// TODO: DList by dlang

use memory::kcache::{KCRc, RefCount};
use core::ptr::Shared;
use core::iter::FromIterator;

/// 同一性の比較及び内部に保持する`LinkedNode`の取り出しに使うトレイト。
pub trait Linker: Clone {
    /// この`Linker`が保持するノードの型。
    type Node: LinkedNode;

    /// 同一性を比較する。
    ///
    /// 値による比較ではなく、参照での比較を行う。
    fn is_same(&self, other: &Self) -> bool;

    /// 保持するノードを参照で返す。
    fn as_ref(&self) -> &Self::Node;

    /// 保持するノードをミュータブル参照で返す。
    fn as_mut(&mut self) -> &mut Self::Node;
}

#[inline(always)]
fn linker_is_same_option<This: Linker>(this: Option<&This>, other: Option<&This>) -> bool {
    match (this, other) {
        (Some(this), Some(other)) => this.is_same(other),
        _ => false
    }
}

/// ノードそのものを表すトレイト。
///
/// 前のノードや次のノードの参照に使う。
pub trait LinkedNode {
    /// この`LinkedNode`を保持する、`Shared`などの型。
    type Linker: Linker;

    /// 前のノードを返す。
    fn get_prev(&self) -> Option<Self::Linker>;

    /// 前のノードを設定する。
    fn set_prev(&mut self, Option<Self::Linker>);

    /// 次のノードを返す。
    fn get_next(&self) -> Option<Self::Linker>;

    /// 次のノードを設定する。
    fn set_next(&mut self, Option<Self::Linker>);
}

/// メモリ確保を伴わない双方向リンクリスト。
pub struct LinkedList<T: LinkedNode> {
    len: usize,
    head: Option<T::Linker>,
    tail: Option<T::Linker>
}

/// `LinkedList`に`IntoIterator`を実装するイテレータ。
pub struct IntoIter<T: LinkedNode> {
    list: LinkedList<T>
}

/// `LinkedList`のイテレータ。
pub struct Iter<T: LinkedNode> {
    len: usize,
    head: Option<T::Linker>,
    tail: Option<T::Linker>
}

impl<T: LinkedNode> LinkedList<T> where T::Linker: Linker<Node=T> {
    /// 空の`LinkedList`を作る。
    #[inline]
    pub const fn new() -> LinkedList<T> {
        LinkedList {
            len: 0,
            head: None,
            tail: None
        }
    }

