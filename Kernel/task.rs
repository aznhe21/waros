use rt::UnsafeOption;
use arch;
use arch::interrupt;
use arch::task::TaskEntity;
use lists::{LinkedList, LinkedNode};
use memory::{self, slab};
use timer;
use core::mem;
use core::ptr;
use core::ops;
use core::sync::atomic::{Ordering, AtomicUsize};

/*
pub const TASK_SWITCH_INTERVAL: usize = ...;
pub const TASK_STACK_SIZE: usize = ...;

struct TaskEntity {
  pub id: usize,
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

pub struct Task {
    entity: *mut TaskEntity
}

impl Task {
    #[inline(always)]
    const fn from_entity(entity: *mut TaskEntity) -> Task {
        Task { entity: entity }
    }

    #[inline(always)]
    fn entity(&self) -> &mut TaskEntity {
        unsafe {
            &mut *self.entity
        }
    }
}

impl ops::Deref for Task {
    type Target = TaskEntity;

    #[inline(always)]
    fn deref(&self) -> &TaskEntity {
        self.entity()
    }
}

impl ops::DerefMut for Task {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut TaskEntity {
        self.entity()
    }
}

pub struct TaskManager {
    tasks: LinkedList<TaskEntity>,
    free_tasks: LinkedList<TaskEntity>,
    current_task: *mut TaskEntity,
    timer: *mut timer::TimerEntity,
    slab: &'static mut slab::SlabAllocator<TaskEntity>
}

impl TaskManager {
    fn init(&mut self) {
        *self = TaskManager {
            tasks: LinkedList::new(),
            free_tasks: LinkedList::new(),
            current_task: ptr::null_mut(),
            timer: unsafe { timer::manager().by_callback(TaskManager::switch_by_timer).entity() },
            slab: memory::check_oom_opt(slab::SlabAllocator::new("Task", mem::align_of::<Task>(), None))
        };

        // 現在のタスクなので値はどうでもいい
        let current_task = memory::check_oom(self.slab.allocate());
        current_task.id = 0;
        current_task.setup_primary();
        self.current_task = current_task;
        self.tasks.push_back(current_task);

        self.reset_timer();
        interrupt::enable();
    }

    pub fn add<T>(&mut self, entry: extern "C" fn(arg: &T), arg: &T) {
        let task = self.free_tasks.pop_front().unwrap_or_else(|| {
            let task = memory::check_oom(self.slab.allocate());
            task.id = task_counter.fetch_add(1, Ordering::SeqCst);
            task.inplace_new();
            task
        });
        task.setup(entry, arg, task_terminated);
        self.tasks.push_back(task);
    }

    #[inline]
    pub fn reset_timer(&mut self) {
        unsafe {
            (*self.timer).reset(arch::task::TASK_SWITCH_INTERVAL);
        }
    }

    fn switch_by_timer(_: timer::TimerId) {
        let man = manager();
        man.reset_timer();
        if man.tasks.len() != 1 {
            man.switch_to_next();
        } else {
            interrupt::enable();
        }
    }

    #[inline]
    pub fn current_task(&mut self) -> Task {
        Task::from_entity(self.current_task)
    }

    #[inline]
    fn forward_task(&mut self) -> Task {
        unsafe {
            // 次のタスクか最初のタスク
            let next = (*self.current_task).get_next();
            self.current_task = if !next.is_null() { next } else { self.tasks.front_ptr() };
            Task::from_entity(self.current_task)
        }
    }

    pub fn switch_to_next(&mut self) {
        let cur_task = self.current_task();
        let next_task = self.forward_task();
        unsafe {
            //let mut sp: *mut u32;
            //asm!("mov $0, sp" : "=r"(sp) ::: "volatile");
            //log!("switching {} to   {} (SP: {:08x})", cur_task.id, next_task.id, sp as usize);
            //arch::print_registers();

            arch::task::switch(cur_task.entity(), next_task.entity());

            //asm!("mov $0, sp" : "=r"(sp) ::: "volatile");
            //log!("returned  {} from {} (SP: {:08x})", cur_task.id, next_task.id, sp as usize);
            //arch::print_registers();
        }
    }

    pub fn terminate(&mut self, task: Task) -> ! {
        interrupt::disable();
        let next_task = self.forward_task();

        self.tasks.remove(task.entity);
        if self.tasks.len() == 0 {
            panic!("There are no tasks");
        }
        // ここでは解放しない
        self.free_tasks.push_back(task.entity);

        self.reset_timer();
        task.entity().terminate();
        unsafe {
            arch::task::leap(next_task.entity());
        }
    }

    #[inline]
    fn terminated(&mut self) -> ! {
        log!("Task terminated");
        let task = self.current_task();
        self.terminate(task);
    }
}

static mut manager_opt: Option<TaskManager> = None;

#[inline]
pub fn init() {
    unsafe {
        manager_opt.into_some().init();
    }
}

#[inline]
pub fn manager() -> &'static mut TaskManager {
    unsafe {
        manager_opt.as_mut().be_some()
    }
}

fn task_terminated() -> ! {
    manager().terminated();
}

