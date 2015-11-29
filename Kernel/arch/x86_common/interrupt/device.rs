use arch::x86_io::inb;
use super::pic::IRQ;

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
    use arch::x86_io::inb;
    use event::{self, Event};
    use drivers::Device;
    use drivers::keyboard::{Keyboard, KeyState};
    use super::super::idt;
    use super::super::pic::IRQ;

    static mut state: KeyState = KeyState::new();

    pub fn init() {
        unsafe {
            idt::set_handler(IRQ::Keyboard, keyboard_handler);
        }
    }

    fn keyboard_handler(_irq: IRQ) {
        IRQ::Keyboard.eoi();
        unsafe {
            let prev_code = state.code;
            state.code = inb(0x60) as u16;
            let key = match state.code {
                0xFA => return,
                // 0xE0 => special key
                code if code & 0x80 == 0 && code & 0x7F == prev_code => Keyboard::Press(state.clone()),
                code if code & 0x80 == 0                             => Keyboard::Down(state.clone()),
                _                                                    => Keyboard::Up(state.clone()),
            };
            *event::queue().emplace_back() = Event::Device(Device::Keyboard(key));
        }
    }
}

mod mouse {
    use arch::x86_io::{outb, inb};
    use event::{self, Event};
    use drivers::Device;
    use drivers::mouse::Mouse;
    use super::super::idt;
    use super::super::pic::IRQ;

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
                    let mouse = Mouse::with_bits(flags, x as i8, -y);
                    *event::queue().emplace_back() = Event::Device(Device::Mouse(mouse));
                    Stage::First
                }
            };
        }
    }
}

#[inline]
pub fn init() {
    IRQ::Mouse.disable();
    IRQ::Keyboard.disable();

    keyboard::init();
    mouse::init();

    IRQ::Keyboard.enable();
    IRQ::Mouse.enable();
}

