use prelude::*;
use arch::interrupt;
use collections::LinkedList;
use memory::{self, slab};
use timer;
use core::mem;
use core::ptr;
use core::slice;
use core::intrinsics;
use core::usize;

const TASK_SWITCH_INTERVAL: usize = 20;
const TASK_STACK_SIZE: usize = 64 * 1024;

extern "C" {
    fn task_switch(csp: *mut *mut u8, cip: *mut *mut u8, nsp: *mut u8, nip: *mut u8);
}

pub struct Task {
    stack: &'static mut [usize],
    sp: *mut u8,
    ip: *mut u8,
    prev: *mut Task,
    next: *mut Task
}

impl_linked_node!(Task { prev: prev, next: next });

pub struct TaskManager {
    tasks: LinkedList<Task>,
    free_tasks: LinkedList<Task>,
    current_task: *mut Task,
    timer: *mut timer::TimerEntity,
    slab: &'static mut slab::SlabAllocator<Task>
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
        self.current_task = current_task;
        self.tasks.push_back(current_task);
    }

    pub fn add<T>(&mut self, f: extern "C" fn(arg: T), arg: T) {
        let task = self.free_tasks.pop_front().unwrap_or_else(|| {
            let task = memory::check_oom(self.slab.allocate());
            let ptr = memory::check_oom_ptr(slab::manager().allocate(TASK_STACK_SIZE, mem::align_of::<usize>()));
            task.stack = unsafe { slice::from_raw_parts_mut(ptr as *mut usize, TASK_STACK_SIZE / usize::BYTES) };
            task
        });

        {
            let stack_len = task.stack.len();
            let sp = &mut task.stack[stack_len - 3 ..];
            sp[0] = task_terminated as usize;
            sp[1] = unsafe { *(&arg as *const T as *const usize) };
            task.sp = sp.as_mut_ptr() as *mut u8;
            task.ip = f as *mut u8;
        }

        self.tasks.push_back(task);
    }

    #[inline]
    pub fn reset_timer(&mut self) {
        unsafe {
            (*self.timer).reset(TASK_SWITCH_INTERVAL);
        }
    }

    fn switch_by_timer(_: timer::TimerId) {
        let man = manager();
        man.reset_timer();
        if man.tasks.len() != 1 {
            man.switch_to_next();
        } else {
            interrupt::sti();
        }
    }

    #[inline]
    pub fn current_task(&mut self) -> &'static mut Task {
        unsafe { &mut *self.current_task }
    }

    #[inline]
    fn forward_task(&mut self) -> &'static mut Task {
        unsafe {
            // 次のタスクか最初のタスク
            let next = (*self.current_task).next;
            self.current_task = if !next.is_null() { next } else { self.tasks.front_ptr() };
            &mut *self.current_task
        }
    }

    pub fn switch_to_next(&mut self) {
        let cur_task = self.current_task();
        let next_task = self.forward_task();

        unsafe {
            task_switch(&mut cur_task.sp, &mut cur_task.ip, next_task.sp, next_task.ip);
        }
    }

    pub fn terminate(&mut self, task: &'static mut Task) -> ! {
        interrupt::cli();
        self.tasks.remove(task);
        if self.tasks.len() == 0 {
            panic!("There are no tasks");
        }
        // ここでは解放しない
        self.free_tasks.push_back(task);

        let next_task = self.forward_task();

        self.reset_timer();
        unsafe {
            asm!("mov $0, %esp
                  sti
                  jmp *$1"
                 :: "r"(next_task.sp), "r"(next_task.ip) :: "volatile");
            intrinsics::unreachable();
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

