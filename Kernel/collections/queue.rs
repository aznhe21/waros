#![allow(dead_code)]

use prelude::*;
use core::mem;

pub struct Queue<'a, T: 'a> {
    #[doc(hidden)]
    pub data: &'a mut [T],
    #[doc(hidden)]
    pub read: usize,
    #[doc(hidden)]
    pub write: usize
}

impl<'a, T> Queue<'a, T> {
    pub fn new(data: &'a mut [T]) -> Queue<T> {
        Queue {
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
        if self.read == self.write {
            self.read = self.step(self.read);
            log!("Queue overflowed");
        }

        self.data[cur] = value;
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

