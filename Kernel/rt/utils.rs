#[inline(always)]
pub const fn align_up(n: usize, align: usize) -> usize {
    n + (align - 1) & !(align - 1)
}

#[inline(always)]
pub fn align_up_ptr<T>(p: *const T, align: usize) -> *const T {
    align_up(p as usize, align) as *const T
}

#[inline(always)]
pub fn align_up_mut_ptr<T>(p: *mut T, align: usize) -> *mut T {
    align_up(p as usize, align) as *mut T
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align() {
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(2, 4), 4);
        assert_eq!(align_up(3, 4), 4);
        assert_eq!(align_up(4, 4), 4);
    }
}

