pub use self::allocator::{KCacheManager, KCacheAllocator, init, manager};
pub use self::boxed::KCBox;
pub use self::rc::KCRc;

mod allocator;
mod boxed;
mod rc;

pub trait RefCount {
    fn add_ref(&mut self);
    fn rel_ref(&mut self) -> usize;
}

