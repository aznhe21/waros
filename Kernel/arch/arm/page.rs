use arch::{self, mach};
use memory;
use memory::buddy::PageFrame;
use memory::kernel::{PhysAddr, VirtAddr};
use core::mem;
use core::slice;
use core::ptr::{self, Shared};
use core::{u32, usize};

const FRAME_SIZE_ADDR: arch::AddrType = arch::FRAME_SIZE as arch::AddrType;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FirstLevelDescriptorType {
    Invalid     = 0b00,
    CoarseTable = 0b01,
    Section     = 0b10,
    FineTable   = 0b11
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DomainAccessControl {
    /// Any access generates a domain fault.
    NoAccess = 0b00,
    /// Accesses are checked against the access permission bits in the section or page descriptor.
    Client   = 0b01,
    /// Reserved.
    Reserved = 0b10,
    /// Accesses are not checked against the access permission bits so a permission fault cannot be generated.
    Manager  = 0b11
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AccessPermission {
    AP0 = 0b00,
    AP1 = 0b01,
    AP2 = 0b10,
    AP3 = 0b11
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FirstLevelDescriptor(u32);

impl FirstLevelDescriptor {
    pub const BYTES: usize = u32::BYTES;
    pub const LEN: usize = 4096;
    pub const SIZE: usize = FirstLevelDescriptor::BYTES * FirstLevelDescriptor::LEN;
    pub const ALIGN: usize = 0x4000;

    const INVALID_DESCRIPTOR:           u32 = 0b10000;
    const COARSE_PAGE_TABLE_DESCRIPTOR: u32 = 0b10001;
    const SECTION_DESCRIPTOR:           u32 = 0b10010;
    const FINE_PAGE_TABLE_DESCRIPTOR:   u32 = 0b10011;

    /// Generates a section translation fault.
    #[inline(always)]
    pub const fn invalid() -> FirstLevelDescriptor {
        FirstLevelDescriptor(FirstLevelDescriptor::INVALID_DESCRIPTOR)
    }

    /// a coarse page table descriptor.
    #[inline(always)]
    pub const fn coarse_table(addr: u32, domain: DomainAccessControl) -> FirstLevelDescriptor {
        FirstLevelDescriptor(FirstLevelDescriptor::COARSE_PAGE_TABLE_DESCRIPTOR |
                             addr & 0xFFFFFC00 |
                             (domain as u32) << 5)
    }

    /// a section descriptor.
    #[inline(always)]
    pub const fn section(addr: u32, ap: AccessPermission, domain: DomainAccessControl, cache: bool,
                     buffer: bool) -> FirstLevelDescriptor {
        FirstLevelDescriptor(FirstLevelDescriptor::SECTION_DESCRIPTOR |
                             addr & 0xFFF00000 |
                             (ap as u32) << 10 |
                             (domain as u32) << 5 |
                             (cache as u32) << 3 |
                             (buffer as u32) << 2)
    }

    /// a fine page table descriptor.
    #[inline(always)]
    pub const fn fine_table(addr: u32, domain: DomainAccessControl) -> FirstLevelDescriptor {
        FirstLevelDescriptor(FirstLevelDescriptor::FINE_PAGE_TABLE_DESCRIPTOR as u32 |
                             addr & 0xFFFFF000 |
                             (domain as u32) << 5)
    }

    #[inline(always)]
    pub fn descriptor_type(&self) -> FirstLevelDescriptorType {
        unsafe {
            mem::transmute(self.0 as u8 & 0b11)
        }
    }

    #[inline(always)]
    pub fn coarse_ptr(&self) -> *mut SecondLevelDescriptor {
        (self.0 & 0xFFFFFC00) as *mut SecondLevelDescriptor
    }

    #[inline(always)]
    pub fn fine_ptr(&self) -> *mut SecondLevelDescriptor {
        (self.0 & 0xFFFFF000) as *mut SecondLevelDescriptor
    }

    pub fn sld(&self) -> Option<&'static mut [SecondLevelDescriptor]> {
        unsafe {
            match self.descriptor_type() {
                FirstLevelDescriptorType::Invalid | FirstLevelDescriptorType::Section => None,
                FirstLevelDescriptorType::CoarseTable =>
                    Some(slice::from_raw_parts_mut(self.coarse_ptr(), SecondLevelDescriptor::COARSE_LEN)),
                FirstLevelDescriptorType::FineTable =>
                    Some(slice::from_raw_parts_mut(self.fine_ptr(), SecondLevelDescriptor::FINE_LEN))
            }
        }
    }

    #[inline]
    pub fn get(&self, addr: VirtAddr) -> Option<&'static mut SecondLevelDescriptor> {
        unsafe {
            match self.descriptor_type() {
                FirstLevelDescriptorType::Invalid | FirstLevelDescriptorType::Section => None,
                FirstLevelDescriptorType::CoarseTable =>
                    Some(&mut *self.coarse_ptr().offset((addr.value() >> 12 & 0xFF) as isize)),
                FirstLevelDescriptorType::FineTable =>
                    Some(&mut *self.fine_ptr().offset((addr.value() >> 10 & 0x3FF) as isize))
            }
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SecondLevelDescriptorType {
    Fault = 0b00,
    Large = 0b01,
    Small = 0b10,
    Tiny  = 0b11
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SecondLevelDescriptor(u32);

impl SecondLevelDescriptor {
    pub const BYTES: usize = u32::BYTES;
    pub const ALIGN: usize = 4;

    pub const COARSE_LEN: usize = 256;
    pub const FINE_LEN: usize = 1024;

    const FAULT_DESCRIPTOR: u32 = 0b00;
    const LARGE_DESCRIPTOR: u32 = 0b01;
    const SMALL_DESCRIPTOR: u32 = 0b10;
    const TINY_DESCRIPTOR:  u32 = 0b11;

    /// Generates a page translation fault.
    #[inline(always)]
    pub const fn fault() -> SecondLevelDescriptor {
        SecondLevelDescriptor(SecondLevelDescriptor::FAULT_DESCRIPTOR)
    }

    /// 64KB page.
    #[inline(always)]
    pub const fn large(addr: PhysAddr, ap0: AccessPermission, ap1: AccessPermission, ap2: AccessPermission,
                       ap3: AccessPermission, cache: bool, buffer: bool) -> SecondLevelDescriptor {
        SecondLevelDescriptor(SecondLevelDescriptor::LARGE_DESCRIPTOR |
                              addr.value() & 0xFFFF0000 |
                              (ap3 as u32) << 10 |
                              (ap2 as u32) << 8 |
                              (ap1 as u32) << 6 |
                              (ap0 as u32) << 4 |
                              (cache as u32) << 3 |
                              (buffer as u32) << 2)
    }

    /// 4KB page.
    #[inline(always)]
    pub const fn small(addr: PhysAddr, ap0: AccessPermission, ap1: AccessPermission, ap2: AccessPermission,
                       ap3: AccessPermission, cache: bool, buffer: bool) -> SecondLevelDescriptor {
        SecondLevelDescriptor(SecondLevelDescriptor::SMALL_DESCRIPTOR |
                              addr.value() & 0xFFFFF000 |
                              (ap3 as u32) << 10 |
                              (ap2 as u32) << 8 |
                              (ap1 as u32) << 6 |
                              (ap0 as u32) << 4 |
                              (cache as u32) << 3 |
                              (buffer as u32) << 2)
    }

    /// 1KB page.
    #[inline(always)]
    pub const fn tiny(addr: PhysAddr, ap: AccessPermission, cache: bool, buffer: bool) -> SecondLevelDescriptor {
        SecondLevelDescriptor(SecondLevelDescriptor::TINY_DESCRIPTOR |
                              addr.value() & 0xFFFFFC00 |
                              (ap as u32) << 4 |
                              (cache as u32) << 3 |
                              (buffer as u32) << 2)
    }

    #[inline]
    pub fn descriptor_type(&self) -> SecondLevelDescriptorType {
        unsafe {
            mem::transmute(self.0 as u8 & 0b11)
        }
    }

    #[inline]
    pub fn large_addr(&self) -> PhysAddr {
        PhysAddr::from_raw(self.0 & 0xFFFF0000)
    }

    #[inline]
    pub fn small_addr(&self) -> PhysAddr {
        PhysAddr::from_raw(self.0 & 0xFFFFF000)
    }

    #[inline]
    pub fn tiny_addr(&self) -> PhysAddr {
        PhysAddr::from_raw(self.0 & 0xFFFFFC00)
    }
}

pub struct PageTable {
    fld_ptr: *mut FirstLevelDescriptor,
    sld_ptr: *mut SecondLevelDescriptor,
}

impl PageTable {
    pub const FLAGS_KERNEL: (bool, bool) = (true, true);

    const LEN: usize = 4096;

    #[inline(always)]
    pub fn new() -> PageTable {
        unsafe {
            let sld_len = SecondLevelDescriptor::COARSE_LEN;
            let sld_size = FirstLevelDescriptor::LEN * SecondLevelDescriptor::BYTES * sld_len;

            let fld_ptr = memory::kernel::allocate_raw(FirstLevelDescriptor::SIZE, FirstLevelDescriptor::ALIGN) as
                *mut FirstLevelDescriptor;
            let sld_ptr = memory::kernel::allocate_raw(sld_size, SecondLevelDescriptor::ALIGN) as
                *mut SecondLevelDescriptor;
            memory::fill32(fld_ptr as *mut u32, FirstLevelDescriptor::invalid().0, FirstLevelDescriptor::LEN);

            PageTable {
                fld_ptr: fld_ptr,
                sld_ptr: sld_ptr
            }
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        unsafe {
            let addr = self.fld_ptr as usize;

            // テーブルのアドレスとフラグを設定
            asm!("mcr p15, 0, $0, c2, c0, 0" :: "r"(addr) :: "volatile");
            /* Set the access control to all-supervisor */
            asm!("mcr p15, 0, $0, c3, c0, 0" :: "r"(!0) :: "volatile");

            self.enable();
        }
    }

    #[inline]
    pub fn enable(&mut self) {
        unsafe {
            // MMUを有効化
            let reg: u32;
            asm!("mrc p15, 0, $0, c1, c0, 0" : "=r"(reg) ::: "volatile");
            asm!("mcr p15, 0, $0, c1, c0, 0" :: "r"(reg | 1 << 0) :: "volatile");
        }
    }

    #[inline]
    pub fn disable(&mut self) {
        unsafe {
            // MMUを無効化
            let reg: u32;
            asm!("mrc p15, 0, $0, c1, c0, 0" : "=r"(reg) ::: "volatile");
            asm!("mcr p15, 0, $0, c1, c0, 0" :: "r"(reg & !(1 << 0)) :: "volatile");
        }
    }

    #[inline]
    pub fn fld(&self) -> &'static mut [FirstLevelDescriptor] {
        unsafe {
            slice::from_raw_parts_mut(self.fld_ptr, PageTable::LEN)
        }
    }

    #[inline]
    pub fn get(&self, addr: VirtAddr) -> &'static mut FirstLevelDescriptor {
        unsafe {
            &mut *self.fld_ptr.offset((addr.value() >> 20) as isize)
        }
    }

    #[inline]
    pub fn get_sld(&self, addr: VirtAddr) -> Option<&'static mut SecondLevelDescriptor> {
        self.get(addr).get(addr)
    }

    // セクションによる初期マッピング
    fn map_all(&mut self) {
        for (idx, fld) in self.fld().iter_mut().enumerate() {
            *fld = FirstLevelDescriptor::section((idx as arch::AddrType) << 20,
                                                 AccessPermission::AP3, DomainAccessControl::Manager,
                                                 true, true);
        }
    }

    // 初期マッピングからすべてFaultするCoarse Tableに置き換え
    fn unmap_all(&mut self) {
        unsafe {
            let sld_len = SecondLevelDescriptor::COARSE_LEN;
            memory::fill32(self.sld_ptr as *mut u32, SecondLevelDescriptor::fault().0,
                FirstLevelDescriptor::LEN * sld_len);

            for i in 0 .. FirstLevelDescriptor::LEN {
                let sld_ptr = self.sld_ptr.offset((i * sld_len) as isize);

                *self.fld_ptr.offset(i as isize) = FirstLevelDescriptor::coarse_table(sld_ptr as u32,
                                                                                      DomainAccessControl::Manager);
            }
        }
    }

    fn map(&mut self, (cache, buffer): (bool, bool), virt_addr: VirtAddr, phys_addr: PhysAddr) {
        let ap = AccessPermission::AP3;

        let sld = SecondLevelDescriptor::small(phys_addr, ap, ap, ap, ap, cache, buffer);
        *self.get_sld(virt_addr).unwrap() = sld;
    }

    fn map_range(&mut self, flags: (bool, bool), virt_addr: VirtAddr, phys_addr: PhysAddr, size: usize) {
        let virt_range = virt_addr.value() .. virt_addr.value() + size;
        let phys_range = phys_addr.value() .. phys_addr.value() + size as arch::AddrType;

        for (virt_addr, phys_addr) in virt_range.step_by(arch::PAGE_SIZE).zip(phys_range.step_by(FRAME_SIZE_ADDR)) {
            self.map(flags, VirtAddr::from_raw(virt_addr), PhysAddr::from_raw(phys_addr));
        }
    }

    pub fn map_direct(&mut self, flags: (bool, bool), phys_addr: PhysAddr, size: usize) {
        assert!(phys_addr.value().checked_add(size as arch::AddrType)
                .map_or(false, |addr| addr <= usize::MAX as arch::AddrType));

        let virt_addr = VirtAddr::from_raw(phys_addr.value() as usize);
        self.map_range(flags, virt_addr, phys_addr, size);
    }

    pub fn map_memory(&mut self, flags: (bool, bool), page: Shared<PageFrame>, size: usize) -> VirtAddr {
        let phys_addr = unsafe { (**page).addr() };
        let virt_addr = VirtAddr::from_raw(phys_addr.value() as usize);

        self.map_direct(flags, phys_addr, size);

        virt_addr
    }
}

static mut kernel_pt: PageTable = PageTable {
    fld_ptr: ptr::null_mut(),
    sld_ptr: ptr::null_mut()
};

#[inline]
pub fn pre_init() {
    unsafe {
        kernel_pt = PageTable::new();

        // All 4GB
        kernel_pt.map_all();

        kernel_pt.reset();
    }
}

#[inline]
pub fn init() {
    unsafe {
        // unmap_allするとすべてのメモリにアクセスできなくなるので無効化
        kernel_pt.disable();

        // All 4GB
        kernel_pt.unmap_all();

        // Vectors
        kernel_pt.map_direct(PageTable::FLAGS_KERNEL,
                             PhysAddr::from_raw(0), usize::BYTES * 8);

        // Registers
        mach::map_pages(&mut kernel_pt);

        // RAM
        let memory_start = arch::kernel_start();
        kernel_pt.map_direct(PageTable::FLAGS_KERNEL,
                             memory_start.as_phys_addr(), memory::kernel::end_addr() - memory_start);

        kernel_pt.enable();
    }
}

#[inline(always)]
pub fn table() -> &'static mut PageTable {
    unsafe {
        &mut kernel_pt
    }
}

