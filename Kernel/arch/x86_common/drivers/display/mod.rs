pub use self::common::*;

use arch::x86_io::{outb, load_eflags, save_eflags};
use arch::interrupt;

#[path="../../../common/drivers/display.rs"]
pub mod common;

pub mod vga_text;
pub mod vga;
pub mod vbe;
pub mod bochs;

/*pub fn suitable() -> Display {
    vga::Vga::new()
}*/

#[inline(always)]
pub fn pre_init() {
}

#[inline(always)]
pub fn init() {
}

fn set_rgb_palette() {
    unsafe {
        set_palette(0, &RGB_TABLE);
    }
}

unsafe fn set_palette(start: u8, table: &[RGB]) {
    let eflags = load_eflags();
    interrupt::cli();
    outb(0x03C8, start);
    for rgb in table {
        outb(0x03C9, rgb.red   / 4);
        outb(0x03C9, rgb.green / 4);
        outb(0x03C9, rgb.blue  / 4);
    }
    save_eflags(eflags);
}

