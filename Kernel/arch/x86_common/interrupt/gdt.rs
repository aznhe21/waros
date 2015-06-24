use prelude::*;
use core::mem;

#[repr(packed)]
struct Gdtr {
    len: u16,
    ptr: u32
}

#[inline(always)]
fn gdt_entry(base: u32, limit: u32, flags: u16) -> u64 {
    (base  as u64 & 0xFF000000 << (56 - 24)) |  // 56-63
    (flags as u64 & 0x0000F0FF << 40) |         // 40-47, 52-55
    (limit as u64 & 0x000F0000 << (48 - 16)) |  // 48-51
    (base  as u64 & 0x00FFFFFF << 16) |         // 16-41
    (limit as u64 & 0x0000FFFF)                 //  0-15
}

static mut boot_gdt: [u64; 5] = [0; 5];
//static mut gdt32: [u64; 32] = [0; 32];

#[inline]
unsafe fn load_gdt<'a>(gdt: &'a mut[u64]) {
    let size = (mem::size_of::<u64>() * gdt.len() - 1) as u16;
    let gdtr = Gdtr { len: size, ptr: (*gdt).as_mut_ptr() as u32 };
    asm!("lgdtl %0" :: "eax"(gdtr) :: "volatile");
}

#[inline(always)]
pub unsafe fn pre_init() {
    //let mut boot_gdt: [u64; GDT_SIZE] = [
    /*boot_gdt = [
        0,
        //0x00CF92000000FFFF,
        //0x00479A280000FFFF
        gdt_entry(0,    0xFFFFF, 0xC09A),
        gdt_entry(0,    0xFFFFF, 0xC092),
        gdt_entry(0,    0xFFFFF, 0xC0FA),
        gdt_entry(0,    0xFFFFF, 0xC0F2)
    ];*/
    boot_gdt[super::GDT_KERNEL_CS] = gdt_entry(0, 0xFFFFF, 0xC09B);
    boot_gdt[super::GDT_KERNEL_DS] = gdt_entry(0, 0xFFFFF, 0xC093);
    boot_gdt[super::GDT_KERNEL_TSS] = gdt_entry(4096, 103, 0x0089);

    load_gdt(&mut boot_gdt);
}

#[inline(always)]
pub unsafe fn init() {
    /*//gdt_entry(0, 0xFFFF, 0xCF92),
    gdt32[1] = 0x00CF92000000FFFF;
    //gdt_entry(0x2800, 0xFFFF, 0x479A),
    gdt32[2] = 0x00479A280000FFFF;
    //gdt_entry(0x2800, 0xFFFF, 0xCF92)
    gdt32[3] = 0x00CF9A280000FFFF;

    load_gdt(&mut gdt32);*/
}

