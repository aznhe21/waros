#![allow(dead_code)]

use core::ptr;
use core::iter;
use core::marker::PhantomData;

#[allow(raw_pointer_derive)]
#[derive(Clone, Copy)]
pub struct LinkedList<T: LinkedNode<T>> {
    len: usize,
    head: *mut T,
    tail: *mut T
}

pub trait LinkedNode<T> {
    fn get_prev(&self) -> *mut T;
    fn set_prev(&mut self, node: *mut T);

    fn get_next(&self) -> *mut T;
    fn set_next(&mut self, *mut T);
}

pub struct IterMut<'a, T: 'a + LinkedNode<T>> {
    len: usize,
    head: *mut T,
    tail: *mut T,
    _marker: PhantomData<&'a mut T>
}

impl<T: LinkedNode<T>> LinkedList<T> {
    #[inline]
    pub const fn new() -> LinkedList<T> {
        LinkedList {
            len: 0,
            head: 0 as *mut T,
            tail: 0 as *mut T
        }
    }

    #[inline]
    pub fn iter_mut<'a>(&self) -> IterMut<'a, T> {
        IterMut {
            len: self.len,
            head: self.head,
            tail: self.tail,
            _marker: PhantomData
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    #[inline]
    pub fn front_mut<'a>(&mut self) -> Option<&'a mut T> {
        unsafe { self.front_ptr().as_mut() }
    }

    #[inline]
    pub fn front_ptr(&mut self) -> *mut T {
        self.head
    }

    #[inline]
    pub fn back_mut<'a>(&mut self) -> Option<&'a mut T> {
        unsafe { self.back_ptr().as_mut() }
    }

    #[inline]
    pub fn back_ptr(&mut self) -> *mut T {
        self.tail
    }

    pub fn push_front(&mut self, node: *mut T) {
        assert!(!node.is_null());

        unsafe {
            (*node).set_next(self.head);
            (*node).set_prev(ptr::null_mut());

            if !self.head.is_null() {
                (*self.head).set_prev(node);
                self.len += 1;
            } else {
                self.tail = node;
                self.len = 1;
            }

            self.head = node;
        }
    }

    pub fn push_back(&mut self, node: *mut T) {
        assert!(!node.is_null());

        unsafe {
            (*node).set_next(ptr::null_mut());
            (*node).set_prev(self.tail);

            if !self.tail.is_null() {
                (*self.tail).set_next(node);
                self.len += 1;
            } else {
                self.head = node;
                self.len = 1;
            }

            self.tail = node;
        }
    }

    pub fn insert(&mut self, node: *mut T, before: *mut T) {
        assert!(!node.is_null());

        unsafe {
            let prev = (*before).get_prev();
            (*node).set_next(before);
            (*node).set_prev(prev);
            (*before).set_prev(node);
            if !prev.is_null() {
                (*prev).set_next(node);
            }

            if self.head == before {
                self.head = node;
            }

            self.len += 1;
        }
    }

    #[inline]
    pub fn pop_front<'a>(&mut self) -> Option<&'a mut T> {
        unsafe { self.pop_front_ptr().as_mut() }
    }

    pub fn pop_front_ptr(&mut self) -> *mut T {
        if self.head.is_null() {
            ptr::null_mut()
        } else {
            let ret = self.head;

            if self.head == self.tail {
                self.head = ptr::null_mut();
                self.tail = ptr::null_mut();
                self.len = 0;
            } else {
                unsafe {
                    self.head = (*ret).get_next();
                    (*self.head).set_prev(ptr::null_mut());
                }

                self.len -= 1;
            }

            ret
        }
    }

    #[inline]
    pub fn pop_back<'a>(&mut self) -> Option<&'a mut T> {
        unsafe { self.pop_back_ptr().as_mut() }
    }

    pub fn pop_back_ptr(&mut self) -> *mut T {
        if self.tail.is_null() {
            ptr::null_mut()
        } else {
            let ret = self.tail;

            if self.head == self.tail {
                self.head = ptr::null_mut();
                self.tail = ptr::null_mut();
                self.len = 0;
            } else {
                unsafe {
                    self.tail = (*ret).get_prev();
                    (*self.tail).set_next(ptr::null_mut());
                }

                self.len -= 1;
            }

            ret
        }
    }

    pub fn remove(&mut self, node: *mut T) {
        assert!(!node.is_null());
        debug_assert!(self.contains(node));

        unsafe {
            let next = (*node).get_next();
            let prev = (*node).get_prev();

            if !next.is_null() {
                (*next).set_prev(prev);
            }
            if !prev.is_null() {
                (*prev).set_next(next);
            }

            if self.head == node {
                self.head = next;
            }

            if self.tail == node {
                self.tail = prev;
            }

            self.len -= 1;
        }
    }

    pub fn contains(&self, node: *mut T) -> bool {
        unsafe {
            let mut ptr = self.head;
            while !ptr.is_null() {
                if ptr == node {
                    return true;
                }
                ptr = (*ptr).get_next();
            }
            false
        }
    }
}

impl<'a, T: 'a + LinkedNode<T>> iter::FromIterator<&'a mut T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item=&'a mut T>>(iterable: I) -> LinkedList<T> {
        let mut iter = iterable.into_iter();
        if let Some(head) = iter.next() {
            let head_ptr = head as *mut T;

            head.set_prev(ptr::null_mut());
            let (len, tail) = iter.fold((1, head), |(len, prev), next| {
                prev.set_next(next);
                next.set_prev(prev);
                (len + 1, next)
            });
            tail.set_next(ptr::null_mut());

            LinkedList {
                len: len,
                head: head_ptr,
                tail: tail as *mut T
            }
        } else {
            LinkedList::new()
        }
    }
}

impl<'a, T: 'a + LinkedNode<T>> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        if self.len == 0 {
            None
        } else {
            let ret = unsafe { &mut *self.head };
            self.head = ret.get_next();
            self.len -= 1;
            Some(ret)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: 'a + LinkedNode<T>> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut T> {
        if self.len == 0 {
            None
        } else {
            let ret = unsafe { &mut *self.tail };
            self.tail = ret.get_prev();
            self.len -= 1;
            Some(ret)
        }
    }
}

