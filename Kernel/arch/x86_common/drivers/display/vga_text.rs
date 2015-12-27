#![allow(dead_code)]

use arch::x86_io::outb;
use drivers::display::{self, DisplaySize, Display};
use core::mem;
use core::cmp::max;

const VGA_ADDRESS: *mut u16 = 0xB8000 as *mut u16;
const VGA_SIZE: (u16, u16) = (80, 25);

#[derive(Clone, Copy)]
pub enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Pink,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightPink,
    Yellow,
    White
}

impl Color {
    fn from_common_color(color: display::Color) -> Color {
        match color {
            display::Color::Black      => Color::Black,
            display::Color::Red        => Color::LightRed,
            display::Color::Lime       => Color::LightGreen,
            display::Color::Yellow     => Color::Yellow,
            display::Color::Blue       => Color::LightBlue,
            display::Color::Purple     => Color::LightPink,
            display::Color::Cyan       => Color::LightCyan,
            display::Color::White      => Color::White,
            display::Color::Gray       => Color::LightGray,
            display::Color::DarkRed    => Color::Red,
            display::Color::Green      => Color::Green,
            display::Color::DarkYellow => Color::Brown,
            display::Color::DarkBlue   => Color::Blue,
            display::Color::DarkPurple => Color::Pink,
            display::Color::DarkCyan   => Color::Cyan,
            display::Color::DarkGray   => Color::DarkGray
        }
    }
}

pub struct Console {
    pub size: (u16, u16),
    pub pos: (u16, u16)
}

impl Console {
    pub fn new() -> Console {
        Console {
            size: VGA_SIZE,
            pos: (0, 0)
        }
    }

    pub fn normalize(&self, (x, y): (u16, u16)) -> (u16, u16) {
        let (width, height) = self.size;
        if x >= width {
            (0, if y + 1 >= height { 0 } else { y + 1 })
        } else {
            (x, if y >= height { 0 } else { y })
        }
    }

    pub fn clear(&mut self, bg: Color) {
        let pos = self.pos;
        let (width, height) = self.size;
        for _ in 0 .. width * height {
            self.put(b' ', Color::Black, bg)
        }
        self.pos = pos;
    }

    pub fn put(&mut self, ch: u8, fg: Color, bg: Color) {
        let (x, y) = self.normalize(self.pos);
        let width = self.size.0;

        self.pos = match ch {
            b'\n' => (0, y + 1),
            b'\r' => (0, y),
            b'\x08' => (max(0, x - 1), y + 1),
            b'\x0b' => (x, y + 1),
            _ => {
                unsafe { *VGA_ADDRESS.offset((x + y * width) as isize) = make_entry(ch, fg, bg) };
                self.normalize((x + 1, y))
            }
        }
    }

    pub fn puts(&mut self, s: &str, fg: Color, bg: Color) {
        for b in s.bytes() {
            self.put(b, fg, bg);
        }
        self.put(b'\n', fg, bg);
    }
}

pub fn clear(bg: Color) {
    let (width, height) = VGA_SIZE;
    for i in 0 .. width * height {
        put(i, 0, b' ', Color::Black, bg)
    }
}

pub fn put(x: u16, y: u16, ch: u8, fg: Color, bg: Color) {
    let width = VGA_SIZE.0;
    unsafe { *VGA_ADDRESS.offset((x + y * width) as isize) = make_entry(ch, fg, bg) };
}

pub fn puts(x: u16, y: u16, s: &str, fg: Color, bg: Color) {
    let bytes = s.as_bytes();
    for i in 0 .. bytes.len() {
        put(x + i as u16, y, bytes[i], fg, bg);
    }
}

pub fn put32(x: u16, y: u16, n: u32, base: u32, fg: Color, bg: Color) {
    let mut t = n / base;
    let mut l = 1;
    while t > 0 {
        l += 1;
        t /= base;
    }

    t = n;
    for i in (x .. x + l).rev() {
        let c = match (t % base) as u8 {
            d @ 0...10 => b'0' + d,
            d @ 10...16 => b'A' - 10 + d,
            _ => unreachable!()
        };
        put(i, y, c, fg, bg);
        t /= base;
    }
}

#[cfg(target_arch="x86")]
pub fn cursor(x: u16, y: u16) {
    unsafe {
        let (lo, hi): (u8, u8) = mem::transmute(x + y * VGA_SIZE.0);
        outb(0x03D4, 0x0E);
        outb(0x03D5, hi);
        outb(0x03D4, 0x0F);
        outb(0x03D5, lo);
    }
}

fn make_entry(b: u8, fg: Color, bg: Color) -> u16 {
    let attr = (fg as u16) | ((bg as u16) << 4);
    (attr << 8) | (b as u16)
}

pub struct VgaText;

impl VgaText {
    pub fn new() -> VgaText {
        VgaText
    }
}

impl Display for VgaText {
    fn is_available() -> bool { true }

    fn resolution(&self) -> (DisplaySize, DisplaySize) {
        (VGA_SIZE.0 as DisplaySize, VGA_SIZE.1 as DisplaySize)
    }

    fn put(&self, ccolor: display::Color, x: DisplaySize, y: DisplaySize) {
        let color = Color::from_common_color(ccolor);
        put(x as u16, y as u16, b' ', Color::Black, color);
    }
}

