use arch::interrupt::device;
use lists::FixedQueue;
use timer;

#[derive(Clone, Copy)]
pub enum Event {
    Timer(timer::TimerId),
    Device(device::Device),
    None
}

pub type EventQueue = FixedQueue<'static, Event>;

static mut q: EventQueue = FixedQueue::new(&mut [Event::None; 128]);

impl Event {
    #[inline]
    pub fn get() -> Event {
        unsafe { q.pop().unwrap_or(Event::None) }
    }
}

#[inline(always)]
pub fn queue() -> &'static mut EventQueue {
    unsafe {
        &mut q
    }
}

