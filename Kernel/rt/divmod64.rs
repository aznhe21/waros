use core::ptr;
use core::mem;
use core::{u32, u64, i8, i64};

/* Effects: if rem != 0, *rem = a % b
 * Returns: a / b
 */
#[no_mangle]
pub unsafe fn __udivmoddi4(a: u64, b: u64, rem: *mut u64) -> u64 {
    let (a_lo, a_hi): (u32, u32) = mem::transmute(a);
    let (b_lo, b_hi): (u32, u32) = mem::transmute(b);

    let (sr, mut r_lo, mut r_hi, mut q_lo, mut q_hi) = match (a_hi, a_lo, b_hi, b_lo) {
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
                let hi = a_hi % b_hi;
                let lo = 0;
                *rem = mem::transmute((lo, hi));
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
                let lo = a_lo;
                let hi = a_hi & (b_hi - 1);
                *rem = mem::transmute((lo, hi));
            }
            return (a_hi >> b_lo.trailing_zeros()) as u64;
        },
        (_, _, _, 0) => {
            /* K K
             * ---
             * K 0
             */
            let mut sr = b_hi.leading_zeros() - a_hi.leading_zeros();
            /* 0 <= sr <= u32::BITS - 2 or sr large */
            if sr > u64::BITS as u32 - 2 {
                if !rem.is_null() {
                    *rem = a;
                }
                return 0;
            } else {
                sr += 1;
                /* 1 <= sr <= u32::BITS - 1 */
                /* q = a << (u64::BITS - sr); */
                let q_lo = 0;
                let q_hi = a_lo << (u32::BITS as u32 - sr);
                /* r = a >> sr; */
                let r_hi = a_hi >> sr;
                let r_lo = (a_hi << (u32::BITS as u32 - sr)) | (a_lo >> sr);
                (sr, r_lo, r_hi, q_lo, q_hi)
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
                let sr = b_lo.trailing_zeros();
                let hi = a_hi >> sr;
                let lo = (a_hi << (u32::BITS as u32 - sr)) | (a_lo >> sr);
                return mem::transmute((lo, hi));
            }
        },
        (_, _, 0, _) => {
            /* K X
             * ---
             * 0 K
             */
            let sr = 1 + u32::BITS as u32 + b_lo.leading_zeros() - a_hi.leading_zeros();
            /* 2 <= sr <= u64::BITS - 1
             * q = a << (u64::BITS - sr);
             * r = a >> sr;
             */
            let (q_lo, q_hi, r_hi, r_lo) = if sr == u32::BITS as u32 {
                (
                    0,
                    a_lo,
                    0,
                    a_hi
                )
            } else if sr < u32::BITS as u32 { // 2 <= sr <= u32::BITS - 1
                (
                    0,
                    a_lo << (u32::BITS as u32 - sr),
                    a_hi >> sr,
                    (a_hi << (u32::BITS as u32 - sr)) | (a_lo >> sr)
                )
            } else { // u32::BITS + 1 <= sr <= u64::BITS - 1
                (
                    a_lo << (u64::BITS as u32 - sr),
                    (a_hi << (u64::BITS as u32 - sr)) | (a_lo >> (sr - u32::BITS as u32)),
                    0,
                    a_hi >> (sr - u32::BITS as u32)
                )
            };
            (sr, r_lo, r_hi, q_lo, q_hi)
        },
        _ => {
            /* K X
             * ---
             * K K
             */
            let mut sr = b_hi.leading_zeros() - a_hi.leading_zeros();
            /* 0 <= sr <= u32::BITS - 1 or sr large */
            if sr > u32::BITS as u32 - 1 {
                if !rem.is_null() {
                    *rem = a;
                }
                return 0;
            } else {
                sr += 1;
                /* 1 <= sr <= u32::BITS */
                /*  q = a << (u64::BITS - sr); */
                let q_lo = 0;
                let (q_hi, r_hi, r_lo) = if sr == u32::BITS as u32 {
                    (
                        a_lo,
                        0,
                        a_hi
                    )
                } else {
                    (
                        a_lo << (u32::BITS as u32 - sr),
                        a_hi >> sr,
                        (a_hi << (u32::BITS as u32 - sr)) | (a_lo >> sr)
                    )
                };
                (sr, r_lo, r_hi, q_lo, q_hi)
            }
        }
    };

    /* Not a special case
     * q and r are initialized with:
     * q = n << (u64::BITS - sr);
     * r = n >> sr;
     * 1 <= sr <= u64::BITS - 1
     */
    let mut carry: u32 = 0;
    for _ in 0 .. sr {
        /* r = (r << 1) | carry */
        r_hi = (r_hi << 1) | (r_lo >> (u32::BITS - 1));
        r_lo = (r_lo << 1) | (q_hi >> (u32::BITS - 1));
        q_hi = (q_hi << 1) | (q_lo >> (u32::BITS - 1));
        q_lo = (q_lo << 1) | carry;
        /* carry = 0;
         * if (r >= d)
         * {
         *      r -= d;
         *      carry = 1;
         * }
         */
        let mut r_all: u64 = mem::transmute((r_lo, r_hi));
        let s = (b as i64 - r_all as i64 - 1) >> (u64::BITS - 1);
        carry = s as u32 & 1;
        r_all -= b & s as u64;
        let (r_lo2, r_hi2) = mem::transmute(r_all);
        r_lo = r_lo2;
        r_hi = r_hi2;
    }
    if !rem.is_null() {
        *rem = mem::transmute((r_lo, r_hi))
    }
    let all: u64 = mem::transmute((q_lo, q_hi));
    (all << 1) | (carry as u64)
}

