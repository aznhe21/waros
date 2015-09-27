pub use self::memory::*;
pub use self::utils::*;

pub mod ptr;
pub mod option;
pub mod iter;
mod memory;
mod utils;

#[cfg(target_arch="x86")]
pub mod divmod64;

