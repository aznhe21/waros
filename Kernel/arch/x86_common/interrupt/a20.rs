/*
use arch::x86_io::{outb, inb, io_delay, set_fs, set_gs, rdfs32, rdgs32, wrfs32};

const TEST_ADDR:  u32 = 4*0x80;
const TEST_SHORT: u32 = 32;
const TEST_LONG:  u32 = 2097152;

unsafe fn empty_8042() -> bool {
    let mut ffs = 32;

    for _ in 0..100000 {
      io_delay();

      let status = inb(0x64);
      if status == 0xFF {
        ffs -= 1;
        if ffs == 0 {
            return false;
        }
      }

      if status & 1 != 0 {
        io_delay();
        inb(0x60);
      } else if status & 2 == 0 {
        return true;
      }
    }

    return false;
}

unsafe fn test(loops: u32) -> bool {
    set_fs(0x0000);
    set_gs(0xFFFF);
    loop {}

    let saved = rdfs32(TEST_ADDR);
    let mut ret = false;

    for ctr in saved + 1 .. saved + 1 + loops {
        wrfs32(TEST_ADDR, ctr);
        io_delay();
        if rdgs32(TEST_ADDR + 0x10) ^ ctr != 0 {
            ret = true;
            break;
        }
    }

    wrfs32(TEST_ADDR, saved);
    ret
}

#[inline(always)]
unsafe fn test_short() -> bool {
    test(TEST_SHORT)
}

#[inline(always)]
unsafe fn test_long() -> bool {
    test(TEST_LONG)
}

#[inline(always)]
unsafe fn enable_bios() {
    //
}

#[inline(always)]
unsafe fn enable_kbc() {
    empty_8042();

    outb(0x64, 0xD1);
    empty_8042();

    outb(0x60, 0xDF);
    empty_8042();

    outb(0x64, 0xFF);
    empty_8042();
}

#[inline(always)]
pub unsafe fn enable_fast() {
    let port_a = inb(0x92);
    if port_a & 2 == 0 {
        outb(0x92, port_a | 0x02 & !0x01);
    }
}

#[inline(always)]
pub unsafe fn enable() -> bool {
    for _ in 0..255 {
        if test_short() {
            return true;
        }

        enable_bios();
        if test_short() {
            return true;
        }

        let kbc_err = empty_8042();

        if test_short() {
            return true;
        }

        if !kbc_err {
            enable_kbc();
            if test_long() {
                return true;
            }
        }

        enable_fast();
        if test_long() {
            return true;
        }
    }
    false
}*/

use arch::x86_io::{outb, inb};

unsafe fn wait() {
    while inb(0x64) & 2 != 0 {
    }
}

unsafe fn wait2() {
    while inb(0x64) & 1 == 0 {
    }
}

#[inline(always)]
pub unsafe fn enable() {
    wait();
    outb(0x64, 0xAD);

    wait();
    outb(0x64, 0xD0);

    wait2();
    let a = inb(0x60);

    wait();
    outb(0x64, 0xD1);

    wait();
    outb(0x60, a | 2);

    wait();
    outb(0x64, 0xAE);

    wait();
}

