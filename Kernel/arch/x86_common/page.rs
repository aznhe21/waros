use prelude::*;
use arch;
use memory;
use memory::kernel::{PhysAddr, VirtAddr};
use core::ops;
use core::u32;

const PDE_OFFSET_MASK: u32 = !(1 << 12 - 1);
const PTE_OFFSET_MASK: u32 = !(1 << 12 - 1);

#[derive(Clone, Copy)]
struct PageDirectoryEntry(u32);
impl PageDirectoryEntry {
    const BYTES: usize = u32::BYTES;
    const LEN: usize = 1024;
    const SIZE: usize = PageDirectoryEntry::BYTES * PageDirectoryEntry::LEN;

    const FLAG_PRESENT:         u16 = 0x001;
    const FLAG_RW:              u16 = 0x002;
    const FLAG_USER:            u16 = 0x004;
    const FLAG_WRITE_THROUGH:   u16 = 0x008;
    const FLAG_CACHE_DISABLE:   u16 = 0x010;
    const FLAG_ACCESSED:        u16 = 0x020;
    const FLAG_4MB:             u16 = 0x080;
    const FLAG_IGNORED:         u16 = 0x100;

    const FLAGS_KERNEL:         u16 = PageDirectoryEntry::FLAG_PRESENT | PageDirectoryEntry::FLAG_RW;

    #[inline(always)]
    pub fn from_addr_4k(addr: u32) -> PageDirectoryEntry {
        PageDirectoryEntry(addr << 12)
    }

    #[inline(always)]
    pub fn page_table_addr(&mut self) -> PhysAddr {
        PhysAddr::from_raw((self.0 & 0xFFFFF000) as usize)
    }

    #[inline]
    pub fn get_pte(&mut self, addr: VirtAddr) -> &mut PageTableEntry {
        unsafe {
            let pt = self.page_table_addr().as_virt_addr().as_mut_ptr::<PageTableEntry>();
            &mut *pt.uoffset(addr.value() >> 12 & 0x03FF)
        }
    }

    #[inline(always)]
    pub fn get_flag_p(&self) -> bool {
        self.0 & (1 << 0) != 0
    }

    #[inline(always)]
    pub fn get_flag_r(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    #[inline(always)]
    pub fn get_flag_u(&self) -> bool {
        self.0 & (1 << 2) != 0
    }

    #[inline(always)]
    pub fn get_flag_w(&self) -> bool {
        self.0 & (1 << 3) != 0
    }

    #[inline(always)]
    pub fn get_flag_d(&self) -> bool {
        self.0 & (1 << 4) != 0
    }

    #[inline(always)]
    pub fn get_flag_a(&self) -> bool {
        self.0 & (1 << 5) != 0
    }

    // 7ビット目は未使用

    #[inline(always)]
    pub fn get_flag_s(&self) -> bool {
        self.0 & (1 << 7) != 0
    }

    #[inline(always)]
    pub fn get_flags(&self) -> u16 {
        (self.0 & 0x1BF) as u16
    }

    #[inline(always)]
    pub fn set_flags(&mut self, flags: u16) {
        self.0 = (self.get_address() << 12) | (flags & 0x1BF) as u32;
    }

    #[inline(always)]
    pub fn get_custom(&self) -> u8 {
        (self.0 >> 8 & 0b111) as u8
    }

    #[inline(always)]
    pub fn get_address(&self) -> u32 {
        self.0 >> 12
    }

    #[inline(always)]
    pub fn set_address(&mut self, addr: u32) {
        self.0 = (addr << 12) | self.get_flags() as u32;
    }
}

#[derive(Clone, Copy)]
struct PageTableEntry(u32);
impl PageTableEntry {
    const BYTES: usize = u32::BYTES;
    const LEN: usize = 1024;
    const SIZE: usize = PageTableEntry::BYTES * PageTableEntry::LEN * PageDirectoryEntry::LEN;

