pub mod display;

#[inline(always)]
pub fn pre_init() {
    display::pre_init();
}

#[inline(always)]
pub fn init() {
    display::init();
}

