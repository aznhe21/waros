use rt;
use arch;
use core::mem;
use core::fmt;
use core::ptr::{self, Unique};
use core::ops::{Add, AddAssign, Sub, SubAssign};

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
        if *self <= end_addr().as_phys_addr() {
            VirtAddr::from_raw(self.value() as usize + arch::KERNEL_BASE)
        } else {
            panic!("as_virt_addr: {:?} > {:?}", self, end_addr().as_phys_addr());
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

    #[inline(always)]
    fn add(self, rhs: arch::AddrType) -> PhysAddr {
        PhysAddr(self.0 + rhs)
    }
}

impl AddAssign<arch::AddrType> for PhysAddr {
    #[inline(always)]
    fn add_assign(&mut self, rhs: arch::AddrType) {
        self.0 += rhs;
    }
}

impl Sub<arch::AddrType> for PhysAddr {
    type Output = PhysAddr;

    #[inline(always)]
    fn sub(self, rhs: arch::AddrType) -> PhysAddr {
        PhysAddr(self.0 - rhs)
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = arch::AddrType;

    #[inline(always)]
    fn sub(self, rhs: PhysAddr) -> arch::AddrType {
        self.0 - rhs.0
    }
}

impl SubAssign<arch::AddrType> for PhysAddr {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: arch::AddrType) {
        self.0 -= rhs;
    }
}

impl fmt::Pointer for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.value() as *const usize), f)
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
        assert!(*self <= end_addr(), "Out of kernel space: {:p} > {:p}", self, end_addr());
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

    #[inline(always)]
    fn add(self, rhs: usize) -> VirtAddr {
        VirtAddr(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtAddr {
    #[inline(always)]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for VirtAddr {
    type Output = VirtAddr;

    #[inline(always)]
    fn sub(self, rhs: usize) -> VirtAddr {
        VirtAddr(self.0 - rhs)
    }
}

impl Sub<VirtAddr> for VirtAddr {
    type Output = usize;

    #[inline(always)]
    fn sub(self, rhs: VirtAddr) -> usize {
        self.0 - rhs.0
    }
}

impl SubAssign<usize> for VirtAddr {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl fmt::Pointer for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.value() as *const usize), f)
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.value() as *const usize), f)
    }
}

enum PreHeap {
    Uninit,
    Available(VirtAddr),
    Post(VirtAddr)
}

static mut memory: PreHeap = PreHeap::Uninit;

#[inline]
pub fn pre_init() {
    unsafe {
        memory = PreHeap::Available(arch::kernel_end());
    }
}

pub fn end_addr() -> VirtAddr {
    unsafe {
        match memory {
            PreHeap::Uninit => VirtAddr::null(),
            PreHeap::Available(address) | PreHeap::Post(address) => address
        }
    }
}

#[inline(always)]
pub fn done() -> VirtAddr {
    unsafe {
        match memory {
            PreHeap::Uninit => panic!("Uninitialized"),
            PreHeap::Available(addr) => {
                let addr = addr.align_up(arch::FRAME_SIZE);
                memory = PreHeap::Post(addr);
                addr
            },
            PreHeap::Post(_) => panic!("Already done")
        }
    }
}

pub unsafe fn allocate_raw(size: usize, align: usize) -> *mut u8 {
    if let PreHeap::Available(addr) = memory {
        let addr = addr.align_up(align);
        memory = PreHeap::Available(addr + size);
        addr.as_mut_ptr()
    } else {
        panic!("You can use the standard memory allocations");
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

