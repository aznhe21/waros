use core::mem;
use core::ops::Range;

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct RGB {
    pub red:   u8,
    pub green: u8,
    pub blue:  u8
}

impl RGB {
    #[inline]
    pub fn as_c8(&self) -> u8 {
        (self.red & 0x03) | (self.green & 0x18) | (self.blue & 0xE0)
    }

    #[inline]
    pub fn as_c15(&self) -> u16 {
        (self.red   as u16 * (1 << 5) / (1 << 8)) << 10 | // 5-bits
        (self.green as u16 * (1 << 5) / (1 << 8)) <<  5 | // 5-bits
        (self.blue  as u16 * (1 << 5) / (1 << 8))         // 5-bits
    }

    #[inline]
    pub fn as_c16(&self) -> u16 {
        (self.red   as u16 * (1 << 5) / (1 << 8)) << 10 | // 5-bits
        (self.green as u16 * (1 << 6) / (1 << 8)) <<  5 | // 6-bits
        (self.blue  as u16 * (1 << 5) / (1 << 8))         // 5-bits
    }

    #[inline]
    pub fn as_c24(&self) -> (u8, u8, u8) {
        (self.blue, self.green, self.red)
    }

    #[inline]
    pub fn as_c32(&self) -> u32 {
        (self.red   as u32) << 16 |
        (self.green as u32) <<  8 |
        (self.blue  as u32)
    }
}

pub const RGB_TABLE: [RGB; 16] = [
    RGB { red: 0x00, green: 0x00, blue: 0x00 },// 0:  Black
    RGB { red: 0xFF, green: 0x00, blue: 0x00 },// 1:  Red
    RGB { red: 0x00, green: 0xFF, blue: 0x00 },// 2:  Lime
    RGB { red: 0xFF, green: 0xFF, blue: 0x00 },// 3:  Yellow
    RGB { red: 0x00, green: 0x00, blue: 0xFF },// 4:  Blue
    RGB { red: 0xFF, green: 0x00, blue: 0xFF },// 5:  Purple
    RGB { red: 0x00, green: 0xFF, blue: 0xFF },// 6:  Cyan
    RGB { red: 0xFF, green: 0xFF, blue: 0xFF },// 7:  White
    RGB { red: 0xC6, green: 0xC6, blue: 0xC6 },// 8:  Gray
    RGB { red: 0x84, green: 0x00, blue: 0x00 },// 9:  Dark Red
    RGB { red: 0x00, green: 0x84, blue: 0x00 },// 10: Green
    RGB { red: 0x84, green: 0x84, blue: 0x00 },// 11: Dark Yellow
    RGB { red: 0x00, green: 0x00, blue: 0x84 },// 12: Dark Blue
    RGB { red: 0x84, green: 0x00, blue: 0x84 },// 13: Dark Purple
    RGB { red: 0x00, green: 0x84, blue: 0x84 },// 14: Dark Cyan
    RGB { red: 0x84, green: 0x84, blue: 0x84 },// 15: Dark Gray
];

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Color {
    /*Black = 0,
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
    White*/
    Black = 0,
    Red,
    Lime,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    Gray,
    DarkRed,
    Green,
    DarkYellow,
    DarkBlue,
    DarkPurple,
    DarkCyan,
    DarkGray
}

impl Color {
    pub fn from_u8(val: u8) -> Option<Color> {
        if val <= Color::DarkGray as u8 {
            Some(unsafe { mem::transmute(val) })
        } else {
            None
        }
    }

    pub fn as_rgb(&self) -> RGB {
        RGB_TABLE[*self as usize]
    }
}

pub type DisplaySize = i32;

pub trait Display {
    fn is_available() -> bool;

    fn log(&self) {
        let (width, height) = self.resolution();
        log!("Display: {}x{}", width, height);
    }

    fn resolution(&self) -> (DisplaySize, DisplaySize);
    fn put(&self, color: Color, x: DisplaySize, y: DisplaySize);

    fn horizontal_line(&self, color: Color, range: Range<DisplaySize>, y: DisplaySize) {
        for x in range {
            self.put(color, x, y);
        }
    }

    fn fill(&self, color: Color, rect: (DisplaySize, DisplaySize, DisplaySize, DisplaySize)) {
        for y in rect.1 .. rect.1 + rect.3 {
            self.horizontal_line(color, rect.0 .. rect.0 + rect.2, y);
        }
    }

    fn clear(&self, color: Color) {
        let res = self.resolution();
        self.fill(color, (0, 0, res.0, res.1));
    }

    fn line(&self, color: Color, from: (DisplaySize, DisplaySize), to: (DisplaySize, DisplaySize)) {
        let dx = (to.0 - from.0).abs();
        let dy = (to.1 - from.1).abs();
        let sx = if from.0 < to.0 { 1 } else { -1 };
        let sy = if from.1 < to.1 { 1 } else { -1 };
        let mut err = if dx > dy { dx } else { -dy } / 2;

        let (mut x, mut y) = from;
        loop {
            self.put(color, x, y);
            if x == to.0 && y == to.1 {
                break;
            }

            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x += sx;
            }
            if e2 < dy {
                err += dx;
                y += sy;
            }
        }
    }
}

pub struct Dummy;

impl Dummy {
    #[inline(always)]
    pub fn new() -> Dummy { Dummy }
}

impl Display for Dummy {
    #[inline(always)]
    fn is_available() -> bool { false }
    #[inline(always)]
    fn log(&self) {}
    #[inline(always)]
    fn resolution(&self) -> (DisplaySize, DisplaySize) { (0, 0) }
    #[inline(always)]
    fn put(&self, _color: Color, _x: DisplaySize, _y: DisplaySize) {}

    #[inline(always)]
    fn horizontal_line(&self, _color: Color, _range: Range<DisplaySize>, _y: DisplaySize) {}
    #[inline(always)]
    fn fill(&self, _color: Color, _rect: (DisplaySize, DisplaySize, DisplaySize, DisplaySize)) {}
    #[inline(always)]
    fn clear(&self, _color: Color) {}
    #[inline(always)]
    fn line(&self, _color: Color, _from: (DisplaySize, DisplaySize), _to: (DisplaySize, DisplaySize)) {}
}

