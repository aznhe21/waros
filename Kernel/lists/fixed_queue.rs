use core::mem;

pub struct FixedQueue<'a, T: 'a> {
    data: &'a mut [T],
    read: usize,
    write: usize
}

impl<'a, T> FixedQueue<'a, T> {
    pub const fn new(data: &'a mut [T]) -> FixedQueue<T> {
        FixedQueue {
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

    #[inline]
    pub fn push(&mut self, value: T) {
        *self.emplace_back() = value;
    }

    pub fn emplace_back(&mut self) -> &mut T {
        let cur = self.write;
        self.write = self.step(self.write);
        if self.read == self.write {
            self.read = self.step(self.read);
            log!("FixedQueue overflowed");
        }

        &mut self.data[cur]
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.read != self.write {
            let cur = self.read;
            self.read = self.step(self.read);

            let mut ret: T = unsafe { mem::uninitialized() };
            mem::swap(&mut self.data[cur], &mut ret);
            Some(ret)
        } else {
            None
        }
    }

    pub fn peek(&self, offset: usize) -> Option<&T> {
        let len = self.len();
        if len > 0 && offset < len {
            let index = self.read + offset;
            if index < self.data.len() {
                Some(&self.data[index])
            } else {
                Some(&self.data[index - self.data.len()])
            }
        } else {
            None
        }
    }

    #[inline]
    fn step(&self, val: usize) -> usize {
        if val + 1 < self.data.len() {
            val + 1
        } else {
            0
        }
    }
}

