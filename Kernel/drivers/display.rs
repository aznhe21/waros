use arch;
use core::fmt;
use core::ops::Range;
use core::ptr::Shared;
use alloc::boxed::Box;

/// 24ビットRGBを表す。
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct Color {
    pub red:   u8,
    pub green: u8,
    pub blue:  u8
}

impl Color {
    pub const BLACK:       Color = Color { red: 0x00, green: 0x00, blue: 0x00 };
    pub const RED:         Color = Color { red: 0xFF, green: 0x00, blue: 0x00 };
    pub const LIME:        Color = Color { red: 0x00, green: 0xFF, blue: 0x00 };
    pub const YELLOW:      Color = Color { red: 0xFF, green: 0xFF, blue: 0x00 };
    pub const BLUE:        Color = Color { red: 0x00, green: 0x00, blue: 0xFF };
    pub const PURPLE:      Color = Color { red: 0xFF, green: 0x00, blue: 0xFF };
    pub const CYAN:        Color = Color { red: 0x00, green: 0xFF, blue: 0xFF };
    pub const WHITE:       Color = Color { red: 0xFF, green: 0xFF, blue: 0xFF };
    pub const GRAY:        Color = Color { red: 0xC6, green: 0xC6, blue: 0xC6 };
    pub const DARK_RED:    Color = Color { red: 0x84, green: 0x00, blue: 0x00 };
    pub const GREEN:       Color = Color { red: 0x00, green: 0x84, blue: 0x00 };
    pub const DARK_YELLOW: Color = Color { red: 0x84, green: 0x84, blue: 0x00 };
    pub const DARK_BLUE:   Color = Color { red: 0x00, green: 0x00, blue: 0x84 };
    pub const DARK_PURPLE: Color = Color { red: 0x84, green: 0x00, blue: 0x84 };
    pub const DARK_CYAN:   Color = Color { red: 0x00, green: 0x84, blue: 0x84 };
    pub const DARK_GRAY:   Color = Color { red: 0x84, green: 0x84, blue: 0x84 };

    #[inline(always)]
    pub const fn new(red: u8, green: u8, blue: u8) -> Color {
        Color {
            red:   red,
            green: green,
            blue:  blue
        }
    }

    #[inline(always)]
    pub const fn from_c15(color: u16) -> Color {
        Color {
            red:   (color >> 10) as u8 * (1 << (8 - 5)),
            green: (color >>  5) as u8 * (1 << (8 - 5)),
            blue:  (color >>  0) as u8 * (1 << (8 - 5))
        }
    }

    #[inline(always)]
    pub const fn from_c16(color: u16) -> Color {
        Color {
            red:   (color >> 10) as u8 * (1 << (8 - 5)),
            green: (color >>  5) as u8 * (1 << (8 - 6)),
            blue:  (color >>  0) as u8 * (1 << (8 - 5))
        }
    }

    #[inline(always)]
    pub const fn from_c32(color: u32) -> Color {
        Color {
            red:   (color >> 16 & 0xFF) as u8,
            green: (color >>  8 & 0xFF) as u8,
            blue:  (color >>  0 & 0xFF) as u8
        }
    }

    /// `Color`を8ビットにより表す。
    #[inline]
    pub fn as_c8(&self) -> u8 {
        (self.red & 0x03) | (self.green & 0x18) | (self.blue & 0xE0)
    }

    /// `Color`を15ビットにより表す。
    #[inline]
    pub fn as_c15(&self) -> u16 {
        (self.red   as u16 / (1 << (8 - 5))) << 10 | // 5-bits
        (self.green as u16 / (1 << (8 - 5))) <<  5 | // 5-bits
        (self.blue  as u16 / (1 << (8 - 5)))         // 5-bits
    }

    /// `Color`を16ビットにより表す。
    #[inline]
    pub fn as_c16(&self) -> u16 {
        (self.red   as u16 / (1 << (8 - 5))) << 10 | // 5-bits
        (self.green as u16 / (1 << (8 - 6))) <<  5 | // 6-bits
        (self.blue  as u16 / (1 << (8 - 5)))         // 5-bits
    }

    /// `Color`を24ビットにより表す。
    #[inline]
    pub fn as_c24(&self) -> (u8, u8, u8) {
        (self.blue, self.green, self.red)
    }

    /// `Color`を32ビットにより表す。
    #[inline]
    pub fn as_c32(&self) -> u32 {
        (self.red   as u32) << 16 |
        (self.green as u32) <<  8 |
        (self.blue  as u32)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:06x}", self.as_c32())
    }
}

pub type DisplaySize = i32;

pub trait Display {
    /// ディスプレイに関するログを出す。
    fn log(&self) {
        let (width, height) = self.resolution();
        log!("Display: {}x{}", width, height);
    }

    /// 解像度を返す。
    fn resolution(&self) -> (DisplaySize, DisplaySize);

    /// ピクセル単位で色を描く。
    fn put(&self, color: Color, x: DisplaySize, y: DisplaySize);

    /// 水平な線を描く。
    fn horizontal_line(&self, color: Color, range: Range<DisplaySize>, y: DisplaySize) {
        for x in range {
            self.put(color, x, y);
        }
    }

    /// 指定したx, y, w, hの範囲内に矩形を描く。
    fn fill(&self, color: Color, rect: (DisplaySize, DisplaySize, DisplaySize, DisplaySize)) {
        for y in rect.1 .. rect.1 + rect.3 {
            self.horizontal_line(color, rect.0 .. rect.0 + rect.2, y);
        }
    }

    /// ディスプレイ全体をクリアする。
    fn clear(&self, color: Color) {
        let res = self.resolution();
        self.fill(color, (0, 0, res.0, res.1));
    }

    /// 線を描画する。
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

/// 画面のない環境向けのダミー実装。
pub struct Dummy;

impl Dummy {
    #[inline(always)]
    pub fn new() -> Dummy { Dummy }

    #[inline(always)]
    pub fn is_available() -> bool { false }
}

impl Display for Dummy {
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

static mut display: Option<Shared<Display>> = None;

/// アーキテクチャに適した`Display`への参照を返す。
/// 未設定の場合は新たに設定される。
pub fn preferred() -> &'static Display {
    unsafe {
        match display {
            Some(ptr) => &**ptr,
            None => {
                let ptr = Shared::new(Box::into_raw(arch::drivers::display::preferred()));
                display = Some(ptr);
                &**ptr
            }
        }
    }
}

/// 現在設定されている`Display`を返す。
#[inline]
pub fn get() -> Option<&'static Display> {
    unsafe {
        display.map(|ptr| &**ptr)
    }
}

/// 新しく`Display`を設定する。
/// 既に設定されている`Display`は破棄される。
#[inline]
pub fn set(value: Box<Display>) {
    unsafe {
        if let Some(ptr) = display {
            Box::from_raw(*ptr);
        }
        display = Some(Shared::new(Box::into_raw(value)));
    }
}

