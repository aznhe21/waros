pub enum Device {
    Keyboard(keyboard::Keyboard),
    Mouse(mouse::Mouse)
}

pub mod keyboard;
pub mod mouse;

pub mod display;