impl<'a, T: 'a + LinkedNode<T>> ExactSizeIterator for IterMut<'a, T> {
}

impl<'a, T: 'a + LinkedNode<T>> Clone for IterMut<'a, T> {
    fn clone(&self) -> IterMut<'a, T> {
        IterMut { len: self.len, head: self.head, tail: self.tail, _marker: self._marker }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Debug)]
    struct Node {
        value: usize,
        prev: *mut Node,
        next: *mut Node
    }
    impl Node {
        const fn new(value: usize) -> Node {
            Node {
                value: value,
                prev: 0 as *mut Node,
                next: 0 as *mut Node
            }
        }
    }
    impl LinkedNode<Node> for Node {
            fn get_prev(&self) -> *mut Node { self.prev }
            fn set_prev(&mut self, node: *mut Node) { self.prev = node; }
            fn get_next(&self) -> *mut Node { self.next }
            fn set_next(&mut self, node: *mut Node) { self.next = node; }
    }

    #[test]
    fn test_linked_list() {
        let mut data: [Node; 4] = [
            Node::new(0xFEDC),
            Node::new(0xBA98),
            Node::new(0x7654),
            Node::new(0x3210),
        ];
        let mut list = LinkedList::new();
        unsafe {
            let ptr = data.as_mut_ptr();
            list.push_back(&mut *ptr.offset(2));
            list.push_front(&mut *ptr.offset(1));
            list.push_front(&mut *ptr.offset(0));
            list.push_back(&mut *ptr.offset(3));
        }

        for i in 0..4 {
            assert_eq!(list.iter().nth(i).unwrap(), &data[i]);
            assert_eq!(list.iter().rev().nth(3 - i).unwrap(), &data[i]);
        }

        unsafe {
            let ptr = data.as_mut_ptr();
            list.remove(&mut *ptr.offset(2));
            list.remove(&mut *ptr.offset(0));
        }
        assert_eq!(list.iter().nth(1).unwrap(), &data[3]);
        assert_eq!(list.iter().rev().nth(1).unwrap(), &data[1]);
    }
}

