use super::super::buddy;
use rt::{self, IterHelper, Force, ForceRef};
use arch;
use lists::{LinkedNode, LinkedList};
use memory::kernel::VirtAddr;
use core::cmp;
use core::fmt;
use core::mem;
use core::slice;
use core::usize;
use core::ptr::{self, Unique, Shared};
use core::marker::PhantomData;

const MAX_ORDER: usize = 13;
const MAX_OBJECT_SIZE: usize = arch::FRAME_SIZE;

type Bufctl = usize;
const BUFCTL_ALLOCATED: Bufctl = usize::MAX - 1;
const BUFCTL_END: Bufctl = usize::MAX;

macro_rules! gen {
    ($($size:expr),*) => {
        [$(($size, concat!("Generic-", $size))),*]
    };
}

const GENERIC_ALLOCATORS: [(usize, &'static str); 17] = gen!(
    8, 16, 32, 64, 96, 128, 192, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072
);

// listにはKCacheAllocatorInnerを型パラメータを無視して格納
pub struct KCacheManager {
    list: LinkedList<KCacheAllocatorInner<()>>,
    allocator: KCacheAllocatorAllocator,
    generic_allocators: [KCacheAllocatorInner<u8>; 17]
}

unsafe impl Send for KCacheManager { }
unsafe impl Sync for KCacheManager { }

impl KCacheManager {
    #[inline(always)]
    fn init(&mut self) {
        self.list = LinkedList::new();
        self.allocator = KCacheAllocatorAllocator::new();

        let inner = &mut self.allocator.0 as *mut _;
        self.add(inner);

        for (&(size, name), allocator) in GENERIC_ALLOCATORS.iter().zip(self.generic_allocators.iter_mut()) {
            *allocator = KCacheAllocatorInner::new(name, 1, None, size);
        }
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
        unsafe {
            self.generic_allocators.iter_mut()
                .find_map(|allocator| if allocator.object_size < size {
                    None
                } else {
                    let ptr = allocator.allocate_uninit();
                    if ptr.is_null() {
                        None
                    } else {
                        let ret = rt::align_up_mut_ptr(ptr, align);
                        // アライメントを揃えた結果が確保した領域を超えたらリトライ
                        if ret.offset(size as isize) > ptr.offset(allocator.object_size as isize) {
                            allocator.free(ptr);
                            None
                        } else {
                            Some(ret)
                        }
                    }
                })
                .unwrap_or(ptr::null_mut())
        }
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
}

struct KCacheAllocatorInner<T> {
    name: &'static str,
    align: usize,
    // color
    ctor: Option<fn(&mut T) -> ()>,
    data_size: usize,
    object_size: usize,
    objects_per_slab: usize,
    total_objects: usize,

    partial_list: LinkedList<Slab<T>>,
    full_list: LinkedList<Slab<T>>,
    free_list: LinkedList<Slab<T>>,

    prev: Option<Shared<KCacheAllocatorInner<()>>>,
    next: Option<Shared<KCacheAllocatorInner<()>>>
}

impl<T> LinkedNode for KCacheAllocatorInner<T> {
    linked_node!(Shared<KCacheAllocatorInner<()>> { prev: prev, next: next });
}

impl<T> KCacheAllocatorInner<T> {
    #[inline(always)]
    const fn check_is_on_slab(object_size: usize) -> bool {
        object_size < MAX_OBJECT_SIZE
    }

    #[inline(always)]
    fn is_on_slab(&self) -> bool {
        Self::check_is_on_slab(self.object_size)
    }

    fn new(name: &'static str, align: usize, ctor: Option<fn(&mut T)>, object_size: usize) -> KCacheAllocatorInner<T> {
        let (data_size, objects_per_slab) = if Self::check_is_on_slab(object_size) {
            let data_size = arch::FRAME_SIZE;
            let capacity = rt::align_up(data_size - mem::size_of::<Slab<T>>() + 1, align) - align;
            let objects_per_slab = capacity / (mem::size_of::<Bufctl>() + object_size);
            (data_size, objects_per_slab)
        } else {
            // 最低でもarch::FRAME_SIZE、あるいはその2^n倍
            let data_size = cmp::max(arch::FRAME_SIZE, (rt::align_up(object_size, align)).next_power_of_two());
            assert!(data_size <= arch::FRAME_SIZE << MAX_ORDER);
            let objects_per_slab = data_size / object_size;
            (data_size, objects_per_slab)
        };

        KCacheAllocatorInner {
            name: name,
            align: align,
            ctor: ctor,
            data_size: data_size,
            object_size: object_size,
            objects_per_slab: objects_per_slab,
            total_objects: 0,

            partial_list: LinkedList::new(),
            full_list: LinkedList::new(),
            free_list: LinkedList::new(),

            prev: None,
            next: None
        }
    }

    unsafe fn allocate_uninit(&mut self) -> *mut T {
        match self.partial_list.front() {
            Some(slab) => {
                let ptr = (**slab).allocate(self);
                if !ptr.is_null() && (**slab).is_empty() {
                    self.partial_list.pop_front();
                    self.full_list.push_back(slab);
                }
                ptr
            },
            None => {
                if self.free_list.is_empty() {
                    self.grow();
                }
                match self.free_list.front() {
                    Some(slab) => {
                        let ptr = (**slab).allocate(self);
                        if !ptr.is_null() {
                            self.free_list.pop_front();
                            if (**slab).is_empty() {
                                self.full_list.push_back(slab);
                            } else {
                                self.partial_list.push_back(slab);
                            }
                        }
                        ptr
                    },
                    None => ptr::null_mut()
                }
            }
        }
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

    fn free(&mut self, ptr: *mut T) {
        unsafe {
            match self.partial_list.iter().find(|&slab| (**slab).matches(self, ptr)) {
                Some(slab) => {
                    if (**slab).free(self, ptr) == self.objects_per_slab {
                        self.partial_list.remove(&slab);
                        self.free_list.push_front(slab);
                    }
                },
                None => {
                    let slab = self.full_list.iter().find(|&slab| (**slab).matches(self, ptr))
                        .expect("Unidentified Freeing Object");
                    (**slab).free(self, ptr);
                    self.full_list.remove(&slab);
                    self.partial_list.push_front(slab);
                }
            }
        }
    }

    /*fn matches(&self, ptr: *mut T) -> bool {
        self.partial_list.iter_mut().chain(self.full_list.iter_mut())
            .any(|slab| slab.matches(self, ptr))
    }*/

    fn grow(&mut self) {
        let data_order = buddy::order_by_size(self.data_size);
        let slab_size = mem::size_of::<Slab<T>>() + mem::size_of::<Bufctl>() * self.objects_per_slab;

        if !self.is_on_slab() {
            let mut manager = manager();
            let align = mem::align_of::<Slab<T>>();
            let slab = manager.allocate(slab_size, align) as *mut Slab<T>;
            if slab.is_null() {
                None
            } else {
                match buddy::manager().allocate(data_order) {
                    Some(page) => {
                        let data_addr = arch::page::table().map_memory(3, 3, page, self.data_size);
                        if data_addr.is_null() {
                            manager.free(slab as *mut u8, align);
                            buddy::manager().free(page);
                            None
                        } else {
                            Some((slab, data_addr))
                        }
                    },
                    None => {
                        manager.free(slab as *mut u8, align);
                        None
                    }
                }
            }
        } else {
            buddy::manager().allocate(data_order).and_then(|page| {
                let slab_addr = arch::page::table().map_memory(3, 3, page, self.data_size);
                if slab_addr.is_null() {
                    buddy::manager().free(page);
                    None
                } else {
                    let data_addr = slab_addr + slab_size;
                    let aligned_addr = rt::align_up(data_addr.value(), self.align);
                    Some((slab_addr.as_mut_ptr(), VirtAddr::from_raw(aligned_addr)))
                }
            })
        }.map_or_else(|| panic!("Unable to allocate a kernel object"), |(slab, data_addr)| {
            unsafe {
                (*slab).init(self, data_addr.as_mut_ptr());
                self.free_list.push_back(Shared::new(slab));
                self.total_objects += self.objects_per_slab;

                if let Some(ctor) = self.ctor {
                    let objects = slice::from_raw_parts_mut(data_addr.as_mut_ptr(), self.objects_per_slab);
                    for obj in objects {
                        ctor(obj);
                    }
                }
            }
        });
    }
}

impl<T> fmt::Debug for KCacheAllocatorInner<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("KCacheAllocator")
            .field("name", &self.name)
            .field("align", &self.align)
            .field("data_size", &self.data_size)
            .field("object_size", &self.object_size)
            .field("objects_per_slab", &self.objects_per_slab)
            .field("total_objects", &self.total_objects)
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
        KCacheAllocator(self.0)
    }
}

#[repr(packed)]
struct Slab<T> {
    ptr: *mut u8,
    index: Bufctl,
    // color

    prev: Option<Shared<Slab<T>>>,
    next: Option<Shared<Slab<T>>>,

    _marker: PhantomData<T>
}

impl<T> LinkedNode for Slab<T> {
    linked_node!(Shared<Slab<T>> { prev: prev, next: next });
}

impl<T> Slab<T> {
    fn init(&mut self, allocator: &KCacheAllocatorInner<T>, ptr: *mut u8) {
        *self = Slab {
            ptr: ptr,
            index: 0,

            prev: None,
            next: None,

            _marker: PhantomData
        };

        let bufctl = unsafe { self.bufctl(allocator.objects_per_slab) };
        let len = bufctl.len();
        for (i, b) in bufctl[..len - 1].iter_mut().enumerate() {
            *b = i + 1;
        }
        bufctl[len - 1] = BUFCTL_END;
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.index == BUFCTL_END
    }

    fn allocate(&mut self, allocator: &KCacheAllocatorInner<T>) -> *mut T {
        if self.index == BUFCTL_END {
            ptr::null_mut()
        } else {
            unsafe {
                let bufctl = self.bufctl(allocator.objects_per_slab);
                let index = self.index;
                self.index = mem::replace(&mut bufctl[index], BUFCTL_ALLOCATED);
                let aligned_size = rt::align_up(allocator.object_size, allocator.align);
                self.ptr.offset((aligned_size * index) as isize) as *mut T
            }
        }
    }

    fn matches(&self, allocator: &KCacheAllocatorInner<T>, ptr: *mut T) -> bool {
        unsafe {
            let ptr_u8 = ptr as *mut u8;
            let last = self.ptr.offset((allocator.object_size * allocator.objects_per_slab) as isize);
            ptr_u8 >= self.ptr && ptr_u8 < last
        }
    }

    fn free(&mut self, allocator: &KCacheAllocatorInner<T>, ptr: *mut T) -> usize {
        assert!(self.matches(allocator, ptr));

        let bufctl = unsafe { self.bufctl(allocator.objects_per_slab) };
        let index = (ptr as usize - self.ptr as usize) / allocator.object_size;
        assert_eq!(bufctl[index], BUFCTL_ALLOCATED);

        bufctl[index] = BUFCTL_END;
        if self.index != BUFCTL_END {
            let mut prev_index = self.index;
            let mut count = 1;
            while bufctl[prev_index] != BUFCTL_END {
                prev_index = bufctl[prev_index];
                count += 1;
            }
            bufctl[prev_index] = index;
            count
        } else {
            self.index = index;
            1
        }
    }

    #[inline(always)]
    unsafe fn bufctl(&mut self, len: usize) -> &'static mut [Bufctl] {
        let bufctl = (self as *mut Self).offset(1) as *mut Bufctl;
        slice::from_raw_parts_mut(bufctl, len)
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

