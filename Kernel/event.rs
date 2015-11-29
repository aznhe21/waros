use rt::UnsafeOption;
use lists::FixedQueue;
use drivers;
use timer;
use core::mem;
use core::slice;
use alloc::heap;

pub enum Event {
    Timer(timer::TimerId),
    Device(drivers::Device)
}

pub type EventQueue = FixedQueue<'static, Event>;

static mut q_opt: Option<EventQueue> = None;

impl Event {
    #[inline]
    pub fn get() -> Option<Event> {
        unsafe { q_opt.as_mut().be_some().pop() }
    }
}

#[inline]
pub fn init() {
    unsafe {
        let len = 128;
        let ptr = heap::allocate(mem::size_of::<Event>() * len, mem::align_of::<Event>()) as *mut Event;
        let slice = slice::from_raw_parts_mut(ptr, len);
        *q_opt.into_some() = FixedQueue::new(slice);
    }
}

#[inline(always)]
pub fn queue() -> &'static mut EventQueue {
    unsafe {
        q_opt.as_mut().be_some()
    }
}