    const FLAG_PRESENT:         u16 = 0x01;
    const FLAG_RW:              u16 = 0x02;
    const FLAG_USER:            u16 = 0x04;
    const FLAG_WRITE_THROUGH:   u16 = 0x08;
    const FLAG_CACHE_DISABLE:   u16 = 0x10;
    const FLAG_ACCESSED:        u16 = 0x20;
    const FLAG_DIRTY:           u16 = 0x40;
    const FLAG_GLOBAL:          u16 = 0x100;

    const FLAGS_KERNEL:         u16 = PageTableEntry::FLAG_PRESENT | PageTableEntry::FLAG_RW;

    #[inline(always)]
    pub fn set_flags(&mut self, flags: u16) {
        self.0 = (self.get_address() << 12) | (flags & 0x17F) as u32;
    }

    #[inline(always)]
    pub fn get_address(&self) -> u32 {
        self.0 >> 12
    }

    #[inline(always)]
    pub fn set_address(&mut self, addr: PhysAddr) {
        self.0 = (addr.value() as u32 & 0xFFFFF000) | (self.0 & 0x00000FFF);
    }
}

pub struct PageTable {
    pd: *mut PageDirectoryEntry,
    pt: *mut PageTableEntry
}

impl PageTable {
    #[inline(always)]
    pub unsafe fn enable() {
        let cr4: u32;
        asm!("mov %cr4, %eax" : "={eax}"(cr4) ::: "volatile");
        asm!("mov %eax, %cr4" :: "{eax}"(cr4 | 0x00000080) :: "volatile");
    }

    #[inline(always)]
    pub unsafe fn disable() {
        let cr4: u32;
        asm!("mov %cr4, %eax" : "={eax}"(cr4) ::: "volatile");
        asm!("mov %eax, %cr4" :: "{eax}"(cr4 & !0x00000080) :: "volatile");
    }

    #[inline(always)]
    pub unsafe fn set(&mut self) {
        let addr = self.pd.as_phys_addr().value();
        asm!("mov %eax, %cr3" :: "{eax}"(addr) :: "volatile");
    }

