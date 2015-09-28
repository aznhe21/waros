use core::mem;
use core::slice;

pub trait SliceHelper {
    unsafe fn translate<T>(&self) -> &[T];
    unsafe fn translate_mut<T>(&mut self) -> &mut [T];
}

impl<S> SliceHelper for [S] {
    #[inline]
    unsafe fn translate<T>(&self) -> &[T] {
        let ptr = self.as_ptr() as *const T;
        let len = self.len() * mem::size_of::<S>() / mem::size_of::<T>();
        slice::from_raw_parts(ptr, len)
    }

    #[inline]
    unsafe fn translate_mut<T>(&mut self) -> &mut [T] {
        let ptr = self.as_mut_ptr() as *mut T;
        let len = self.len() * mem::size_of::<S>() / mem::size_of::<T>();
        slice::from_raw_parts_mut(ptr, len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate() {
        let buf = [0x00u8, 0xFF, 0x11, 0xEE];
        let slice = unsafe { buf.translate::<u16>() };
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0], 0xFF00);
        assert_eq!(slice[1], 0xEE11);
    }

    #[test]
    fn test_translate_mut() {
        let mut buf = [0u8; 4];
        {
            let slice = unsafe { buf.translate_mut::<u32>() };
            assert_eq!(slice.len(), 1);
            slice[0] = 0xEE11FF00;
        }

        assert_eq!(buf, [0x00, 0xFF, 0x11, 0xEE]);
    }
}

