use arch::multiboot;
use arch::page;

pub mod rust;
pub mod kernel;
pub mod buddy;
pub mod slab;

#[inline]
pub fn init_by_multiboot(mmap: &[multiboot::MemoryMap]) {
    self::kernel::init();
    let pt = page::PageTable::new();

    self::buddy::init_by_multiboot(mmap);
    self::slab::init();

    page::init(pt);
}

#[inline]
pub fn total_size() -> u64 {
    self::buddy::manager().total_size()
}

#[inline]
pub fn free_size() -> u64 {
    self::buddy::manager().free_size()
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