    #[inline]
    pub fn new() -> PageTable {
        let pd_ptr = memory::kernel::allocate_aligned(PageDirectoryEntry::SIZE, arch::PAGE_SIZE);
        let pt_ptr = memory::kernel::allocate_aligned(PageTableEntry::SIZE, arch::PAGE_SIZE);

        let base_addr = pt_ptr.as_phys_addr().value() as u32 >> 12;

        for i in 0 .. PageDirectoryEntry::LEN {
            unsafe {
                *pd_ptr.uoffset(i) = PageDirectoryEntry::from_addr_4k(base_addr + i as u32);
            }
        }
        unsafe {
            memory::fill32(pt_ptr as *mut u32, 0, PageTableEntry::LEN * PageDirectoryEntry::LEN / u32::BYTES);
        }

        PageTable {
            pd: pd_ptr,
            pt: pt_ptr
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        unsafe {
            Self::disable();
            self.set();
            Self::enable();
        }
    }

    #[inline]
    fn get_pde(&mut self, addr: VirtAddr) -> &mut PageDirectoryEntry {
        let index = addr.value() >> 22 & 0x03FF;
        unsafe { &mut *self.pd.uoffset(index) }
    }

    pub fn map(&mut self, desc_flags: u16, table_flags: u16, virt_addr: VirtAddr, phys_addr: PhysAddr) {
        let pde = self.get_pde(virt_addr);
        //assert!(pde.get_flag_p());
        pde.set_flags(desc_flags);

        let pte = pde.get_pte(virt_addr);
        pte.set_flags(table_flags);
        pte.set_address(phys_addr);
    }

    pub fn map_range(&mut self, desc_flags: u16, table_flags: u16, virt_range: ops::Range<VirtAddr>,
                     phys_range: ops::Range<PhysAddr>)
    {
        let virt_addr_range = virt_range.start.value() .. virt_range.end.value();
        let phys_addr_range = phys_range.start.value() .. phys_range.end.value();

        for (vaddr, paddr) in virt_addr_range.step_by(arch::PAGE_SIZE).zip(phys_addr_range.step_by(arch::FRAME_SIZE)) {
            self.map(desc_flags, table_flags, VirtAddr::from_raw(vaddr), PhysAddr::from_raw(paddr));
        }
    }

    pub fn map_direct(&mut self, desc_flags: u16, table_flags: u16, addr_range: ops::Range<PhysAddr>) {
        let virt_addr_range = VirtAddr::from_raw(addr_range.start.value())
            .. VirtAddr::from_raw(addr_range.end.value());
        self.map_range(desc_flags, table_flags, virt_addr_range, addr_range);
    }
}

//const PDE_PTE_SIZE: usize = PageDirectoryEntry::SIZE + PageTableEntry::SIZE;

pub static mut kernel_pt: PageTable = PageTable {
    pd: 0 as *mut PageDirectoryEntry,
    pt: 0 as *mut PageTableEntry
    /*pd: &mut [PageDirectoryEntry(0); PageDirectoryEntry::LEN],
    pt: &mut [PageTableEntry(0); PageTableEntry::LEN]*/
};

#[inline]
pub fn init() {
    /*let (pd, base_addr) = unsafe {
        kernel_pt = PageTable {
            pd: slice::from_raw_parts_mut(
                     memory::kernel::allocate_aligned(PageDirectoryEntry::SIZE, arch::PAGE_SIZE),
                     PageDirectoryEntry::LEN
            ),
            pt: slice::from_raw_parts_mut(
                memory::kernel::allocate_aligned(PageTableEntry::SIZE, arch::PAGE_SIZE),
                PageTableEntry::LEN
            )
        };
        (
            &mut kernel_pt.pd,
            (virt_to_phys_addr(kernel_pt.pd.as_ptr() as usize) + PageDirectoryEntry::SIZE) as u32 >> 12
        )
    };

    let mut i = 0_u32;
    for pde in pd.iter_mut() {
        *pde = PageDirectoryEntry::from_addr(base_addr + i);
        i += 1;
    }

    unsafe {
        PageTable::disable();
        kernel_pt.set();
        PageTable::enable();
    }*/

    log!("Kernel is positioned: {:X} .. {:X}",
         arch::kernel_start().value(), arch::kernel_end().value());

    unsafe {
        kernel_pt = PageTable::new();
        kernel_pt.map_range(PageDirectoryEntry::FLAGS_KERNEL, PageTableEntry::FLAGS_KERNEL,
                            arch::kernel_start() .. memory::kernel::kernel_space_end(),
                            arch::kernel_start().as_phys_addr() .. memory::kernel::kernel_space_end().as_phys_addr());

        kernel_pt.reset();
    }
}

/*#[inline(always)]
pub fn virt_to_phys_addr(addr: usize) -> usize {
    assert!(virtual_addr <= arch::KERNEL_BASE, "Out of kernel space: {:X} > {:X}", virtual_addr, arch::KERNEL_BASE);
    virtual_addr - arch::KERNEL_BASE
}

#[inline(always)]
pub fn phys_to_virt_addr(physical_addr: usize) -> usize {
    if physical_addr <= virt_to_phys_addr(arch::kernel_end()) {
        physical_addr + arch::KERNEL_BASE
    } else {
        unimplemented!();
        //phys_addr_to_frame()
    }
}*/

/*#[inline]
pub fn fix<T>(ptr: *mut T, page_count: usize) -> Option<*mut T> {
    const KERNEL_SIZE: usize = arch::KERNEL_END - arch::KERNEL_BASE;
    let addr: usize = unsafe { mem::transmute(ptr) };
    if addr < KERNEL_SIZE && (KERNEL_SIZE - addr >> 10) > page_count {
        Some(unsafe { mem::transmute(arch::KERNEL_BASE + addr) })
    } else {
        None
    }
}

#[inline]
pub fn map_auto<T>(ptr: *mut T, count: usize) -> Option<*mut T> {
    if let Some(fixed) = fix(ptr, count) {
        Some(fixed)
    } else {
        None
    }
}*/

