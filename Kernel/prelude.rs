/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * prelude.rs
 * - Definitions meant to be used in every module
 *
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */
pub use rt::ptr::{UOffset, UOffsetMut};
pub use rt::option::UnsafeOption;
pub use rt::iter::IterHelper;
pub use rt::slice::SliceHelper;

