use rt::Register;
use arch::page::PageTable;
use super::super::PERIPHERAL_BASE;
use super::pic;
use timer;
use core::sync::atomic::{self, Ordering};

const SYSTIMER_BASE: Register<u32> = PERIPHERAL_BASE.offset(0x3000);

/// System Timer Control/Status
const SYSTIMER_CS:  Register<u32> = SYSTIMER_BASE.offset(0x00);
/// System Timer Counter Lower 32 bits
const SYSTIMER_CLO: Register<u32> = SYSTIMER_BASE.offset(0x04);
/// System Timer Counter Higher 32 bits
const SYSTIMER_CHI: Register<u32> = SYSTIMER_BASE.offset(0x08);
/// System Timer Compare 0
const SYSTIMER_C0:  Register<u32> = SYSTIMER_BASE.offset(0x0C);
/// System Timer Compare 1
const SYSTIMER_C1:  Register<u32> = SYSTIMER_BASE.offset(0x10);
/// System Timer Compare 2
const SYSTIMER_C2:  Register<u32> = SYSTIMER_BASE.offset(0x14);
/// System Timer Compare 3
const SYSTIMER_C3:  Register<u32> = SYSTIMER_BASE.offset(0x18);

const INTERVAL_MS: u32 = 10;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Timer {
    M0 = 0,
    M1,
    M2,
    M3
}

impl Timer {
    const REGS: [Register<u32>; 4] = [SYSTIMER_C0, SYSTIMER_C1, SYSTIMER_C2, SYSTIMER_C3];

    #[inline(always)]
    const fn reg(self) -> Register<u32> {
        Self::REGS[self as usize]
    }

    #[inline(always)]
    pub const fn bit(self) -> u32 {
        1 << self as u32
    }

    #[inline]
    pub fn enable(self) {
        SYSTIMER_CS.store(SYSTIMER_CS.load() | self.bit());
    }

    #[inline]
    pub fn disable(self) {
        SYSTIMER_CS.store(SYSTIMER_CS.load() & !self.bit());
    }

    #[inline]
    pub fn tick(self) -> u32 {
        self.reg().load()
    }

    #[inline]
    pub fn set(self, tick: u32) {
        self.reg().store(tick);
    }
}

#[inline(always)]
pub fn map_pages(table: &mut PageTable) {
    table.map_direct(PageTable::FLAGS_KERNEL,
                     SYSTIMER_CS.addr(), (SYSTIMER_C3 - SYSTIMER_CS) as usize);
}

#[inline]
pub unsafe fn init() {
    pic::IRQ::SysTimer1.set_handler(irq_handler);
    pic::IRQ::SysTimer1.enable();

    Timer::M1.set(SYSTIMER_CLO.load().wrapping_add(INTERVAL_MS * 1000));
    Timer::M1.enable();
}

unsafe fn irq_handler(_irq: pic::IRQ) {
    Timer::M1.set(Timer::M1.tick().wrapping_add(INTERVAL_MS * 1000));
    Timer::M1.enable();

    timer::manager().tick(INTERVAL_MS as usize);
}

pub fn clock() -> u64 {
    loop {
        // カウンタをアトミックに読み出す
        let hi = SYSTIMER_CHI.load();
        let lo = SYSTIMER_CLO.load();

        atomic::fence(Ordering::Acquire);
        if hi == SYSTIMER_CHI.load() {
            return (hi as u64) << 32 | lo as u64;
        }
    }
}

