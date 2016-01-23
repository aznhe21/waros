pub use self::vga_text::{Console, Color as VgaColor, VgaText};
pub use self::vbe::Vbe;
pub use self::bochs::Bochs;

use drivers::display::{self, Display};
use arch::x86_io::{outb, load_eflags, save_eflags};
use arch::interrupt;
use alloc::boxed::Box;

pub mod vga_text;
pub mod vbe;
pub mod bochs;

#[inline]
pub unsafe fn preferred() -> Box<Display> {
    if bochs::Bochs::is_available() {
        Box::new(bochs::Bochs::new(640, 480))
    } else if vbe::Vbe::is_available() {
        Box::new(vbe::Vbe::new())
    } else {
        Box::new(display::Dummy::new())
    }
}

fn set_rgb_palette() {
    unsafe {
        set_palette(0, &display::RGB_TABLE);
    }
}

unsafe fn set_palette(start: u8, table: &[display::RGB]) {
    let eflags = load_eflags();
    interrupt::disable();
    outb(0x03C8, start);
    for rgb in table {
        outb(0x03C9, rgb.red   / 4);
        outb(0x03C9, rgb.green / 4);
        outb(0x03C9, rgb.blue  / 4);
    }
    save_eflags(eflags);
}

