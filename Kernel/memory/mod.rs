#![allow(dead_code)]

use prelude::*;
use self::kernel::PhysAddr;
use core::usize;
use core::mem;
use core::ptr;
use core::slice;
use multiboot;

pub mod rust;
pub mod kernel;

struct PhysicalMemory {
    total_size: usize,
    total_blocks: usize,
    allocated_blocks: usize,
    free_blocks: usize,
    first: *mut PhysicalMemoryBlock
}

impl PhysicalMemory {
    const BLOCK_SIZE: usize = 0x1000;

    pub fn iter(&self) -> PhysicalMemoryIter {
        PhysicalMemoryIter { block: self.first }
    }

    pub fn block_of(&self, ptr: *const u8) -> Option<&'static mut PhysicalMemoryBlock> {
        unsafe {
            self.iter()
                .filter(|block| ptr >= block.data_ptr() && block.data_ptr().uoffset(block.data_size()) < ptr)
                .next()
        }
    }
}

struct PhysicalMemoryIter {
    block: *mut PhysicalMemoryBlock
}

impl Iterator for PhysicalMemoryIter {
    type Item = &'static mut PhysicalMemoryBlock;

    #[inline]
    fn next(&mut self) -> Option<&'static mut PhysicalMemoryBlock> {
        unsafe {
            let ret = self.block;
            self.block = ret.as_mut().map_or(ptr::null_mut(), |block| block.next);
            ret.as_mut()
        }
    }
}

struct PhysicalMemoryBlock {
    next: *mut PhysicalMemoryBlock,
    size: usize
}

impl PhysicalMemoryBlock {
    pub fn new(next: *mut PhysicalMemoryBlock, mblock: &multiboot::MemoryMap) -> *mut PhysicalMemoryBlock {
        unsafe {
            let block = &mut *PhysAddr::from_raw(mblock.base_addr as usize).as_virt_addr().as_mut_ptr();
            *block = PhysicalMemoryBlock {
                next: next,
                size: mblock.length as usize
            };
            mem::transmute(&block)
        }
    }

    #[inline]
    fn bmap_len(&self) -> usize {
        (self.size + usize::BITS - 1) / usize::BITS
    }

    #[inline]
    fn bmap_ptr(&self) -> *mut usize {
        unsafe {
            let ptr: *mut u8 = mem::transmute(self);
            ptr.uoffset(mem::size_of::<PhysicalMemoryBlock>()) as *mut usize
        }
    }

    #[inline]
    fn bmap(&self) -> &[usize] {
        unsafe {
            slice::from_raw_parts(self.bmap_ptr(), self.bmap_len())
        }
    }

    #[inline]
    fn bmap_mut(&mut self) -> &mut [usize] {
        unsafe {
            slice::from_raw_parts_mut(self.bmap_ptr(), self.bmap_len())
        }
    }

    #[inline]
    fn data_ptr(&self) -> *const u8 {
        unsafe {
            let ptr: *const u8 = mem::transmute(self);
            ptr.uoffset(mem::size_of::<PhysicalMemoryBlock>() + self.bmap_len() * mem::size_of::<usize>())
        }
    }

    #[inline]
    fn data_ptr_mut(&mut self) -> *mut u8 {
        self.data_ptr() as *mut u8
    }

    #[inline]
    pub fn data_size(&self) -> usize {
        self.size - mem::size_of::<PhysicalMemoryBlock>() - self.bmap_len() * mem::size_of::<usize>()
    }

    #[inline]
    pub fn occupied(&self, index: usize) -> bool {
        self.bmap()[index / usize::BITS] & (1 << (index % usize::BITS)) == 0
    }

    pub fn addr<T>(&mut self, index: usize) -> *mut T {
        unsafe {
            self.data_ptr().uoffset(index * PhysicalMemory::BLOCK_SIZE) as *mut T
        }
    }

