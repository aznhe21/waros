use self::kernel::PhysAddr;
use arch;
use arch::page;
use core::intrinsics;
use core::iter;
use core::ops::Range;

pub mod rust;
pub mod kernel;
pub mod buddy;
pub mod kcache;

pub const MAX_ADDR: PhysAddr = PhysAddr::from_raw(arch::AddrType::max_value());

#[inline]
pub fn pre_init() {
    kernel::pre_init();
}

pub fn init_by_iter<I: Iterator<Item=Range<PhysAddr>>>(size: arch::AddrType, f: I) {
    buddy::init_by_iter(size, f);
    kcache::init();

    page::init();
}

#[inline(always)]
pub fn init_by_manual(range: Range<PhysAddr>) {
    init_by_iter(range.end.value() - range.start.value(), iter::once(range));
}

pub unsafe fn init_by_detection(max_addr: PhysAddr) {
    let max_addr = max_addr.as_virt_addr();
    let mut addr = arch::kernel_end().align_up(arch::FRAME_SIZE);

    loop {
        let ptr_us: *mut usize = addr.as_mut_ptr();
        let old = intrinsics::volatile_load(ptr_us);
        let new = !old;

        // 適当な値を入れてみて、その値が読み出せなければメモリ範囲外
        intrinsics::volatile_store(ptr_us, new);
        if intrinsics::volatile_load(ptr_us) != new {
            break;
        }

        // 元に戻す
        intrinsics::volatile_store(ptr_us, old);

        if addr >= max_addr {
            break;
        }

        addr += arch::PAGE_SIZE;
    }

    let range = arch::kernel_start().as_phys_addr() .. addr.as_phys_addr();
    debug_log!("Memory detected: {:?}", range);

    init_by_manual(range);
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
pub fn check_oom_opt<T>(opt: Option<T>) -> T {
    opt.expect("Out of memory")
}

#[allow(improper_ctypes)]
extern "C" {
    fn memory_fill8(ptr: *mut u8, val: u8, size: usize);
    fn memory_fill16(ptr: *mut u16, val: u16, size: usize);
    fn memory_fill32(ptr: *mut u32, val: u32, size: usize);
    fn memory_fill64(ptr: *mut u64, val: u64, size: usize);
}

#[inline(always)]
pub unsafe fn fill8(ptr: *mut u8, val: u8, size: usize) {
    memory_fill8(ptr, val, size);
}

#[inline(always)]
pub unsafe fn fill16(ptr: *mut u16, val: u16, size: usize) {
    memory_fill16(ptr, val, size);
}

#[inline(always)]
pub unsafe fn fill32(ptr: *mut u32, val: u32, size: usize) {
    memory_fill32(ptr, val, size);
}

#[inline(always)]
pub unsafe fn fill64(ptr: *mut u64, val: u64, size: usize) {
    memory_fill64(ptr, val, size);
}

#[inline(always)]
#[cfg(target_pointer_width="32")]
pub unsafe fn fillus(ptr: *mut usize, val: usize, size: usize) {
    memory_fill32(ptr as *mut u32, val as u32, size);
}

#[inline(always)]
#[cfg(target_pointer_width="64")]
pub unsafe fn fillus(ptr: *mut usize, val: usize, size: usize) {
    memory_fill64(ptr as *mut u64, val as u64, size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill() {
        macro_rules! fill {
            ($t: ident, $val: expr, $fill: path) => ({
                use std::{$t};
                const LEN: usize = 16 / $t::BYTES;
                let mut arr: [$t; LEN] = [0; LEN];

                $fill(arr.as_mut_ptr(), 1, 0);
                assert_eq!(arr, [0; LEN]);

                $fill(arr.as_mut_ptr(), $val, LEN / 2);
                assert_eq!(arr[0..LEN/2], [$val; LEN/2]);
                assert_eq!(arr[LEN/2..], [0; LEN/2]);

                $fill(arr.as_mut_ptr(), $val, LEN);
                assert_eq!(arr, [$val; LEN]);
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

