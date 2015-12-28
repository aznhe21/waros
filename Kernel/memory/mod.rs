use rt;
use arch;
use arch::page;
use core::intrinsics;
use core::mem;
use core::slice;
use core::usize;

#[cfg(any(target_arch="x86_64", target_arch="x86"))]
use arch::multiboot;

pub mod rust;
pub mod kernel;
pub mod buddy;
pub mod slab;

pub const MAX_ADDR: *mut usize = usize::MAX as *mut usize;

#[inline]
pub fn pre_init() {
    kernel::pre_init();
}

fn init_by_frames<F: FnOnce(&mut [buddy::PageFrame])>(len: usize, init_frames: F) {
    let frames = unsafe {
        slice::from_raw_parts_mut(kernel::allocate_raw(
            mem::size_of::<buddy::PageFrame>() * len,
            mem::align_of::<buddy::PageFrame>()
        ) as *mut buddy::PageFrame, len)
    };
    init_frames(frames);

    buddy::init_by_frames(frames);
    slab::init();

    page::init();
}

pub fn init_by_manual(start: usize, end: usize) {
    init_by_frames((end - start) / arch::FRAME_SIZE, |frames| {
        let kernel_end = kernel::memory_end().as_phys_addr().value();
        for (i, frame) in frames.iter_mut().enumerate() {
            let addr = (start + i * arch::FRAME_SIZE) as u64;
            *frame = buddy::PageFrame::new(kernel::PhysAddr::from_raw(addr), addr <= kernel_end);
        }
    });
}

pub fn init_by_detection(max_addr: *mut usize) {
    unsafe {
        let mut ptr = rt::align_up_mut_ptr(arch::kernel_end().as_mut_ptr::<usize>(), arch::FRAME_SIZE);

        loop {
            let old = intrinsics::volatile_load(ptr);
            let new = old ^ usize::MAX;

            // 適当な値を入れてみて、その値が読み出せなければメモリ範囲外
            intrinsics::volatile_store(ptr, new);
            if intrinsics::volatile_load(ptr) != new {
                break;
            }

            // 元に戻す
            intrinsics::volatile_store(ptr, old);

            if ptr >= max_addr {
                break;
            }

            ptr = ptr.offset(arch::PAGE_SIZE as isize);
        }

        let kernel_start = arch::kernel_start().as_phys_addr().value() as usize;
        let len = (ptr as usize - kernel_start) / arch::FRAME_SIZE;

        init_by_frames(len, |frames| {
            let kernel_end = kernel::memory_end().as_phys_addr().value();
            for (i, frame) in frames.iter_mut().enumerate() {
                let addr = (kernel_start + i * arch::FRAME_SIZE) as u64;
                *frame = buddy::PageFrame::new(kernel::PhysAddr::from_raw(addr), addr <= kernel_end);
            }
        });
    }
}

#[cfg(any(target_arch="x86_64", target_arch="x86"))]
pub fn init_by_multiboot(mmap: &[multiboot::MemoryMap]) {
    let len = mmap.iter()
        .filter(|region| region.type_ == multiboot::MemoryType::Usable)
        .map(|region| (region.length / arch::FRAME_SIZE as u64) as usize)
        .sum::<usize>();

    init_by_frames(len, |frames| {
        let kernel_end = kernel::memory_end().as_phys_addr().value();
        let mut i = 0;
        for region in mmap.iter().filter(|region| region.type_ == multiboot::MemoryType::Usable) {
            let base_addr_end = region.base_addr + (region.length & !(arch::FRAME_SIZE as u64 - 1));
            for addr in (region.base_addr .. base_addr_end).step_by(arch::FRAME_SIZE as u64) {
                frames[i] = buddy::PageFrame::new(kernel::PhysAddr::from_raw(addr), addr <= kernel_end);
                i += 1;
            }
        }
    });
}

#[inline]
pub fn total_size() -> u64 {
    buddy::manager().total_size()
}

#[inline]
pub fn free_size() -> u64 {
    buddy::manager().free_size()
}

#[inline(always)]
pub fn check_oom_ptr<T>(ptr: *mut T) -> *mut T {
    assert!(!ptr.is_null(), "Out of memory");
    ptr
}

#[inline(always)]
pub fn check_oom<T>(ptr: *mut T) -> &'static mut T {
    unsafe { &mut *check_oom_ptr(ptr) }
}

#[inline(always)]
pub fn check_oom_opt<T>(opt: Option<T>) -> T {
    match opt {
        Some(val) => val,
        None => panic!("Out of memory")
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

#[inline(always)]
#[cfg(target_pointer_width="32")]
pub unsafe extern "C" fn fillus(ptr: *mut usize, val: usize, size: usize) {
    memory_fill32(ptr as *mut u32, val as u32, size);
}

#[inline(always)]
#[cfg(target_pointer_width="64")]
pub unsafe extern "C" fn fillus(ptr: *mut usize, val: usize, size: usize) {
    memory_fill64(ptr as *mut u64, val as u64, size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill() {
        macro_rules! fill {
            ($t: ty, $val: expr, $fill: path) => ({
                let mut arr: [$t; 256] = [0; 256];
                $fill(arr.as_mut_ptr(), $val, arr.len());
                for &x in arr.iter() {
                    assert_eq!(x, $val);
                }
            })
        }

        unsafe {
            fill!(u8, 0x12, fill8);
            fill!(u16, 0xFEED, fill16);
            fill!(u32, 0xDEADBEEF, fill32);
            fill!(u64, 0xC0CAC01AADD511FE, fill64);
        }
    }
}

