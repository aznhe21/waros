#![allow(dead_code)]

use rt::Register;
use arch::page::PageTable;
use super::super::PERIPHERAL_BASE;
use core::mem;

const PIC_BASE: Register<u32> = PERIPHERAL_BASE.offset(0xB200);

const IRQ_BASIC_PENDING:  Register<u32> = PIC_BASE.offset(0x00);
const IRQ_PENDING_1:      Register<u32> = PIC_BASE.offset(0x04);
const IRQ_PENDING_2:      Register<u32> = PIC_BASE.offset(0x08);
const FIQ_CONTROL:        Register<u32> = PIC_BASE.offset(0x0C);
const ENABLE_IRQS_1:      Register<u32> = PIC_BASE.offset(0x10);
const ENABLE_IRQS_2:      Register<u32> = PIC_BASE.offset(0x14);
const ENABLE_BASIC_IRQS:  Register<u32> = PIC_BASE.offset(0x18);
const DISABLE_IRQS_1:     Register<u32> = PIC_BASE.offset(0x1C);
const DISABLE_IRQS_2:     Register<u32> = PIC_BASE.offset(0x20);
const DISABLE_BASIC_IRQS: Register<u32> = PIC_BASE.offset(0x24);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IRQ {
    // 0-31 IRQ1
    SysTimer0 = 0,
    SysTimer1,
    SysTimer2,
    SysTimer3,
    Aux = 29,
    // 32-63 IRQ2
    I2cSpiSlv = 43,
    Pwa0 = 45,
    Pwa1,
    Smi = 48,
    Gpio0,
    Gpio1,
    Gpio2,
    Gpio3,
    I2c,
    Spi,
    Pcm,
    Uart = 57,
    // 64-71 Basic IRQ
    Timer = 64,
    Mailbox,
    Doorbell0,
    Doorbell1,
    Gpu0Halted,
    Gpu1Halted,
    AccessError1,
    AccessError0,
}

impl IRQ {
    pub fn from_u8(num: u8) -> Option<IRQ> {
        match num {
            0  => Some(IRQ::SysTimer0),
            1  => Some(IRQ::SysTimer1),
            2  => Some(IRQ::SysTimer2),
            3  => Some(IRQ::SysTimer3),
            29 => Some(IRQ::Aux),

            43 => Some(IRQ::I2cSpiSlv),
            45 => Some(IRQ::Pwa0),
            46 => Some(IRQ::Pwa1),
            48 => Some(IRQ::Smi),
            49 => Some(IRQ::Gpio0),
            50 => Some(IRQ::Gpio1),
            51 => Some(IRQ::Gpio2),
            52 => Some(IRQ::Gpio3),
            53 => Some(IRQ::I2c),
            54 => Some(IRQ::Spi),
            55 => Some(IRQ::Pcm),
            57 => Some(IRQ::Uart),

            64 => Some(IRQ::Timer),
            65 => Some(IRQ::Mailbox),
            66 => Some(IRQ::Doorbell0),
            67 => Some(IRQ::Doorbell1),
            68 => Some(IRQ::Gpu0Halted),
            69 => Some(IRQ::Gpu1Halted),
            70 => Some(IRQ::AccessError1),
            71 => Some(IRQ::AccessError0),

            _  => None
        }
    }

    pub fn enable(self) {
        match self as u32 {
            bit @ 0  ... 31 => ENABLE_IRQS_1.store(1 << bit),
            bit @ 32 ... 63 => ENABLE_IRQS_2.store(1 << (bit - 32)),
            bit @ 64 ... 71 => ENABLE_BASIC_IRQS.store(1 << (bit - 64)),
            _ => unreachable!()
        }
    }

    pub fn disable(self) {
        match self as u32 {
            bit @ 0  ... 31 => DISABLE_IRQS_1.store(1 << bit),
            bit @ 32 ... 63 => DISABLE_IRQS_2.store(1 << (bit - 32)),
            bit @ 64 ... 71 => DISABLE_BASIC_IRQS.store(1 << (bit - 64)),
            _ => unreachable!()
        }
    }

    #[inline]
    pub fn set_handler(self, handler: IrqHandler) {
        unsafe {
            irq_handlers[self as usize] = handler;
        }
    }
}

#[inline(always)]
pub fn map_pages(table: &mut PageTable) {
    table.map_direct(PageTable::FLAGS_KERNEL,
                     IRQ_BASIC_PENDING.addr(), (DISABLE_BASIC_IRQS - IRQ_BASIC_PENDING) as usize);
}

#[inline]
pub unsafe fn init() {
    DISABLE_IRQS_1.store(!0);
    DISABLE_IRQS_2.store(!0);
    DISABLE_BASIC_IRQS.store(!0);
}

pub type IrqHandler = unsafe fn(IRQ);
static mut irq_handlers: [IrqHandler; 72] = [irq_null_handler; 72];

unsafe fn irq_null_handler(irq: IRQ) {
    log!("Unhandled IRQ: {:?}", irq);
}

#[inline(always)]
fn first_set_bit(val: u32) -> u32 {
    31 - val.leading_zeros()
}

#[no_mangle]
pub unsafe extern "C" fn irq_handler() {
    let mut mask;

    mask = IRQ_PENDING_1.load();
    while mask != 0 {
        let bit = first_set_bit(mask);
        mask ^= 1 << bit;
        irq_handlers[bit as usize](mem::transmute(bit as u8));
    }

    mask = IRQ_PENDING_2.load();
    while mask != 0 {
        let bit = first_set_bit(mask);
        mask ^= 1 << bit;
        irq_handlers[bit as usize](mem::transmute(bit as u8 + 32));
    }

    mask = IRQ_BASIC_PENDING.load();
    while mask != 0 {
        let bit = first_set_bit(mask);
        mask ^= 1 << bit;
        irq_handlers[bit as usize](mem::transmute(bit as u8 + 64));
    }
}

