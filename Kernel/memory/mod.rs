#![allow(dead_code)]

use core::prelude::*;
use core::u32;
use core::mem;
use core::ptr;
use core::intrinsics::{volatile_load, volatile_store};
//use arch;
use multiboot::MultibootInfo;

pub mod rust;

/*pub fn init(begin: usize, end: usize) {
}

pub fn size() -> usize {
    0
}

fn allocate(size: usize, align: usize) -> *mut u8 {
    0 as *mut u8
}

fn free(ptr: *mut u8) {
}

fn reallocate(ptr: *mut u8, size: usize, align: usize) -> *mut u8 {
    0 as *mut u8
}*/

struct Block {
    size: usize,
    prev: *mut Block,
    next: *mut Block
}

impl Block {
    fn next(&mut self, size: usize) -> Option<&mut Block> {
        if self.data_size() >= size {
            Some(self)
        } else if !self.next.is_null() {
            unsafe { (*self.next).next(size) }
        } else {
            None
        }
    }

    fn find<T>(&mut self, ptr: *mut T) -> Option<&mut Block> {
        if unsafe { self.data_ptr() } == ptr {
            Some(self)
        } else if !self.next.is_null() {
            unsafe { (*self.next).find(ptr) }
        } else {
            None
        }
    }

    #[inline]
    unsafe fn data_ptr<T>(&mut self) -> *mut T {
        let ptr: *mut Block = mem::transmute(&self);
        mem::transmute(ptr.offset(1))
    }

    /*#[inline]
    fn data<T>(&mut self) -> &'static mut T {
        unsafe { &mut *self.data_ptr() }
    }*/

    #[inline]
    fn data_size(&self) -> usize {
        self.size - mem::size_of::<Block>()
    }
}

struct BlockList {
    first: *mut Block,
    last: *mut Block
}

impl BlockList {
    fn next(&mut self, size: usize) -> Option<&mut Block> {
        unsafe { (*self.first).next(size) }
    }

    fn find<T>(&mut self, ptr: *mut T) -> Option<&mut Block> {
        unsafe { (*self.first).find(ptr) }
    }

    fn add(&mut self, target: &mut Block) {
        target.prev = self.last;
        target.next = ptr::null_mut();
        if self.first.is_null() {
            self.first = target;
        }
        if !self.last.is_null() {
            unsafe { (*self.last).next = target };
        }
        self.last = target;
    }

    fn remove(&mut self, target: &mut Block) {
        if self.first == target {
            self.first = target.next;
        }
        if self.last == target {
            self.last = target.prev;
        }
    }

    fn remove_and_insert(&mut self, old: &mut Block, new: &mut Block) {
        new.prev = old.prev;
        new.next = old.next;
        if !old.next.is_null() {
            unsafe { (*old.next).prev = new };
        }
        if !old.prev.is_null() {
            unsafe { (*old.prev).next = new };
        }
        if self.first == old {
            self.first = new;
        }
        if self.last == old {
            self.last = new;
        }
    }
}

static mut free_list: BlockList = BlockList { first: 0 as *mut Block, last: 0 as *mut Block };
static mut allocated: BlockList = BlockList { first: 0 as *mut Block, last: 0 as *mut Block };
static mut cached_size: usize = 0;

unsafe fn fetch_size(begin: usize, end: usize) -> usize {
    let mut ptr = begin as *mut u32;

    while ptr < end as *mut u32 {
        let val = volatile_load(ptr);
        volatile_store(ptr, 0x12345678);
        if volatile_load(ptr) != 0x12345678 {
            break;
        }
        volatile_store(ptr, val);
        ptr = ptr.offset(1024 * 1024 / u32::BYTES as isize);
    }

    ptr as usize
}

pub fn init(_multiboot: &'static MultibootInfo) {
    /*unsafe {
        arch::begin_memory_direct_access();
        cached_size = fetch_size(begin, end);
        arch::end_memory_direct_access();

        let block: *mut Block = mem::transmute(begin);
        *block = Block {
            size: cached_size,
            prev: ptr::null_mut(),
            next: ptr::null_mut()
        };

        free_list = BlockList {
            first: block,
            last: block
        };
    }*/
}

pub fn size() -> usize {
    //unsafe { cached_size }
    ::multiboot::info().mem_size().unwrap_or(0) as usize
}

pub fn allocate<T>(size: usize, _align: usize) -> *mut T {
    unsafe {
        free_list.next(size).map_or(ptr::null_mut(), |block| {
            let ptr = block.data_ptr::<T>();

            let next = &mut *(ptr.offset(size as isize) as *mut Block);
            next.size = block.data_size() - size;
            free_list.remove_and_insert(block, next);

            block.size = mem::size_of::<Block>() + size;
            allocated.add(block);

            ptr
        })
    }
}

pub fn reallocate<T>(ptr: *mut T, _size: usize, _align: usize) -> *mut T {
    unsafe {
        allocated.find(ptr).map_or(ptr::null_mut(), |_block| {
            //
            0 as *mut T
        })
    }
}

pub fn new<T>() -> Option<&'static mut T> {
    unsafe {
        allocate(mem::size_of::<T>(), mem::min_align_of::<T>()).as_mut()
    }
}

pub fn free<T>(ptr: *mut T) {
    unsafe {
        match allocated.find(ptr) {
            Some(block) => {
                allocated.remove(block);
                free_list.add(block);
            },
            None => {}
        }
    }
}

pub fn delete<T>(data: &'static mut T) {
    unsafe {
        free::<T>(mem::transmute(data));
    }
}

