use rt::Register;
use arch::page::PageTable;

pub mod interrupt;

pub const GPIO_BASE:       Register<u32> = Register::new(0x101E4000);
pub const UART0_BASE:      Register<u32> = Register::new(0x101F1000);

#[inline(always)]
pub fn map_pages(table: &mut PageTable) {
    interrupt::pic::map_pages(table);
    interrupt::pit::map_pages(table);
    super::gpio::map_pages(table);
    super::serial::map_pages(table);
}

