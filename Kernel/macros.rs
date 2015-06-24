/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * macros.rs
 * - Macros used by the kernel
 *
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

/// A very primitive logging macro
///
/// Obtaines a logger instance (locking the log channel) with the current module name passed
/// then passes the standard format! arguments to it
macro_rules! log {
    ($($arg:tt)*) => ({
        // Import the Writer trait (required by write!)
        use core::fmt::Write;
        let _ = write!(&mut ::logging::Writer::get(module_path!()), $($arg)*);
    })
}

macro_rules! debug_log {
    ($($arg:tt)*) => (if cfg!(debug_assertions) { log!($($arg)*); })
}

/*
 * int!(0x00, < { "eax" = hoge }, > { hoge = "eax" });
 */
macro_rules! int {
    ( $no:expr ) => {
        asm!(concat!("int $", $no));
    };
    ( $no:expr, < { $($reg1:tt = $val1:expr),* }, > { $($val2:ident = $reg2:tt),* }) => ({
        $(
            asm!(concat!("mov $0, %", $reg1) :: "i"($val1) :: "volatile");
        )*
        asm!(concat!("int $$", $no) :::: "volatile");
        //asm!("mov %al, $0" : "=r"(ax) ::: "volatile");
        $(
            asm!(concat!("mov $0, %", $reg2) : "=r"($val2) ::: "volatile");
        )*
    });
}

