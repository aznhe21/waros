use super::super::buddy;
use rt::{self, Force, ForceRef};
use arch;
use lists::{LinkedNode, DList};
use core::fmt;
use core::mem;
use core::ptr::{self, Unique, Shared};
use core::sync::atomic::{Ordering, AtomicUsize};

/*macro_rules! gen {
    ($($size:expr),*) => {
        [$(($size, concat!("Generic-", $size))),*]
    };
}

const GENERIC_ALLOCATORS: [(usize, &'static str); 17] = gen!(
    8, 16, 32, 64, 96, 128, 192, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072
);*/

// listにはKCacheAllocatorInnerを型パラメータを無視して格納
pub struct KCacheManager {
    list: DList<KCacheAllocatorInner<()>>,
    allocator: KCacheAllocatorAllocator,
    //generic_allocators: [KCacheAllocatorInner<u8>; 17]
}

unsafe impl Send for KCacheManager { }
unsafe impl Sync for KCacheManager { }

impl KCacheManager {
    #[inline(always)]
    fn init(&mut self) {
        self.list = DList::new();

        self.allocator = KCacheAllocatorAllocator::new();
        let inner = &mut self.allocator.0 as *mut _;
        self.add(inner);

        /*for (&(size, name), allocator) in GENERIC_ALLOCATORS.iter().zip(self.generic_allocators.iter_mut()) {
            *allocator = KCacheAllocatorInner::new(name, 1, None, size);
        }*/
    }

    #[inline]
    fn add<T>(&mut self, allocator: *mut KCacheAllocatorInner<T>) {
        unsafe {
            self.list.push_back(Shared::new(allocator as *mut KCacheAllocatorInner<()>));
        }
    }

    pub fn usable_size(&mut self, size: usize, align: usize) -> usize {
        panic!("Unimplemented method: KCacheManager::usable_size({:?}, {:?})", size, align)
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        let alloc_size = rt::align_up(size + 1, align);
        buddy::manager().allocate(buddy::order_by_size(alloc_size)).and_then(|page| {
            let addr = arch::page::table().map_memory(3, 3, page, alloc_size);
            if addr.is_null() {
                buddy::manager().free(page);
                None
            } else {
                Some(rt::align_up_mut_ptr(addr.as_mut_ptr(), align))
            }
        })
        .unwrap_or(ptr::null_mut())
    }

    pub fn reallocate_inplace(&mut self, ptr: *mut u8, size: usize, align: usize) -> usize {
        panic!("Unimplemented method: KCacheManager::reallocate_inplace({:?}, {:?}, {:?})", ptr, size, align);
    }

    pub fn reallocate(&mut self, ptr: *mut u8, size: usize, align: usize) -> *mut u8 {
        panic!("Unimplemented method: KCacheManager::reallocate({:?}, {:?}, {:?})", ptr, size, align);
    }

    pub fn free(&mut self, ptr: *mut u8, align: usize) {
        log!("Unimplemented method: KCacheManager::free({:?}, {:?})", ptr, align);
        arch::print_backtrace();
    }
}

// KCacheAllocatorの領域を確保するAllocator
struct KCacheAllocatorAllocator(KCacheAllocatorInner<KCacheAllocatorInner<()>>);

impl KCacheAllocatorAllocator {
    #[inline]
    fn new() -> KCacheAllocatorAllocator {
        let size = mem::size_of::<KCacheAllocator<KCacheAllocator<()>>>();
        let align = mem::align_of::<KCacheAllocator<KCacheAllocator<()>>>();
        KCacheAllocatorAllocator(KCacheAllocatorInner::new("Slab", align, None, size))
    }

    fn allocate<T>(&mut self, val: KCacheAllocatorInner<T>) -> Option<KCacheAllocator<T>> {
        debug_assert_eq!(mem::size_of::<KCacheAllocatorInner<()>>(), mem::size_of::<KCacheAllocatorInner<T>>());
        unsafe {
            let inner = self.0.allocate_uninit() as *mut KCacheAllocatorInner<T>;
            if inner.is_null() {
                None
            } else {
                ptr::write(inner, val);
                Some(KCacheAllocator(Shared::new(inner)))
            }
        }
    }

