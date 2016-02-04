use rt::Register;
use arch::page::PageTable;
use super::mach::GPIO_BASE;

pub const GPFSEL0:      Register<u32> = GPIO_BASE.offset(0x00);
pub const GPFSEL1:      Register<u32> = GPIO_BASE.offset(0x04);
pub const GPFSEL2:      Register<u32> = GPIO_BASE.offset(0x08);
pub const GPFSEL3:      Register<u32> = GPIO_BASE.offset(0x0C);
pub const GPFSEL4:      Register<u32> = GPIO_BASE.offset(0x10);
pub const GPFSEL5:      Register<u32> = GPIO_BASE.offset(0x14);

pub const GPSET0:       Register<u32> = GPIO_BASE.offset(0x1C);
pub const GPSET1:       Register<u32> = GPIO_BASE.offset(0x20);

pub const GPCLR0:       Register<u32> = GPIO_BASE.offset(0x28);
pub const GPCLR1:       Register<u32> = GPIO_BASE.offset(0x2C);

pub const GPLEV0:       Register<u32> = GPIO_BASE.offset(0x34);
pub const GPLEV1:       Register<u32> = GPIO_BASE.offset(0x38);

pub const GPEDS0:       Register<u32> = GPIO_BASE.offset(0x40);
pub const GPEDS1:       Register<u32> = GPIO_BASE.offset(0x44);

pub const GPREN0:       Register<u32> = GPIO_BASE.offset(0x4C);
pub const GPREN1:       Register<u32> = GPIO_BASE.offset(0x50);

pub const GPFEN0:       Register<u32> = GPIO_BASE.offset(0x58);
pub const GPFEN1:       Register<u32> = GPIO_BASE.offset(0x5C);

pub const GPHEN0:       Register<u32> = GPIO_BASE.offset(0x64);
pub const GPHEN1:       Register<u32> = GPIO_BASE.offset(0x68);

pub const GPLEN0:       Register<u32> = GPIO_BASE.offset(0x70);
pub const GPLEN1:       Register<u32> = GPIO_BASE.offset(0x74);

pub const GPAREN0:      Register<u32> = GPIO_BASE.offset(0x7C);
pub const GPAREN1:      Register<u32> = GPIO_BASE.offset(0x80);

pub const GPAFEN0:      Register<u32> = GPIO_BASE.offset(0x88);
pub const GPAFEN1:      Register<u32> = GPIO_BASE.offset(0x8C);

// Controls actuation of pull up/down to ALL GPIO pins.
pub const GPPUD:        Register<u32> = GPIO_BASE.offset(0x94);

// Controls actuation of pull up/down for specific GPIO pin.
pub const GPPUDCLK0:    Register<u32> = GPIO_BASE.offset(0x98);
pub const GPPUDCLK1:    Register<u32> = GPIO_BASE.offset(0x9C);

pub const TEST:         Register<u32> = GPIO_BASE.offset(0xB0);

