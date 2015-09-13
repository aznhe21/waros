use super::slab;

#[no_mangle]
pub extern fn rust_allocate(size: usize, align: usize) -> *mut u8 {
    slab::manager().allocate(size, align)
}

#[no_mangle]
pub extern fn rust_deallocate(ptr: *mut u8, _old_size: usize, align: usize) {
    slab::manager().free(ptr, align)
}

#[no_mangle]
pub extern fn rust_reallocate(ptr: *mut u8, _old_size: usize, size: usize, align: usize) -> *mut u8 {
    slab::manager().reallocate(ptr, size, align)
}

#[no_mangle]
pub extern fn rust_reallocate_inplace(ptr: *mut u8, _old_size: usize, size: usize, align: usize) -> usize {
    slab::manager().reallocate_inplace(ptr, size, align)
}

#[no_mangle]
pub extern fn rust_usable_size(size: usize, align: usize) -> usize {
    slab::manager().usable_size(size, align)
}

#[no_mangle]
pub extern fn rust_stats_print() {
    slab::manager().stats_print()
}

