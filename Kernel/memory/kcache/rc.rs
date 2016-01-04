use super::{RefCount, KCacheAllocator};
use core::mem;
use core::intrinsics;
use core::ptr::Shared;
use core::fmt::{self, Display, Debug};
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};

pub struct KCRc<T: RefCount> {
    allocator: KCacheAllocator<T>,
    ptr: Shared<T>
}

impl<T: RefCount> KCRc<T> {
    pub fn new(allocator: KCacheAllocator<T>, x: T) -> Option<KCRc<T>> {
        unsafe {
            allocator.allocate(x).map(|ptr| {
                (**ptr).add_ref();
                KCRc {
                    allocator: allocator,
                    ptr: Shared::new(*ptr)
                }
            })
        }
    }

    #[inline(always)]
    pub unsafe fn into_raw(this: KCRc<T>) -> *mut T {
        let ptr = *this.ptr;
        mem::forget(this);
        ptr
    }

    #[inline(always)]
    pub unsafe fn from_raw(allocator: KCacheAllocator<T>, raw: *mut T) -> KCRc<T> {
        KCRc {
            allocator: allocator,
            ptr: Shared::new(raw)
        }
    }
}

unsafe impl<T: RefCount + Send> Send for KCRc<T> { }
unsafe impl<T: RefCount + Sync> Sync for KCRc<T> { }

impl<T: RefCount + PartialEq> PartialEq for KCRc<T> {
    #[inline(always)]
    fn eq(&self, other: &KCRc<T>) -> bool {
        PartialEq::eq(&**self, &**other)
    }

    #[inline(always)]
    fn ne(&self, other: &KCRc<T>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: RefCount + PartialOrd> PartialOrd for KCRc<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &KCRc<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }

    #[inline(always)]
    fn lt(&self, other: &KCRc<T>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }

    #[inline(always)]
    fn le(&self, other: &KCRc<T>) -> bool {
        PartialOrd::le(&**self, &**other)
    }

    #[inline(always)]
    fn ge(&self, other: &KCRc<T>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }

    #[inline(always)]
    fn gt(&self, other: &KCRc<T>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: RefCount + Ord> Ord for KCRc<T> {
    #[inline(always)]
    fn cmp(&self, other: &KCRc<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: RefCount + Eq> Eq for KCRc<T> {}

impl<T: RefCount + Hash> Hash for KCRc<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: RefCount + Display> Display for KCRc<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T: RefCount + Debug> Debug for KCRc<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: RefCount> fmt::Pointer for KCRc<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ptr: *const T = &**self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

impl<T: RefCount + Iterator> Iterator for KCRc<T> {
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

impl<T: RefCount + DoubleEndedIterator> DoubleEndedIterator for KCRc<T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<T::Item> {
        (**self).next_back()
    }
}

impl<T: RefCount + ExactSizeIterator> ExactSizeIterator for KCRc<T> { }

impl<T: RefCount> Deref for KCRc<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe {
            &**self.ptr
        }
    }
}

impl<T: RefCount> DerefMut for KCRc<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut **self.ptr
        }
    }
}

impl<T: RefCount> Clone for KCRc<T> {
    fn clone(&self) -> KCRc<T> {
        unsafe {
            (**self.ptr).add_ref();
        }
        KCRc {
            allocator: self.allocator.clone(),
            ptr: self.ptr
        }
    }
}

impl<T: RefCount> Drop for KCRc<T> {
    fn drop(&mut self) {
        unsafe {
            if (**self.ptr).rel_ref() == 1 {
                intrinsics::drop_in_place(*self.ptr);
                self.allocator.free(*self.ptr);
            }
        }
    }
}

