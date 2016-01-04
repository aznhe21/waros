use rt::{IterHelper, Force, ForceRef};
use arch;
use super::kernel::PhysAddr;
use lists::LinkedList;
use core::cmp;
use core::mem;
use core::usize;
use core::ptr::Shared;

const MAX_ORDER: usize = 11;

pub struct BuddyManager {
    frames: &'static mut [PageFrame],
    orders: [LinkedList<PageFrame>; MAX_ORDER]
}

unsafe impl Send for BuddyManager { }
unsafe impl Sync for BuddyManager { }

impl BuddyManager {
    #[inline(always)]
    fn init(&mut self, frames: &'static mut [PageFrame]) {
        self.frames = frames;
        for order in self.orders.iter_mut() {
            *order = LinkedList::new();
        }

        let order = cmp::min(MAX_ORDER - 1, usize::BITS - (self.frames.len() - 1).leading_zeros() as usize);

        let mut idx = 0;
        let mut cur_len = self.frames.len();
        let mut frame_len = 1 << order;
        for cur_order in (0 .. order + 1).rev() {
            while cur_len >= frame_len {
                let frame = &mut self.frames[idx];
                frame.order = cur_order;
                self.orders[cur_order].push_front(unsafe { Shared::new(frame) });

                idx += frame_len;
                cur_len -= frame_len;
            }
            frame_len >>= 1;
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
                    for cur_order in (order .. matched_order).rev() {
                        let new_frame = (**frame).divide_into(cur_order);
                        (*new_frame).order = cur_order;
                        self.orders[cur_order].push_front(Shared::new(new_frame));
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
                let count = 1 << order;
                let buddy = &mut self.frames[top_index ^ count];

                if buddy.using || buddy.order != order {
                    break;
                }

                self.orders[order].remove(&Shared::new(buddy));
                top_index &= !count;
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
    pub const fn new(addr: PhysAddr, using: bool) -> PageFrame {
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
pub fn init_by_frames(frames: &'static mut [PageFrame]) {
    MANAGER.setup().init(frames);
}

#[inline(always)]
pub fn manager() -> ForceRef<BuddyManager> {
    MANAGER.as_ref()
}

#[inline(always)]
pub fn order_by_size(size: usize) -> usize {
    debug_assert!(size > 0);
    let nframes = (size + arch::FRAME_SIZE - 1) / arch::FRAME_SIZE;
    usize::BITS - (nframes - 1).leading_zeros() as usize
}

