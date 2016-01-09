use core::sync::atomic::{AtomicPtr, Ordering};
use core::ptr;
use alloc::boxed::Box;

struct QueueNode<T> {
    data: T,
    next: AtomicPtr<QueueNode<T>>
}

/// データをアトミックに入出力できるキュー。
pub struct SyncQueue<T> {
    head: AtomicPtr<QueueNode<T>>
}

/// キューに加えたデータを削除するために使われる。
pub struct QueueTicket<T>(*mut QueueNode<T>, *mut QueueNode<T>);

impl<T> QueueNode<T> {
    #[inline(always)]
    fn new(data: T) -> QueueNode<T> {
        QueueNode {
            data: data,
            next: AtomicPtr::new(ptr::null_mut())
        }
    }
}

impl<T> SyncQueue<T> {
    /// 空の`SyncQueue`を作る。
    #[inline(always)]
    pub const fn new() -> SyncQueue<T> {
        SyncQueue {
            head: AtomicPtr::new(ptr::null_mut())
        }
    }

    /// キューが空ならば`true`を返す。
    #[inline]
    pub fn is_empty(&self) -> bool {
        !self.head.load(Ordering::Relaxed).is_null()
    }

    /// キューの先頭のデータを取り出す。
    #[inline]
    pub fn front(&self) -> Option<&T> {
        let node = self.head.load(Ordering::Relaxed);
        if node.is_null() {
            None
        } else {
            Some(unsafe { &(*node).data })
        }
    }

    /// キューにデータをアトミックに加える。
    /// この関数の戻り値を使うと、キューの途中にあるデータを削除できる。
    pub fn push(&mut self, data: T) -> QueueTicket<T> {
        unsafe {
            let node = Box::into_raw(Box::new(QueueNode::new(data)));
            let mut tail = self.head.compare_and_swap(ptr::null_mut(), node, Ordering::SeqCst);
            if tail.is_null() {
                // 先頭の要素を置き換えられた
                return QueueTicket(ptr::null_mut(), node);
            }

            // 最後の要素として新しい要素を挿入する
            while !(*tail).next.compare_and_swap(ptr::null_mut(), node, Ordering::SeqCst).is_null() {
                tail = (*tail).next.load(Ordering::Acquire);
                debug_assert!(!tail.is_null());
            }

            QueueTicket(tail, node)
        }
    }

    /// キューからデータをアトミックに取り出す。
    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            loop {
                let head = self.head.load(Ordering::Acquire);
                if head.is_null() {
                    // キューが空
                    return None;
                }

                let next = (*head).next.load(Ordering::Acquire);
                if self.head.compare_and_swap(head, next, Ordering::SeqCst) == head {
                    // 先頭の要素を２番目の要素と置き換えられた
                    let node = Box::from_raw(head);
                    return Some(node.data);
                }
            }
        }
    }

    /// `push`によって返された`QueueTicket`を用いてキューからデータをアトミックに削除する。
    /// 既に`pop`されたキューに対してこの操作を行うことはできない。
    pub fn remove(&mut self, ticket: QueueTicket<T>) {
        unsafe {
            let QueueTicket(prev, node) = ticket;
            let next = (*node).next.load(Ordering::Acquire);

            loop {
                let head = if prev.is_null() || self.head.load(Ordering::Acquire) == node {
                    // 先頭の要素として挿入されたか、または既に直前の要素がpopされた
                    &self.head
                } else {
                    // まだprevはpopされていない
                    &(*prev).next
                };
                if head.compare_and_swap(node, next, Ordering::SeqCst) == node {
                    // 先頭の要素あるいは直前の要素の次の要素を直後の要素と置き換えられた
                    Box::from_raw(node);
                    return;
                }
            }
        }
    }

    /// `push`によって返された`QueueTicket`を用いてキューにデータが含まれているかアトミックに判定する。
    pub fn contains(&self, ticket: &QueueTicket<T>) -> bool {
        unsafe {
            let &QueueTicket(_, node) = ticket;

            let mut top = self.head.load(Ordering::Acquire);
            while !top.is_null() {
                if top == node {
                    return true;
                }
                top = (*top).next.load(Ordering::Acquire);
            }

            return false;
        }
    }
}

