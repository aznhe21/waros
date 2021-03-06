#![allow(dead_code)]

use super::pic::IRQ;
use core::mem;

const INT_DIVISION_BY_ZERO:             u8 = 0x00;
const INT_DEBUGGER:                     u8 = 0x01;
const INT_NMI:                          u8 = 0x02;
const INT_BREAKPOINT:                   u8 = 0x03;
const INT_OVERFLOW:                     u8 = 0x04;
const INT_BOUNDS:                       u8 = 0x05;
const INT_INVALID_OPCODE:               u8 = 0x06;
const INT_COPROCESSOR_NOT_AVAILABLE:    u8 = 0x07;
const INT_DOUBLE_FAULT:                 u8 = 0x08;
const INT_COPROCESSOR_SEGMENT_OVERRUN:  u8 = 0x09;
const INT_INVALID_TASK_STATE_SEGMENT:   u8 = 0x0A;
const INT_SEGMENT_NOT_PRESENT:          u8 = 0x0B;
const INT_STACK_FAULT:                  u8 = 0x0C;
const INT_GENERAL_PROTECTION_FAULT:     u8 = 0x0D;
const INT_PAGE_FAULT:                   u8 = 0x0E;
const INT_MATH_FAULT:                   u8 = 0x10;
const INT_ALIGNMENT_CHECK:              u8 = 0x11;
const INT_MACHINE_CHECK:                u8 = 0x12;
const INT_SIMD_FLOATINGPOINT_EXCEPTION: u8 = 0x13;

const IDT_FLAGS_INTGATE_16BIT: u8 = 0x06;
const IDT_FLAGS_TSKGATE:       u8 = 0x05;
const IDT_FLAGS_CALL_GATE:     u8 = 0x0C;
const IDT_FLAGS_INTGATE_32BIT: u8 = 0x0E;
const IDT_FLAGS_TRPGATE:       u8 = 0x0F;
const IDT_FLAGS_DPL_LV0:       u8 = 0x00;
const IDT_FLAGS_DPL_LV1:       u8 = 0x20;
const IDT_FLAGS_DPL_LV2:       u8 = 0x40;
const IDT_FLAGS_DPL_LV3:       u8 = 0x60;
const IDT_FLAGS_PRESENT:       u8 = 0x80;

const IDT_SIZE: usize = 256;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GateDescriptor {
    offset_lo: u16,
    selector:  u16,
    reserved:  u8,
    flags:     u8,
    offset_hi: u16
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    addr:  u32
}

type InterruptHandler = unsafe extern "C" fn() -> ();

struct InterruptDescriptorTable {
    idt: [GateDescriptor; IDT_SIZE]
}

impl InterruptDescriptorTable {
    unsafe fn set_idt(&mut self, idtr: usize, handler: InterruptHandler, flags: u8) {
        let (lo, hi): (u16, u16) = mem::transmute(handler);
        self.idt[idtr] = GateDescriptor {
            selector:  super::KERNEL_CS as u16,
            offset_lo: lo,
            offset_hi: hi,
            reserved:  0,
            flags:     flags
        };
    }

    #[inline(always)]
    unsafe fn set_exception(&mut self, idtr: usize, handler: InterruptHandler) {
        self.set_idt(idtr, handler, IDT_FLAGS_PRESENT | IDT_FLAGS_TRPGATE);
    }

    #[inline(always)]
    unsafe fn set_interrupt(&mut self, idtr: usize, handler: InterruptHandler) {
        self.set_idt(idtr, handler, IDT_FLAGS_PRESENT | IDT_FLAGS_INTGATE_32BIT);
    }

    #[inline(always)]
    unsafe fn set_user_interrupt(&mut self, idtr: usize, handler: InterruptHandler) {
        self.set_idt(idtr, handler, IDT_FLAGS_PRESENT | IDT_FLAGS_TRPGATE | IDT_FLAGS_DPL_LV3);
    }

    unsafe fn load(&mut self) {
        static mut idtr: Idtr = Idtr { limit: 0, addr: 0 };

        idtr = Idtr {
            limit: (mem::size_of::<GateDescriptor>() * IDT_SIZE - 1) as u16,
            addr: self.idt.as_mut_ptr() as u32
        };
        asm!("lidtl ($0)" :: "r"(&idtr) :: "volatile");
    }
}

