use rt::{Force, ForceRef, IntBlocker};
use lists::{LinkedList, SortedList};
use event::{Event, EventQueue};
use core::cmp::Ordering;
use core::iter::FromIterator;
use core::mem;
use core::ptr;
use core::intrinsics;

pub type TimerId = u16;

pub enum TimerHandler {
    Unset,
    Queue(ForceRef<EventQueue>),
    Callback(fn(TimerId) -> ())
}

pub struct TimerManager {
    timer_pool: [TimerEntity; 256],
    free_timers: LinkedList<TimerEntity>,
    ticking_timers: SortedList<TimerEntity>,
    counter: usize
}

unsafe impl Send for TimerManager { }
unsafe impl Sync for TimerManager { }

impl TimerManager {
    #[inline(always)]
    pub fn init(&mut self) {
        for (i, timer) in self.timer_pool.iter_mut().enumerate() {
            *timer = TimerEntity::new(i as TimerId);
        }
        self.free_timers = LinkedList::from_iter(&mut self.timer_pool[..]);
        self.ticking_timers = SortedList::new(TimerEntity::cmp);
        self.counter = 0;
    }

    fn with_handler(&mut self, handler: TimerHandler) -> TimerId {
        let timer = self.free_timers.pop_front().expect("Not enough timers");
        timer.handler = handler;
        timer.id
    }

    fn remove(&mut self, timer: &'static mut TimerEntity) {
        if self.ticking_timers.contains(timer) {
            self.ticking_timers.remove(timer);
        }
        self.free_timers.push_back(timer);
    }

    pub fn tick(&mut self, count: usize) {
        unsafe {
            self.counter = self.counter.wrapping_add(count);
            let mut callbacks = LinkedList::new();

            loop {
                let timer_ptr = self.ticking_timers.front_ptr();
                if timer_ptr.is_null() || (*timer_ptr).tick > self.counter {
                    break;
                }
                self.ticking_timers.remove(timer_ptr);
                match (*timer_ptr).handler {
                    TimerHandler::Unset => unreachable!(),
                    TimerHandler::Queue(ref mut queue) => queue.push(Event::Timer((*timer_ptr).id)),
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

struct TimerEntity {
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
            handler: TimerHandler::Unset,
            tick: 0,
            prev: ptr::null_mut(),
            next: ptr::null_mut()
        }
    }

    pub fn reset(&'static mut self, delay: usize) {
        let _blocker = IntBlocker::new();

        let mut man = manager();
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
    #[inline]
    pub fn with_queue(queue: ForceRef<EventQueue>) -> Timer {
        Timer(manager().with_handler(TimerHandler::Queue(queue)))
    }

    #[inline]
    pub fn with_callback(callback: fn(TimerId) -> ()) -> Timer {
        Timer(manager().with_handler(TimerHandler::Callback(callback)))
    }

    #[inline]
    fn entity(&self) -> &'static mut TimerEntity {
        unsafe {
            mem::transmute(&mut manager().timer_pool[self.0 as usize])
        }
    }

    #[inline]
    pub fn reset(&self, delay: usize) {
        self.entity().reset(delay);
    }

    #[inline(always)]
    pub fn id(&self) -> TimerId {
        self.0
    }
}

impl Drop for Timer {
    #[inline]
    fn drop(&mut self) {
        manager().remove(self.entity());
    }
}

pub struct UnmanagedTimer(TimerId);

impl UnmanagedTimer {
    #[inline]
    pub unsafe fn with_queue(queue: ForceRef<EventQueue>) -> UnmanagedTimer {
        UnmanagedTimer(manager().with_handler(TimerHandler::Queue(queue)))
    }

    #[inline]
    pub unsafe fn with_callback(callback: fn(TimerId) -> ()) -> UnmanagedTimer {
        UnmanagedTimer(manager().with_handler(TimerHandler::Callback(callback)))
    }

    #[inline]
    fn entity(&self) -> &'static mut TimerEntity {
        unsafe {
            mem::transmute(&mut manager().timer_pool[self.0 as usize])
        }
    }

    #[inline]
    pub fn reset(&self, delay: usize) {
        self.entity().reset(delay);
    }

    #[inline(always)]
    pub fn id(&self) -> TimerId {
        self.0
    }

    #[inline]
    pub fn drop(&self) {
        manager().remove(self.entity());
    }
}

impl Clone for UnmanagedTimer {
    #[inline]
    fn clone(&self) -> UnmanagedTimer {
        UnmanagedTimer(self.0)
    }
}

static MANAGER: Force<TimerManager> = Force::new();

#[inline]
pub fn init() {
    MANAGER.setup().init();
}

#[inline(always)]
pub fn manager() -> ForceRef<TimerManager> {
    MANAGER.as_ref()
}

