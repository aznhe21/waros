use rt::divmod;

#[no_mangle]
pub unsafe extern "C" fn __udivmoddi4(a: u64, b: u64, rem: *mut u64) -> u64 {
    divmod::udivmod64(a, b, rem)
}

#[no_mangle]
pub unsafe extern "C" fn __udivdi3(a: u64, b: u64) -> u64 {
    divmod::udiv64(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn __umoddi3(a: u64, b: u64) -> u64 {
    divmod::umod64(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn __divdi3(a: i64, b: i64) -> i64 {
    divmod::idiv64(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn __moddi3(a: i64, b: i64) -> i64 {
    divmod::imod64(a, b)
}

