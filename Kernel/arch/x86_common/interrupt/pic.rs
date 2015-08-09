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

#[inline]
pub fn enable_irq(irq: u8) {
    match irq {
        0...7  => {
            enable_port(PORT_MASTER_PIC_IMR, 1 << irq);
        },
        8...16 => {
            enable_port(PORT_SLAVE_PIC_IMR, 1 << (irq - 8));
            enable_port(PORT_MASTER_PIC_IMR, 1 << 2);
        },
        _ => {
        }
    }
}

#[inline]
pub fn disable_irq(irq: u8) {
    match irq {
        0...7  => disable_port(PORT_MASTER_PIC_IMR, 1 << irq),
        8...16 => disable_port(PORT_SLAVE_PIC_IMR, 1 << (irq - 8)),
        _ => {}
    }
}

#[inline]
pub fn eoi(irq: u8) {
    unsafe {
        match irq {
            0...7 => outb(PORT_MASTER_PIC_COMMAND, 0x60 | irq),
            8...16 => {
                outb(PORT_SLAVE_PIC_COMMAND, 0x60 | (irq - 8));
                outb(PORT_MASTER_PIC_COMMAND, 0x62);
            },
            _ => {}
        }
    }
}

