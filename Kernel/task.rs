use rt::{Force, ForceRef, IntBlocker};
use arch;
use arch::interrupt;
use arch::task::TaskEntity;
use lists::{LinkedNode, DList};
use memory;
use memory::kcache::{KCacheAllocator, KCBox};
use timer;
use core::result;
use core::mem;
use core::usize;
use core::ptr::{self, Shared};
use core::sync::atomic::{Ordering, AtomicUsize};
use alloc::boxed::{Box, FnBox};

/*
pub const TASK_SWITCH_INTERVAL: usize = ...;

struct TaskEntity {
    ...
}

impl TaskEntity {
    pub fn new() -> TaskEntity {
        ...
    }

    pub fn setup(&mut self, entry: extern "C" fn(usize), arg: usize, return_to: fn() -> !) {
        ...
    }

    pub fn setup_primary(&mut self) {
        ...
    }

    pub fn terminate(&mut self) {
        ...
    }
}

/// Switch to the next task
pub unsafe fn switch(cur_task: &mut TaskEntity, next_task: &mut TaskEntity) { ... }

/// Switch the task due to it is terminated
pub unsafe fn leap(next_task: &mut TaskEntity) -> ! { ... }
*/

#[allow(non_upper_case_globals)]
static task_counter: AtomicUsize = AtomicUsize::new(1);

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Error {
    /// タスクが無効。
    InvalidTask,
    /// タスクが実行中。
    InRunning,
    /// タスクの状態が不正。
    InvalidState
}

pub type Result<T> = result::Result<T, Error>;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum State {
    Runnable,
    Suspended,
    Free
}

#[derive(PartialEq, Eq, PartialOrd, Ord,Debug, Clone, Copy)]
#[repr(u8)]
pub enum Priority {
    Critical = 4,
    High     = 3,
    Middle   = 2,
    Low      = 1,
    Idle     = 0
}

const PRIORITY_LEN: usize = 5;

impl Priority {
    #[inline]
    fn from_u8(num: u8) -> Priority {
        debug_assert!((num as usize) < PRIORITY_LEN);
        unsafe { mem::transmute(num) }
    }
}

pub struct TaskData {
    pub id: usize,
    pub timer: timer::Timer,
    pub state: State,
    pub priority: Priority,
    pub entity: TaskEntity,
    prev: Option<Shared<TaskData>>,
    next: Option<Shared<TaskData>>
}

impl_linked_node!(Shared<TaskData> { prev: prev, next: next });

impl TaskData {
    #[inline]
    fn new() -> TaskData {
        TaskData {
            id: usize::MAX,
            timer: timer::Timer::with_callback(TaskManager::resume_by_timer),
            state: State::Free,
            priority: Task::DEFAULT_PRIORITY,
            entity: TaskEntity::new(),
            prev: None,
            next: None
        }
    }

    #[inline]
    fn setup(&mut self, id: usize, entry: extern "C" fn(usize), arg: usize, return_to: fn() -> !) {
        self.id = id;
        self.state = State::Runnable;
        self.priority = Task::DEFAULT_PRIORITY;
        self.entity.setup(entry, arg, return_to);
    }

    #[inline]
    fn setup_primary(&mut self) {
        self.id = 0;
        self.state = State::Runnable;
        self.entity.setup_primary();
    }

    fn terminate(&mut self) {
        self.entity.terminate();
        self.timer.clear();
    }
}

#[derive(Clone)]
pub struct Task {
    id: usize,
    ptr: Shared<TaskData>
}

impl Task {
    pub const DEFAULT_PRIORITY: Priority = Priority::Middle;

    #[inline(always)]
    fn new(ptr: Shared<TaskData>) -> Task {
        unsafe {
            Task {
                id: (**ptr).id,
                ptr: ptr
            }
        }
    }

    #[inline]
    pub fn this() -> Task {
        manager().running_task.clone()
    }

    #[inline(always)]
    pub fn id(&self) -> usize {
        self.id
    }

