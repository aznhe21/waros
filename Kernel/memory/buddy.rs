use rt::{IterHelper, Force, ForceRef};
use arch;
use super::kernel::{self, PhysAddr};
use lists::DList;
use core::cmp;
use core::mem;
use core::slice;
use core::usize;
use core::ops::Range;
use core::ptr::Shared;

const MAX_ORDER: usize = 11;
const FRAME_SIZE_ADDR: arch::AddrType = arch::FRAME_SIZE as arch::AddrType;

pub struct BuddyManager {
    frames: &'static mut [PageFrame],
    orders: [DList<PageFrame>; MAX_ORDER]
}

unsafe impl Send for BuddyManager { }
unsafe impl Sync for BuddyManager { }

impl BuddyManager {
    fn init<I: Iterator<Item=Range<PhysAddr>>>(&mut self, size: arch::AddrType, f: I) {
        let len = (size / FRAME_SIZE_ADDR) as usize;

        self.frames = unsafe {
            slice::from_raw_parts_mut(kernel::allocate_raw(
                mem::size_of::<PageFrame>() * len,
                mem::align_of::<PageFrame>()
            ) as *mut PageFrame, len)
        };
        let kernel_end = kernel::done().as_phys_addr();

        let f = f.filter_map(|range| {
            // カーネル領域は使えない
            let start = cmp::max(range.start.align_up(FRAME_SIZE_ADDR), kernel_end);
            let end = range.end.align_down(FRAME_SIZE_ADDR);

            // 1フレームに満たないなら無視
            if start + FRAME_SIZE_ADDR > end {
                None
            } else {
                // 開始アドレスとブロックの大きさ
                Some((start, ((end - start) / FRAME_SIZE_ADDR) as usize))
            }
        });

        for frames in self.orders.iter_mut() {
            *frames = DList::new();
        }

        let mut total = 0;
        let mut i = 0;
        for (mut addr, mut nframes) in f {
            total += nframes;

            let order = cmp::min(MAX_ORDER - 1, usize::BITS - (nframes - 1).leading_zeros() as usize);
            let mut len = 1 << order;

            for order in (0 .. order + 1).rev() {
                while nframes >= len {
                    // PageFrameを初期化し、オーダーを設定する
                    let top_index = i;
                    let end = i + len;
                    while i < end {
                        self.frames[i] = PageFrame::new(addr, false);
                        addr += FRAME_SIZE_ADDR;
                        i += 1;
                    }

                    self.frames[top_index].order = order;
                    self.orders[order].push_front(unsafe { Shared::new(&mut self.frames[top_index]) });

                    nframes -= len;
                }
                len >>= 1;
            }
        }

        // [PageFrame]を確保して減った分のページを埋める
        for frame in &mut self.frames[total..] {
            *frame = PageFrame::new(PhysAddr::null(), true);
        }
    }

    pub fn allocate(&mut self, order: usize) -> Option<Shared<PageFrame>> {
        assert!(order < MAX_ORDER);
        self.orders[order..]
            .iter_mut()
            .enumerate()
            .find_map(|(i, frames)| frames.pop_front().map(|frame| (order + i, frame)))
            .map(|(matched_order, frame)| {
                unsafe {
                    // 分割
                    for div_order in (order .. matched_order).rev() {
                        let div_frame = (**frame).divide_into(div_order);
                        (*div_frame).using = false;
                        (*div_frame).order = div_order;
                        self.orders[div_order].push_front(Shared::new(div_frame));
                    }

                    (**frame).using = true;
                    (**frame).order = order;

                    frame
                }
            })
    }

    pub fn free(&mut self, frame: Shared<PageFrame>) {
        unsafe {
            assert!((**frame).using);

            let mut top_index = self.frames
                .iter()
                .position(|f| f as *const _ == *frame)
                .expect("Invalid page frame");
            let mut order = (**frame).order;

            while order < MAX_ORDER {
                let len = 1 << order;
                let buddy_index = top_index ^ len;
                if !self.is_contiguous_to(top_index, buddy_index) {
                    break;
                }

                let buddy = &mut self.frames[buddy_index];
                if buddy.using || buddy.order != order {
                    break;
                }

                self.orders[order].remove(&Shared::new(buddy));
                top_index &= !len;
                order += 1;
            }

            let top = &mut self.frames[top_index];
            top.using = false;
            top.order = order;
            self.orders[order].push_front(Shared::new(top));
        }
    }

    pub fn total_size(&self) -> u64 {
        self.frames.len() as u64 * arch::FRAME_SIZE as u64
    }

    pub fn free_size(&self) -> u64 {
        self.frames.iter().filter(|frame| !frame.using).fold(0, |acc, _| acc + arch::FRAME_SIZE as u64)
    }

    #[inline]
    fn is_contiguous_to(&self, a: usize, b: usize) -> bool {
        self.frames[a].addr + FRAME_SIZE_ADDR * (b - a) as arch::AddrType == self.frames[b].addr
    }
}

pub struct PageFrame {
    using: bool,
    order: usize,
    addr: PhysAddr,
    prev: Option<Shared<PageFrame>>,
    next: Option<Shared<PageFrame>>
}

impl PageFrame {
    #[inline(always)]
    const fn new(addr: PhysAddr, using: bool) -> PageFrame {
        PageFrame {
            using: using,
            order: 0,
            addr: addr,
            prev: None,
            next: None
        }
    }

    #[inline]
    unsafe fn divide_into(&mut self, order: usize) -> *mut PageFrame {
        (self as *mut PageFrame).offset((1 << order) as isize)
    }

    #[inline(always)]
    pub fn order(&self) -> usize {
        self.order
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        (1 << self.order) * arch::FRAME_SIZE
    }

    #[inline(always)]
    pub fn addr(&self) -> PhysAddr {
        self.addr
    }
}

impl_linked_node!(Shared<PageFrame> { prev: prev, next: next });

static MANAGER: Force<BuddyManager> = Force::new();

#[inline]
pub fn init_by_iter<I: Iterator<Item=Range<PhysAddr>>>(size: arch::AddrType, f: I) {
    MANAGER.setup().init(size, f);
}

#[inline(always)]
pub fn manager() -> ForceRef<BuddyManager> {
    MANAGER.as_ref()
}

#[inline(always)]
pub fn order_by_size(size: usize) -> Option<usize> {
    debug_assert!(size > 0);
    let nframes = (size + arch::FRAME_SIZE - 1) / arch::FRAME_SIZE;
    let order = usize::BITS - (nframes - 1).leading_zeros() as usize;
    if order >= MAX_ORDER {
        None
    } else {
        Some(order)
    }
}