    /// 前進イテレータを提供する。
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            len: self.len,
            head: self.head.clone(),
            tail: self.tail.clone()
        }
    }

    /// このリストの長さを返す。
    ///
    /// この操作はO(1)で完了する。
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// このリストが空ならば`true`を返す。
    ///
    /// この操作はO(1)で完了する。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// このリストを空にする。
    ///
    /// この操作はO(1)で完了する。
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// 先頭の要素の`Linker`を返す。リストが空ならば`None`を返す。
    #[inline]
    pub fn front(&self) -> Option<T::Linker> {
        self.head.clone()
    }

    /// 末尾の要素の`Linker`を返す。リストが空ならば`None`を返す。
    #[inline]
    pub fn back(&self) -> Option<T::Linker> {
        self.tail.clone()
    }

    /// リストの先頭に要素を挿入する。
    ///
    /// この操作はO(1)で完了する。ただし、デバッグ時はこの限りでない。
    pub fn push_front(&mut self, mut node: T::Linker) {
        debug_assert!(!self.contains(&node));

        node.as_mut().set_next(self.head.clone());
        node.as_mut().set_prev(None);

        match self.head {
            Some(ref mut head) => {
                head.as_mut().set_prev(Some(node.clone()));
                self.len += 1;
            },
            None => {
                self.tail = Some(node.clone());
                self.len = 1;
            }
        }

        self.head = Some(node);
    }

    /// リストの末尾に要素を挿入する。
    ///
    /// この操作はO(1)で完了する。ただし、デバッグ時はこの限りでない。
    pub fn push_back(&mut self, mut node: T::Linker) {
        debug_assert!(!self.contains(&node));

        node.as_mut().set_next(None);
        node.as_mut().set_prev(self.tail.clone());

        match self.tail {
            Some(ref mut tail) => {
                tail.as_mut().set_next(Some(node.clone()));
                self.len += 1;
            }
            None => {
                self.head = Some(node.clone());
                self.len = 1;
            }
        }

        self.tail = Some(node);
    }

    /// `before`の後ろに要素を挿入する。
    ///
    /// この操作はO(1)で完了する。ただし、デバッグ時はこの限りでない。
    pub fn insert(&mut self, mut node: T::Linker, mut before: T::Linker) {
        debug_assert!(!self.contains(&node));
        debug_assert!(self.contains(&before));

        // nodeに前後を設定
        let mut prev = before.as_mut().get_prev();
        node.as_mut().set_next(Some(before.clone()));
        node.as_mut().set_prev(prev.clone());
        if let Some(ref mut prev) = prev {
            prev.as_mut().set_next(Some(node.clone()));
        }
        before.as_mut().set_prev(Some(node.clone()));

        // 先頭を更新
        if linker_is_same_option(self.head.as_ref(), Some(&before)) {
            self.head = Some(node);
        }

        self.len += 1;
    }

    /// 先頭の要素を削除して返す。リストが空ならば`None`を返す。
    ///
    /// この操作はO(1)で完了する。
    pub fn pop_front(&mut self) -> Option<T::Linker> {
        debug_assert!(self.head.is_some() || (self.tail.is_none() && self.len == 0));

        match self.head.take() {
            Some(val) => {
                if linker_is_same_option(self.tail.as_ref(), Some(&val)) {
                    self.tail = None;
                    self.len = 0;
                } else {
                    self.head = val.as_ref().get_next();
                    self.head.as_mut().unwrap().as_mut().set_prev(None);
                    self.len -= 1;
                }

                Some(val)
            },
            None => None
        }
    }

    /// 末尾の要素を削除して返す。リストが空ならば`None`を返す。
    ///
    /// この操作はO(1)で完了する。
    pub fn pop_back(&mut self) -> Option<T::Linker> {
        debug_assert!(self.tail.is_some() || (self.head.is_none() && self.len == 0));

        match self.tail.take() {
            Some(val) => {
                if linker_is_same_option(self.head.as_ref(), Some(&val)) {
                    self.head = None;
                    self.len = 0;
                } else {
                    self.tail = val.as_ref().get_prev();
                    self.tail.as_mut().unwrap().as_mut().set_next(None);
                    self.len -= 1;
                }

                Some(val)
            },
            None => None
        }
    }

    /// 指定した要素と同じ要素を削除する。
    ///
    /// この操作はO(1)で完了する。ただし、デバッグ時はこの限りでない。
    pub fn remove(&mut self, node: &T::Linker) {
        debug_assert!(self.contains(node));

        let mut next = node.as_ref().get_next();
        let mut prev = node.as_ref().get_prev();

        // nodeの前後を更新
        if let Some(ref mut next) = next {
            next.as_mut().set_prev(prev.clone());
        }
        if let Some(ref mut prev) = prev {
            prev.as_mut().set_next(next.clone());
        }

        // 先頭/末尾を更新
        if linker_is_same_option(self.head.as_ref(), Some(&node)) {
            self.head = next.clone();
        }

        if linker_is_same_option(self.tail.as_ref(), Some(&node)) {
            self.tail = prev.clone();
        }

        self.len -= 1;

        debug_assert!(self.head.is_some() || self.len == 0);
    }

    /// 指定した要素がこのリストに含まれているならば`true`を返す。
    pub fn contains(&self, node: &T::Linker) -> bool {
        let mut ptr = self.head.clone();
        while let Some(ref val) = ptr.take() {
            if val.is_same(node) {
                return true;
            }
            ptr = val.as_ref().get_next();
        }
        false
    }
}

