use rt::{Force, ForceRef, IntBlocker};
use lists::{DList, SortedList};
use event::{Event, EventQueue};
use core::cmp::Ordering;
use core::iter::FromIterator;
use core::ptr::Shared;

pub type TimerId = u16;

pub enum TimerHandler {
    Unset,
    Queue(ForceRef<EventQueue>),
    Callback(fn(TimerId) -> ())
}

pub struct TimerManager {
    timer_pool: [TimerEntity; 256],
    free_timers: DList<TimerEntity>,
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
        self.free_timers = DList::from_iter(self.timer_pool.iter_mut().map(|timer| unsafe { Shared::new(timer) }));
        self.ticking_timers = SortedList::new(TimerEntity::cmp);
        self.counter = 0;
    }

    fn with_handler(&mut self, handler: TimerHandler) -> TimerId {
        unsafe {
            let timer = self.free_timers.pop_front().expect("Not enough timers");
            (**timer).handler = handler;
            (**timer).id
        }
    }

    fn remove(&mut self, timer: Shared<TimerEntity>) {
        if self.ticking_timers.contains(&timer) {
            self.ticking_timers.remove(&timer);
        }
        self.free_timers.push_back(timer);
    }

    pub fn tick(&mut self, count: usize) {
        unsafe {
            self.counter = self.counter.wrapping_add(count);

            while let Some(timer) = self.ticking_timers.front() {
                if (**timer).tick > self.counter {
                    break;
                }
                self.ticking_timers.remove(&timer);
                match (**timer).handler {
                    TimerHandler::Unset => unreachable!(),
                    TimerHandler::Queue(ref mut queue) => queue.push(Event::Timer((**timer).id)),
                    TimerHandler::Callback(cb) => {
                        cb((**timer).id);
                        break;
                    }
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
    prev: Option<Shared<TimerEntity>>,
    next: Option<Shared<TimerEntity>>
}

impl_linked_node!(Shared<TimerEntity> { prev: prev, next: next });

impl TimerEntity {
    #[inline]
    fn new(id: TimerId) -> TimerEntity {
        TimerEntity {
            id: id,
            handler: TimerHandler::Unset,
            tick: 0,
            prev: None,
            next: None
        }
    }

    pub fn reset(this: Shared<TimerEntity>, delay: usize) {
        unsafe {
            let _blocker = IntBlocker::new();

            let mut man = manager();
            let counter = man.counter();
            if counter < (**this).tick {
                // リストの最後に移動
                man.ticking_timers.remove(&this);
            }

            (**this).tick = counter + delay;
            man.ticking_timers.push(this);
        }
    }

    pub fn clear(this: Shared<TimerEntity>) {
        unsafe {
            let _blocker = IntBlocker::new();

            let mut man = manager();
            if man.ticking_timers.contains(&this) {
                man.ticking_timers.remove(&this);
            }
            (**this).tick = 0;
        }
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
    fn entity(&self) -> Shared<TimerEntity> {
        unsafe {
            Shared::new(&mut manager().timer_pool[self.0 as usize])
        }
    }

    #[inline(always)]
    pub fn id(&self) -> TimerId {
        self.0
    }

    #[inline(always)]
    pub fn reset(&self, delay: usize) {
        TimerEntity::reset(self.entity(), delay);
    }

    #[inline(always)]
    pub fn clear(&self) {
        TimerEntity::clear(self.entity());
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
    fn entity(&self) -> Shared<TimerEntity> {
        unsafe {
            Shared::new(&mut manager().timer_pool[self.0 as usize])
        }
    }

    #[inline(always)]
    pub fn id(&self) -> TimerId {
        self.0
    }

    #[inline(always)]
    pub fn reset(&self, delay: usize) {
        TimerEntity::reset(self.entity(), delay);
    }

    #[inline(always)]
    pub fn clear(&self) {
        TimerEntity::clear(self.entity());
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

