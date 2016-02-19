use arch;
use memory::kernel::PhysAddr;
use core::intrinsics;
use core::ops::Sub;
use core::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Register<T> {
    addr: PhysAddr,
    _marker: PhantomData<*mut T>
}

impl<T> Register<T> {
    #[inline(always)]
    pub const fn new(addr: arch::AddrType) -> Register<T> {
        Register {
            addr: PhysAddr::from_raw(addr),
            _marker: PhantomData
        }
    }

    #[inline(always)]
    pub const fn addr(&self) -> PhysAddr {
        self.addr
    }

    #[inline(always)]
    pub const fn offset(&self, offset: arch::AddrType) -> Register<T> {
        Register::new(self.addr.value() + offset)
    }

    #[inline(always)]
    pub const fn as_ptr(&self) -> *const T {
        self.addr.value() as *const T
    }

    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.addr.value() as *mut T
    }

    #[inline(always)]
    pub fn load(&self) -> T {
        unsafe {
            intrinsics::volatile_load(self.as_ptr())
        }
    }

    #[inline(always)]
    pub fn store(&self, val: T) {
        unsafe {
            intrinsics::volatile_store(self.as_mut_ptr(), val);
        }
    }
}

impl<T> Sub<Register<T>> for Register<T> {
    type Output = arch::AddrType;

    #[inline(always)]
    fn sub(self, rhs: Register<T>) -> arch::AddrType {
        self.addr - rhs.addr
    }
}

