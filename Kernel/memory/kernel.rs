use rt;
use arch;
use core::mem;
use core::fmt;
use core::ptr::{self, Unique};
use core::ops::{Add, Sub};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(arch::AddrType);

impl PhysAddr {
    #[inline(always)]
    pub const fn from_raw(addr: arch::AddrType) -> PhysAddr {
        PhysAddr(addr)
    }

    #[inline(always)]
    pub const fn null() -> PhysAddr {
        PhysAddr(0)
    }

    #[inline(always)]
    pub const fn value(&self) -> arch::AddrType {
        self.0
    }

    #[inline(always)]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn as_virt_addr(&self) -> VirtAddr {
        if *self <= kernel_memory().as_phys_addr() {
            VirtAddr::from_raw(self.value() as usize + arch::KERNEL_BASE)
        } else {
            panic!("as_virt_addr: {:?} > {:?}", self, kernel_memory().as_phys_addr());
            // phys_addr_to_frame
        }
    }

    #[inline(always)]
    pub fn align_up(&self, align: arch::AddrType) -> PhysAddr {
        PhysAddr(rt::align_up(self.0, align))
    }

    #[inline(always)]
    pub fn align_down(&self, align: arch::AddrType) -> PhysAddr {
        PhysAddr(rt::align_down(self.0, align))
    }
}

impl Add<arch::AddrType> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: arch::AddrType) -> PhysAddr {
        PhysAddr(self.0 + rhs)
    }
}

impl Sub<arch::AddrType> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: arch::AddrType) -> PhysAddr {
        PhysAddr(self.0 - rhs)
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.value() as *const usize), f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);

impl VirtAddr {
    #[inline(always)]
    pub const fn from_raw(addr: usize) -> VirtAddr {
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
    pub const fn null() -> VirtAddr {
        VirtAddr(0)
    }

    #[inline(always)]
    pub const fn value(&self) -> usize {
        self.0
    }

    #[inline(always)]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub fn as_phys_addr(&self) -> PhysAddr {
        assert!(*self <= kernel_memory(), "Out of kernel space: {:?} > {:?}", self, kernel_memory());
        PhysAddr((self.value() - arch::KERNEL_BASE) as arch::AddrType)
    }

    #[inline(always)]
    pub const fn as_ptr<T>(&self) -> *const T {
        self.value() as *const T
    }

    #[inline(always)]
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.value() as *mut T
    }

    #[inline(always)]
    pub fn align_up(&self, align: usize) -> VirtAddr {
        VirtAddr(rt::align_up(self.0, align))
    }

    #[inline(always)]
    pub fn align_down(&self, align: usize) -> VirtAddr {
        VirtAddr(rt::align_down(self.0, align))
    }
}

impl Add<usize> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: usize) -> VirtAddr {
        VirtAddr(self.0 + rhs)
    }
}

impl Sub<usize> for VirtAddr {
    type Output = VirtAddr;

    fn sub(self, rhs: usize) -> VirtAddr {
        VirtAddr(self.0 - rhs)
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.value() as *const usize), f)
    }
}

enum KernelMemory {
    Uninit,
    Available(VirtAddr),
    End(VirtAddr)
}

impl KernelMemory {
    #[inline(always)]
    pub fn addr(&self) -> VirtAddr {
        match *self {
            KernelMemory::Uninit => VirtAddr::null(),
            KernelMemory::Available(address) => address,
            KernelMemory::End(address) => address
        }
    }
}

static mut memory: KernelMemory = KernelMemory::Uninit;

#[inline]
pub fn pre_init() {
    unsafe {
        memory = KernelMemory::Available(arch::kernel_end());
    }
}

#[inline(always)]
pub fn kernel_memory() -> VirtAddr {
    unsafe { memory.addr() }
}

#[inline(always)]
pub fn memory_end() -> VirtAddr {
    unsafe {
        match memory {
            KernelMemory::Uninit => panic!("Uninitialized"),
            KernelMemory::Available(addr) => {
                let addr = addr.align_up(arch::FRAME_SIZE);
                memory = KernelMemory::End(addr);
                addr
            },
            KernelMemory::End(_) => panic!("Already ended")
        }
    }
}

pub unsafe fn allocate_raw(size: usize, align: usize) -> *mut u8 {
    if let KernelMemory::Available(addr) = memory {
        let addr = addr.align_up(align);
        memory = KernelMemory::Available(addr + size);
        addr.as_mut_ptr()
    } else {
        panic!("Unable to allocate after kernel space");
    }
}

#[inline]
pub unsafe fn allocate_uninit<T>() -> Unique<T> {
    Unique::new(allocate_raw(mem::size_of::<T>(), mem::align_of::<T>()) as *mut T)
}

#[inline]
pub fn allocate<T>(x: T) -> Unique<T> {
    unsafe {
        let p = allocate_uninit();
        ptr::write(*p, x);
        p
    }
}

