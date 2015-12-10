use arch::interrupt;

#[must_use]
pub struct IntBlocker {
    state: usize
}

impl IntBlocker {
    #[inline]
    pub fn new() -> IntBlocker {
        IntBlocker {
            state: interrupt::stop()
        }
    }
}

impl Drop for IntBlocker {
    #[inline]
    fn drop(&mut self) {
        interrupt::restore(self.state);
    }
}

