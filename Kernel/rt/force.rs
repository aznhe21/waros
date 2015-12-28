use core::cell::UnsafeCell;
use core::ptr::Shared;
use core::fmt::{self, Display, Debug};
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};

/// You must feel the Force around you. Here, between you, me, the tree, the rock, everywhere!
pub struct Force<T>(UnsafeCell<Option<T>>);
pub struct ForceRef<T>(Shared<T>);

impl<T> Force<T> {
    #[inline(always)]
    pub const fn new() -> Force<T> {
        Force(UnsafeCell::new(None))
    }

    #[inline(always)]
    pub fn setup(&self) -> &mut T {
        unsafe {
            &mut *(self.0.get() as *mut T)
        }
    }

    #[inline(always)]
    pub fn as_ref(&self) -> ForceRef<T> {
        unsafe {
            ForceRef(Shared::new(self.0.get() as *mut T))
        }
    }
}

unsafe impl<T: Send> Send for Force<T> { }
unsafe impl<T: Sync> Sync for Force<T> { }

impl<T: PartialEq> PartialEq for ForceRef<T> {
    #[inline(always)]
    fn eq(&self, other: &ForceRef<T>) -> bool {
        PartialEq::eq(&**self, &**other)
    }

    #[inline(always)]
    fn ne(&self, other: &ForceRef<T>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: PartialOrd> PartialOrd for ForceRef<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &ForceRef<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }

    #[inline(always)]
    fn lt(&self, other: &ForceRef<T>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }

    #[inline(always)]
    fn le(&self, other: &ForceRef<T>) -> bool {
        PartialOrd::le(&**self, &**other)
    }

    #[inline(always)]
    fn ge(&self, other: &ForceRef<T>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }

    #[inline(always)]
    fn gt(&self, other: &ForceRef<T>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: Ord> Ord for ForceRef<T> {
    #[inline(always)]
    fn cmp(&self, other: &ForceRef<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: Eq> Eq for ForceRef<T> { }

impl<T: Hash> Hash for ForceRef<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: Display> Display for ForceRef<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T: Debug> Debug for ForceRef<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T> fmt::Pointer for ForceRef<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ptr: *const T = &**self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

impl<T: Iterator> Iterator for ForceRef<T> {
    type Item = T::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<T::Item> {
        (**self).next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

impl<T: DoubleEndedIterator> DoubleEndedIterator for ForceRef<T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<T::Item> {
        (**self).next_back()
    }
}

impl<T: ExactSizeIterator> ExactSizeIterator for ForceRef<T> { }

impl<T> Deref for ForceRef<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe {
            &**self.0
        }
    }
}

impl<T> DerefMut for ForceRef<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut **self.0
        }
    }
}

