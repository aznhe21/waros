use core::mem;
use arch::x86_io::outb;

const PIT_REG_COUNTER0: u16 = 0x0040;
const PIT_REG_COUNTER1: u16 = 0x0041;
const PIT_REG_COUNTER2: u16 = 0x0042;
const PIT_REG_CONTROL:  u16 = 0x0043;

const DEF_PIT_CLOCK: u32 = 1193182;

//
const DEF_PIT_COM_MASK_BINCOUNT: u8 = 0x01;
const DEF_PIT_COM_MASK_MODE:     u8 = 0x0E;
const DEF_PIT_COM_MASK_RL:       u8 = 0x30;
const DEF_PIT_COM_MASK_COUNTER:  u8 = 0xC0;

// binary count
const DEF_PIT_COM_BINCOUNT_BIN: u8 = 0x00;
const DEF_PIT_COM_BINCOUNT_BCD: u8 = 0x01;

// counter mode
const DEF_PIT_COM_MODE_TERMINAL:   u8 = 0x00;
const DEF_PIT_COM_MODE_PROGONE:    u8 = 0x02;
const DEF_PIT_COM_MODE_RATEGEN:    u8 = 0x04;
const DEF_PIT_COM_MODE_SQUAREWAVE: u8 = 0x06;
const DEF_PIT_COM_MODE_SOFTTRIG:   u8 = 0x08;
const DEF_PIT_COM_MODE_HARDTRIG:   u8 = 0x0A;

// data transfer
const DEF_PIT_COM_RL_LATCH:   u8 = 0x00;
const DEF_PIT_COM_RL_LSBONLY: u8 = 0x10;
const DEF_PIT_COM_RL_MSBONLY: u8 = 0x20;
const DEF_PIT_COM_RL_DATA:    u8 = 0x30;

// counter
const DEF_PIT_COM_COUNTER0: u8 = 0x00;
const DEF_PIT_COM_COUNTER1: u8 = 0x40;
const DEF_PIT_COM_COUNTER2: u8 = 0x80;

#[inline(always)]
pub unsafe fn pre_init() {
}

#[inline(always)]
pub unsafe fn init() {
    static FREQ: u32 = 100;
    let counter = (DEF_PIT_CLOCK / FREQ) as u16;
    let command = DEF_PIT_COM_COUNTER0 | DEF_PIT_COM_RL_DATA | DEF_PIT_COM_MODE_SQUAREWAVE;

    let (lo, hi): (u8, u8) = mem::transmute(counter);

    outb(PIT_REG_CONTROL, command);
    outb(PIT_REG_CONTROL, lo);
    outb(PIT_REG_CONTROL, hi);
}
