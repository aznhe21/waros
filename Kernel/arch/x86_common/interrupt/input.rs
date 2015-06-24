use arch::x86_io::inb;
pub use self::keyboard::Key;
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

mod keyboard {
    use prelude::*;
    use arch::x86_io::{outb, inb};
    use collections::queue::Queue;

    pub enum Key {
        Down(u8),
        Up(u8)
    }

    static mut queue: Queue<'static, u8> = Queue { data: &mut [0; 64], read: 0, write: 0 };

    pub fn init() {
        unsafe {
            super::super::idt::set_handler(1, keyboard_handler);
        }
    }

    fn keyboard_handler(_irq: u32) {
        unsafe {
            outb(0x20, 0x61);
            let data = inb(0x60);
            if data != 0xFA {
                queue.push(data);
            }
        }
    }

    pub fn event() -> Option<Key> {
        unsafe {
            /*if queue.peek(0).unwrap_or(0) == 0xE0 {
             * special key
            }*/
            queue.pop().map(|code| {
                if code & 0x80 == 0 {
                    Key::Down(code)
                } else {
                    Key::Up(code & 0x7F)
                }
            })
        }
    }
}

mod mouse {
    use prelude::*;
    use arch::x86_io::{outb, inb};
    use collections::queue::Queue;

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

    static mut queue: Queue<'static, u8> = Queue { data: &mut [0; 64], read: 0, write: 0 };

    pub fn init() {
        unsafe {
            if !init_mouse() {
                panic!("Failed to initialize the mouse");
            }

            super::super::idt::set_handler(12, mouse_handler);
        }
    }

    fn mouse_handler(_irq: u32) {
        unsafe {
            outb(0xA0, 0x64);
            outb(0x20, 0x62);
            queue.push(inb(0x60));
        }
    }

    pub fn event() -> Option<Mouse> {
        unsafe {
            while queue.peek(0).map_or(false, |data| data & 0x08 == 0) {
                queue.pop();
            }

            if queue.len() >= 3 {
                let flags = queue.pop().unwrap();
                let x = queue.pop().unwrap() as i8 as i32;
                let y = queue.pop().unwrap() as i8 as i32;
                Some(Mouse {
                    x: x,
                    y: -y,
                    left: flags & 0x01 != 0,
                    middle: false,
                    right: flags & 0x02 != 0
                })
            } else {
                None
            }
        }
    }
}

pub enum Event {
    Keyboard(Key),
    Mouse(Mouse),
    None
}

pub fn init() {
    super::pic::disable_irq(12);// Mouse
    super::pic::disable_irq(1);// Keyboard

    keyboard::init();
    mouse::init();

    super::pic::enable_irq(1);// Keyboard
    super::pic::enable_irq(12);// Mouse
}

pub fn get() -> Event {
    keyboard::event().map_or_else(
        || mouse::event().map_or(Event::None, Event::Mouse),
        Event::Keyboard
    )
}

