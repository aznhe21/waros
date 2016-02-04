use rt::Register;
use arch::page::PageTable;

pub mod interrupt;

pub const PERIPHERAL_BASE: Register<u32> = Register::new(0x20000000);

pub const GPIO_BASE:       Register<u32> = PERIPHERAL_BASE.offset(0x200000);
pub const UART0_BASE:      Register<u32> = PERIPHERAL_BASE.offset(0x201000);

#[inline(always)]
pub fn map_pages(table: &mut PageTable) {
    interrupt::pic::map_pages(table);
    interrupt::pit::map_pages(table);
    super::gpio::map_pages(table);
    super::serial::map_pages(table);
}

