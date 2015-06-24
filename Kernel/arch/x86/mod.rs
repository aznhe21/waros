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

use prelude::*;
use core::mem;
use core::fmt::Write;
use logging::Writer;

// x86 port IO
#[path = "../x86_common/io.rs"]
mod x86_io;

// Serial output channel
#[path = "../x86_common/serial.rs"]
pub mod serial;

#[path = "../x86_common/interrupt/mod.rs"]
pub mod interrupt;

#[path = "../x86_common/drivers/mod.rs"]
pub mod drivers;

pub const KERNEL_BASE: usize = 0xC0000000;

extern {
    fn into_protection(entry: fn());
}

#[no_mangle]
pub fn x86_prep_page_table(buf: &mut [u32; 1024]) {
    for i in 0u32 .. 1024 {
        buf[i as usize] = i * 0x1000 + 3;
    }
}

#[no_mangle]
pub fn x86_pre_init() {
    //log!("WARos: pre boot");

    interrupt::pre_init();
    drivers::pre_init();

    let main = super::kmain;
    unsafe {
        /*while !serial::is_transmit_empty() {
            // Do nothing
        }*/

        into_protection(main);
    }
}

#[inline(always)]
pub fn indirect_pointer<T>(ptr: *const T) -> *const T {
    (ptr as usize + KERNEL_BASE) as *const T
}

#[inline(always)]
pub fn indirect_pointer_mut<T>(ptr: *mut T) -> *mut T {
    (ptr as usize + KERNEL_BASE) as *mut T
}

#[inline(always)]
pub unsafe fn indirect_unwrap<T>(ptr: *const T) -> &'static T {
    &*indirect_pointer(ptr)
}

#[inline(always)]
pub unsafe fn indirect_unwrap_mut<T>(ptr: *mut T) -> &'static T {
    &*indirect_pointer_mut(ptr)
}

#[inline(always)]
pub unsafe extern "C" fn begin_memory_direct_access() {
    // Enable cache
    let cr0: u32;
    asm!("movl %cr0, %eax" : "={eax}"(cr0) ::: "volatile");
    asm!("movl %eax, %cr0" :: "{eax}"(cr0 | 0x60000000) :: "volatile");
}

#[inline(always)]
pub unsafe extern "C" fn end_memory_direct_access() {
    // Disable cache
    let cr0: u32;
    asm!("movl %cr0, %eax" : "={eax}"(cr0) ::: "volatile");
    asm!("movl %eax, %cr0" :: "{ax}"(cr0 & !0x60000000) :: "volatile");
}

pub fn print_backtrace() {
    let mut bp: u32;
    unsafe {
        asm!("mov %ebp, $0" : "=r"(bp) ::: "volatile");
    }

    let mut writer = Writer::get_without_module();
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
        let ptr: *const [u32; 2] = unsafe { mem::transmute(bp) };
        let newbp = unsafe { (*ptr)[0] };
        let newip = unsafe { (*ptr)[1] };
        if newbp <= bp {
            Some((0, newip))
        } else {
            Some((newbp, newip))
        }
    }
}

