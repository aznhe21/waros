use core::num::One;
use core::ops::{Add, Sub, Not, BitAnd};

#[inline(always)]
pub fn align_up<T: Copy + One + Add<Output=T> + Sub<Output=T> + Not<Output=T> + BitAnd<Output=T>>(n: T, align: T) -> T {
    n + (align - T::one()) & !(align - T::one())
}

#[inline(always)]
pub fn align_down<T: One + Sub<Output=T> + Not<Output=T> + BitAnd<Output=T>>(n: T, align: T) -> T {
    n & !(align - T::one())
}

#[inline(always)]
pub fn align_up_ptr<T>(p: *const T, align: usize) -> *const T {
    align_up(p as usize, align) as *const T
}

#[inline(always)]
pub fn align_down_ptr<T>(p: *const T, align: usize) -> *const T {
    align_down(p as usize, align) as *const T
}

#[inline(always)]
pub fn align_up_mut_ptr<T>(p: *mut T, align: usize) -> *mut T {
    align_up(p as usize, align) as *mut T
}

#[inline(always)]
pub fn align_down_mut_ptr<T>(p: *mut T, align: usize) -> *mut T {
    align_down(p as usize, align) as *mut T
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up_down() {
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(2, 4), 4);
        assert_eq!(align_up(3, 4), 4);
        assert_eq!(align_up(4, 4), 4);

        assert_eq!(align_down(1, 4), 0);
        assert_eq!(align_down(2, 4), 0);
        assert_eq!(align_down(3, 4), 0);
        assert_eq!(align_down(4, 4), 4);
    }
}

