pub use self::atags::init_memory;
use memory;
use memory::kernel::VirtAddr;

#[cfg(target_mach="versatile")] #[path="mach/versatile/mod.rs"]
pub mod mach;
#[cfg(target_mach="bcm")] #[path="mach/bcm/mod.rs"]
pub mod mach;

pub mod aeabi;

// Atags data
pub mod atags;

// Serial output channel
pub mod serial;

pub mod gpio;

pub mod page;

pub mod interrupt;

pub mod task;

pub mod drivers;

pub const PAGE_SIZE: usize = 0x1000;
pub const FRAME_SIZE: usize = 0x1000;
pub const KERNEL_BASE: usize = 0;

pub type AddrType = u32;

extern {
    static __kernel_start: u8;
    static __kernel_end: u8;
}

#[inline(always)]
pub fn kernel_start() -> VirtAddr {
    let addr = &__kernel_start as *const u8 as usize;
    VirtAddr::from_raw(addr)
}

#[inline(always)]
pub fn kernel_end() -> VirtAddr {
    let addr = &__kernel_end as *const u8 as usize;
    VirtAddr::from_raw(addr)
}

#[inline(always)]
pub fn kernel_size() -> usize {
    kernel_end() - kernel_start()
}

#[no_mangle]
pub unsafe extern "C" fn arm_main(_r0: u32, _r1: u32, _atags: u32) {
    interrupt::pre_init();
    gpio::pre_init();
    serial::pre_init();
    memory::pre_init();
    page::pre_init();

    log!("WARos: pre boot");
}

pub fn print_backtrace() {
    log!("Stacktrace not available");
}

pub fn print_registers() {
    unsafe {
        let mut r: [u32; 15] = [0; 15];
        asm!("stmia $0!, {r0-r12, lr}
              str   sp,  [$0]" :: "r"(r.as_mut_ptr()) :: "volatile");

        log!("Registers
    r0:  {:08X}, r1:  {:08X}, r2:  {:08X}, r3:  {:08X}, r4:  {:08X},
    r5:  {:08X}, r6:  {:08X}, r7:  {:08X}, r8:  {:08X}, r9:  {:08X},
    r10: {:08X}, r11: {:08X}, r12: {:08X}, sp:  {:08X}, lr:  {:08X}",
          r[0],  r[1],  r[2],  r[3],  r[4],
          r[5],  r[6],  r[7],  r[8],  r[9],
          r[10], r[11], r[12], r[14], r[13]
        );
    }
}

