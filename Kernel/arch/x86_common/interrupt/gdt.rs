use arch::interrupt;
use core::u64;

extern "C" {
    fn flush_gdt(cs: u16, ds: u16);
}

#[repr(C, packed)]
struct Gdtr {
    len: u16,
    ptr: u32
}

const fn gdt_entry(base: u32, limit: u32, flags: u16) -> u64 {
    ((base  as u64 & 0xFF000000) << (56 - 24)) |  // 56-63
    ((flags as u64 & 0x0000F0FF) << 40) |         // 40-47, 52-55
    ((limit as u64 & 0x000F0000) << (48 - 16)) |  // 48-51
    ((base  as u64 & 0x00FFFFFF) << 16) |         // 16-41
    ((limit as u64 & 0x0000FFFF))                 //  0-15
}

#[inline(always)]
pub unsafe fn pre_init() {
    static mut boot_gdt: [u64; interrupt::GDT_BOOT_ENTRIES] = [0; interrupt::GDT_BOOT_ENTRIES];
    boot_gdt[interrupt::GDT_ENTRY_BOOT_CS] = gdt_entry(0, 0xFFFFF, 0xC09B);
    boot_gdt[interrupt::GDT_ENTRY_BOOT_DS] = gdt_entry(0, 0xFFFFF, 0xC093);

    static mut gdtr: Gdtr = Gdtr {
        len: (interrupt::GDT_BOOT_ENTRIES * u64::BYTES - 1) as u16,
        ptr: 0
    };
    gdtr.ptr = boot_gdt.as_mut_ptr() as u32;

    asm!("lgdtl ($0)" :: "r"(&gdtr) :: "volatile");
}

#[inline]
pub unsafe fn init() {
    static mut init_gdt: [u64; interrupt::GDT_ENTRIES] = [0; interrupt::GDT_ENTRIES];
    init_gdt[interrupt::GDT_ENTRY_KERNEL_CS] = gdt_entry(0, 0xFFFFF, 0xC09A);
    init_gdt[interrupt::GDT_ENTRY_KERNEL_DS] = gdt_entry(0, 0xFFFFF, 0xC092);
    init_gdt[interrupt::GDT_ENTRY_DEFAULT_USER_CS] = gdt_entry(0, 0xFFFFF, 0xC0FA);
    init_gdt[interrupt::GDT_ENTRY_DEFAULT_USER_DS] = gdt_entry(0, 0xFFFFF, 0xC0F2);

    static mut gdtr: Gdtr = Gdtr {
        len: (interrupt::GDT_ENTRIES * u64::BYTES - 1) as u16,
        ptr: 0
    };
    gdtr.ptr = init_gdt.as_mut_ptr() as u32;

    asm!("lgdtl ($0)" :: "r"(&gdtr) :: "volatile");
    flush_gdt(interrupt::KERNEL_CS as u16, interrupt::KERNEL_DS as u16);
}

