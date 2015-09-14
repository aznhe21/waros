#![allow(dead_code)]

use collections::linked_list::{LinkedList, LinkedNode, IterMut};
use core::cmp::Ordering::{self, Less};

pub struct SortedList<T: LinkedNode<T>> {
    inner_list: LinkedList<T>,
    compare: fn(&T, &T) -> Ordering
}

impl<T: LinkedNode<T>> SortedList<T> {
    #[inline]
    pub const fn new(compare: fn(&T, &T) -> Ordering) -> SortedList<T> {
        SortedList {
            inner_list: LinkedList::new(),
            compare: compare
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.inner_list.iter_mut()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner_list.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner_list.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner_list.clear();
    }

    #[inline]
    pub fn front_mut<'a>(&mut self) -> Option<&'a mut T> {
        self.inner_list.front_mut()
    }

    #[inline]
    pub fn front_ptr(&mut self) -> *mut T {
        self.inner_list.front_ptr()
    }

    #[inline]
    pub fn back_mut<'a>(&mut self) -> Option<&'a mut T> {
        self.inner_list.back_mut()
    }

    #[inline]
    pub fn back_ptr(&mut self) -> *mut T {
        self.inner_list.back_ptr()
    }

    pub fn push(&mut self, node: *mut T) {
        assert!(!node.is_null());

        unsafe {
            let mut cand = self.inner_list.front_ptr();
            // while node >= cand
            while !cand.is_null() && (self.compare)(&*node, &*cand) != Less {
                cand = (*cand).get_next();
            }

            if !cand.is_null() {
                self.inner_list.insert(node, cand)
            } else {
                self.inner_list.push_back(node)
            }
        }
    }

    #[inline]
    pub fn pop_front<'a>(&mut self) -> Option<&'a mut T> {
        self.inner_list.pop_front()
    }

    #[inline]
    pub fn pop_front_ptr(&mut self) -> *mut T {
        self.inner_list.pop_front_ptr()
    }

    #[inline]
    pub fn pop_back<'a>(&mut self) -> Option<&'a mut T> {
        self.inner_list.pop_back()
    }

    #[inline]
    pub fn pop_back_ptr(&mut self) -> *mut T {
        self.inner_list.pop_back_ptr()
    }

    #[inline]
    pub fn remove(&mut self, node: *mut T) {
        self.inner_list.remove(node)
    }

    #[inline]
    pub fn contains(&self, node: *mut T) -> bool {
        self.inner_list.contains(node)
    }
}

