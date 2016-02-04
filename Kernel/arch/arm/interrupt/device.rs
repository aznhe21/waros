#[derive(Clone, Copy)]
pub enum Device {
    KeyDown(u8),
    KeyUp(u8),
    Mouse(())
}

#[inline]
pub unsafe fn init() {
}

