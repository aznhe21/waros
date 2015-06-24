use core::iter::Iterator;

#[no_mangle]
pub unsafe extern fn memset(buf: *mut u8, ch: i32, n: usize) -> *mut u8 {
    for i in 0 .. n as isize {
        *buf.offset(i) = ch as u8;
    }

    buf
}

#[no_mangle]
pub unsafe extern fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if src < dst {
        for i in (0 .. n as isize).rev() {
            *dst.offset(i) = *src.offset(i);
        }
    } else {
        for i in 0 .. n as isize {
            *dst.offset(i) = *src.offset(i);
        }
    }

    dst
}

#[no_mangle]
pub unsafe extern fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0 .. n as isize {
        *dst.offset(i) = *src.offset(i);
    }

    dst
}

#[inline]
pub unsafe fn strlen(ptr: *const u8) -> usize {
    let mut p = ptr;
    while *p != 0 {
        p = p.offset(1);
    }

    p as usize - ptr as usize
}
