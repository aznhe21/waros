use core::ptr;
use core::mem;
use core::{u32, u64, i8, i64};

/* Effects: if rem != 0, *rem = a % b
 * Returns: a / b
 */
#[no_mangle]
pub unsafe fn __udivmoddi4(a: u64, b: u64, rem: *mut u64) -> u64 {
    #[allow(non_upper_case_globals)]
    const u32_BITS: u32 = u32::BITS as u32;
    #[allow(non_upper_case_globals)]
    const u64_BITS: u32 = u64::BITS as u32;

    let (a_lo, a_hi): (u32, u32) = mem::transmute(a);
    let (b_lo, b_hi): (u32, u32) = mem::transmute(b);

    let mut sr: u32;

    let mut r_t: (u32, u32) = mem::uninitialized();
    let mut q_t: (u32, u32) = mem::uninitialized();
    let r_all: &mut u64 = mem::transmute(&mut r_t);
    let q_all: &mut u64 = mem::transmute(&mut q_t);
    let (q_lo, q_hi) = (&mut q_t.0, &mut q_t.1);
    let (r_lo, r_hi) = (&mut r_t.0, &mut r_t.1);

    match (a_hi, a_lo, b_hi, b_lo) {
        (0, _, 0, _) => {
            /* 0 X
             * ---
             * 0 X
             */
            if !rem.is_null() {
                *rem = (a_lo % b_lo) as u64;
            }
            return (a_lo / b_lo) as u64;
        },
        (0, _, _, _) => {
            /* 0 X
             * ---
             * K X
             */
            if !rem.is_null() {
                *rem = a_lo as u64;
            }
            return 0;
        },
        (_, _, 0, 0) => {
            /* K X
             * ---
             * 0 0
             */
            if !rem.is_null() {
                *rem = (a_hi % b_lo) as u64;
            }
            return (a_hi / b_lo) as u64;
        },
        (_, 0, _, 0) => {
            /* K 0
             * ---
             * K 0
             */
            if !rem.is_null() {
                *r_hi = a_hi % b_hi;
                *r_lo = 0;
                *rem = *r_all;
            }
            return (a_hi / b_hi) as u64;
        },
        //(_, _, _, 0) if b_hi & (b_hi - 1) == 0 => { /* if b is a power of 2 */
        (_, _, _, 0) if b_hi.is_power_of_two() => {
            /* K K
             * ---
             * K 0
             */
            if !rem.is_null() {
                *r_lo = a_lo;
                *r_hi = a_hi & (b_hi - 1);
                *rem = *r_all;
            }
            return (a_hi.wrapping_shr(b_hi.trailing_zeros())) as u64;
        },
        (_, _, _, 0) => {
            /* K K
             * ---
             * K 0
             */
            sr = b_hi.leading_zeros().wrapping_sub(a_hi.leading_zeros());
            /* 0 <= sr <= u32::BITS - 2 or sr large */
            if sr > u64_BITS - 2 {
                if !rem.is_null() {
                    *rem = a;
                }
                return 0;
            } else {
                sr += 1;
                /* 1 <= sr <= u32::BITS - 1 */
                /* q = a << (u64::BITS - sr); */
                *q_lo = 0;
                *q_hi = a_lo.wrapping_shl(u32_BITS.wrapping_sub(sr));
                /* r = a >> sr; */
                *r_hi = a_hi.wrapping_shr(sr);
                *r_lo = a_hi.wrapping_shl(u32_BITS.wrapping_sub(sr)) | a_lo.wrapping_shr(sr);
            }
        },
        (_, _, 0, _) if b_lo.is_power_of_two() => {
            /* K X
             * ---
             * 0 K
             */
            if !rem.is_null() {
                *rem = (a_lo & (b_lo - 1)) as u64;
            }
            if b_lo == 1 {
                return a;
            } else {
                sr = b_lo.trailing_zeros();
                *q_hi = a_hi >> sr;
                *q_lo = (a_hi << (u32_BITS - sr)) | (a_lo >> sr);
                return *q_all;
            }
        },
        (_, _, 0, _) => {
            /* K X
             * ---
             * 0 K
             */
            sr = 1 + u32_BITS + b_lo.leading_zeros() - a_hi.leading_zeros();
            /* 2 <= sr <= u64::BITS - 1
             * q = a << (u64::BITS - sr);
             * r = a >> sr;
             * if (sr == u32::BITS)
             * {
             *     q_lo = 0;
             *     q_hi = a_lo;
             *     r_hi = 0;
             *     r_lo = a_hi;
             * }
             * else if (sr < u32::BITS)  // 2 <= sr <= u32::BITS - 1
             * {
             *     q_lo = 0;
             *     q_hi = a_lo << (u32::BITS - sr);
             *     r_hi = a_hi >> sr;
             *     r_lo = (a_hi << (u32::BITS - sr)) | (a_lo >> sr);
             * }
             * else              // u32::BITS + 1 <= sr <= u64::BITS - 1
             * {
             *     q_lo = a_lo << (u64::BITS - sr);
             *     q_hi = (a_hi << (u64::BITS - sr)) |
             *            (a_lo >> (sr - u32::BITS));
             *     r_hi = 0;
             *     r_lo = a_hi >> (sr - u32::BITS);
             * }
             */
            *q_lo = a_lo.wrapping_shl(u64_BITS - sr) &
                   (u32_BITS.wrapping_sub(sr) as i32 >> (u32_BITS - 1)) as u32;
            *q_hi = (a_lo.wrapping_shl(u32_BITS.wrapping_sub(sr))                   &
                   (sr.wrapping_sub(u32_BITS + 1) as i32 >> (u32_BITS - 1)) as u32) |
                   ((a_hi.wrapping_shl(u64_BITS - sr)                               |
                   a_lo.wrapping_shr(sr.wrapping_sub(u32_BITS)))                    &
                   (u32_BITS.wrapping_sub(sr) as i32 >> (u32_BITS - 1)) as u32);
            *r_hi = a_hi.wrapping_shr(sr) &
                   (sr.wrapping_sub(u32_BITS) as i32 >> (u32_BITS - 1)) as u32;
            *r_lo = (a_hi.wrapping_shr(sr.wrapping_sub(u32_BITS))                     &
                   ((u32_BITS - 1).wrapping_sub(sr) as i32 >> (u32_BITS - 1)) as u32) |
                   ((a_hi.wrapping_shl(u32_BITS.wrapping_sub(sr))                     |
                   a_lo.wrapping_shr(sr))                                             &
                   (sr.wrapping_sub(u32_BITS) as i32 >> (u32_BITS - 1)) as u32);
        },
        _ => {
            /* K X
             * ---
             * K K
             */
            sr = b_hi.leading_zeros().wrapping_sub(a_hi.leading_zeros());
            /* 0 <= sr <= u32::BITS - 1 or sr large */
            if sr > u32_BITS - 1 {
                if !rem.is_null() {
                    *rem = a;
                }
                return 0;
            } else {
                sr += 1;
                /* 1 <= sr <= u32::BITS */
                /*  q = a << (u64::BITS - sr); */
                *q_lo = 0;
                *q_hi = a_lo << u32_BITS.wrapping_sub(sr);
                /* r = a >> sr;
                 * if (sr < u32::BITS)
                 * {
                 *     r_hi = a_hi >> sr;
                 *     r_lo = (a_hi << (u32::BITS - sr)) | (a_lo >> sr);
                 * }
                 * else
                 * {
                 *     r_hi = 0;
                 *     r_lo = a_hi;
                 * }
                 */
                *r_hi = (a_hi >> sr) &
                      (sr.wrapping_sub(u32_BITS) as i32 >> (u32_BITS - 1)) as u32;
                *r_lo = (a_hi << u32_BITS.wrapping_sub(sr)) |
                      ((a_lo >> sr)                         &
                      (sr.wrapping_sub(u32_BITS) as i32 >> (u32_BITS - 1)) as u32);
            }
        }
    }

    /* Not a special case
     * q and r are initialized with:
     * q = a << (u64::BITS - sr);
     * r = a >> sr;
     * 1 <= sr <= u64::BITS - 1
     */
    let mut carry: u32 = 0;
    for _ in 0 .. sr {
        /* r = (r << 1) | carry */
        *r_hi = (*r_hi << 1) | (*r_lo >> (u32_BITS - 1));
        *r_lo = (*r_lo << 1) | (*q_hi >> (u32_BITS - 1));
        *q_hi = (*q_hi << 1) | (*q_lo >> (u32_BITS - 1));
        *q_lo = (*q_lo << 1) | carry;
        /* carry = 0;
         * if (r >= d)
         * {
         *      r -= d;
         *      carry = 1;
         * }
         */
        let s = b.wrapping_sub(*r_all).wrapping_sub(1) as i64 >> (u64_BITS - 1);
        carry = (s & 1) as u32;
        *r_all -= b & s as u64;
    }
    *q_all = (*q_all << 1) | (carry as u64);
    if !rem.is_null() {
        *rem = *r_all;
    }
    *q_all
}

