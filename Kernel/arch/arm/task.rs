use rt;
use core::usize;
use collections::Vec;

pub const TASK_SWITCH_INTERVAL: usize = 20;
const TASK_STACK_SIZE: usize = 64 * 1024;

type Registers = [u32; 11];// r4-r12
const REG_INIT: Registers = [0; 11];

#[allow(improper_ctypes)]
extern "C" {
    fn task_entry() -> !;
    fn task_switch(cur_regs: &mut Registers, next_regs: &mut Registers);
    fn task_leap(regs: &mut Registers) -> !;
}

pub struct TaskEntity {
    stack: Vec<usize>,
    regs: Registers
}

impl TaskEntity {
    #[inline(always)]
    pub fn new() -> TaskEntity {
        unsafe {
            let stack_len = TASK_STACK_SIZE / usize::BYTES;
            let mut stack = Vec::with_capacity(stack_len);
            stack.set_len(stack_len);

            TaskEntity {
                stack: stack,
                regs: REG_INIT
            }
        }
    }

    #[inline(always)]
    pub fn setup(&mut self, entry: extern "C" fn(usize), arg: usize, return_to: fn() -> !) {
        let stack_len = self.stack.len();
        self.regs[4  - 4] = arg as u32;
        self.regs[5  - 4] = return_to as u32;
        self.regs[6  - 4] = entry as u32;
        self.regs[12 - 4] = 0x5F;
        self.regs[13 - 4] = rt::align_down_mut_ptr(self.stack[stack_len..].as_mut_ptr(), 8) as u32;
        self.regs[14 - 4] = task_entry as u32;
    }

    #[inline(always)]
    pub fn setup_primary(&mut self) {
        // 現在のタスクなので値はどうでもいい
    }

    #[inline(always)]
    pub fn terminate(&mut self) {
        self.regs = REG_INIT;
    }
}

#[inline]
pub unsafe fn switch(cur_task: &mut TaskEntity, next_task: &mut TaskEntity) {
    task_switch(&mut cur_task.regs, &mut next_task.regs);
}

#[inline]
pub unsafe fn leap(next_task: &mut TaskEntity) -> ! {
    task_leap(&mut next_task.regs);
}

