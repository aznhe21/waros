use arch::{self, page};
use arch::x86_io::outb;
use memory::kernel::PhysAddr;
use drivers::display::{self, DisplaySize, Display};
use core::mem;
use core::u16;
use core::cmp::max;

const VGA_ADDRESS: arch::AddrType = 0xB8000;
const VGA_PTR: *mut u16 = VGA_ADDRESS as *mut u16;
const VGA_SIZE: (u16, u16) = (80, 25);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Black = 0,
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
    pub fn from_common(color: display::Color) -> Color {
        const LO_LO:  u8 = 0;
        const LO_HI:  u8 = 84;
        const MI_LO:  u8 = 85;
        const MI_HI:  u8 = 170;
        const HI_LO:  u8 = 171;
        const HI_HI:  u8 = 255;
        match color {
            display::Color { red: LO_LO...LO_HI, green: LO_LO...LO_HI, blue: LO_LO...LO_HI } => Color::Black,
            display::Color { red: HI_LO...HI_HI, green: LO_LO...LO_HI, blue: LO_LO...LO_HI } => Color::LightRed,
            display::Color { red: LO_LO...LO_HI, green: HI_LO...HI_HI, blue: LO_LO...LO_HI } => Color::LightGreen,
            display::Color { red: HI_LO...HI_HI, green: HI_LO...HI_HI, blue: LO_LO...LO_HI } => Color::Yellow,
            display::Color { red: LO_LO...LO_HI, green: LO_LO...LO_HI, blue: HI_LO...HI_HI } => Color::LightBlue,
            display::Color { red: HI_LO...HI_HI, green: LO_LO...LO_HI, blue: HI_LO...HI_HI } => Color::LightPink,
            display::Color { red: LO_LO...LO_HI, green: HI_LO...HI_HI, blue: HI_LO...HI_HI } => Color::LightCyan,
            display::Color { red: HI_LO...HI_HI, green: HI_LO...HI_HI, blue: HI_LO...HI_HI } => Color::White,
            display::Color { red: MI_LO...MI_HI, green: LO_LO...LO_HI, blue: LO_LO...LO_HI } => Color::Red,
            display::Color { red: LO_LO...LO_HI, green: MI_LO...MI_HI, blue: LO_LO...LO_HI } => Color::Green,
            display::Color { red: MI_LO...MI_HI, green: MI_LO...MI_HI, blue: LO_LO...LO_HI } => Color::Brown,
            display::Color { red: LO_LO...LO_HI, green: LO_LO...LO_HI, blue: MI_LO...MI_HI } => Color::Blue,
            display::Color { red: MI_LO...MI_HI, green: LO_LO...LO_HI, blue: MI_LO...MI_HI } => Color::Pink,
            display::Color { red: LO_LO...LO_HI, green: MI_LO...MI_HI, blue: MI_LO...MI_HI } => Color::Cyan,
            display::Color { red: MI_LO...MI_HI, green: MI_LO...MI_HI, blue: MI_LO...MI_HI } => Color::DarkGray,
            _                                                                                => Color::LightGray
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
                unsafe {
                    *VGA_PTR.offset((x + y * width) as isize) = make_entry(ch, fg, bg);
                }
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
    unsafe {
        *VGA_PTR.offset((x + y * width) as isize) = make_entry(ch, fg, bg);
    }
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
        let res = VGA_SIZE.0 as usize * VGA_SIZE.1 as usize;
        let vram = PhysAddr::from_raw(VGA_ADDRESS);
        let vram_end = vram + (res * u16::BYTES) as arch::AddrType;
        page::table().map_direct(page::PageTable::FLAGS_KERNEL, vram .. vram_end);

        VgaText
    }

    pub fn is_available() -> bool { true }
}

impl Display for VgaText {
    fn resolution(&self) -> (DisplaySize, DisplaySize) {
        (VGA_SIZE.0 as DisplaySize, VGA_SIZE.1 as DisplaySize)
    }

    fn put(&self, color: display::Color, x: DisplaySize, y: DisplaySize) {
        let color = Color::from_common(color);
        put(x as u16, y as u16, b' ', Color::Black, color);
    }
}