/* Returns: a / b */
#[no_mangle]
pub unsafe fn __udivdi3(a: u64, b: u64) -> u64 {
    __udivmoddi4(a, b, ptr::null_mut())
}

/* Returns: a % b */
#[no_mangle]
pub unsafe fn __umoddi3(a: u64, b: u64) -> u64 {
    let mut r: u64 = mem::uninitialized();
    __udivmoddi4(a, b, &mut r);
    r
}

/* Returns: a / b */
#[no_mangle]
pub unsafe fn __divdi3(mut a: i64, mut b: i64) -> i64 {
    const BITS_IN_DWORD_M1: i64 = (i64::BYTES * i8::BITS) as i64 - 1;
    let mut s_a = a >> BITS_IN_DWORD_M1;           /* s_a = a < 0 ? -1 : 0 */
    let s_b = b >> BITS_IN_DWORD_M1;               /* s_b = b < 0 ? -1 : 0 */
    a = (a ^ s_a) - s_a;                           /* negate if s_a == -1 */
    b = (b ^ s_b) - s_b;                           /* negate if s_b == -1 */
    s_a ^= s_b;                                    /*sign of quotient */
    (__udivmoddi4(a as u64, b as u64, ptr::null_mut()) as i64 ^ s_a) - s_a  /* negate if s_a == -1 */
}

