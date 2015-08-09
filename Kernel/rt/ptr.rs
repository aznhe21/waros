use core::marker::Sized;
use core::intrinsics;

pub trait UOffset<T: ?Sized> {
    unsafe fn uoffset(self, count: usize) -> *const T where T: Sized;
    unsafe fn uoffset_rev(self, count: usize) -> *const T where T: Sized;

    unsafe fn uoffset8(self, count: usize) -> *const T where T: Sized;
    unsafe fn uoffset8_rev(self, count: usize) -> *const T where T: Sized;
}

impl<T: ?Sized> UOffset<T> for *const T {
    #[inline(always)]
    unsafe fn uoffset(self, count: usize) -> *const T where T: Sized {
        intrinsics::overflowing_add(
            self as usize,
            intrinsics::overflowing_mul(count, intrinsics::size_of::<T>())
        ) as *const T
    }

    #[inline(always)]
    unsafe fn uoffset_rev(self, count: usize) -> *const T where T: Sized {
        intrinsics::overflowing_sub(
            self as usize,
            intrinsics::overflowing_mul(count, intrinsics::size_of::<T>())
        ) as *const T
    }

    #[inline(always)]
    unsafe fn uoffset8(self, count: usize) -> *const T where T: Sized {
        intrinsics::overflowing_add(self as usize, count) as *const T
    }

    #[inline(always)]
    unsafe fn uoffset8_rev(self, count: usize) -> *const T where T: Sized {
        intrinsics::overflowing_sub(self as usize, count) as *const T
    }
}

pub trait UOffsetMut<T: ?Sized> {
    unsafe fn uoffset(self, count: usize) -> *mut T where T: Sized;
    unsafe fn uoffset_rev(self, count: usize) -> *mut T where T: Sized;

    unsafe fn uoffset8(self, count: usize) -> *mut T where T: Sized;
    unsafe fn uoffset8_rev(self, count: usize) -> *mut T where T: Sized;
}

impl<T: ?Sized> UOffsetMut<T> for *mut T {
    #[inline(always)]
    unsafe fn uoffset(self, count: usize) -> *mut T where T: Sized {
        intrinsics::overflowing_add(
            self as usize,
            intrinsics::overflowing_mul(count, intrinsics::size_of::<T>())
        ) as *mut T
    }

    #[inline(always)]
    unsafe fn uoffset_rev(self, count: usize) -> *mut T where T: Sized {
        intrinsics::overflowing_sub(
            self as usize,
            intrinsics::overflowing_mul(count, intrinsics::size_of::<T>())
        ) as *mut T
    }

    #[inline(always)]
    unsafe fn uoffset8(self, count: usize) -> *mut T where T: Sized {
        intrinsics::overflowing_add(self as usize, count) as *mut T
    }

    #[inline(always)]
    unsafe fn uoffset8_rev(self, count: usize) -> *mut T where T: Sized {
        intrinsics::overflowing_sub(self as usize, count) as *mut T
    }
}

