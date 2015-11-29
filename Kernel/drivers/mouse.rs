pub struct Mouse {
    pub buttons: [bool; 8],// TODO: bitfields
    pub x: i8,
    pub y: i8
}

impl Mouse {
    #[inline]
    pub const fn new() -> Mouse {
        Mouse {
            buttons: [false; 8],
            x: 0,
            y: 0
        }
    }

    #[inline]
    pub const fn with_bits(bits: u8, x: i8, y: i8) -> Mouse {
        Mouse {
            buttons: [
                bits & (1 << 0) != 0,
                bits & (1 << 1) != 0,
                bits & (1 << 2) != 0,
                bits & (1 << 3) != 0,
                bits & (1 << 4) != 0,
                bits & (1 << 5) != 0,
                bits & (1 << 6) != 0,
                bits & (1 << 7) != 0,
            ],
            x: x,
            y: y
        }
    }

    #[inline]
    pub fn left(&self) -> bool {
        self.buttons[0]
    }

    #[inline]
    pub fn right(&self) -> bool {
        self.buttons[1]
    }

    #[inline]
    pub fn middle(&self) -> bool {
        self.buttons[2]
    }
}