static mut idt: InterruptDescriptorTable = InterruptDescriptorTable {
    idt: [GateDescriptor { selector: 0, offset_lo: 0, offset_hi: 0, reserved: 0, flags: 0 }; IDT_SIZE]
};

extern "C" {
    fn idt_null_handler();
    fn idt_06_handler();
    fn idt_0c_handler();
    fn idt_0d_handler();
    fn idt_0e_handler();

    fn irq_handler_0();
    fn irq_handler_1();
    fn irq_handler_2();
    fn irq_handler_3();
    fn irq_handler_4();
    fn irq_handler_5();
    fn irq_handler_6();
    fn irq_handler_7();
    fn irq_handler_8();
    fn irq_handler_9();
    fn irq_handler_10();
    fn irq_handler_11();
    fn irq_handler_12();
    fn irq_handler_13();
    fn irq_handler_14();
    fn irq_handler_15();
}

#[inline]
pub unsafe fn pre_init() {
    static mut idtr: Idtr = Idtr { limit: 0, addr: 0 };
    asm!("lidtl ($0)" :: "r"(&idtr) :: "volatile");
}

#[inline]
pub unsafe fn init() {
    for i in 0 .. IDT_SIZE {
        idt.set_exception(i, idt_null_handler);
    }

    idt.set_exception(0x06, idt_06_handler);
    idt.set_exception(0x0C, idt_0c_handler);
    idt.set_exception(0x0D, idt_0d_handler);
    idt.set_exception(0x0E, idt_0e_handler);

    idt.set_interrupt(0x20, irq_handler_0);
    idt.set_interrupt(0x21, irq_handler_1);
    idt.set_interrupt(0x22, irq_handler_2);
    idt.set_interrupt(0x23, irq_handler_3);
    idt.set_interrupt(0x24, irq_handler_4);
    idt.set_interrupt(0x25, irq_handler_5);
    idt.set_interrupt(0x26, irq_handler_6);
    idt.set_interrupt(0x27, irq_handler_7);
    idt.set_interrupt(0x28, irq_handler_8);
    idt.set_interrupt(0x29, irq_handler_9);
    idt.set_interrupt(0x2A, irq_handler_10);
    idt.set_interrupt(0x2B, irq_handler_11);
    idt.set_interrupt(0x2C, irq_handler_12);
    idt.set_interrupt(0x2D, irq_handler_13);
    idt.set_interrupt(0x2E, irq_handler_14);
    idt.set_interrupt(0x2F, irq_handler_15);

    idt.load();
}

#[no_mangle]
pub unsafe extern "C" fn idt_empty_handler(esp: *const u32) {
    panic!("Unhandled interrupt at {:p}", *esp as *const u8);
}

#[no_mangle]
pub unsafe extern "C" fn invalid_opcode_handler(esp: *const u32) {
    panic!("Invalid opcode (may out of memory occurred) at {:p}", *esp as *const u8);
}

#[inline]
fn selector_error_panic(message: &str, code: u32) -> ! {
    panic!("{} at {}{} of {}",
           message,
           if code & 1 != 0 { "external " } else { "" },
           match code >> 1 & 0b11 {
               0b00 => "GDT",
               0b01 => "IDT",
               0b10 => "LDT",
               _    => "IDT",
           },
           code >> 3);
}

#[no_mangle]
pub unsafe extern "C" fn stack_segment_fault_handler(code: u32) {
    selector_error_panic("Stack-segment fault", code);
}

#[no_mangle]
pub unsafe extern "C" fn general_protection_fault_handler(code: u32) {
    selector_error_panic("General protection fault", code);
}

#[no_mangle]
pub unsafe extern "C" fn page_fault_handler(esp: *const u32, address: u32) {
    panic!("Page fault {} to {:p} at {:p}", *esp.offset(0), address as *const u8, *esp.offset(1) as *const u8);
}

pub type IrqHandler = fn(IRQ);
static mut irq_handlers: [IrqHandler; 16] = [irq_null_handler; 16];

#[inline(always)]
pub unsafe fn set_handler(irq: IRQ, handler: IrqHandler) {
    irq_handlers[irq as usize] = handler;
}

fn irq_null_handler(irq: IRQ) {
    log!("Unhandled IRQ: {}", irq as u8);
}

#[no_mangle]
pub unsafe extern "C" fn irq_common_handler(irq: IRQ) {
    irq_handlers[irq as usize](irq);
}

