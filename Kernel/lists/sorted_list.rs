use super::linked_list::{Linker, LinkedNode, DList, Iter};
use core::cmp::Ordering::{self, Less};

pub struct SortedList<T: LinkedNode> {
    inner_list: DList<T>,
    compare: fn(&T, &T) -> Ordering
}

impl<T: LinkedNode> SortedList<T> where T::Linker: Linker<Node=T> {
    #[inline]
    pub fn new(compare: fn(&T, &T) -> Ordering) -> SortedList<T> {
        SortedList {
            inner_list: DList::new(),
            compare: compare
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<T> {
        self.inner_list.iter()
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
    pub fn front(&mut self) -> Option<T::Linker> {
        self.inner_list.front()
    }

    #[inline]
    pub fn back(&mut self) -> Option<T::Linker> {
        self.inner_list.back()
    }

    pub fn push(&mut self, node: T::Linker) {
        let mut next = self.inner_list.front();
        while let Some(cand) = next.take() {
            // if node < cand
            if (self.compare)(node.as_ref(), cand.as_ref()) == Less {
                self.inner_list.insert(node, cand);
                return;
            }

            next = cand.as_ref().get_next();
        }

        self.inner_list.push_back(node);
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<T::Linker> {
        self.inner_list.pop_front()
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<T::Linker> {
        self.inner_list.pop_back()
    }

    #[inline]
    pub fn remove(&mut self, node: &T::Linker) {
        self.inner_list.remove(node)
    }

    #[inline]
    pub fn contains(&self, node: &T::Linker) -> bool {
        self.inner_list.contains(node)
    }
}

