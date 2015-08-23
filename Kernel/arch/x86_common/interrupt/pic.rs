use arch::x86_io::{inb, outb};

const PORT_MASTER_PIC_COMMAND: u16 = 0x0020;
const PORT_MASTER_PIC_STATUS:  u16 = 0x0020;
const PORT_MASTER_PIC_DATA:    u16 = 0x0021;
const PORT_MASTER_PIC_IMR:     u16 = 0x0021;

const PORT_SLAVE_PIC_COMMAND: u16 = 0x00A0;
const PORT_SLAVE_PIC_STATUS:  u16 = 0x00A0;
const PORT_SLAVE_PIC_DATA:    u16 = 0x00A1;
const PORT_SLAVE_PIC_IMR:     u16 = 0x00A1;

const PIC_ICW1: u8 = 0x11;

const PIC_MASTER_ICW2: u8 = 0x20;
const PIC_MASTER_ICW3: u8 = 0x04;
const PIC_MASTER_ICW4: u8 = 0x01;

const PIC_SLAVE_ICW2:  u8 = 0x28;
const PIC_SLAVE_ICW3:  u8 = 0x02;
const PIC_SLAVE_ICW4:  u8 = 0x01;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum IRQ {
    PIT = 0,
    Keyboard,
    Cascade,
    COM2,
    COM1,
    LPT2,
    FloppyDisk,
    LPT1,
    CMOSClock,
    Free1,
    Free2,
    Free3,
    Mouse,
    FPU,
    PrimaryHD,
    SecondaryHD
}

#[inline(always)]
pub unsafe fn pre_init() {
    outb(PORT_MASTER_PIC_IMR, 0xFF);
    outb(PORT_SLAVE_PIC_IMR, 0xFF);
}

#[inline]
pub unsafe fn init() {
    // master
    outb(PORT_MASTER_PIC_COMMAND, PIC_ICW1);     // ICW1
    outb(PORT_MASTER_PIC_DATA, PIC_MASTER_ICW2); // ICW2
    outb(PORT_MASTER_PIC_DATA, PIC_MASTER_ICW3); // ICW3
    outb(PORT_MASTER_PIC_DATA, PIC_MASTER_ICW4); // ICW4

    // slave
    outb(PORT_SLAVE_PIC_COMMAND, PIC_ICW1);    // ICW1
    outb(PORT_SLAVE_PIC_DATA, PIC_SLAVE_ICW2); // ICW2
    outb(PORT_SLAVE_PIC_DATA, PIC_SLAVE_ICW3); // ICW3
    outb(PORT_SLAVE_PIC_DATA, PIC_SLAVE_ICW4); // ICW4

    // disable all interrupts
    outb(PORT_MASTER_PIC_IMR, 0xFF);
    outb(PORT_SLAVE_PIC_IMR, 0xFF);
}

#[inline]
fn enable_port(port: u16, mask: u8) {
    unsafe {
        outb(port, inb(port) & !mask);
    }
}

#[inline]
fn disable_port(port: u16, mask: u8) {
    unsafe {
        outb(port, inb(port) | mask);
    }
}

impl IRQ {
    #[inline]
    pub fn is_master(self) -> bool {
        match self {
            IRQ::PIT |
            IRQ::Keyboard |
            IRQ::Cascade |
            IRQ::COM2 |
            IRQ::COM1 |
            IRQ::LPT2 |
            IRQ::FloppyDisk |
            IRQ::LPT1
            => true,

            IRQ::CMOSClock |
            IRQ::Free1 |
            IRQ::Free2 |
            IRQ::Free3 |
            IRQ::Mouse |
            IRQ::FPU |
            IRQ::PrimaryHD |
            IRQ::SecondaryHD
            => false
        }
    }

    #[inline]
    pub fn enable(self) {
        if self.is_master() {
            enable_port(PORT_MASTER_PIC_IMR, 1 << (self as u8));
        } else {
            enable_port(PORT_SLAVE_PIC_IMR, 1 << (self as u8 - 8));
            enable_port(PORT_MASTER_PIC_IMR, 1 << 2);
        }
    }

    #[inline]
    pub fn disable(self) {
        if self.is_master() {
            disable_port(PORT_MASTER_PIC_IMR, 1 << (self as u8));
        } else {
            disable_port(PORT_SLAVE_PIC_IMR, 1 << (self as u8 - 8));
        }
    }

    #[inline]
    pub fn eoi(self) {
        unsafe {
            if self.is_master() {
                outb(PORT_MASTER_PIC_COMMAND, 0x60 | (self as u8));
            } else {
                outb(PORT_SLAVE_PIC_COMMAND, 0x60 | (self as u8 - 8));
                outb(PORT_MASTER_PIC_COMMAND, 0x60 | 0x02);
            }
        }
    }
}

