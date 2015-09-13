use arch::x86_io::outb;
use timer;
use super::pic::IRQ;

const PIT_REG_COUNTER0: u16 = 0x0040;
const PIT_REG_COUNTER1: u16 = 0x0041;
const PIT_REG_COUNTER2: u16 = 0x0042;
const PIT_REG_CONTROL:  u16 = 0x0043;

const PIT_CLOCK: u32 = 1193182;

//
const PIT_COM_MASK_BINCOUNT: u8 = 0x01;
const PIT_COM_MASK_MODE:     u8 = 0x0E;
const PIT_COM_MASK_RL:       u8 = 0x30;
const PIT_COM_MASK_COUNTER:  u8 = 0xC0;

// binary count
const PIT_COM_BINCOUNT_BIN: u8 = 0x00;
const PIT_COM_BINCOUNT_BCD: u8 = 0x01;

// counter mode
const PIT_COM_MODE_TERMINAL:   u8 = 0x00;
const PIT_COM_MODE_PROGONE:    u8 = 0x02;
const PIT_COM_MODE_RATEGEN:    u8 = 0x04;
const PIT_COM_MODE_SQUAREWAVE: u8 = 0x06;
const PIT_COM_MODE_SOFTTRIG:   u8 = 0x08;
const PIT_COM_MODE_HARDTRIG:   u8 = 0x0A;

// data transfer
const PIT_COM_RL_LATCH:   u8 = 0x00;
const PIT_COM_RL_LSBONLY: u8 = 0x10;
const PIT_COM_RL_MSBONLY: u8 = 0x20;
const PIT_COM_RL_DATA:    u8 = 0x30;

// counter
const PIT_COM_COUNTER0: u8 = 0x00;
const PIT_COM_COUNTER1: u8 = 0x40;
const PIT_COM_COUNTER2: u8 = 0x80;

const FREQ: u32 = 100;

#[inline(always)]
pub unsafe fn pre_init() {
}

#[inline]
pub unsafe fn init() {
    const COUNTER: u16 = (PIT_CLOCK / FREQ) as u16;
    const COMMAND: u8 = PIT_COM_COUNTER0 | PIT_COM_RL_DATA | PIT_COM_MODE_SQUAREWAVE;

    outb(PIT_REG_CONTROL, COMMAND);
    outb(PIT_REG_COUNTER0, (COUNTER >> 0 & 0xFF) as u8);
    outb(PIT_REG_COUNTER0, (COUNTER >> 8 & 0xFF) as u8);

    super::idt::set_handler(IRQ::PIT, pit_handler);
    IRQ::PIT.enable();
}

fn pit_handler(_irq: IRQ) {
    IRQ::PIT.eoi();
    timer::manager().tick(1000 / FREQ as usize);
}

