/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * arch/x86/debug.rs
 * - Debug output channel
 *
 * Writes debug to the standard PC serial port (0x3F8 .. 0x3FF)
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

#[inline]
pub unsafe fn is_transmit_empty() -> bool {
    super::x86_io::inb(0x3F8+5) & 0x20 != 0
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
    // Wait for the serial port's fifo to not be empty
    while !is_transmit_empty() {
        // Do nothing
    }
    // Send the byte out the serial port
    super::x86_io::outb(0x3F8, b);

    // Also send to the bochs 0xE9 hack
    super::x86_io::outb(0xE9, b);
}

