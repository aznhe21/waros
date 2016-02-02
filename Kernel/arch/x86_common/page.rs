use arch;
use memory;
use memory::buddy::PageFrame;
use memory::kernel::{PhysAddr, VirtAddr};
use core::slice;
use core::ptr::{self, Shared};
use core::{u32, usize};

const FRAME_SIZE_ADDR: arch::AddrType = arch::FRAME_SIZE as arch::AddrType;

// TODO: Support PAE
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
    pub const fn from_4kb_aligned(addr: u32) -> PageDirectoryEntry {
        PageDirectoryEntry(addr << 12)
    }

    #[inline(always)]
    pub fn page_table(&mut self) -> *mut PageTableEntry {
        PhysAddr::from_raw((self.0 & 0xFFFFF000) as arch::AddrType).as_virt_addr().as_mut_ptr()
    }

    #[inline]
    pub fn get_pte(&mut self, addr: VirtAddr) -> &mut PageTableEntry {
        let index = addr.value() >> 12 & 0x03FF;
        debug_assert!(index < PageTableEntry::LEN);
        unsafe {
            &mut *self.page_table().offset(index as isize)
        }
    }

    #[inline]
    pub fn as_slice(&mut self) -> &'static mut [PageTableEntry] {
        unsafe {
            slice::from_raw_parts_mut(self.page_table(), PageTableEntry::LEN)
        }
    }

    #[inline(always)]
    pub fn get_flags(&self) -> u16 {
        (self.0 & 0x1BF) as u16
    }

    #[inline(always)]
    pub fn set_flags(&mut self, flags: u16) {
        self.0 = (self.0 & 0xFFFFFE00) | (flags & 0x1BF) as u32;
    }

    #[inline(always)]
    pub fn get_custom(&self) -> u8 {
        (self.0 >> 9 & 0b111) as u8
    }

    #[inline(always)]
    pub fn get_raw_address(&self) -> u32 {
        self.0 >> 12
    }

    #[inline(always)]
    pub fn get_address(&self) -> PhysAddr {
        PhysAddr::from_raw(self.get_raw_address() as arch::AddrType)
    }

    #[inline(always)]
    pub fn set_address(&mut self, addr: PhysAddr) {
        self.0 = (addr.value() as u32 & 0xFFFFF000) | (self.0 & 0x00000FFF);
    }
}

struct PageTableEntry(u32);
impl PageTableEntry {
    const BYTES: usize = u32::BYTES;
    const LEN: usize = 1024;
    const TOTAL_LEN: usize = PageTableEntry::LEN * PageDirectoryEntry::LEN;
    const SIZE: usize = PageTableEntry::BYTES * PageTableEntry::TOTAL_LEN;

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
    pub fn get_flags(&self) -> u16 {
        (self.0 & 0x17F) as u16
    }

    #[inline(always)]
    pub fn set_flags(&mut self, flags: u16) {
        self.0 = (self.0 & 0xFFFFF000) | (flags & 0x17F) as u32;
    }

    #[inline(always)]
    pub fn get_raw_address(&self) -> u32 {
        self.0 >> 12
    }

    #[inline(always)]
    pub fn get_address(&self) -> PhysAddr {
        PhysAddr::from_raw(self.get_raw_address() as arch::AddrType)
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
    pub const FLAGS_KERNEL: (u16, u16) = (PageDirectoryEntry::FLAGS_KERNEL, PageTableEntry::FLAGS_KERNEL);

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
        let addr = VirtAddr::from_ptr(self.pd).as_phys_addr().value() as u32;
        asm!("mov %eax, %cr3" :: "{eax}"(addr) :: "volatile");
    }

