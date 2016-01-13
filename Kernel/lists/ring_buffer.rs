use core::ptr;

pub struct RingBuffer<'a, T: 'a> {
    data: &'a mut [T],
    read: usize,
    write: usize
}

impl<'a, T> RingBuffer<'a, T> {
    // data.len() should be power of two
    pub const fn new(data: &'a mut [T]) -> RingBuffer<T> {
        RingBuffer {
            data: data,
            read: 0,
            write: 0
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.read = 0;
        self.write = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.write >= self.read {
            self.write - self.read
        } else {
            self.write + self.data.len() - self.read
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool { self.read == self.write }

    pub fn push(&mut self, value: T) {
        let cur = self.write;
        self.write = self.step(self.write);
        if self.is_empty() {
            self.read = self.step(self.read);
        }

        unsafe {
            ptr::write(&mut self.data[cur], value);
        }
    }

    pub fn try_push(&mut self, value: T) -> bool {
        let cur = self.write;
        self.write = self.step(self.write);
        if self.is_empty() {
            false
        } else {
            unsafe {
                ptr::write(&mut self.data[cur], value);
            }
            true
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let cur = self.read;
            self.read = self.step(self.read);

            unsafe {
                Some(ptr::read(&mut self.data[cur]))
            }
        }
    }

    pub fn peek(&self, offset: usize) -> Option<&T> {
        let len = self.len();
        if len > 0 && offset < len {
            let index = self.read + offset;
            Some(&self.data[index & (self.data.len() - 1)])
        } else {
            None
        }
    }

    #[inline]
    fn step(&self, val: usize) -> usize {
        (val + 1) & (self.data.len() - 1)
    }
}

