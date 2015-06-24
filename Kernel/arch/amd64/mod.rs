/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * arch/amd64/mod.rs
 * - Top-level file for amd64 architecture
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

// x86 port IO
#[path = "../x86_common/io.rs"]
mod x86_io;

// Serial output channel
#[path = "../x86_common/serial.rs"]
pub mod serial;

#[path = "../x86_common/interrupt/mod.rs"]
pub mod interrupt;

