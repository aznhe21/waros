use core::prelude::*;
use rt;
use arch;

pub trait AsPhysAddr<T: ?Sized> {
    fn as_phys_addr(self) -> PhysAddr where T: Sized;
}

impl<T: ?Sized> AsPhysAddr<T> for *const T {
    #[inline(always)]
    fn as_phys_addr(self) -> PhysAddr where T: Sized {
        VirtAddr(self as usize).as_phys_addr()
    }
}

impl<T: ?Sized> AsPhysAddr<T> for *mut T {
    #[inline(always)]
    fn as_phys_addr(self) -> PhysAddr where T: Sized {
        VirtAddr(self as usize).as_phys_addr()
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct PhysAddr(usize);

impl PhysAddr {
    #[inline(always)]
    pub fn from_raw(addr: usize) -> PhysAddr {
        PhysAddr(addr)
    }

    #[inline(always)]
    pub fn from_ptr<T>(ptr: *const T) -> PhysAddr {
        PhysAddr(ptr as usize)
    }

    #[inline(always)]
    pub fn from_mut_ptr<T>(ptr: *mut T) -> PhysAddr {
        PhysAddr(ptr as usize)
    }

    #[inline(always)]
    pub fn value(&self) -> usize {
        self.0
    }

    pub fn as_virt_addr(&self) -> VirtAddr {
        if *self <= kernel_space_end().as_phys_addr() {
            VirtAddr(self.0 + arch::KERNEL_BASE)
        } else {
            panic!("as_virt_addr: {:X} > {:X}", self.0, kernel_space_end().as_phys_addr().0);
            // phys_addr_to_frame
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct VirtAddr(usize);

impl VirtAddr {
    #[inline(always)]
    pub fn from_raw(addr: usize) -> VirtAddr {
        VirtAddr(addr)
    }

    #[inline(always)]
    pub fn from_ptr<T>(ptr: *const T) -> VirtAddr {
        VirtAddr(ptr as usize)
    }

    #[inline(always)]
    pub fn from_mut_ptr<T>(ptr: *mut T) -> VirtAddr {
        VirtAddr(ptr as usize)
    }

    #[inline(always)]
    pub fn value(&self) -> usize {
        self.0
    }

    #[inline(always)]
    pub fn as_phys_addr(&self) -> PhysAddr {
        assert!(*self <= kernel_space_end(), "Out of kernel space: {:X} > {:X}", self.0, arch::KERNEL_BASE);
        PhysAddr(self.0 - arch::KERNEL_BASE)
    }

    #[inline(always)]
    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[inline(always)]
    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

static mut address: usize = 0;

#[inline]
pub fn init() {
    unsafe {
        address = arch::kernel_end().value();
    }
}

#[inline(always)]
pub fn kernel_space_end() -> VirtAddr {
    VirtAddr::from_raw(unsafe { address })
}

#[inline]
pub fn allocate<T>(size: usize) -> *mut T {
    unsafe {
        let addr = address;
        address += size;
        addr as *mut T
    }
}

#[inline]
pub fn allocate_aligned<T>(size: usize, align: usize) -> *mut T {
    unsafe {
        address = rt::align_up(address, align);
    }
    allocate(size)
}

