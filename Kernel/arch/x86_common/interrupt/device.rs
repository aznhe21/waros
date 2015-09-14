use arch::x86_io::inb;
use event::EventQueue;
use super::pic::IRQ;
pub use self::mouse::Mouse;

const TIMEOUT: usize = 50000;

unsafe fn wait_for_read() -> bool {
    for _ in 0..TIMEOUT {
        if inb(0x64) & 1 != 0 {
            return true;
        }
    }
    false
}

unsafe fn wait_for_write() -> bool {
    for _ in 0..TIMEOUT {
        if inb(0x64) & 3 == 0 {
            return true;
        }
        inb(0x60);
    }
    false
}

static mut queue: Option<&'static mut EventQueue> = None;

mod keyboard {
    use prelude::*;
    use arch::x86_io::inb;
    use event::Event;
    use super::super::idt;
    use super::super::pic::IRQ;
    use super::Device::{KeyDown, KeyUp};

    pub fn init() {
        unsafe {
            idt::set_handler(IRQ::Keyboard, keyboard_handler);
        }
    }

    fn keyboard_handler(_irq: IRQ) {
        IRQ::Keyboard.eoi();
        unsafe {
            let key = match inb(0x60) {
                0xFA => return,
                // 0xE0 => special key
                code if code & 0x80 == 0 => KeyDown(code),
                code                     => KeyUp(code & 0x7F),
            };
            super::queue.as_mut().be_some().push(Event::Device(key));
        }
    }
}

mod mouse {
    use prelude::*;
    use arch::x86_io::{outb, inb};
    use event::Event;
    use super::super::idt;
    use super::super::pic::IRQ;

    #[derive(Clone, Copy)]
    pub struct Mouse {
        pub x: i32,
        pub y: i32,
        pub left: bool,
        pub middle: bool,
        pub right: bool
    }

    #[inline(always)]
    unsafe fn init_mouse() -> bool {
        outb(0x64, 0x20);
        if !super::wait_for_read() {
            return false;
        }
        let data = inb(0x60);

        outb(0x64, 0x60);
        if !super::wait_for_write() {
            return false;
        }

        outb(0x60, data & !0x30 | 0x03);
        if super::wait_for_read() {
            inb(0x60);
        }

        outb(0x64, 0xD4);
        if !super::wait_for_write() {
            return false;
        }

        outb(0x60, 0xF4);
        if !super::wait_for_read() {
            return false;
        }

        inb(0x60);
        true
    }

    enum Stage {
        First,
        Second(u8),
        Third(u8, i8),
    }

    static mut stage: Stage = Stage::First;

    pub fn init() {
        unsafe {
            if !init_mouse() {
                panic!("Failed to initialize the mouse");
            }

            idt::set_handler(IRQ::Mouse, mouse_handler);
        }
    }

    fn mouse_handler(_irq: IRQ) {
        IRQ::Mouse.eoi();
        unsafe {
            let data = inb(0x60);

            stage = match stage {
                Stage::First if data & 0x08 == 0 => Stage::First,
                Stage::First                     => Stage::Second(data),
                Stage::Second(flags)             => Stage::Third(flags, data as i8),
                Stage::Third(flags, x)           => {
                    let y = data as i8;
                    super::queue.as_mut().be_some().push(Event::Device(super::Device::Mouse(Mouse {
                        x: x as i32,
                        y: -y as i32,
                        left: flags & 0x01 != 0,
                        middle: false,// TODO
                        right: flags & 0x02 != 0,
                    })));
                    Stage::First
                }
            };
        }
    }
}

#[derive(Clone, Copy)]
pub enum Device {
    KeyDown(u8),
    KeyUp(u8),
    Mouse(Mouse)
}

#[inline]
pub fn init(q: &'static mut EventQueue) {
    IRQ::Mouse.disable();
    IRQ::Keyboard.disable();

    unsafe {
        queue = Some(q);
    }
    keyboard::init();
    mouse::init();

    IRQ::Keyboard.enable();
    IRQ::Mouse.enable();
}

