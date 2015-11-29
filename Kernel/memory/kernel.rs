use rt;
use arch;
use core::fmt;
use core::ops::{Add, Sub};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct PhysAddr(u64);

impl PhysAddr {
    #[inline(always)]
    pub const fn from_raw(addr: u64) -> PhysAddr {
        PhysAddr(addr)
    }

    #[inline(always)]
    pub const fn null() -> PhysAddr {
        PhysAddr(0)
    }

    #[inline(always)]
    pub const fn value(&self) -> u64 {
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
}

impl Add<u64> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: u64) -> PhysAddr {
        PhysAddr(self.0 + rhs)
    }
}

impl Sub<u64> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: u64) -> PhysAddr {
        PhysAddr(self.0 - rhs)
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.value() as *const usize), f)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
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
        PhysAddr((self.value() - arch::KERNEL_BASE) as u64)
    }

    #[inline(always)]
    pub const fn as_ptr<T>(&self) -> *const T {
        self.value() as *const T
    }

    #[inline(always)]
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.value() as *mut T
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
pub fn init() {
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
            KernelMemory::Available(address) => {
                let addr = VirtAddr::from_raw(rt::align_up(address.value(), arch::FRAME_SIZE));
                memory = KernelMemory::End(addr);
                addr
            },
            KernelMemory::End(_) => panic!("Already ended")
        }
    }
}

#[inline]
pub fn allocate(size: usize, align: usize) -> *mut u8 {
    unsafe {
        if let KernelMemory::Available(VirtAddr(old_addr)) = memory {
            let addr = rt::align_up(old_addr, align);

            memory = KernelMemory::Available(VirtAddr::from_raw(addr + size));
            addr as *mut u8
        } else {
            panic!("Unable to allocate after kernel space");
        }
    }
}

