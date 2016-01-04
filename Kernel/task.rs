use rt::{Force, ForceRef, IntBlocker};
use arch;
use arch::interrupt;
use arch::task::TaskEntity;
use lists::{LinkedNode, DList};
use memory;
use memory::kcache::KCacheAllocator;
use timer;
use core::mem;
use core::ptr::{self, Shared};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{Ordering, AtomicUsize};

/*
pub const TASK_SWITCH_INTERVAL: usize = ...;
pub const TASK_STACK_SIZE: usize = ...;

struct TaskEntity {
    pub id: usize,
    pub timer: timer::UnmanagedTimer,
    pub state: task::State,
    pub priority: task::Priority,
    ...
}

impl TaskEntity {
    pub fn inplace_new(&mut self) {
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

impl LinkedNode for TaskEntity { ... }

/// Switch to next task
pub unsafe fn switch(cur_task: &mut TaskEntity, next_task: &mut TaskEntity) -> ! { ... }

/// Switch the task due to current task is terminated
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

pub struct Task {
    entity: Shared<TaskEntity>
}

impl Task {
    pub const DEFAULT_PRIORITY: Priority = Priority::Middle;

    #[inline(always)]
    fn from_entity(entity: *mut TaskEntity) -> Task {
        unsafe {
            Task { entity: Shared::new(entity) }
        }
    }

    #[inline(always)]
    fn entity(&self) -> &mut TaskEntity {
        unsafe {
            &mut **self.entity
        }
    }

    #[inline]
    pub fn this() -> Task {
        Task::from_entity(*manager().running_task)
    }

    #[inline]
    pub fn is_running(&self) -> bool {
        manager().running_task().entity() as *const _ == self.entity() as *const _
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

impl Deref for Task {
    type Target = TaskEntity;

    #[inline(always)]
    fn deref(&self) -> &TaskEntity {
        self.entity()
    }
}

impl DerefMut for Task {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut TaskEntity {
        self.entity()
    }
}

impl PartialEq for Task {
    #[inline(always)]
    fn eq(&self, other: &Task) -> bool {
        *self.entity == *other.entity
    }
}

impl Eq for Task { }

pub struct TaskManager {
    runnable_tasks: [DList<TaskEntity>; PRIORITY_LEN],
    suspended_tasks: DList<TaskEntity>,
    free_tasks: DList<TaskEntity>,
    running_task: Shared<TaskEntity>,
    current_priority: Priority,
    timer: timer::UnmanagedTimer,
    kcache: KCacheAllocator<TaskEntity>
}

unsafe impl Send for TaskManager { }
unsafe impl Sync for TaskManager { }

impl TaskManager {
    #[inline(always)]
    fn init(&mut self) {
        unsafe {
            let _blocker = IntBlocker::new();

            let kcache = memory::check_oom_opt(KCacheAllocator::new("Task", mem::align_of::<Task>(), None));
            let primary_task = Shared::new(memory::check_oom(kcache.allocate_uninit()));
            TaskManager::init_task(primary_task, 0);
            (**primary_task).setup_primary();

            ptr::write(self, TaskManager {
                runnable_tasks: mem::uninitialized(),
                suspended_tasks: DList::new(),
                free_tasks: DList::new(),
                running_task: primary_task,
                current_priority: Task::DEFAULT_PRIORITY,
                timer: timer::UnmanagedTimer::with_callback(TaskManager::switch_by_timer),
                kcache: kcache
            });

            for list in self.runnable_tasks.iter_mut() {
                *list = DList::new();
            }

            self.runnable_tasks[(**primary_task).priority as usize].push_back(primary_task);

            // CPU返還タスク
            self.add(yield_task, &()).set_priority(Priority::Idle);

            self.reset_timer();
        }
    }

    fn init_task(entity: Shared<TaskEntity>, id: usize) {
        unsafe {
            (**entity).id = id;
            (**entity).priority = Task::DEFAULT_PRIORITY;
            (**entity).timer = timer::UnmanagedTimer::with_callback(TaskManager::resume_by_timer);
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
        unsafe {
            let task = self.free_tasks.pop_front().unwrap_or_else(|| {
                let task = Shared::new(memory::check_oom(self.kcache.allocate_uninit()));
                TaskManager::init_task(task, task_counter.fetch_add(1, Ordering::SeqCst));
                (**task).inplace_new();
                task
            });
            (**task).priority = Task::DEFAULT_PRIORITY;
            (**task).state = State::Runnable;
            (**task).setup(entry, arg, task_terminated);
            self.runnable_tasks[(**task).priority as usize].push_back(task);

            Task::from_entity(*task)
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
            let entity = manager().suspended_tasks.iter()
                .find(|&entity| (**entity).timer.id() == timer)
                .unwrap();
            Task::from_entity(*entity).resume();
        }
    }

    fn set_priority(&mut self, task: &Task, priority: Priority) {
        let _blocker = IntBlocker::new();

        let entity = task.entity();
        match entity.state {
            State::Runnable => {
                self.runnable_tasks[entity.priority as usize].remove(&task.entity);
                entity.priority = priority;
                self.runnable_tasks[entity.priority as usize].push_back(task.entity);
            },
            State::Suspended => {
                entity.priority = priority;
            },
            State::Free => panic!("Unable to modify a free task")
        }
    }

    fn suspend(&mut self, task: &Task) {
        let entity = task.entity();
        assert_eq!(entity.state, State::Runnable);
        entity.state = State::Suspended;

        if task.is_running() {
            interrupt::disable();
            self.runnable_tasks[entity.priority as usize].remove(&task.entity);
            self.suspended_tasks.push_back(task.entity);
            let next_task = self.forward_task();
            self.switch_task(task, &next_task);
        } else {
            self.runnable_tasks[entity.priority as usize].remove(&task.entity);
            self.suspended_tasks.push_back(task.entity);
        }
    }

    fn resume(&mut self, task: &Task) {
        let entity = task.entity();
        assert_eq!(entity.state, State::Suspended);
        entity.state = State::Runnable;

        self.suspended_tasks.remove(&task.entity);
        self.runnable_tasks[entity.priority as usize].push_back(task.entity);
        self.switch_to_next();
    }

    fn sleep(&mut self, task: &Task, duration: usize) {
        task.entity().timer.reset(duration);
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
    fn running_task(&mut self) -> Task {
        Task::from_entity(*self.running_task)
    }

    #[inline]
    fn forward_task(&mut self) -> Task {
        unsafe {
            // 次のタスクか最初のタスク
            let next = match (**self.running_task).get_next() {
                Some(next) => next,
                _ => {
                    self.current_priority = self.highest_priority();
                    self.runnable_tasks[self.current_priority as usize].front().unwrap()
                }
            };

            debug_assert!(*self.running_task != *next);
            self.running_task = next;

            Task::from_entity(*self.running_task)
        }
    }

    #[inline]
    fn switch_task(&mut self, cur_task: &Task, next_task: &Task) {
        unsafe {
            arch::task::switch(cur_task.entity(), next_task.entity());
        }
    }

    pub fn switch_to_next(&mut self) {
        let cur_task = self.running_task();
        let next_task = self.forward_task();
        self.switch_task(&cur_task, &next_task);
    }

    fn remove_task(&mut self, task: &Task) {
        let entity = task.entity();
        match task.state {
            State::Runnable => {
                self.runnable_tasks[entity.priority as usize].remove(&task.entity);

                // ここでは解放しない
                entity.state = State::Free;
                self.free_tasks.push_back(task.entity);
            },
            State::Suspended => {
                self.suspended_tasks.remove(&task.entity);

                entity.state = State::Free;
                self.free_tasks.push_back(task.entity);
            },
            State::Free => panic!("Unable to remove a free task")
        }

        task.entity().terminate();
    }

    pub fn terminate(&mut self, task: Task) {
        let _blocker = IntBlocker::new();

        assert!(!task.is_running());
        self.remove_task(&task);
    }

    fn terminated(&mut self) -> ! {
        debug_log!("Task terminated");

        interrupt::disable();
        let cur_task = self.running_task();
        let next_task = self.forward_task();
        self.remove_task(&cur_task);

        self.reset_timer();
        unsafe {
            arch::task::leap(next_task.entity());
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