    #[inline]
    fn new() -> PageTable {
        unsafe {
            let pd_ptr = memory::kernel::allocate_raw(PageDirectoryEntry::SIZE, arch::PAGE_SIZE) as
                *mut PageDirectoryEntry;
            let pt_ptr = memory::kernel::allocate_raw(PageTableEntry::SIZE, arch::PAGE_SIZE) as
                *mut PageTableEntry;

            let base_addr = VirtAddr::from_ptr(pt_ptr).as_phys_addr().value() as u32 >> 12;

            for i in 0 .. PageDirectoryEntry::LEN {
                *pd_ptr.offset(i as isize) = PageDirectoryEntry::from_4kb_aligned(base_addr + i as u32);
            }
            memory::fill32(pt_ptr as *mut u32, 0, PageTableEntry::LEN * PageDirectoryEntry::LEN / u32::BYTES);

            PageTable {
                pd: pd_ptr,
                pt: pt_ptr
            }
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

    #[inline(always)]
    fn get_pde_index(addr: VirtAddr) -> usize {
        addr.value() >> 22 & 0x03FF
    }

    #[inline]
    fn get_pde(&mut self, addr: VirtAddr) -> &mut PageDirectoryEntry {
        unsafe { &mut *self.pd.offset(PageTable::get_pde_index(addr) as isize) }
    }

    #[inline]
    fn as_slice(&mut self) -> &'static mut [PageDirectoryEntry] {
        unsafe { slice::from_raw_parts_mut(self.pd, PageDirectoryEntry::LEN) }
    }

    fn map(&mut self, (desc_flags, table_flags): (u16, u16), virt_addr: VirtAddr, phys_addr: PhysAddr) {
        let pde = self.get_pde(virt_addr);
        //assert!(!pde.get_flag_p());
        pde.set_flags(desc_flags);

        let pte = pde.get_pte(virt_addr);
        pte.set_flags(table_flags);
        pte.set_address(phys_addr);
    }

    fn map_range(&mut self, flags: (u16, u16), virt_addr: VirtAddr, phys_addr: PhysAddr, size: usize)
    {
        let virt_range = virt_addr.value() .. virt_addr.value() + size;
        let phys_range = phys_addr.value() .. phys_addr.value() + size as arch::AddrType;

        for (virt_addr, phys_addr) in virt_range.step_by(arch::PAGE_SIZE).zip(phys_range.step_by(FRAME_SIZE_ADDR)) {
            self.map(flags, VirtAddr::from_raw(virt_addr), PhysAddr::from_raw(phys_addr));
        }
    }

    pub fn map_direct(&mut self, flags: (u16, u16), phys_addr: PhysAddr, size: usize) {
        assert!(phys_addr.value().checked_add(size as arch::AddrType)
                .map_or(false, |addr| addr <= usize::MAX as arch::AddrType));

        let virt_addr = VirtAddr::from_raw(phys_addr.value() as usize);
        self.map_range(flags, virt_addr, phys_addr, size);
    }

    fn find_free_addr(&mut self, size: usize) -> VirtAddr {
        let map_pages = (size + arch::PAGE_SIZE - 1) / arch::PAGE_SIZE;
        let pde_index = 1;
        let mut begin_addr = 0;
        let mut free_pages = 0;

        //for (pde, pde_addr) in self.as_slice()[pde_index..].iter_mut().zip((pde_index << 22 ..).step_by(1 << 22)) {
        //    for (pte, pte_addr) in pde.as_slice().iter_mut().zip((pde_addr..).step_by(1 << 12)) {
        for (i, pde) in self.as_slice()[pde_index..].iter_mut().enumerate() {
            let pde_addr = (pde_index + i) << 22;
            for (j, pte) in pde.as_slice().iter_mut().enumerate() {
                let pte_addr = pde_addr + (j << 12);
                if pte.get_flags() & PageTableEntry::FLAG_PRESENT != 0 {
                    begin_addr = 0;
                } else {
                    if begin_addr == 0 {
                        begin_addr = pte_addr;
                        free_pages = 0;
                    }

                    free_pages += 1;
                    if free_pages >= map_pages {
                        return VirtAddr::from_raw(begin_addr);
                    }
                }
            }
        }

        VirtAddr::null()
    }

    pub fn map_memory(&mut self, flags: (u16, u16), page: Shared<PageFrame>, size: usize) -> VirtAddr {
        let virt_addr = self.find_free_addr(size);
        let phys_addr = unsafe { (**page).addr() };
        if !virt_addr.is_null() {
            self.map_range(flags, virt_addr, phys_addr, size);
        } else {
            debug_log!("Unable to map a page {:p}", unsafe { (**page).addr() });
        }

        virt_addr
    }
}

static mut kernel_pt: PageTable = PageTable {
    pd: ptr::null_mut(),
    pt: ptr::null_mut()
};

#[inline]
pub fn pre_init() {
    unsafe {
        kernel_pt = PageTable::new();
    }
}

#[inline]
pub fn init() {
    unsafe {
        let memory_start = PhysAddr::from_raw(0);
        let memory_start_virt = memory_start.as_virt_addr();
        let size = memory::kernel::end_addr() - memory_start_virt;
        kernel_pt.map_range(PageTable::FLAGS_KERNEL, memory_start_virt, memory_start, size);

        kernel_pt.reset();
    }
}

#[inline(always)]
pub fn table() -> &'static mut PageTable {
    unsafe {
        &mut kernel_pt
    }
}