impl<T: LinkedNode> FromIterator<T::Linker> for LinkedList<T> where T::Linker: Linker<Node=T> {
    fn from_iter<I: IntoIterator<Item=T::Linker>>(iterable: I) -> LinkedList<T> {
        let mut iter = iterable.into_iter();
        if let Some(mut head) = iter.next() {
            head.as_mut().set_prev(None);
            let (len, mut tail) = iter.fold((1, head.clone()), |(len, mut prev), mut next| {
                prev.as_mut().set_next(Some(next.clone()));
                next.as_mut().set_prev(Some(prev.clone()));
                (len + 1, next)
            });
            tail.as_mut().set_next(None);

            LinkedList {
                len: len,
                head: Some(head),
                tail: Some(tail)
            }
        } else {
            LinkedList::new()
        }
    }
}

impl<T: LinkedNode> IntoIterator for LinkedList<T> where T::Linker: Linker<Node=T> {
    type Item = T::Linker;
    type IntoIter = IntoIter<T>;

    #[inline(always)]
    fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }
}

impl<'a, T: LinkedNode> IntoIterator for &'a LinkedList<T> where T::Linker: Linker<Node=T> {
    type Item = T::Linker;
    type IntoIter = Iter<T>;

    #[inline(always)]
    fn into_iter(self) -> Iter<T> {
        self.iter()
    }
}

impl<'a, T: LinkedNode> IntoIterator for &'a mut LinkedList<T> where T::Linker: Linker<Node=T> {
    type Item = T::Linker;
    type IntoIter = Iter<T>;

    #[inline(always)]
    fn into_iter(self) -> Iter<T> {
        self.iter()
    }
}

impl<T: LinkedNode> Iterator for IntoIter<T> where T::Linker: Linker<Node=T> {
    type Item = T::Linker;

    #[inline(always)]
    fn next(&mut self) -> Option<T::Linker> {
        self.list.pop_front()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T: LinkedNode> DoubleEndedIterator for IntoIter<T> where T::Linker: Linker<Node=T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<T::Linker> {
        self.list.pop_back()
    }
}

impl<T: LinkedNode> ExactSizeIterator for IntoIter<T> where T::Linker: Linker<Node=T> {
}

impl<T: LinkedNode> Iterator for Iter<T> where T::Linker: Linker<Node=T> {
    type Item = T::Linker;

    fn next(&mut self) -> Option<T::Linker> {
        if self.len == 0 {
            None
        } else {
            let ret = self.head.take().unwrap();
            self.head = ret.as_ref().get_next();
            self.len -= 1;
            Some(ret)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T: LinkedNode> DoubleEndedIterator for Iter<T> where T::Linker: Linker<Node=T> {
    fn next_back(&mut self) -> Option<T::Linker> {
        if self.len == 0 {
            None
        } else {
            let ret = self.tail.take().unwrap();
            self.tail = ret.as_ref().get_prev();
            self.len -= 1;
            Some(ret)
        }
    }
}

impl<T: LinkedNode> ExactSizeIterator for Iter<T> where T::Linker: Linker<Node=T> {
}

impl<T: LinkedNode> Clone for Iter<T> where T::Linker: Linker<Node=T> {
    #[inline(always)]
    fn clone(&self) -> Iter<T> {
        Iter { len: self.len, head: self.head.clone(), tail: self.tail.clone() }
    }
}

impl<T: LinkedNode> Linker for Shared<T> {
    type Node = T;

    #[inline(always)]
    fn is_same(&self, other: &Shared<T>) -> bool {
        **self == **other
    }

    #[inline(always)]
    fn as_ref(&self) -> &T {
        unsafe {
            &***self
        }
    }

    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        unsafe {
            &mut ***self
        }
    }
}

impl<T: LinkedNode + RefCount> Linker for KCRc<T> {
    type Node = T;

    #[inline(always)]
    fn is_same(&self, other: &KCRc<T>) -> bool {
        &**self as *const T == &**other as *const T
    }

    #[inline(always)]
    fn as_ref(&self) -> &T {
        &**self
    }

    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    pub struct RefLinker<'a, T: 'a>(&'a mut T);

    impl<'a, T: 'a + LinkedNode> RefLinker<'a, T> {
        #[inline(always)]
        pub const fn new(x: &'a mut T) -> RefLinker<'a, T> {
            RefLinker(x)
        }
    }

    impl<'a, T: 'a + LinkedNode> PartialEq for RefLinker<'a, T> {
        #[inline(always)]
        fn eq(&self, other: &RefLinker<'a, T>) -> bool {
            self.0 as *const _ == other.0 as *const _
        }
    }

    impl<'a, T: 'a + LinkedNode> Eq for RefLinker<'a, T> { }

    impl<'a, T: 'a + LinkedNode> Clone for RefLinker<'a, T> {
        #[inline(always)]
        fn clone(&self) -> RefLinker<'a, T> {
            unsafe {
                RefLinker(*(&self.0 as *const &'a mut T))
            }
        }
    }

    impl<'a, T: 'a + LinkedNode> ::core::ops::Deref for RefLinker<'a, T> {
        type Target = T;

        #[inline(always)]
        fn deref(&self) -> &T {
            self.0
        }
    }

    impl<'a, T: 'a + LinkedNode> ::core::ops::DerefMut for RefLinker<'a, T> {
        #[inline(always)]
        fn deref_mut(&mut self) -> &mut T {
            self.0
        }
    }

