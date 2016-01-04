use super::kcache;

#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    kcache::manager().allocate(size, align)
}

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, _old_size: usize, align: usize) {
    kcache::manager().free(ptr, align)
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, _old_size: usize, size: usize, align: usize) -> *mut u8 {
    kcache::manager().reallocate(ptr, size, align)
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(ptr: *mut u8, _old_size: usize, size: usize, align: usize) -> usize {
    kcache::manager().reallocate_inplace(ptr, size, align)
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, align: usize) -> usize {
    kcache::manager().usable_size(size, align)
}

