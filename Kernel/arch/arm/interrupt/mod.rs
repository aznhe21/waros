pub mod pic;
pub mod pit;
pub mod device;

#[no_mangle]
pub unsafe fn undefined_entry() -> ! {
    super::serial::puts("undef\n");
    loop {}
}

#[no_mangle]
pub unsafe fn prefetch_entry() -> ! {
    super::serial::puts("prefetch abort\n");
    loop {}
}

#[no_mangle]
pub unsafe fn data_entry() -> ! {
    super::serial::puts("data abort\n");
    loop {}
}

#[no_mangle]
pub unsafe fn unhandled_interrupt() -> ! {
    super::serial::puts("unhandled interrupt\n");
    loop {}
}

#[inline(always)]
pub fn enable() {
    unsafe {
        asm!("cpsie if" :::: "volatile");
    }
}

#[inline(always)]
pub fn wait() {
    unsafe {
        asm!("wfi" :::: "volatile");
    }
}

#[inline(always)]
pub fn enable_wait() {
    enable();
    wait();
}

#[inline(always)]
pub fn disable() {
    unsafe {
        asm!("cpsid if" :::: "volatile");
    }
}

pub fn start() -> usize {
    unsafe {
        let ret: usize;
        asm!("mrs r0, cpsr
              and r0, r0, #0x80
              mov $0, r0
              cpsie if" : "=r"(ret) :: "r0" : "volatile");
        ret
    }
}

pub fn stop() -> usize {
    unsafe {
        let ret: usize;
        asm!("mrs r0, cpsr
              and r0, r0, #0x80
              mov $0, r0
              cpsid if" : "=r"(ret) :: "r0" : "volatile");
        ret
    }
}

pub fn restore(state: usize) {
    unsafe {
        asm!("mrs r0, cpsr
              orr r0, r0, $0
              msr cpsr_c, r0" :: "r"(state) : "r0" : "volatile");
    }
}

#[inline(always)]
pub fn pre_init() {
    self::disable();
}

#[inline]
pub fn init() {
    unsafe {
        pic::init();
        pit::init();
        device::init();
    }
    self::enable();
}