/* Returns: a % b */
#[no_mangle]
pub unsafe fn __moddi3(mut a: i64, mut b: i64) -> i64 {
    const BITS_IN_DWORD_M1: i64 = (i64::BYTES * i8::BITS) as i64 - 1;
    let mut s = b >> BITS_IN_DWORD_M1;  /* s = b < 0 ? -1 : 0 */
    b = (b ^ s) - s;                    /* negate if s == -1 */
    s = a >> BITS_IN_DWORD_M1;          /* s = a < 0 ? -1 : 0 */
    a = (a ^ s) - s;                    /* negate if s == -1 */

    let mut r: u64 = mem::uninitialized();
    __udivmoddi4(a as u64, b as u64, &mut r);
    return (r as i64 ^ s) - s;          /* negate if s == -1 */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divmod64() {
        unsafe {
            /* 0 X
             * ---
             * 0 X
             */
            assert_eq!(__udivdi3(15, 5), 3);
            assert_eq!(__udivdi3(14, 4), 3);
            assert_eq!(__udivdi3(13, 3), 4);

            assert_eq!(__umoddi3(15, 5), 0);
            assert_eq!(__umoddi3(14, 4), 2);
            assert_eq!(__umoddi3(13, 3), 1);

            /* 0 X
             * ---
             * K X
             */
            assert_eq!(__udivdi3(15, (3 << 32) + 3), 0);
            assert_eq!(__umoddi3(15, (3 << 32) + 3), 15);

            /* K X
             * ---
             * 0 0
             */
            // see below

            /* K 0
             * ---
             * K 0
             */
            assert_eq!(__udivdi3(15 << 32, 3 << 32), 5);
            assert_eq!(__umoddi3(15 << 32, 3 << 32), 0);

            /* K K
             * ---
             * K 0
             */
            // b is a power of 2
            assert_eq!(__udivdi3((15 << 32) + 15, 2 << 32), 7);
            assert_eq!(__umoddi3((15 << 32) + 15, 2 << 32), 4294967311);

            /* K K
             * ---
             * K 0
             */
            assert_eq!(__udivdi3((15 << 32) + 15, 3 << 32), 5);
            assert_eq!(__umoddi3((15 << 32) + 15, 3 << 32), 15);

            /* K X
             * ---
             * 0 K
             */
            assert_eq!(__udivdi3((15 << 32) + 15, 3), 21474836485);
            assert_eq!(__umoddi3((15 << 32) + 15, 3), 0);

            /* K X
             * ---
             * K K
             */
            assert_eq!(__udivdi3((15 << 32) + 15, (3 << 32) + 3), 5);
            assert_eq!(__umoddi3((15 << 32) + 15, (3 << 32) + 3), 0);

            /* i64 */
            assert_eq!(__divdi3((-15 << 32) + 15, 2 << 32), -7);
            assert_eq!(__moddi3((-15 << 32) + 15, 2 << 32), -4294967281);
        }
    }

    #[test]
    #[should_panic(expected = "attempted to divide by zero")]
    fn test_divmod64_panic() {
        unsafe {
            /* K X
             * ---
             * 0 0
             */
            __udivdi3((15 << 32) + 15, 0);
        }
    }
}

