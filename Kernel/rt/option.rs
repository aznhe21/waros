use rt::ptr::UOffsetMut;
use core::option::Option::{self, Some, None};
use core::mem;
use core::intrinsics;

pub trait UnsafeOption<T> {
    unsafe fn into_some<'a>(&'a mut self) -> &'a mut T;
    unsafe fn be_some(self) -> T;
}

impl<T> UnsafeOption<T> for Option<T> {
    unsafe fn into_some<'a>(&'a mut self) -> &'a mut T {
        if mem::size_of::<T>() != mem::size_of::<Option<T>>() {
            let opt_ptr = self as *mut _;
            match mem::align_of::<T>() {
                1 => *(opt_ptr as *mut u8)  = 1,
                2 => *(opt_ptr as *mut u16) = 1,
                4 => *(opt_ptr as *mut u32) = 1,
                8 => *(opt_ptr as *mut u64) = 1,
                _ => panic!("Unknown align")
            }
            &mut *((opt_ptr as *mut u8).uoffset(mem::align_of::<T>()) as *mut T)
        } else {
            mem::transmute(self)
        }
    }

    unsafe fn be_some(self) -> T {
        match self {
            Some(val) => val,
            None => intrinsics::unreachable()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_some() {
        unsafe {
            let mut opt: Option<u32> = None;
            assert!(opt.is_none());

            assert!(!opt.into_some().is_none());
        }

        unsafe {
            let mut opt: Option<&'static str> = None;
            assert!(opt.is_none());

            assert!(opt.into_some().is_none());

            *opt.into_some() = "";
            assert!(!opt.is_none());
        }
    }

    #[test]
    fn test_be_some() {
        unsafe {
            let opt = Some(1);
            assert_eq!(opt.be_some(), 1);
        }
    }
}