    #[inline(always)]
    fn free<T>(&mut self, val: Shared<KCacheAllocatorInner<T>>) {
        self.0.free(*val as *mut KCacheAllocatorInner<()>);
    }
}

struct KCacheAllocatorInner<T> {
    rc: AtomicUsize,
    name: &'static str,
    align: usize,
    _ctor: Option<fn(&mut T) -> ()>,
    object_size: usize,

    prev: Option<Shared<KCacheAllocatorInner<()>>>,
    next: Option<Shared<KCacheAllocatorInner<()>>>
}

impl<T> LinkedNode for KCacheAllocatorInner<T> {
    linked_node!(Shared<KCacheAllocatorInner<()>> { prev: prev, next: next });
}

impl<T> KCacheAllocatorInner<T> {
    #[inline(always)]
    fn new(name: &'static str, align: usize, ctor: Option<fn(&mut T)>, object_size: usize) -> KCacheAllocatorInner<T> {
        KCacheAllocatorInner {
            rc: AtomicUsize::new(1),
            name: name,
            align: align,
            _ctor: ctor,
            object_size: object_size,

            prev: None,
            next: None
        }
    }

    #[inline]
    unsafe fn allocate_uninit(&mut self) -> *mut T {
        manager().allocate(self.object_size, self.align) as *mut T
    }

    fn allocate(&mut self, x: T) -> Option<Unique<T>> {
        unsafe {
            let p = self.allocate_uninit();
            if p.is_null() {
                None
            } else {
                ptr::write(p, x);
                Some(Unique::new(p))
            }
        }
    }

    #[inline]
    fn free(&mut self, ptr: *mut T) {
        manager().free(ptr as *mut u8, self.align);
    }
}

impl<T> fmt::Debug for KCacheAllocatorInner<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("KCacheAllocator")
            .field("name", &self.name)
            .field("align", &self.align)
            .field("object_size", &self.object_size)
            .finish()
    }
}

pub struct KCacheAllocator<T>(Shared<KCacheAllocatorInner<T>>);

impl<T> KCacheAllocator<T> {
    pub fn new(name: &'static str, align: usize, ctor: Option<fn(&mut T)>) -> Option<KCacheAllocator<T>> {
        let mut manager = manager();
        let size = mem::size_of::<T>();
        let inner = KCacheAllocatorInner::new(name, align, ctor, size);
        manager.allocator.allocate(inner).map(|allocator| {
            manager.add(*allocator.0);
            allocator
        })
    }

    #[inline(always)]
    fn mut_inner(&self) -> &mut KCacheAllocatorInner<T> {
        unsafe { &mut **self.0 }
    }

    #[inline(always)]
    pub fn name(&self) -> &str {
        self.mut_inner().name
    }

    #[inline(always)]
    pub unsafe fn allocate_uninit(&self) -> *mut T {
        self.mut_inner().allocate_uninit()
    }

    #[inline(always)]
    pub fn allocate(&self, x: T) -> Option<Unique<T>> {
        self.mut_inner().allocate(x)
    }

    #[inline(always)]
    pub fn free(&self, ptr: *mut T) {
        self.mut_inner().free(ptr)
    }
}

impl<T> Clone for KCacheAllocator<T> {
    #[inline(always)]
    fn clone(&self) -> KCacheAllocator<T> {
        unsafe {
            (**self.0).rc.fetch_add(1, Ordering::SeqCst);
        }
        KCacheAllocator(self.0)
    }
}

impl<T> Drop for KCacheAllocator<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if (**self.0).rc.fetch_sub(1, Ordering::SeqCst) == 1 {
                manager().allocator.free(self.0);
            }
        }
    }
}

static MANAGER: Force<KCacheManager> = Force::new();

#[inline]
pub fn init() {
    MANAGER.setup().init();
}

#[inline(always)]
pub fn manager() -> ForceRef<KCacheManager> {
    MANAGER.as_ref()
}

