use drivers::display::{Display, Dummy};
use alloc::boxed::Box;

#[inline(always)]
pub unsafe fn preferred() -> Box<Display> {
    Box::new(Dummy::new())
}

