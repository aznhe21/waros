use rt::{Force, ForceRef};
use lists::FixedQueue;
use drivers;
use timer;
use core::ops::{Deref, DerefMut};
use alloc::raw_vec::RawVec;
use alloc::boxed::Box;

pub enum Event {
    Timer(timer::TimerId),
    Device(drivers::Device)
}

pub struct EventQueue(FixedQueue<'static, Event>);

unsafe impl Send for EventQueue { }
unsafe impl Sync for EventQueue { }

impl Deref for EventQueue {
    type Target = FixedQueue<'static, Event>;
    fn deref(&self) -> &FixedQueue<'static, Event> {
        &self.0
    }
}

impl DerefMut for EventQueue {
    fn deref_mut(&mut self) -> &mut FixedQueue<'static, Event> {
        &mut self.0
    }
}

static Q: Force<EventQueue> = Force::new();

impl Event {
    #[inline]
    pub fn get() -> Option<Event> {
        queue().pop()
    }
}

#[inline]
pub fn init() {
    unsafe {
        let len = 128;
        let slice: Box<[Event]> = RawVec::with_capacity(len).into_box();
        *Q.setup() = EventQueue(FixedQueue::new(&mut *Box::into_raw(slice)));
    }
}

#[inline(always)]
pub fn queue() -> ForceRef<EventQueue> {
    Q.as_ref()
}

