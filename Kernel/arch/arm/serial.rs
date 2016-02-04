#![allow(dead_code)]

use rt::Register;
use arch::page::PageTable;
use super::mach::UART0_BASE;
use super::gpio::{GPPUD, GPFSEL1, GPPUDCLK0};

const UART0_DR:     Register<u32> = UART0_BASE.offset(0x00);
const UART0_RSRECR: Register<u32> = UART0_BASE.offset(0x04);
const UART0_FR:     Register<u32> = UART0_BASE.offset(0x18);
const UART0_ILPR:   Register<u32> = UART0_BASE.offset(0x20);
const UART0_IBRD:   Register<u32> = UART0_BASE.offset(0x24);
const UART0_FBRD:   Register<u32> = UART0_BASE.offset(0x28);
const UART0_LCRH:   Register<u32> = UART0_BASE.offset(0x2C);
const UART0_CR:     Register<u32> = UART0_BASE.offset(0x30);
const UART0_IFLS:   Register<u32> = UART0_BASE.offset(0x34);
const UART0_IMSC:   Register<u32> = UART0_BASE.offset(0x38);
const UART0_RIS:    Register<u32> = UART0_BASE.offset(0x3C);
const UART0_MIS:    Register<u32> = UART0_BASE.offset(0x40);
const UART0_ICR:    Register<u32> = UART0_BASE.offset(0x44);
const UART0_DMACR:  Register<u32> = UART0_BASE.offset(0x48);
const UART0_ITCR:   Register<u32> = UART0_BASE.offset(0x80);
const UART0_ITIP:   Register<u32> = UART0_BASE.offset(0x84);
const UART0_ITOP:   Register<u32> = UART0_BASE.offset(0x88);
const UART0_TDR:    Register<u32> = UART0_BASE.offset(0x8C);

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
    // Disable UART0
    UART0_CR.store(0);

    let mut ra = GPFSEL1.load();
    ra &= !(7 << 12);// gpio14
    ra |= 4 << 12;   // alt0
    ra &= !(7 << 15);// gpio15
    ra |= 4 << 15;   // alt0
    GPFSEL1.store(ra);

    // Disable pull up/down for all GPIO pins & delay for 150 cycles
    GPPUD.store(0);
    delay!(150);

    // Disable pull up/down for pin 14,15 & delay for 150 cycles.
    GPPUDCLK0.store(1 << 14 | 1 << 15);
    delay!(150);

    // Write 0 to GPPUDCLK0 to make it take effect.
    GPPUDCLK0.store(0);

    // Clear pending interrupts.
    UART0_ICR.store(0x7FF);

    // Set integer & fractional part of baud rate.
    // Divider = UART_CLOCK/(16 * Baud)
    // Fraction part register = (Fractional part * 64) + 0.5
    // UART_CLOCK = 3000000; Baud = 115200.

    // Divider = 3000000 / (16 * 115200) = 1.627 = ~1.
    // Fractional part register = (.627 * 64) + 0.5 = 40.6 = ~40.
    UART0_IBRD.store(1);
    UART0_FBRD.store(40);

    // Enable FIFO & 8 bit data transmissio (1 stop bit, no parity).
    UART0_LCRH.store(1 << 4 | 1 << 5 | 1 << 6);

    // Mask all interrupts.
    /*UART0_IMSC.store(1 << 1 | 1 << 4 | 1 << 5 | 1 << 6 |
                     1 << 7 | 1 << 8 | 1 << 9 | 1 << 10);*/

    // Enable UART0, receive & transfer part of UART.
    UART0_CR.store(1 << 0 | 1 << 8 | 1 << 9);
}

#[inline(always)]
pub fn map_pages(table: &mut PageTable) {
    table.map_direct(PageTable::FLAGS_KERNEL,
                     UART0_DR.addr(), (UART0_TDR - UART0_DR) as usize);
}

#[inline]
pub fn is_transmit_empty() -> bool {
    UART0_FR.load() & (1 << 5) == 0
}

#[inline]
pub fn is_receive_empty() -> bool {
    UART0_FR.load() & (1 << 4) != 0
}

/// Write a string to the output channel
///
/// This method is unsafe because it does port accesses without synchronisation
#[inline]
pub unsafe fn puts(s: &str) {
    for b in s.bytes() {
        putb(b);
    }
}

/// Write a single byte to the output channel
///
/// This method is unsafe because it does port accesses without synchronisation
pub unsafe fn putb(b: u8) {
    if b == b'\n' {
        putb(b'\r');
    }
    // Wait for the serial port's fifo to not be empty
    while !is_transmit_empty() {
        // Do nothing
    }
    // Send the byte out the serial port
    UART0_DR.store(b as u32);
}

pub unsafe fn getc() -> u8 {
    while is_receive_empty() {
    }

    UART0_DR.load() as u8
}