    pub fn allocate(&mut self, alloc_bits: usize, size: usize, _align: usize) -> *mut u8 {
        unsafe {
            let data_slice = slice::from_raw_parts_mut(self.data_ptr_mut() as *mut usize, self.bmap_len());
            for (cur_bits, addr) in self.bmap_mut().iter_mut().zip(data_slice) {
                let mut bits = *cur_bits;
                for j in 0 .. usize::BITS {
                    if bits & alloc_bits == alloc_bits {
                        *cur_bits &= !(alloc_bits << j);
                        *addr = size;
                        let ptr: *mut usize = mem::transmute(addr);
                        log!("Allocated Addr: {:p}", ptr);
                        return ptr.uoffset(1) as *mut u8;
                    }
                    bits = bits.wrapping_shr(1);
                }
            }
            ptr::null_mut()
        }
    }

    pub fn reallocate(&mut self, _ptr: *mut u8, _size: usize, _align: usize) -> *mut u8 {
        0 as *mut u8
    }

    pub fn free(&mut self, ptr: *mut u8) {
        unsafe {
            let index = self.data_ptr() as usize - ptr as usize;
            let total = *(ptr as *mut usize).uoffset_rev(1);
            let bits_len = total + usize::BITS - 1;
            let bits = (1 << bits_len / usize::BITS) - 1;
            self.bmap_mut()[index / usize::BITS] |= bits << ((index % usize::BITS) - bits_len);
        }
    }
}

static mut physical_memory: PhysicalMemory = PhysicalMemory {
    total_size: 0, total_blocks: 0, allocated_blocks: 0, free_blocks: 0,
    first: 0 as *mut PhysicalMemoryBlock
};

#[inline]
pub fn init(_mmap: &[multiboot::MemoryMap]) {
    kernel::init();

    /*for block in mmap {
        log!("{}: Addr {:X} Length {:X}", block.type_.description(), block.base_addr, block.length);
    }

    let size = mmap.iter()
        .filter(|block| block.type_ == multiboot::MemoryType::Usable)
        .fold(0, |acc, block| acc + block.length as usize);
    let blocks = size / PhysicalMemory::BLOCK_SIZE;

    let first_block = mmap.iter()
        .filter(|block| block.type_ == multiboot::MemoryType::Usable)
        .rev()
        .fold(0 as *mut PhysicalMemoryBlock, |next, block| PhysicalMemoryBlock::new(next, block));

    unsafe {
        physical_memory = PhysicalMemory {
            total_size: size,
            total_blocks: blocks,
            allocated_blocks: blocks,
            free_blocks: 0,
            first: first_block
        };
    }*/
}

#[inline]
pub fn size() -> usize {
    unsafe { physical_memory.total_size }
}

pub fn allocate(size: usize, align: usize) -> *mut u8 {
    let total = size + usize::BYTES;
    let bits = (1 << (total + usize::BITS - 1) / usize::BITS) - 1;
    unsafe {
        physical_memory.iter().map(|block| block.allocate(bits, total, align))
            .filter(|ptr| !ptr.is_null())
            .next()
            .unwrap_or(ptr::null_mut())
    }
}

pub fn reallocate(ptr: *mut u8, size: usize, align: usize) -> *mut u8 {
    unsafe {
        match physical_memory.block_of(ptr) {
            Some(block) => block.reallocate(ptr, size, align),
            None => ptr::null_mut()
        }
    }
}

pub fn free(ptr: *mut u8) {
    unsafe {
        if let Some(block) = physical_memory.block_of(ptr) {
            block.free(ptr);
        }
    }
}

#[allow(improper_ctypes)]
extern "C" {
    fn memory_fill8(ptr: *mut u8, val: u8, size: usize);
    fn memory_fill16(ptr: *mut u16, val: u16, size: usize);
    fn memory_fill32(ptr: *mut u32, val: u32, size: usize);
    fn memory_fill64(ptr: *mut u64, val: u64, size: usize);
}

#[inline(always)]
pub unsafe extern "C" fn fill8(ptr: *mut u8, val: u8, size: usize) {
    memory_fill8(ptr, val, size);
}

#[inline(always)]
pub unsafe extern "C" fn fill16(ptr: *mut u16, val: u16, size: usize) {
    memory_fill16(ptr, val, size);
}

#[inline(always)]
pub unsafe extern "C" fn fill32(ptr: *mut u32, val: u32, size: usize) {
    memory_fill32(ptr, val, size);
}

#[inline(always)]
pub unsafe extern "C" fn fill64(ptr: *mut u64, val: u64, size: usize) {
    memory_fill64(ptr, val, size);
}

