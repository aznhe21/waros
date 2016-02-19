#![allow(dead_code)]

use rt::Register;
use arch::page::PageTable;
use core::mem;

const PIC: Register<u32> = Register::new(0x10140000);

const VIC_IRQ_STATUS:   Register<u32> = PIC.offset(0x00);
const VIC_FIQ_STATUS:   Register<u32> = PIC.offset(0x04);
const VIC_INT_SELECT:   Register<u32> = PIC.offset(0x0C);
const VIC_INT_ENABLE:   Register<u32> = PIC.offset(0x10);
const VIC_INT_EN_CLEAR: Register<u32> = PIC.offset(0x14);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IRQ {
    WatchDog = 0, // Watchdog timer
    Software,     // Software interrupt
    CommsRx,      // Debug communications receive interrupt
    CommsTx,      // Debug communications transmit interrupt
    Timer01,      // Timers on development chip
    Timer23,      // Timers on development chip
    GPIO0,        // GPIO controller in development chip
    GPIO1,        // GPIO controller in development chip
    GPIO2,        // GPIO controller in development chip
    GPIO3,        // GPIO controller in development chip
    RTC,          // Real time clock in development chip
    SSP,          // Synchronous serial port in development chip
    UART0,        // UART0 on development chip
    UART1,        // UART1 on development chip
    UART2,        // UART2 on development chip
    SCI0,         // Smart Card interface in development chip
    CLCD,         // CLCD controller in development chip
    DMA,          // DMA controller in development chip
    PWRFAIL,      // Power failure from FPGA
    MBX,          // Graphics processor on development chip
    GND,          // Reserved
}

impl IRQ {
    pub fn from_u8(num: u8) -> Option<IRQ> {
        unsafe {
            match num {
                0...20 => Some(mem::transmute(num)),
                _ => None
            }
        }
    }

    #[inline(always)]
    pub const fn bit(self) -> u32 {
        1 << self as u32
    }

    pub fn enable(self) {
        let int = VIC_INT_ENABLE.load();
        VIC_INT_ENABLE.store(int | self.bit());
    }

    pub fn disable(self) {
        let int = VIC_INT_ENABLE.load();
        VIC_INT_ENABLE.store(int & !self.bit());
    }

    #[inline]
    pub fn set_handler(self, handler: IrqHandler) {
        unsafe {
            irq_handlers[self as usize] = handler;
        }
    }
}

#[inline]
pub unsafe fn init() {
    VIC_INT_ENABLE.store(0);
}

#[inline(always)]
pub fn map_pages(table: &mut PageTable) {
    table.map_direct(PageTable::FLAGS_KERNEL,
                     VIC_IRQ_STATUS.addr(), (VIC_INT_EN_CLEAR - VIC_IRQ_STATUS) as usize);
}

pub type IrqHandler = unsafe fn(IRQ);
static mut irq_handlers: [IrqHandler; 32] = [irq_null_handler; 32];

unsafe fn irq_null_handler(irq: IRQ) {
    log!("Unhandled IRQ: {:?}", irq);
}

#[no_mangle]
pub unsafe extern "C" fn irq_handler() {
    let mut mask = VIC_IRQ_STATUS.load();

    while mask != 0 {
        let bit = 31 - mask.leading_zeros();
        mask ^= 1 << bit;
        irq_handlers[bit as usize](mem::transmute(bit as u8));
    }
}