    #[inline(always)]
    fn data(&self) -> &mut TaskData {
        unsafe {
            &mut **self.ptr
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        manager().task_is_valid(self.id())
    }

    #[inline]
    pub fn priority(&self) -> Result<Priority> {
        let _blocker = IntBlocker::new();

        if !self.is_valid() {
            return Err(Error::InvalidTask)
        }

        Ok(self.data().priority)
    }

    #[inline]
    pub fn is_running(&self) -> bool {
        &manager().running_task == self
    }

    #[inline]
    pub fn is_primary(&self) -> bool {
        &manager().primary_task == self
    }

    #[inline]
    pub fn terminate(&self) -> Result<()> {
        manager().terminate(self)
    }

    #[inline(always)]
    pub fn set_priority(&self, priority: Priority) -> Result<()> {
        manager().set_priority(self, priority)
    }

    #[inline(always)]
    pub fn suspend(&self) -> Result<()> {
        manager().suspend(self)
    }

    #[inline(always)]
    pub fn resume(&self) -> Result<()> {
        manager().resume(self, true)
    }

    #[inline(always)]
    pub fn resume_later(&self) -> Result<()> {
        manager().resume(self, false)
    }
}

impl PartialEq for Task {
    #[inline(always)]
    fn eq(&self, other: &Task) -> bool {
        self.id == other.id
    }
}

impl Eq for Task { }

struct TaskManager {
    runnable_tasks: [DList<TaskData>; PRIORITY_LEN],
    suspended_tasks: DList<TaskData>,
    free_tasks: DList<TaskData>,
    running_task: Task,
    current_priority: Priority,
    primary_task: Task,
    timer: timer::UnmanagedTimer,
    kcache: KCacheAllocator<TaskData>
}

unsafe impl Send for TaskManager { }
unsafe impl Sync for TaskManager { }

impl TaskManager {
    #[inline(always)]
    fn init(&mut self) {
        unsafe {
            let _blocker = IntBlocker::new();

            let kcache = memory::check_oom_opt(KCacheAllocator::new("Task", mem::align_of::<TaskData>(), None));
            let mut primary_box = memory::check_oom_opt(KCBox::new(kcache.clone(), TaskData::new()));
            primary_box.setup_primary();

            let primary_task = Task::new(Shared::new(KCBox::into_raw(primary_box)));
            ptr::write(self, TaskManager {
                runnable_tasks: mem::uninitialized(),
                suspended_tasks: DList::new(),
                free_tasks: DList::new(),
                running_task: primary_task.clone(),
                current_priority: Task::DEFAULT_PRIORITY,
                primary_task: primary_task.clone(),
                timer: timer::UnmanagedTimer::with_callback(TaskManager::switch_by_timer),
                kcache: kcache
            });

            for list in self.runnable_tasks.iter_mut() {
                *list = DList::new();
            }

            self.push_task(primary_task.ptr);

            // CPU返還タスク
            let r = self.add(yield_task, 0).set_priority(Priority::Idle);
            debug_assert!(r.is_ok());

            self.reset_timer();
        }
    }

    fn task_is_valid(&self, id: usize) -> bool {
        let list_has_task = |list: &DList<TaskData>| -> bool {
            unsafe {
                list.iter().any(|data| (**data).id == id)
            }
        };

        self.runnable_tasks.iter().rev().any(&list_has_task) || list_has_task(&self.suspended_tasks)
    }

    // 実行可能状態タスクのある最も高い優先度を返す。
    fn highest_priority(&self) -> Priority {
        let (priority, _) = self.runnable_tasks.iter()
            .enumerate()
            .rev()
            .find(|&(_, tasks)| !tasks.is_empty())
            .unwrap();
        Priority::from_u8(priority as u8)
    }

    // 指定したタスク以外で、実行可能状態タスクのある最も高い優先度を返す。
    fn highest_priority_without(&self, task: &Task) -> Priority {
        let (priority, _) = self.runnable_tasks.iter()
            .enumerate()
            .rev()
            // !tasks.is_empty() && tasks != [task]
            .find(|&(_, tasks)| match tasks.len() {
                0 => false,
                1 => tasks.front().map_or(ptr::null_mut(), |p| *p) != *task.ptr,
                _ => true
            })
            .unwrap();
        Priority::from_u8(priority as u8)
    }