    impl<'a, T: 'a + LinkedNode> Linker for RefLinker<'a, T> {
        type Node = T;

        #[inline(always)]
        fn is_same(&self, other: &RefLinker<'a, T>) -> bool {
            self.0 as *const _ == other.0 as *const _
        }

        #[inline(always)]
        fn as_ref(&self) -> &T {
            self.0
        }

        #[inline(always)]
        fn as_mut(&mut self) -> &mut T {
            self.0
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    struct Node<'a> {
        value: usize,
        prev: Option<RefLinker<'a, Node<'a>>>,
        next: Option<RefLinker<'a, Node<'a>>>
    }
    impl<'a> Node<'a> {
        const fn new(value: usize) -> Node<'a> {
            Node {
                value: value,
                prev: None,
                next: None
            }
        }
    }
    impl<'a> LinkedNode for Node<'a> {
        type Linker = RefLinker<'a, Node<'a>>;
        fn get_prev(&self) -> Option<RefLinker<'a, Node<'a>>> { self.prev.clone() }
        fn set_prev(&mut self, node: Option<RefLinker<'a, Node<'a>>>) { self.prev = node; }
        fn get_next(&self) -> Option<RefLinker<'a, Node<'a>>> { self.next.clone() }
        fn set_next(&mut self, node: Option<RefLinker<'a, Node<'a>>>) { self.next = node; }
    }

    #[test]
    fn test_linked_list() {
        let mut data: [Node; 4] = [
            Node::new(0xFEDC),
            Node::new(0xBA98),
            Node::new(0x7654),
            Node::new(0x3210),
        ];
        let mut list = LinkedList::<Node>::new();

        unsafe {
            let ptr = data.as_mut_ptr();
            list.push_back(RefLinker::new(&mut *ptr.offset(2)));
            list.push_front(RefLinker::new(&mut *ptr.offset(1)));
            list.push_front(RefLinker::new(&mut *ptr.offset(0)));
            list.push_back(RefLinker::new(&mut *ptr.offset(3)));
        }

        assert_eq!(list.front(), Some(RefLinker::new(&mut data[0])));
        assert_eq!(list.back(), Some(RefLinker::new(&mut data[3])));

        assert_eq!(list.len(), 4);
        assert_eq!(list.iter().count(), 4);
        assert_eq!(list.iter().rev().count(), 4);

        for i in 0..4 {
            assert_eq!(*list.iter().nth(i).unwrap(), data[i]);
            assert_eq!(*list.iter().rev().nth(3 - i).unwrap(), data[i]);
        }

        unsafe {
            let ptr = data.as_mut_ptr();
            list.remove(&RefLinker::new(&mut *ptr.offset(2)));
            list.remove(&RefLinker::new(&mut *ptr.offset(0)));
        }

        assert_eq!(*list.iter().nth(1).unwrap(), data[3]);
        assert_eq!(*list.iter().rev().nth(1).unwrap(), data[1]);
    }
}

