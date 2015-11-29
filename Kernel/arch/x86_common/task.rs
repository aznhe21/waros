use core::intrinsics;
use core::ptr;
use core::usize;
use collections::Vec;

pub const TASK_SWITCH_INTERVAL: usize = 20;
pub const TASK_STACK_SIZE: usize = 64 * 1024;

#[allow(improper_ctypes)]
extern "C" {
    fn task_switch(csp: &mut *mut (), cip: &mut *mut (), nsp: *mut (), nip: *mut ());
}

pub struct TaskEntity {
    pub id: usize,
    stack: Vec<usize>,
    sp: *mut (),
    ip: *mut (),
    prev: *mut TaskEntity,
    next: *mut TaskEntity
}

impl_linked_node!(TaskEntity { prev: prev, next: next });

impl TaskEntity {
    #[inline]
    pub fn inplace_new(&mut self) {
        let stack_len = TASK_STACK_SIZE / usize::BYTES;
        unsafe {
            // 不定値でdropしないようptr::writeで書き込む
            ptr::write(&mut self.stack, Vec::with_capacity(stack_len));
            self.stack.set_len(stack_len);
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
        // do nothing
    }

    #[inline(always)]
    pub fn terminate(&mut self) {
        // do nothing
    }
}

#[inline]
pub unsafe fn switch(cur_task: &mut TaskEntity, next_task: &mut TaskEntity) {
    task_switch(&mut cur_task.sp, &mut cur_task.ip, next_task.sp, next_task.ip);
}

#[inline]
pub unsafe fn leap(next_task: &mut TaskEntity) -> ! {
    asm!("mov $0, %esp
          sti
          jmp *$1"
         :: "r"(next_task.sp), "r"(next_task.ip) :: "volatile");
    intrinsics::unreachable();
}

