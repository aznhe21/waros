use rt::{Force, ForceRef, IntBlocker};
use arch;
use arch::interrupt;
use arch::task::TaskEntity;
use lists::{LinkedNode, DList};
use memory;
use memory::kcache::{KCacheAllocator, KCBox};
use timer;
use core::mem;
use core::usize;
use core::ptr::{self, Shared};
use core::sync::atomic::{Ordering, AtomicUsize};

/*
pub const TASK_SWITCH_INTERVAL: usize = ...;

struct TaskEntity {
    ...
}

impl TaskEntity {
    pub fn new() -> TaskEntity {
        ...
    }

    pub fn setup<T>(&mut self, entry: extern "C" fn(arg: &T), arg: &T, return_to: fn() -> !) {
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
#[repr(u8)]
pub enum State {
    Runnable,
    Suspended,
    Free
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Priority {
    Critical = 0,
    High,
    Middle,
    Low,
    Idle
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
    pub timer: timer::UnmanagedTimer,
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
            timer: unsafe { timer::UnmanagedTimer::with_callback(TaskManager::resume_by_timer) },
            state: State::Free,
            priority: Task::DEFAULT_PRIORITY,
            entity: TaskEntity::new(),
            prev: None,
            next: None
        }
    }

    #[inline]
    fn setup<T>(&mut self, id: usize, entry: extern "C" fn(arg: &T), arg: &T, return_to: fn() -> !) {
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
    ptr: Shared<TaskData>
}

impl Task {
    pub const DEFAULT_PRIORITY: Priority = Priority::Middle;

    #[inline(always)]
    fn new(ptr: Shared<TaskData>) -> Task {
        Task {
            ptr: ptr
        }
    }

    #[inline]
    pub fn this() -> Task {
        manager().running_task.clone()
    }

    #[inline(always)]
    fn data(&self) -> &mut TaskData {
        unsafe {
            &mut **self.ptr
        }
    }

    #[inline]
    pub fn is_running(&self) -> bool {
        &manager().running_task == self
    }

    #[inline]
    pub fn exit(&self) -> ! {
        assert!(self.is_running());
        manager().terminated();
    }

    #[inline(always)]
    pub fn set_priority(&self, priority: Priority) {
        manager().set_priority(self, priority);
    }

    #[inline(always)]
    pub fn suspend(&self) {
        manager().suspend(self);
    }

    #[inline(always)]
    pub fn resume(&self) {
        manager().resume(self)
    }

    #[inline(always)]
    pub fn sleep(&self, duration: usize) {
        manager().sleep(self, duration);
    }

    #[inline(always)]
    pub fn yielding(&self) {
        assert!(self.is_running());
        manager().yielding();
    }
}

impl PartialEq for Task {
    #[inline(always)]
    fn eq(&self, other: &Task) -> bool {
        *self.ptr == *other.ptr
    }
}

impl Eq for Task { }

pub struct TaskManager {
    runnable_tasks: [DList<TaskData>; PRIORITY_LEN],
    suspended_tasks: DList<TaskData>,
    free_tasks: DList<TaskData>,
    running_task: Task,
    current_priority: Priority,
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
                timer: timer::UnmanagedTimer::with_callback(TaskManager::switch_by_timer),
                kcache: kcache
            });

            for list in self.runnable_tasks.iter_mut() {
                *list = DList::new();
            }

            self.runnable_tasks[(**primary_task.ptr).priority as usize].push_back(primary_task.ptr);

            // CPU返還タスク
            self.add(yield_task, &()).set_priority(Priority::Idle);

            self.reset_timer();
        }
    }

    // 実行可能状態タスクのある最も高い優先度を返す
    fn highest_priority(&self) -> Priority {
        let (priority, _) = self.runnable_tasks.iter()
            .enumerate()
            .find(|&(_, tasks)| !tasks.is_empty())
            .unwrap();
        Priority::from_u8(priority as u8)
    }

    pub fn add<T>(&mut self, entry: extern "C" fn(arg: &T), arg: &T) -> Task {
        let _blocker = IntBlocker::new();

        unsafe {
            let data = self.free_tasks.pop_front().unwrap_or_else(|| {
                let b = memory::check_oom_opt(KCBox::new(self.kcache.clone(), TaskData::new()));
                Shared::new(KCBox::into_raw(b))
            });
            (**data).setup(task_counter.fetch_add(1, Ordering::SeqCst), entry, arg, task_terminated);
            self.runnable_tasks[(**data).priority as usize].push_back(data);

            Task::new(data)
        }
    }

    #[inline]
    pub fn reset_timer(&mut self) {
        self.timer.reset(arch::task::TASK_SWITCH_INTERVAL);
    }

    fn switch_by_timer(_: timer::TimerId) {
        manager().yielding();
    }

    fn resume_by_timer(timer: timer::TimerId) {
        unsafe {
            let data = manager().suspended_tasks.iter()
                .find(|&data| (**data).timer.id() == timer)
                .unwrap();
            Task::new(data).resume();
        }
    }

    fn set_priority(&mut self, task: &Task, priority: Priority) {
        let data = task.data();
        match data.state {
            State::Runnable => {
                self.runnable_tasks[data.priority as usize].remove(&task.ptr);
                data.priority = priority;
                self.runnable_tasks[data.priority as usize].push_back(task.ptr);
            },
            State::Suspended => {
                data.priority = priority;
            },
            State::Free => panic!("Unable to modify a free task")
        }
    }

    fn suspend(&mut self, task: &Task) {
        let data = task.data();
        assert_eq!(data.state, State::Runnable);
        data.state = State::Suspended;

        if task.is_running() {
            interrupt::disable();
            self.runnable_tasks[data.priority as usize].remove(&task.ptr);
            self.suspended_tasks.push_back(task.ptr);
            let next_task = self.forward_task();
            self.switch_task(task, &next_task);
        } else {
            self.runnable_tasks[data.priority as usize].remove(&task.ptr);
            self.suspended_tasks.push_back(task.ptr);
        }
    }

    fn resume(&mut self, task: &Task) {
        let data = task.data();
        assert_eq!(data.state, State::Suspended);
        data.state = State::Runnable;

        self.suspended_tasks.remove(&task.ptr);
        self.runnable_tasks[data.priority as usize].push_back(task.ptr);
        self.switch_to_next();
    }

    fn sleep(&mut self, task: &Task, duration: usize) {
        task.data().timer.reset(duration);
        task.suspend();
    }

    fn yielding(&mut self) {
        self.reset_timer();
        if self.runnable_tasks[self.current_priority as usize].len() != 1 {
            self.switch_to_next();
        } else {
            interrupt::enable();
        }
    }

    #[inline]
    fn forward_task(&mut self) -> Task {
        // 次のタスクか最初のタスク
        let next = Task::new(match self.running_task.data().get_next() {
            Some(next) => next,
            _ => {
                self.current_priority = self.highest_priority();
                self.runnable_tasks[self.current_priority as usize].front().unwrap()
            }
        });

        debug_assert!(self.running_task != next);
        self.running_task = next;

        self.running_task.clone()
    }

    #[inline]
    fn switch_task(&mut self, cur_task: &Task, next_task: &Task) {
        unsafe {
            arch::task::switch(&mut cur_task.data().entity, &mut next_task.data().entity);
        }
    }

    pub fn switch_to_next(&mut self) {
        let cur_task = Task::this();
        let next_task = self.forward_task();
        self.switch_task(&cur_task, &next_task);
    }

    fn remove_task(&mut self, task: &Task) {
        let data = task.data();

        match data.state {
            State::Runnable => self.runnable_tasks[data.priority as usize].remove(&task.ptr),
            State::Suspended => self.suspended_tasks.remove(&task.ptr),
            State::Free => panic!("Unable to remove a free task")
        }

        // ここでは解放しない
        data.state = State::Free;
        self.free_tasks.push_back(task.ptr);

        data.terminate();
    }

    pub fn terminate(&mut self, task: Task) {
        let _blocker = IntBlocker::new();

        assert!(!task.is_running());
        self.remove_task(&task);
    }

    fn terminated(&mut self) -> ! {
        debug_log!("Task terminated");

        interrupt::disable();
        let cur_task = Task::this();
        let next_task = self.forward_task();
        self.remove_task(&cur_task);

        self.reset_timer();
        unsafe {
            arch::task::leap(&mut next_task.data().entity);
        }
    }
}

static MANAGER: Force<TaskManager> = Force::new();

#[inline]
pub fn init() {
    MANAGER.setup().init();
}

#[inline(always)]
pub fn manager() -> ForceRef<TaskManager> {
    MANAGER.as_ref()
}

fn task_terminated() -> ! {
    manager().terminated();
}

extern "C" fn yield_task(_: &()) {
    loop {
        interrupt::wait();
    }
}

