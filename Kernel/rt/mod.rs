pub use self::memory::*;

pub mod ptr;
mod memory;

#[cfg(target_arch="x86")]
pub mod divmod64;

