#[no_mangle]
pub extern fn rust_allocate(size: usize, align: usize) -> *mut u8 {
    super::allocate(size, align)
}

#[no_mangle]
pub extern fn rust_deallocate(ptr: *mut u8, _old_size: usize, _align: usize) {
    super::free(ptr)
}

#[no_mangle]
pub extern fn rust_reallocate(ptr: *mut u8, _old_size: usize, size: usize, align: usize) -> *mut u8 {
    super::reallocate(ptr, size, align)
}

#[no_mangle]
pub extern fn rust_reallocate_inplace(_ptr: *mut u8, old_size: usize, _size: usize, _align: usize) -> usize {
    old_size
}

#[no_mangle]
pub extern fn rust_usable_size(size: usize, _align: usize) -> usize {
    size
}

#[no_mangle]
pub extern fn rust_stats_print() {
}

