use core::marker::Sized;
use core::intrinsics;

pub trait UOffset<T: Sized> {
    unsafe fn uoffset(self, count: usize) -> *const T;
    unsafe fn uoffset_rev(self, count: usize) -> *const T;
}

impl<T: Sized> UOffset<T> for *const T {
    #[inline(always)]
    unsafe fn uoffset(self, count: usize) -> *const T {
        intrinsics::arith_offset(self, count as isize)
    }

    #[inline(always)]
    unsafe fn uoffset_rev(self, count: usize) -> *const T {
        intrinsics::arith_offset(self, -(count as isize))
    }
}

pub trait UOffsetMut<T: Sized> {
    unsafe fn uoffset(self, count: usize) -> *mut T;
    unsafe fn uoffset_rev(self, count: usize) -> *mut T;
}

impl<T: Sized> UOffsetMut<T> for *mut T {
    #[inline(always)]
    unsafe fn uoffset(self, count: usize) -> *mut T {
        intrinsics::arith_offset(self as *const T, count as isize) as *mut T
    }

    #[inline(always)]
    unsafe fn uoffset_rev(self, count: usize) -> *mut T {
        intrinsics::arith_offset(self as *const T, -(count as isize)) as *mut T
    }
}

