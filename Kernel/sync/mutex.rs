use sync::SyncQueue;
use task::{self, Task};
use rt::IntBlocker;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// `lock`メソッドのエラーを表す列挙型。
pub enum LockError {
    /// `Mutex`が破棄された。
    Destroyed
}

/// `try_lock_for`メソッドのエラーを表す列挙型。
pub enum TryLockForError {
    /// `Mutex`が破棄された。
    Destroyed,
    /// 指定した時間内にロックを取得できなかった。
    WouldBlock
}

/// `try_lock`メソッドのエラーを表す列挙型。
pub enum TryLockError {
    /// 即座にロックを取得できなかった。
    WouldBlock
}

/// 単純なミューテックス。
/// staticに宣言できる。
pub struct PrimitiveMutex {
    locked: AtomicBool,
    queue: UnsafeCell<SyncQueue<Task>>
}

/// データへの排他的なアクセスを提供するミューテックス。
pub struct Mutex<T: ?Sized> {
    inner: PrimitiveMutex,
    data: UnsafeCell<T>
}

/// スコープの間だけロックを実現するRAIIの実装。
/// この構造体がdropされるとミューテックスは自動的にアンロックされる。
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a PrimitiveMutex,
    data: &'a UnsafeCell<T>
}

/// `Mutex`の`lock`メソッドに特殊化された`Result`。
pub type LockResult<Guard> = Result<Guard, LockError>;

/// `Mutex`の`try_lock_for`メソッドに特殊化された`Result`。
pub type TryLockForResult<Guard> = Result<Guard, TryLockForError>;

/// `Mutex`の`try_lock`メソッドに特殊化された`Result`。
pub type TryLockResult<Guard> = Result<Guard, TryLockError>;

unsafe impl Send for PrimitiveMutex { }
unsafe impl Sync for PrimitiveMutex { }

impl PrimitiveMutex {
    /// 未ロック状態で利用可能なミューテックスを作る。
    #[inline(always)]
    pub const fn new() -> PrimitiveMutex {
        PrimitiveMutex {
            locked: AtomicBool::new(false),
            queue: UnsafeCell::new(SyncQueue::new())
        }
    }

    /// ミューテックスをロックする。
    /// 既にロックされている場合はロックが解除されるまでタスクをブロックする。
    pub fn lock(&self) -> LockResult<()> {
        if self.locked.swap(true, Ordering::Acquire) {
            let this_task = task::this();

            let q = unsafe { &mut *self.queue.get() };
            let ticket = q.push(this_task.clone());

            // Wait until unlocked
            loop {
                let _ = task::this().suspend();
                let _blocker = IntBlocker::new();

                if !self.locked.swap(true, Ordering::SeqCst) {
                    if q.front() == Some(&this_task) {
                        break;
                    }

                    self.locked.store(false, Ordering::Release);
                } else {
                    if !q.contains(&ticket) {
                        task::yield_now();// Back to a task which is destroying a mutex
                        return Err(LockError::Destroyed);
                    }
                }
            }

            q.pop();
        }

        Ok(())
    }

    /// ミューテックスをロックする。
    /// 既にロックされている場合は`duration`で指定した時間が経過するかロックが解除されるまでタスクをブロックする。
    pub fn try_lock_for(&self, duration: usize) -> TryLockForResult<()> {
        if self.locked.swap(true, Ordering::Acquire) {
            let this_task = task::this();

            let q = unsafe { &mut *self.queue.get() };
            let ticket = q.push(this_task.clone());

            task::sleep(duration);
            let _blocker = IntBlocker::new();

            if self.locked.swap(true, Ordering::SeqCst) {
                if !q.contains(&ticket) {
                    task::yield_now();// Back to a task which is destroying a mutex
                    return Err(TryLockForError::Destroyed);
                }

                q.remove(ticket);
                return Err(TryLockForError::WouldBlock);
            }

            if q.front() != Some(&task::this()) {
                self.locked.store(false, Ordering::Release);
                q.remove(ticket);
                return Err(TryLockForError::WouldBlock);
            }

            q.pop();
        }

        Ok(())
    }

    /// ミューテックスのロックを試みる。
    /// 既にロックされている場合は即座に`false`を返す。
    /// ロックできた場合は`true`を返す。
    pub fn try_lock(&self) -> bool {
        !self.locked.swap(true, Ordering::SeqCst)
    }

    /// ロックされたミューテックスを解除する。
    pub fn unlock(&self) {
        if self.locked.swap(false, Ordering::SeqCst) {
            let q = unsafe { &mut *self.queue.get() };
            loop {
                // タスクの起動に失敗したら次のタスクでやり直し
                match q.front().map(Task::resume) {
                    Some(Ok(_)) | None => break,
                    Some(Err(_)) => {}
                }
                q.pop();
            }
        }
    }

    /// ミューテックスを破棄する。
    pub unsafe fn destroy(&mut self) {
        self.locked.store(true, Ordering::Release);
        let q = &mut *self.queue.get();
        while let Some(task) = q.pop() {
            let _ = task.resume_later();
            let _ = task::run_now(&task);
        }
        task::yield_now();
    }
}

impl<T> Mutex<T> {
    /// 未ロック状態で利用可能なミューテックスを作る。
    #[inline(always)]
    pub fn new(value: T) -> Mutex<T> {
        Mutex {
            inner: PrimitiveMutex::new(),
            data: UnsafeCell::new(value)
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// ミューテックスをロックする。
    /// 既にロックされている場合はロックが解除されるまでタスクをブロックする。
    pub fn lock(&self) -> LockResult<MutexGuard<T>> {
        try!(self.inner.lock());
        Ok(MutexGuard::new(&self.inner, &self.data))
    }

    /// ミューテックスをロックする。
    /// 既にロックされている場合は`duration`で指定した時間が経過するかロックが解除されるまでタスクをブロックする。
    pub fn try_lock_for(&self, duration: usize) -> TryLockForResult<MutexGuard<T>> {
        try!(self.inner.try_lock_for(duration));
        Ok(MutexGuard::new(&self.inner, &self.data))
    }

    /// ミューテックスのロックを試みる。
    /// 既にロックされている場合は即座に`Err`を返す。
    pub fn try_lock(&self) -> TryLockResult<MutexGuard<T>> {
        if self.inner.try_lock() {
            Ok(MutexGuard::new(&self.inner, &self.data))
        } else {
            Err(TryLockError::WouldBlock)
        }
    }
}

impl<T: ?Sized> Drop for Mutex<T> {
    fn drop(&mut self) {
        unsafe {
            self.inner.destroy();
        }
    }
}

unsafe impl<T: ?Sized> Send for Mutex<T> { }
unsafe impl<T: ?Sized> Sync for Mutex<T> { }

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    #[inline]
    fn new(lock: &'a PrimitiveMutex, data: &'a UnsafeCell<T>) -> MutexGuard<'a, T> {
        MutexGuard {
            lock: lock,
            data: data
        }
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.unlock()
    }
}