macro_rules! delay {
    ($count: expr) => {
        asm!(concat!(
             "   mov  r1, #", $count, "
              1: subs r1, r1, #1
                 bne  1b") ::: "r1", "cc" : "volatile");
    }
}

#[inline]
pub unsafe fn pre_init() {
    // Pullup all
    GPPUD.store(0x02);
    delay!(150);

    GPPUDCLK0.store(0xFFFFFFFF);
    GPPUDCLK1.store(0xFFFFFFFF);
    delay!(150);

    GPPUDCLK0.store(0);
    GPPUDCLK1.store(0);

    GPFSEL0.store(0);
    GPFSEL1.store(0);
    GPFSEL2.store(0);
    GPFSEL3.store(0);
    GPFSEL4.store(0);
    GPFSEL5.store(0);
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    Input  = 0b000,
    Output = 0b001,
    Alt0   = 0b100,
    Alt1   = 0b101,
    Alt2   = 0b110,
    Alt3   = 0b111,
    Alt4   = 0b011,
    Alt5   = 0b010
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Low,
    High
}

impl Level {
    #[inline(always)]
    pub fn is_low(self) -> bool {
        match self {
            Level::Low  => true,
            Level::High => false
        }
    }

    #[inline(always)]
    pub fn is_high(self) -> bool {
        !self.is_low()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventDetectionType {
    NoDetect,
    RisingEdge,
    FallingEdge,
    Transition,
    High,
    Low,
    AsyncRisingEdge,
    AsyncFallingEdge
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PullUpDown {
    Disable = 0b00,
    PullDown = 0b01,
    PullUp = 0b10
}

pub struct Channel(u8);

macro_rules! regs_shift {
    ($this:expr, ($($reg0s:expr),*), ($($reg1s:expr),*)) => {
        match $this.0 {
            0  ... 31 => ($($reg0s, )* $this.0),
            32 ... 53 => ($($reg1s, )* $this.0 - 32),
            _ => unreachable!()
        }
    };
}

impl Channel {
    pub fn with_mode(channel: u8, mode: Mode) -> Channel {
        let (reg, bit) = match channel {
            0  ... 9  => (GPFSEL0, channel),
            10 ... 19 => (GPFSEL1, channel - 10),
            20 ... 29 => (GPFSEL2, channel - 20),
            30 ... 39 => (GPFSEL3, channel - 30),
            40 ... 49 => (GPFSEL4, channel - 40),
            50 ... 53 => (GPFSEL5, channel - 50),
            _ => panic!("Invalid GPIO channel")
        };

        let shift = bit * 3;
        reg.store(reg.load() & !(0b111 << shift) | (mode as u32) << shift);

        Channel(channel)
    }

    #[inline(always)]
    pub fn with_input(channel: u8) -> Channel {
        Channel::with_mode(channel, Mode::Input)
    }

    #[inline(always)]
    pub fn with_output(channel: u8) -> Channel {
        Channel::with_mode(channel, Mode::Output)
    }

    pub fn set(&self) {
        let (reg, shift) = regs_shift!(self, (GPSET0), (GPSET1));
        reg.store(1 << shift);
    }

    pub fn clear(&self) {
        let (reg, shift) = regs_shift!(self, (GPCLR0), (GPCLR1));
        reg.store(1 << shift);
    }

    pub fn level(&self) -> Level {
        let (reg, shift) = regs_shift!(self, (GPLEV0), (GPLEV1));

        if reg.load() & (1 << shift) == 0 {
            Level::Low
        } else {
            Level::High
        }
    }

    pub fn event_status(&self) -> bool {
        let (reg, shift) = regs_shift!(self, (GPEDS0), (GPEDS1));
        reg.load() & (1 << shift) == 1
    }

    pub fn clear_event(&self) {
        let (reg, shift) = regs_shift!(self, (GPEDS0), (GPEDS1));
        reg.store(reg.load() | (1 << shift));
    }

    pub fn set_event_type(&self, event: EventDetectionType) {
        match event {
            EventDetectionType::NoDetect => {
                let (reg_ren, reg_fen, reg_hen, reg_len, reg_aren, reg_afen, shift) = regs_shift!(
                    self,
                    (GPREN0, GPFEN0, GPHEN0, GPLEN0, GPAREN0, GPAFEN0),
                    (GPREN1, GPFEN1, GPHEN1, GPLEN1, GPAREN1, GPAFEN1)
                );
                reg_ren.store(reg_ren.load() & !(1 << shift));
                reg_fen.store(reg_fen.load() & !(1 << shift));
                reg_hen.store(reg_hen.load() & !(1 << shift));
                reg_len.store(reg_len.load() & !(1 << shift));
                reg_aren.store(reg_aren.load() & !(1 << shift));
                reg_afen.store(reg_afen.load() & !(1 << shift));
            },
            EventDetectionType::RisingEdge => {
                let (reg, shift) = regs_shift!(self, (GPREN0), (GPREN1));
                reg.store(reg.load() | (1 << shift));
            },
            EventDetectionType::FallingEdge => {
                let (reg, shift) = regs_shift!(self, (GPFEN0), (GPFEN1));
                reg.store(reg.load() | (1 << shift));
            },
            EventDetectionType::Transition => {
                let (reg_ren, reg_fen, shift) = regs_shift!(self, (GPREN0, GPFEN0), (GPREN1, GPFEN1));
                reg_ren.store(reg_ren.load() | (1 << shift));
                reg_fen.store(reg_fen.load() | (1 << shift));
            },
            EventDetectionType::High => {
                let (reg, shift) = regs_shift!(self, (GPHEN0), (GPHEN1));
                reg.store(reg.load() | (1 << shift));
            },
            EventDetectionType::Low => {
                let (reg, shift) = regs_shift!(self, (GPLEN0), (GPLEN1));
                reg.store(reg.load() | (1 << shift));
            },
            EventDetectionType::AsyncRisingEdge => {
                let (reg, shift) = regs_shift!(self, (GPAREN0), (GPAREN1));
                reg.store(reg.load() | (1 << shift));
            },
            EventDetectionType::AsyncFallingEdge => {
                let (reg, shift) = regs_shift!(self, (GPAFEN0), (GPAFEN1));
                reg.store(reg.load() | (1 << shift));
            }
        }
    }

    pub fn set_pull(&self, pud: PullUpDown) {
        unsafe {
            let (reg, shift) = regs_shift!(self, (GPEDS0), (GPEDS1));

            GPPUD.store(pud as u32);
            delay!(150);

            reg.store(reg.load() | (1 << shift));
            delay!(150);

            GPPUD.store(0);
            reg.store(0);
        }
    }
}

#[inline(always)]
pub fn map_pages(table: &mut PageTable) {
    table.map_direct(PageTable::FLAGS_KERNEL,
                     GPFSEL0.addr(), (TEST - GPFSEL0) as usize);
}

