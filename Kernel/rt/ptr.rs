use core::marker::Sized;

pub trait UOffset<T: ?Sized> {
    unsafe fn uoffset(self, count: usize) -> *mut T where T: Sized;
}

impl<T: ?Sized> UOffset<T> for *mut T {
    #[inline]
    unsafe fn uoffset(self, count: usize) -> *mut T where T: Sized {
        (self as usize + count) as *mut T
    }
}


