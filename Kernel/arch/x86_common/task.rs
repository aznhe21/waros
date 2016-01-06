use core::ptr;
use core::usize;
use collections::Vec;

pub const TASK_SWITCH_INTERVAL: usize = 20;
const TASK_STACK_SIZE: usize = 64 * 1024;

#[allow(improper_ctypes)]
extern "C" {
    fn task_switch(csp: &mut *mut (), cip: &mut *mut (), nsp: *mut (), nip: *mut ());
    fn task_leap(sp: *mut (), ip: *mut ()) -> !;
}

pub struct TaskEntity {
    stack: Vec<usize>,
    sp: *mut (),
    ip: *mut ()
}

impl TaskEntity {
    #[inline]
    pub fn new() -> TaskEntity {
        unsafe {
            let stack_len = TASK_STACK_SIZE / usize::BYTES;
            let mut stack = Vec::with_capacity(stack_len);
            stack.set_len(stack_len);

            TaskEntity {
                stack: stack,
                sp: ptr::null_mut(),
                ip: ptr::null_mut()
            }
        }
    }

    #[inline]
    pub fn setup<T>(&mut self, entry: extern "C" fn(arg: &T), arg: &T, return_to: fn() -> !) {
        let stack_len = self.stack.len();
        let sp = &mut self.stack[stack_len - 3 ..];
        sp[0] = return_to as usize;
        sp[1] = arg as *const T as usize;
        self.sp = sp.as_mut_ptr() as *mut ();
        self.ip = entry as *mut ();
    }

    #[inline(always)]
    pub fn setup_primary(&mut self) {
        // 現在のタスクなので値はどうでもいい
    }

    #[inline(always)]
    pub fn terminate(&mut self) {
        self.sp = ptr::null_mut();
        self.ip = ptr::null_mut();
    }
}

#[inline]
pub unsafe fn switch(cur_task: &mut TaskEntity, next_task: &mut TaskEntity) {
    task_switch(&mut cur_task.sp, &mut cur_task.ip, next_task.sp, next_task.ip);
}

#[inline]
pub unsafe fn leap(next_task: &mut TaskEntity) -> ! {
    task_leap(next_task.sp, next_task.ip);
}

