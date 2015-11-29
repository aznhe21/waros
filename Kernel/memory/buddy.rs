#![allow(dead_code)]

use prelude::*;
use arch;
use arch::multiboot;
use super::kernel::PhysAddr;
use lists::LinkedList;
use core::cmp;
use core::ptr;
use core::mem;
use core::slice;
use core::usize;

const MAX_ORDER: usize = 11;

pub struct BuddyManager {
    frames: &'static mut [PageFrame],
    orders: [LinkedList<PageFrame>; MAX_ORDER]
}

impl BuddyManager {
    #[inline(always)]
    fn init(&mut self, frames: &'static mut [PageFrame]) {
        self.frames = frames;
        for order in self.orders.iter_mut() {
            *order = LinkedList::new();
        }

        let order = cmp::min(MAX_ORDER - 1, usize::BITS - self.frames.len().leading_zeros() as usize);

        let mut idx = 0;
        let mut cur_len = self.frames.len();
        let mut frame_len = 1 << order;
        for cur_order in (0 .. order + 1).rev() {
            while cur_len >= frame_len {
                let frame = &mut self.frames[idx];
                frame.order = cur_order;
                self.orders[cur_order].push_front(frame);

                idx += frame_len;
                cur_len -= frame_len;
            }
            frame_len >>= 1;
        }
    }

    pub fn allocate(&mut self, order: usize) -> Option<&'static mut PageFrame> {
        assert!(order < MAX_ORDER);
        self.orders[order..]
            .iter_mut()
            .enumerate()
            .find_map(|(i, frames)| frames.pop_front().map(|frame| (order + i, frame)))
            .map(|(matched_order, frame)| {
                // 分割
                for cur_order in (order .. matched_order).rev() {
                    let new_frame = frame.divide_by(cur_order);
                    new_frame.order = cur_order;
                    self.orders[cur_order].push_front(new_frame);
                }

                frame.using = true;
                frame.order = order;
                frame
            })
    }

    pub fn free(&mut self, frame: &'static mut PageFrame) {
        assert!(frame.using);

        let frame_ptr = frame as *const PageFrame;
        let mut top_index = self.frames
            .iter()
            .position(|f| f as *const _ == frame_ptr)
            .expect("Invalid page frame");
        let mut order = frame.order;

        while order < MAX_ORDER {
            let count = 1 << order;
            let buddy = unsafe { &mut *self.frames.as_mut_ptr().uoffset(top_index ^ count) };

            if buddy.using || buddy.order != order {
                break;
            }

            self.orders[order].remove(buddy);
            top_index &= !count;
            order += 1;
        }

        unsafe {
            let top = self.frames.as_mut_ptr().uoffset(top_index);
            (*top).using = false;
            (*top).order = order;
            self.orders[order].push_front(top);
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
    prev: *mut PageFrame,
    next: *mut PageFrame
}

impl PageFrame {
    #[inline(always)]
    fn new(addr: PhysAddr, using: bool) -> PageFrame {
        PageFrame {
            using: using,
            order: 0,
            addr: addr,
            prev: ptr::null_mut(),
            next: ptr::null_mut()
        }
    }

    #[inline]
    fn divide_by(&mut self, order: usize) -> &mut PageFrame {
        unsafe { &mut *(self as *mut PageFrame).uoffset(1 << (order - 1)) }
    }

    #[inline(always)]
    pub fn addr(&self) -> PhysAddr {
        self.addr
    }
}

impl_linked_node!(PageFrame { prev: prev, next: next });

static mut manager_opt: Option<BuddyManager> = None;

#[inline]
pub fn init_by_multiboot(mmap: &[multiboot::MemoryMap]) {
    let len = mmap.iter()
        .filter(|region| region.type_ == multiboot::MemoryType::Usable)
        .map(|region| (region.length / arch::FRAME_SIZE as u64) as usize)
        .sum::<usize>();

    let frames = unsafe {
        slice::from_raw_parts_mut(super::kernel::allocate(
            mem::size_of::<PageFrame>() * len,
            mem::align_of::<PageFrame>()
        ) as *mut PageFrame, len)
    };

    let kernel_end = super::kernel::memory_end().as_phys_addr().value();

    let mut i = 0;
    for region in mmap.iter().filter(|region| region.type_ == multiboot::MemoryType::Usable) {
        let base_addr_end = region.base_addr + (region.length & !(arch::FRAME_SIZE as u64 - 1));
        for addr in (region.base_addr .. base_addr_end).step_by(arch::FRAME_SIZE as u64) {
            frames[i] = PageFrame::new(PhysAddr::from_raw(addr), addr <= kernel_end);
            i += 1;
        }
    }

    unsafe {
        manager_opt.into_some().init(frames);
    }
}

#[inline]
pub fn manager() -> &'static mut BuddyManager {
    unsafe {
        manager_opt.as_mut().be_some()
    }
}

