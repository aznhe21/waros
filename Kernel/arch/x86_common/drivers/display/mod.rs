pub use self::vga_text::{Console, Color as VgaColor, VgaText};
pub use self::vbe::Vbe;
pub use self::bochs::Bochs;

use drivers::display::{self, Display};
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

