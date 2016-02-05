pub mod gdt;
pub mod idt;
pub mod pic;
pub mod pit;

mod a20;
pub mod device;

pub const GDT_ENTRY_BOOT_CS:  usize = 2;
pub const GDT_ENTRY_BOOT_DS:  usize = 3;
pub const GDT_BOOT_ENTRIES:   usize = 4;

pub const BOOT_CS:  usize = GDT_ENTRY_BOOT_CS  * 8;
pub const BOOT_DS:  usize = GDT_ENTRY_BOOT_DS  * 8;

pub const GDT_ENTRY_KERNEL_CS:          usize = 12;
pub const GDT_ENTRY_KERNEL_DS:          usize = 13;
pub const GDT_ENTRY_DEFAULT_USER_CS:    usize = 14;
pub const GDT_ENTRY_DEFAULT_USER_DS:    usize = 15;
pub const GDT_ENTRY_TSS:                usize = 16;
pub const GDT_ENTRY_LDT:                usize = 17;
pub const GDT_ENTRIES:                  usize = 18;

pub const KERNEL_CS: usize = GDT_ENTRY_KERNEL_CS * 8;
pub const KERNEL_DS: usize = GDT_ENTRY_KERNEL_DS * 8;

#[inline(always)]
pub fn enable() {
    unsafe {
        asm!("sti" :::: "volatile");
    }
}

#[inline(always)]
pub fn wait() {
    unsafe {
        asm!("hlt" :::: "volatile");
    }
}

#[inline(always)]
pub fn enable_wait() {
    unsafe {
        asm!("sti \n hlt" :::: "volatile");
    }
}

#[inline(always)]
pub fn disable() {
    unsafe {
        asm!("cli" :::: "volatile");
    }
}

pub fn start() -> usize {
    unsafe {
        let ret: usize;
        asm!("pushf
              pop   %eax
              andl  $$(1<<9), %eax
              mov   %eax, $0
              sti" : "=r"(ret) :: "eax" : "volatile");
        ret
    }
}

pub fn stop() -> usize {
    unsafe {
        let ret: usize;
        asm!("pushf
              pop   %eax
              andl  $$(1<<9), %eax
              mov   %eax, $0
              cli" : "=r"(ret) :: "eax" : "volatile");
        ret
    }
}

#[cfg(target_arch="x86")]
pub fn restore(state: usize) {
    unsafe {
        asm!("pushf
              pop    %eax
              andl   $$(~(1<<9)), %eax
              orl    $0, %eax
              push   %eax
              popf" :: "m"(state) : "eax" : "volatile");
    }
}

#[cfg(target_arch="x86_64")]
pub fn restore(state: usize) {
    unsafe {
        asm!("pushf
              pop    %rax
              andl   $$(~(1<<9)), %rax
              orl    $0, %rax
              push   %rax
              popf" :: "m"(state) : "rax" : "volatile");
    }
}

#[inline(always)]
pub fn pre_init() {
    unsafe {
        gdt::pre_init();
        idt::pre_init();
        pic::pre_init();
        pit::pre_init();

        a20::enable();
        self::disable();
    }
}

#[inline]
pub fn init() {
    unsafe {
        gdt::init();
        idt::init();
        pic::init();
        pit::init();
        device::init();

        self::enable();
    }
}

