/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * arch/x86/mod.rs
 * - Top-level file for x86 architecture
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */
#![allow(dead_code)]

use memory;
use memory::kernel::VirtAddr;
use core::fmt::Write;
use logging::Writer;

// x86 port IO
#[path = "../x86_common/io.rs"]
mod x86_io;

// Runtime library (64bit division)
pub mod rt;

// Multiboot data
#[path = "../x86_common/multiboot.rs"]
pub mod multiboot;

// Serial output channel
#[path = "../x86_common/serial.rs"]
pub mod serial;

#[path = "../x86_common/page.rs"]
pub mod page;

#[path = "../x86_common/interrupt/mod.rs"]
pub mod interrupt;

#[path = "../x86_common/task.rs"]
pub mod task;

#[path = "../x86_common/drivers/mod.rs"]
pub mod drivers;

pub const PAGE_SIZE: usize = 0x1000;
pub const FRAME_SIZE: usize = 0x1000;
pub const KERNEL_BASE: usize = 0xC0000000;

extern {
    static __kernel_start: u8;
    static __kernel_end: u8;
}

#[inline(always)]
pub fn kernel_start() -> VirtAddr {
    let addr = &__kernel_start as *const u8 as usize;
    VirtAddr::from_raw(addr - 0x00100000)
}

#[inline(always)]
pub fn kernel_end() -> VirtAddr {
    let addr = &__kernel_end as *const u8 as usize;
    VirtAddr::from_raw(addr)
}

#[inline(always)]
pub fn kernel_size() -> usize {
    kernel_end().value() - kernel_start().value()
}

#[no_mangle]
pub fn x86_prep_page_table(buf: &mut [u32; 1024 * 16]) {
    for i in 0u32 .. 1024 * 16 {
        buf[i as usize] = (i << 12) | 3;
        //buf[i as usize] = i * 0x1000 + 3;
    }
}

#[no_mangle]
pub fn x86_pre_init() {
    interrupt::pre_init();

    log!("WARos: pre boot");
}

#[no_mangle]
pub fn x86_init() {
    log!("WARos: Switched to protected mode");

    multiboot::init();
    memory::init_by_multiboot(multiboot::info().mmap().expect("Memory map not provided"));
}

pub fn print_backtrace() {
    let mut bp: u32;
    unsafe {
        asm!("mov %ebp, $0" : "=r"(bp) ::: "volatile");
    }

    let mut writer = Writer::get(module_path!());
    let _ = write!(&mut writer, "Backtrace: {:x}", bp);

    while let Some((newbp, ip)) = backtrace(bp) {
        let _ = write!(&mut writer, " > {:x}", ip);
        bp = newbp;
    }
}

pub fn backtrace(bp: u32) -> Option<(u32, u32)> {
    if bp == 0 || bp % 4 != 0 {
        None
    } else {
        let ptr = bp as *const [u32; 2];
        let newbp = unsafe { (*ptr)[0] };
        let newip = unsafe { (*ptr)[1] };
        if newbp <= bp {
            Some((0, newip))
        } else {
            Some((newbp, newip))
        }
    }
}