    #[inline]
    fn current_tasks(&self) -> &DList<TaskData> {
        &self.runnable_tasks[self.current_priority as usize]
    }

    #[inline]
    fn push_task(&mut self, data: Shared<TaskData>) {
        unsafe {
            self.runnable_tasks[(**data).priority as usize].push_back(data);
        }
    }

    #[inline]
    fn remove_task(&mut self, data: Shared<TaskData>) {
        unsafe {
            self.runnable_tasks[(**data).priority as usize].remove(&data);
        }
    }

    fn add(&mut self, entry: extern "C" fn(usize), arg: usize) -> Task {
        let _blocker = IntBlocker::new();

        unsafe {
            let data = self.free_tasks.pop_front().unwrap_or_else(|| {
                let b = memory::check_oom_opt(KCBox::new(self.kcache.clone(), TaskData::new()));
                Shared::new(KCBox::into_raw(b))
            });
            (**data).setup(task_counter.fetch_add(1, Ordering::SeqCst), entry, arg, task_terminated);
            self.push_task(data);

            Task::new(data)
        }
    }

    #[inline]
    fn reset_timer(&mut self) {
        self.timer.reset(arch::task::TASK_SWITCH_INTERVAL);
    }

    #[inline]
    fn is_switch_needed(&self) -> bool {
        self.highest_priority() > self.current_priority
    }

    #[inline]
    fn can_switch(&self) -> bool {
        self.current_tasks().len() != 1 || self.is_switch_needed()
    }

    fn switch_by_timer(_: timer::TimerId) {
        let mut man = manager();

        if man.can_switch() {
            man.switch_to_next();
        } else {
            man.reset_timer();
        }
    }

    fn yield_now(&mut self) {
        if self.can_switch() {
            self.switch_to_next();
        } else {
            let state = interrupt::start();
            interrupt::wait();
            interrupt::restore(state);
        }
    }

    /// スイッチした場合`true`を返す。
    fn switch_if_needed(&mut self) -> bool {
        let _blocker = IntBlocker::new();

        if self.is_switch_needed() {
            self.switch_to_next();
            true
        } else {
            false
        }
    }

    // 内部変数を次のタスクに移行し、次のタスクを返す
    #[inline]
    fn forward_task(&mut self) -> Task {
        let highest_priority = self.highest_priority_without(&self.running_task);
        let next = Task::new(if self.current_priority != highest_priority {
            self.current_priority = highest_priority;
            self.current_tasks().front().unwrap()
        } else {
            // 次のタスクか最初のタスク
            LinkedNode::get_next(self.running_task.data()).unwrap_or_else(|| self.current_tasks().front().unwrap())
        });

        debug_assert!(self.running_task != next);
        self.running_task = next;

        self.running_task.clone()
    }

    fn switch_to_next(&mut self) {
        self.reset_timer();

        let cur_task = Task::this();
        let next_task = self.forward_task();
        unsafe {
            arch::task::switch(&mut cur_task.data().entity, &mut next_task.data().entity);
        }
    }

    fn terminate_task(&mut self, task: &Task) -> Result<()> {
        let data = task.data();

        match data.state {
            State::Runnable => self.remove_task(task.ptr),
            State::Suspended => self.suspended_tasks.remove(&task.ptr),
            State::Free => return Err(Error::InvalidState)
        }

        // ここでは解放しない
        data.state = State::Free;
        self.free_tasks.push_back(task.ptr);

        data.terminate();

        Ok(())
    }

    fn terminate(&mut self, task: &Task) -> Result<()> {
        let _blocker = IntBlocker::new();

        if !task.is_valid() {
            return Err(Error::InvalidTask)
        }
        if task.is_running() {
            return Err(Error::InRunning)
        }

        let r = self.terminate_task(task);
        debug_assert!(r.is_ok());
        self.switch_if_needed();

        Ok(())
    }

    fn terminated(&mut self) -> ! {
        debug_log!("Task terminated");

        interrupt::disable();
        let cur_task = Task::this();
        let next_task = self.forward_task();
        let r = self.terminate_task(&cur_task);
        debug_assert!(r.is_ok());

        self.reset_timer();
        unsafe {
            arch::task::leap(&mut next_task.data().entity);
        }
    }

