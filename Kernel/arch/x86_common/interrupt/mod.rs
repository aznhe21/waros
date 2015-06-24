// #![feature(asm)]

pub mod gdt;
pub mod idt;
pub mod pic;
pub mod rtc;

mod a20;
pub mod input;

const GDT_KERNEL_CS: usize = 2;
const GDT_KERNEL_DS: usize = 3;
const GDT_KERNEL_TSS: usize = 4;

const KERNEL_CS: usize = GDT_KERNEL_CS * 8;
const KERNEL_DS: usize = GDT_KERNEL_DS * 8;
const KERNEL_TSS: usize = GDT_KERNEL_TSS * 8;
//USER_CS
//USER_DS

#[inline(always)]
pub extern "C" fn hlt() {
    unsafe {
        asm!("hlt" :::: "volatile");
    }
}

#[inline(always)]
pub extern "C" fn sti() {
    unsafe {
        asm!("sti" :::: "volatile");
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
        rtc::pre_init();

        a20::enable();
        cli();
    }
}

#[inline(always)]
pub fn init() {
    unsafe {
        gdt::init();
        idt::init();
        pic::init();
        rtc::init();
        input::init();
    }
}