/* Returns: a / b */
#[no_mangle]
pub unsafe fn __udivdi3(a: u64, b: u64) -> u64 {
    __udivmoddi4(a, b, ptr::null_mut())
}

/* Returns: a % b */
#[no_mangle]
pub unsafe fn __umoddi3(a: u64, b: u64) -> u64 {
    let r: u64 = mem::uninitialized();
    __udivmoddi4(a, b, mem::transmute(&r));
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

    let r: u64 = mem::uninitialized();
    __udivmoddi4(a as u64, b as u64, mem::transmute(&r));
    return (r as i64 ^ s) - s;          /* negate if s == -1 */
}

/*#[cfg(test)]
mod tests {
    #[test]
    fn test_divmod64() {
        /* 0 X
         * ---
         * 0 X
         */
        assert_eq!(__udivdi3(15, 5), 3);
        assert_eq!(__udivdi3(15, 4), 3);
        assert_eq!(__udivdi3(15, 3), 5);

        /* 0 X
         * ---
         * K X
         */
        assert_eq!(__udivdi3(15, 3 << 32 | 3), 0);

        /* K X
         * ---
         * 0 0
         */
        //assert_eq!(__udivdi3(15 << 32 | 15, 0), );

        /* K 0
         * ---
         * K 0
         */
        assert_eq!(__udivdi3(15 << 32, 3 << 32), 5);

        /* K K
         * ---
         * K 0
         */
        // b is a power of 2
        assert_eq!(__udivdi3(15 << 32 | 15, 2 << 32, 7));

        /* K K
         * ---
         * K 0
         */
        assert_eq!(__udivdi3(15 << 32 | 15, 3 << 32), 5);

        /* K X
         * ---
         * 0 K
         */
        assert_eq!(__udivdi3(15 << 32 | 15, 3), 5);

        /* K X
         * ---
         * K K
         */
        assert_eq!(__udivdi3(15 << 32 | 15, 3 << 32 | 3), ?);

        assert_eq!(__umoddi3(15, 5), 0);
        assert_eq!(__umoddi3(15, 4), 3);
        assert_eq!(__umoddi3(15, 3), 0);
    }
}*/