    fn set_priority(&mut self, task: &Task, priority: Priority) -> Result<()> {
        let _blocker = IntBlocker::new();

        if !task.is_valid() {
            return Err(Error::InvalidTask)
        }

        let data = task.data();
        match data.state {
            State::Runnable => if data.priority != priority {
                self.remove_task(task.ptr);
                data.priority = priority;
                self.push_task(task.ptr);

                if task.is_running() {
                    self.current_priority = priority;
                }
            },
            State::Suspended => data.priority = priority,
            State::Free => return Err(Error::InvalidState)
        }

        Ok(())
    }

    fn resume_by_timer(timer_id: timer::TimerId) {
        unsafe {
            // sleep中にresumeされる場合がある
            if let Some(data) = manager().suspended_tasks.iter().find(|&data| (**data).timer.id() == timer_id) {
                let r = Task::new(data).resume_later();
                debug_assert!(r.is_ok());
            }
        }
    }

    fn suspend(&mut self, task: &Task) -> Result<()> {
        let _blocker = IntBlocker::new();

        if !task.is_valid() {
            return Err(Error::InvalidTask)
        }

        let data = task.data();
        if data.state != State::Runnable {
            return Err(Error::InvalidState)
        }
        data.state = State::Suspended;

        self.remove_task(task.ptr);
        self.suspended_tasks.push_back(task.ptr);

        if task.is_running() {
            self.switch_to_next();
        }

        Ok(())
    }

    fn resume(&mut self, task: &Task, now: bool) -> Result<()> {
        let _blocker = IntBlocker::new();

        if !task.is_valid() {
            return Err(Error::InvalidTask)
        }

        let data = task.data();
        if data.state != State::Suspended {
            return Err(Error::InvalidState)
        }
        data.state = State::Runnable;

        self.suspended_tasks.remove(&task.ptr);
        self.push_task(task.ptr);

        if now && data.priority > self.current_priority {
            self.switch_to_next();
        }

        Ok(())
    }

    fn sleep(&mut self, duration: usize) {
        let task = Task::this();
        task.data().timer.reset(duration);
        assert!(task.suspend().is_ok());
    }
}

fn task_terminated() -> ! {
    manager().terminated();
}

extern "C" fn yield_task(_: usize) {
    loop {
        interrupt::wait();
    }
}

static MANAGER: Force<TaskManager> = Force::new();

#[inline]
pub fn init() {
    MANAGER.setup().init();
}

#[inline(always)]
fn manager() -> ForceRef<TaskManager> {
    MANAGER.as_ref()
}

#[inline(always)]
pub fn add(entry: extern "C" fn(usize), arg: usize) -> Task {
    manager().add(entry, arg)
}

pub fn spawn<F: FnOnce() + Send + 'static>(f: F) -> Task
{
    let _blocker = IntBlocker::new();

    let main = move || {
        let r = Task::this().set_priority(Task::DEFAULT_PRIORITY);
        debug_assert!(r.is_ok());
        manager().switch_if_needed();// Release the execution right

        // TODO: catch exceptions
        f();
    };

    let mut man = manager();
    let p: Box<FnBox()> = Box::new(main);
    let task = man.add(spawn_entry, Box::into_raw(Box::new(p)) as usize);

    // Switch to the spawning task immediately
    let r = task.set_priority(Priority::Critical);
    debug_assert!(r.is_ok());
    mem::drop(_blocker);
    if !man.switch_if_needed() {
        sleep(0);
    }

    return task;

    extern "C" fn spawn_entry(main: usize) {
        let main = main as *mut Box<FnBox()>;
        unsafe {
            Box::from_raw(main)();
        }
    }
}

// TODO: try

#[inline(always)]
pub fn this() -> Task {
    Task::this()
}

#[inline(always)]
pub fn exit() -> ! {
    manager().terminated();
}

#[inline(always)]
pub fn sleep(duration: usize) {
    manager().sleep(duration);
}

#[inline(always)]
pub fn yield_now() {
    manager().yield_now();
}

