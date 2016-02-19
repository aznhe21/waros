use rt::divmod;

extern "C" {
    fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn memset(dst: *mut u8, ch: i32, n: usize) -> *mut u8;
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_unwind_cpp_pr0() {
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_unwind_cpp_pr1() {
}

#[no_mangle]
pub unsafe extern "C" fn __udivmodsi4(a: u32, b: u32, rem: *mut u32) -> u32 {
    divmod::udivmod32(a, b, rem)
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_uidiv(a: u32, b: u32) -> u32 {
    divmod::udiv32(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn __umodsi3(a: u32, b: u32) -> u32 {
    divmod::umod32(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_idiv(a: i32, b: i32) -> i32 {
    divmod::idiv32(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn __modsi3(a: i32, b: i32) -> i32 {
    divmod::imod32(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_uldivmod(num: u64, den: u64, rem_p: *mut u64) -> u64 {
    divmod::udivmod64(num, den, rem_p)
}

#[no_mangle]
pub unsafe extern "C" fn __umoddi3(a: u64, b: u64) -> u64 {
    divmod::umod64(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memcpy(dst: *mut u8, src: *const u8, n: usize) {
    memcpy(dst, src, n);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memcpy4(dst: *mut u8, src: *const u8, n: usize) {
    memcpy(dst, src, n);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memcpy8(dst: *mut u8, src: *const u8, n: usize) {
    memcpy(dst, src, n);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memmove(dst: *mut u8, src: *const u8, n: usize) {
    memmove(dst, src, n);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memmove4(dst: *mut u8, src: *const u8, n: usize) {
    memmove(dst, src, n);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memmove8(dst: *mut u8, src: *const u8, n: usize) {
    memmove(dst, src, n);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memset(dst: *mut u8, n: usize, c: i32) {
    memset(dst, c, n);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memset4(dst: *mut u8, n: usize, c: i32) {
    memset(dst, c, n);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memset8(dst: *mut u8, n: usize, c: i32) {
    memset(dst, c, n);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memclr(dst: *mut u8, n: usize) {
    __aeabi_memset(dst, n, 0);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memclr4(dst: *mut u8, n: usize) {
    __aeabi_memset4(dst, n, 0);
}

#[no_mangle]
pub unsafe extern "C" fn __aeabi_memclr8(dst: *mut u8, n: usize) {
    __aeabi_memset8(dst, n, 0);
}

