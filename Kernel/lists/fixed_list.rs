use core::mem;
use core::ptr;
use core::ops;
use core::fmt;
use core::marker::PhantomData;

pub struct FixedList<'a, T: 'a> {
    data: &'a mut [T],
    len: usize,
}

pub struct Iter<'a, T: 'a> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>
}

pub struct IterMut<'a, T: 'a> {
    ptr: *mut T,
    end: *mut T,
    _marker: PhantomData<&'a mut T>
}

impl<'a, T> FixedList<'a, T> {
    pub const fn new(data: &'a mut [T]) -> FixedList<'a, T> {
        FixedList {
            data: data,
            len: 0
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            ptr: self.data.as_ptr(),
            end: unsafe { self.data.as_ptr().offset(self.len as isize) },
            _marker: PhantomData
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            ptr: self.data.as_mut_ptr(),
            end: unsafe { self.data.as_mut_ptr().offset(self.len as isize) },
            _marker: PhantomData
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len != 0
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        for i in 0 .. self.len {
            unsafe { ptr::drop_in_place(&mut self.data[i]) };
        }

        self.len = 0;
    }

    #[inline]
    pub fn get(&mut self, i: usize) -> Option<&mut T> {
        if i >= self.len {
            None
        } else {
            Some(&mut self.data[i])
        }
    }

    #[inline]
    pub fn swap(&mut self, i: usize, j: usize) {
        self.data.swap(i, j);
    }

    #[inline]
    pub fn front(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            Some(&self.data[0])
        }
    }

    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            Some(&mut self.data[0])
        }
    }

    #[inline]
    pub fn back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            Some(&self.data[self.len - 1])
        }
    }

    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            Some(&mut self.data[self.len - 1])
        }
    }

    pub fn push_back(&mut self, value: T) {
        assert!(self.len + 1 <= self.data.len(), "FixedList overflowed");

        self.data[self.len] = value;
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len = self.len - 1;
            let mut ret = unsafe { mem::uninitialized() };
            mem::swap(&mut self.data[self.len], &mut ret);
            Some(ret)
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len && index < self.data.len());

        unsafe {
            let ptr = self.data.as_mut_ptr().offset(index as isize);
            ptr::copy(ptr, ptr.offset(1), self.len - index);
            ptr::write(ptr, value);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len);
        self.len -= 1;

        unsafe {
            let ptr = self.data.as_mut_ptr().offset(index as isize);
            let ret = ptr::read(ptr);
            ptr::copy(ptr.offset(1), ptr, self.len - index);
            ret
        }
    }

    pub fn remove_for(&mut self, index: usize, len: usize) {
        if len > 0 {
            assert!(index + len <= self.len);
            self.len -= len;

            unsafe {
                let ptr = self.data.as_mut_ptr().offset(index as isize);
                ptr::copy(ptr.offset(len as isize), ptr, self.len - (index + len - 1));
            }
        }
    }
}

/*impl<'a, T> Drop for FixedList<'a, T> {
    fn drop(&mut self) {
        self.clear();
    }
}*/

impl<'a, T> ops::Index<usize> for FixedList<'a, T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        assert!(i < self.len);
        &self.data[i]
    }
}

impl<'a, T> ops::IndexMut<usize> for FixedList<'a, T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        assert!(i < self.len);
        &mut self.data[i]
    }
}


impl<'a, T> IntoIterator for &'a FixedList<'a, T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut FixedList<'a, T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<'a, A> Extend<A> for FixedList<'a, A> {
    fn extend<T: IntoIterator<Item=A>>(&mut self, iter: T) {
        for val in iter {
            self.push_back(val);
        }
    }
}

impl<'a, T: 'a + Copy> Extend<&'a T> for FixedList<'a, T> {
    fn extend<I: IntoIterator<Item=&'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for FixedList<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "["));

        for (i, e) in self.iter().enumerate() {
            if i != 0 { try!(write!(f, ", ")); }
            try!(write!(f, "{:?}", *e));
        }

        write!(f, "]")
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        if self.ptr == self.end {
            None
        } else {
            unsafe {
                let old = self.ptr;
                self.ptr = self.ptr.offset(1);
                Some(&*old)
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.ptr as usize - self.end as usize) / mem::size_of::<T>();
        (size, Some(size))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Iter<'a, T> { Iter { ptr: self.ptr, end: self.end, _marker: self._marker } }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        if self.ptr == self.end {
            None
        } else {
            unsafe {
                let old = self.ptr;
                self.ptr = self.ptr.offset(1);
                Some(&mut *old)
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.ptr as usize - self.end as usize) / mem::size_of::<T>();
        (size, Some(size))
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
}

impl<'a, T> Clone for IterMut<'a, T> {
    fn clone(&self) -> IterMut<'a, T> { IterMut { ptr: self.ptr, end: self.end, _marker: self._marker } }
}

