/*/// DO NOT USE THIS FUNCTION
#[no_mangle]
pub unsafe extern "C" fn memset(buf: *mut u8, ch: i32, n: usize) -> *mut u8 {
    for i in 0 .. n {
        *buf.uoffset(i) = ch as u8;
    }

    buf
}*/

/// DO NOT USE THIS FUNCTION
#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
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

/// DO NOT USE THIS FUNCTION
#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0 .. n as isize {
        *dst.offset(i) = *src.offset(i);
    }

    dst
}

#[inline]
pub unsafe fn strlen(ptr: *const u8) -> usize {
    let mut ps = ptr as *const usize;
    while *ps != 0 {
        ps = ps.offset(1);
    }

    let mut p8 = ps as *const u8;
    while *p8 != 0 {
        p8 = p8.offset(1);
    }

    p8 as usize - ptr as usize
}

