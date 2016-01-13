use rt::{Force, ForceRef};
use lists::RingBuffer;
use drivers;
use timer;
use core::ops::{Deref, DerefMut};

pub enum Event {
    Timer(timer::TimerId),
    Device(drivers::Device)
}

pub struct EventQueue(RingBuffer<'static, Event>);

unsafe impl Send for EventQueue { }
unsafe impl Sync for EventQueue { }

impl Deref for EventQueue {
    type Target = RingBuffer<'static, Event>;
    fn deref(&self) -> &RingBuffer<'static, Event> {
        &self.0
    }
}

impl DerefMut for EventQueue {
    fn deref_mut(&mut self) -> &mut RingBuffer<'static, Event> {
        &mut self.0
    }
}

static Q: Force<EventQueue> = Force::new();
static Q_BUF: Force<[Event; 128]> = Force::new();

#[inline]
pub fn init() {
    let slice = ForceRef::as_mut(Q_BUF.as_ref());
    *Q.setup() = EventQueue(RingBuffer::new(slice));
}

#[inline(always)]
pub fn queue() -> ForceRef<EventQueue> {
    Q.as_ref()
}

