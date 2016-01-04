use super::KCacheAllocator;
use memory;
use core::mem;
use core::intrinsics;
use core::ptr::Unique;
use core::fmt::{self, Display, Debug};
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};

pub struct KCBox<T> {
    allocator: KCacheAllocator<T>,
    ptr: Unique<T>
}

impl<T> KCBox<T> {
    pub fn new(allocator: KCacheAllocator<T>, x: T) -> Option<KCBox<T>> {
        allocator.allocate(x).map(|ptr| KCBox {
            allocator: allocator,
            ptr: ptr
        })
    }

    #[inline(always)]
    pub unsafe fn into_raw(this: KCBox<T>) -> *mut T {
        let ptr = *this.ptr;
        mem::forget(this);
        ptr
    }

    #[inline(always)]
    pub unsafe fn from_raw(allocator: KCacheAllocator<T>, raw: *mut T) -> KCBox<T> {
        KCBox {
            allocator: allocator,
            ptr: Unique::new(raw)
        }
    }
}

impl<T: PartialEq> PartialEq for KCBox<T> {
    #[inline(always)]
    fn eq(&self, other: &KCBox<T>) -> bool {
        PartialEq::eq(&**self, &**other)
    }

    #[inline(always)]
    fn ne(&self, other: &KCBox<T>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: PartialOrd> PartialOrd for KCBox<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &KCBox<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }

    #[inline(always)]
    fn lt(&self, other: &KCBox<T>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }

    #[inline(always)]
    fn le(&self, other: &KCBox<T>) -> bool {
        PartialOrd::le(&**self, &**other)
    }

    #[inline(always)]
    fn ge(&self, other: &KCBox<T>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }

    #[inline(always)]
    fn gt(&self, other: &KCBox<T>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: Ord> Ord for KCBox<T> {
    #[inline(always)]
    fn cmp(&self, other: &KCBox<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: Eq> Eq for KCBox<T> {}

impl<T: Hash> Hash for KCBox<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: Display> Display for KCBox<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T: Debug> Debug for KCBox<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T> fmt::Pointer for KCBox<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ptr: *const T = &**self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

impl<T: Iterator> Iterator for KCBox<T> {
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

impl<T: DoubleEndedIterator> DoubleEndedIterator for KCBox<T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<T::Item> {
        (**self).next_back()
    }
}

impl<T: ExactSizeIterator> ExactSizeIterator for KCBox<T> { }

impl<T> Deref for KCBox<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe {
            self.ptr.get()
        }
    }
}

impl<T> DerefMut for KCBox<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            self.ptr.get_mut()
        }
    }
}

impl<T: Clone> Clone for KCBox<T> {
    fn clone(&self) -> KCBox<T> {
        memory::check_oom_opt(KCBox::new(self.allocator.clone(), (**self).clone()))
    }

    #[inline(always)]
    fn clone_from(&mut self, source: &KCBox<T>) {
        (**self).clone_from(&**source);
    }
}

impl<T> Drop for KCBox<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            intrinsics::drop_in_place(*self.ptr);
            self.allocator.free(*self.ptr);
        }
    }
}
