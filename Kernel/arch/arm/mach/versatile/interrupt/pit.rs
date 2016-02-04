#![allow(dead_code)]

use rt::Register;
use arch;
use arch::page::PageTable;
use super::pic::IRQ;
use timer;
use core::sync::atomic::{self, Ordering};

const TIMER_LOAD:    arch::AddrType = 0x00;
const TIMER_VALUE:   arch::AddrType = 0x04;
const TIMER_CONTROL: arch::AddrType = 0x08;
const TIMER_INTCLR:  arch::AddrType = 0x0C;
const TIMER_MIS:     arch::AddrType = 0x14;

const TIMER0_BASE:    Register<u32> = Register::new(0x101E2000);
const TIMER0_LOAD:    Register<u32> = TIMER0_BASE.offset(TIMER_LOAD);
const TIMER0_VALUE:   Register<u32> = TIMER0_BASE.offset(TIMER_VALUE);
const TIMER0_CONTROL: Register<u32> = TIMER0_BASE.offset(TIMER_CONTROL);
const TIMER0_INTCLR:  Register<u32> = TIMER0_BASE.offset(TIMER_INTCLR);
const TIMER0_MIS:     Register<u32> = TIMER0_BASE.offset(TIMER_MIS);

const TIMER1_BASE:    Register<u32> = Register::new(0x101E2020);
const TIMER1_LOAD:    Register<u32> = TIMER1_BASE.offset(TIMER_LOAD);
const TIMER1_VALUE:   Register<u32> = TIMER1_BASE.offset(TIMER_VALUE);
const TIMER1_CONTROL: Register<u32> = TIMER1_BASE.offset(TIMER_CONTROL);
const TIMER1_INTCLR:  Register<u32> = TIMER1_BASE.offset(TIMER_INTCLR);
const TIMER1_MIS:     Register<u32> = TIMER1_BASE.offset(TIMER_MIS);

const TIMER_ENABLE:     u32 = 1 << 7;
const TIMER_PERIODIC:   u32 = 1 << 6;
const TIMER_INT_ENABLE: u32 = 1 << 5;
const TIMER_DIV_16:     u32 = 0b01 << 2;
const TIMER_DIV_256:    u32 = 0b10 << 2;
const TIMER_32BIT:      u32 = 1 << 1;
const TIMER_ONESHOT:    u32 = 1 << 0;

const FREQ: u32 = 100;

#[inline]
pub unsafe fn init() {
    TIMER0_LOAD.store(1000 * 1000 / FREQ);
    TIMER0_CONTROL.store(TIMER_ENABLE | TIMER_PERIODIC | TIMER_32BIT | TIMER_INT_ENABLE);
    TIMER0_INTCLR.store(0);

    TIMER1_CONTROL.store(TIMER_ENABLE | TIMER_32BIT);

    IRQ::Timer01.set_handler(irq_handler);
    IRQ::Timer01.enable();
}

#[inline(always)]
pub fn map_pages(table: &mut PageTable) {
    table.map_direct(PageTable::FLAGS_KERNEL,
                     TIMER0_LOAD.addr(), (TIMER0_MIS - TIMER0_LOAD) as usize);
    table.map_direct(PageTable::FLAGS_KERNEL,
                     TIMER1_LOAD.addr(), (TIMER1_MIS - TIMER1_LOAD) as usize);
}

unsafe fn irq_handler(irq: IRQ) {
    match irq {
        IRQ::Timer01 if TIMER0_MIS.load() == 1 => {
            TIMER0_INTCLR.store(0);

            atomic::fence(Ordering::Acquire);
            timer::manager().tick(1000 / FREQ as usize);
        },
        _ => unreachable!()
    }
}

pub fn clock() -> u64 {
    !TIMER1_VALUE.load() as u64
}

