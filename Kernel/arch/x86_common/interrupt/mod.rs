pub mod gdt;
pub mod idt;
pub mod pic;
pub mod pit;

mod a20;
pub mod device;

const GDT_ENTRY_BOOT_CS:  usize = 2;
const GDT_ENTRY_BOOT_DS:  usize = 3;
//const GDT_ENTRY_BOOT_TSS: usize = 4;
const GDT_BOOT_ENTRIES:   usize = 4;

const BOOT_CS:  usize = GDT_ENTRY_BOOT_CS  * 8;
const BOOT_DS:  usize = GDT_ENTRY_BOOT_DS  * 8;
//const BOOT_TSS: usize = GDT_ENTRY_BOOT_TSS * 8;

const GDT_ENTRY_KERNEL_CS:          usize = 12;
const GDT_ENTRY_KERNEL_DS:          usize = 13;
const GDT_ENTRY_DEFAULT_USER_CS:    usize = 14;
const GDT_ENTRY_DEFAULT_USER_DS:    usize = 15;
const GDT_ENTRY_TSS:                usize = 16;
const GDT_ENTRY_LDT:                usize = 17;
const GDT_ENTRIES:                  usize = 18;

const KERNEL_CS: usize = GDT_ENTRY_KERNEL_CS * 8;
const KERNEL_DS: usize = GDT_ENTRY_KERNEL_DS * 8;

#[inline(always)]
pub extern "C" fn sti() {
    unsafe {
        asm!("sti" :::: "volatile");
    }
}

#[inline(always)]
pub extern "C" fn hlt() {
    unsafe {
        asm!("hlt" :::: "volatile");
    }
}

#[inline(always)]
pub extern "C" fn sti_hlt() {
    unsafe {
        asm!("sti \n hlt" :::: "volatile");
    }
}

#[inline(always)]
pub extern "C" fn cli() {
    unsafe {
        asm!("cli" :::: "volatile");
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
        cli();
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

        sti();
    }
}

