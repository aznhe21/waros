use rt::UnsafeOption;
use arch::interrupt;
use lists::{LinkedList, SortedList};
use event::{Event, EventQueue};
use core::cmp::Ordering;
use core::iter::FromIterator;
use core::mem;
use core::ptr;
use core::intrinsics;

pub type TimerId = u16;

pub enum TimerHandler {
    Queue(*mut EventQueue),
    Callback(fn(TimerId) -> ())
}

pub struct TimerManager {
    timer_pool: [TimerEntity; 256],
    free_timers: LinkedList<TimerEntity>,
    ticking_timers: SortedList<TimerEntity>,
    counter: usize
}

impl TimerManager {
    pub fn init(&'static mut self) {
        for (i, timer) in self.timer_pool.iter_mut().enumerate() {
            *timer = TimerEntity::new(i as TimerId);
        }
        self.free_timers = LinkedList::from_iter(&mut self.timer_pool[..]);
        self.ticking_timers = SortedList::new(TimerEntity::cmp);
        self.counter = 0;
    }

    fn by_handler(&mut self, handler: TimerHandler) -> Timer {
        let timer = self.free_timers.pop_front().expect("Not enough timers");
        timer.handler = handler;
        Timer(timer.id)
    }

    #[inline]
    pub fn by_queue(&mut self, queue: &'static mut EventQueue) -> Timer {
        self.by_handler(TimerHandler::Queue(queue))
    }

    #[inline]
    pub fn by_callback(&mut self, callback: fn(TimerId) -> ()) -> Timer {
        self.by_handler(TimerHandler::Callback(callback))
    }

    fn remove(&mut self, timer: &'static mut TimerEntity) {
        if self.ticking_timers.contains(timer) {
            self.ticking_timers.remove(timer);
        }
        self.free_timers.push_back(timer);
    }

    pub fn tick(&mut self, count: usize) {
        unsafe {
            self.counter = intrinsics::overflowing_add(self.counter, count);
            let mut callbacks = LinkedList::new();

            loop {
                let timer_ptr = self.ticking_timers.front_ptr();
                if timer_ptr.is_null() || (*timer_ptr).tick > self.counter {
                    break;
                }
                self.ticking_timers.remove(timer_ptr);
                match (*timer_ptr).handler {
                    TimerHandler::Queue(queue) => (*queue).push(Event::Timer((*timer_ptr).id)),
                    TimerHandler::Callback(_) => callbacks.push_back(timer_ptr)
                }
            }

            for timer in callbacks.iter_mut() {
                match timer.handler {
                    TimerHandler::Callback(callback) => callback(timer.id),
                    _ => intrinsics::unreachable()
                }
            }
        }
    }

    pub fn counter(&self) -> usize {
        self.counter
    }
}

pub struct TimerEntity {
    id: TimerId,
    handler: TimerHandler,
    tick: usize,
    prev: *mut TimerEntity,
    next: *mut TimerEntity
}

impl_linked_node!(TimerEntity { prev: prev, next: next });

impl TimerEntity {
    #[inline]
    fn new(id: TimerId) -> TimerEntity {
        TimerEntity {
            id: id,
            handler: TimerHandler::Queue(ptr::null_mut()),
            tick: 0,
            prev: ptr::null_mut(),
            next: ptr::null_mut()
        }
    }

    pub fn reset(&'static mut self, delay: usize) {
        interrupt::disable();
        let man = manager();
        let counter = man.counter();
        if counter < self.tick {
            // リストの最後に移動
            man.ticking_timers.remove(self);
        }

        self.tick = counter + delay;
        man.ticking_timers.push(self);
    }

    fn cmp(a: &TimerEntity, b: &TimerEntity) -> Ordering {
        a.tick.cmp(&b.tick)
    }
}

pub struct Timer(TimerId);

impl Timer {
    unsafe fn own_entity(&self) -> &'static mut TimerEntity {
        &mut manager().timer_pool[self.0 as usize]
    }

    #[inline]
    pub unsafe fn entity(self) -> &'static mut TimerEntity {
        let entity = self.own_entity();
        mem::forget(self);
        entity
    }

    #[inline]
    pub fn reset(&self, delay: usize) {
        unsafe {
            self.own_entity().reset(delay);
        }
    }

    #[inline(always)]
    pub fn id(&self) -> TimerId {
        self.0
    }
}

impl Drop for Timer {
    #[inline]
    fn drop(&mut self) {
        manager().remove(unsafe { self.own_entity() });
    }
}

static mut manager_opt: Option<TimerManager> = None;

#[inline]
pub fn init() {
    unsafe {
        manager_opt.into_some().init();
    }
}

#[inline]
pub fn manager() -> &'static mut TimerManager {
    unsafe {
        manager_opt.as_mut().be_some()
    }
}

