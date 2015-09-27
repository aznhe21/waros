#![allow(dead_code)]

use prelude::*;
use rt;
use arch;
use lists::{LinkedList, LinkedNode};
use memory::kernel::VirtAddr;
use core::cmp;
use core::fmt;
use core::mem;
use core::ptr;
use core::slice;
use core::usize;

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

// listにはSlabAllocatorを型パラメータを無視して格納
pub struct SlabManager {
    list: LinkedList<SlabAllocator<()>>,
    allocator: SlabAllocator<SlabAllocator<()>>,
    generic_allocators: [SlabAllocator<u8>; 17]
}

impl SlabManager {
    #[inline]
    fn init(&'static mut self) {
        self.list = LinkedList::new();
        let size = mem::size_of::<SlabAllocator<SlabAllocator<()>>>();
        let align = mem::align_of::<SlabAllocator<SlabAllocator<()>>>();
        if !self.allocator.init("Slab", align, None, size) {
            panic!("Failed to initialize a slab allocator");
        }

        let allocator_ptr = &mut self.allocator as *mut _;
        self.add(allocator_ptr);

        for (&(size, name), allocator) in GENERIC_ALLOCATORS.iter().zip(self.generic_allocators.iter_mut()) {
            if !allocator.init(name, 1, None, size) {
                panic!("Failed to initialize a general slab allocator");
            }
        }
    }

    #[inline]
    fn add<T>(&mut self, allocator: *mut SlabAllocator<T>) {
        self.list.push_back(allocator as *mut SlabAllocator<()>);
    }

    pub fn usable_size(&mut self, size: usize, align: usize) -> usize {
        panic!("Unimplemented method: SlabManager::usable_size({:?}, {:?})", size, align)
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        let alloc_size = rt::align_up(size, align);
        self.generic_allocators.iter_mut()
            .filter_map(|allocator| if allocator.object_size >= alloc_size {
                let ptr = allocator.allocate();
                if !ptr.is_null() {
                    unsafe {
                        Some(ptr.uoffset(alloc_size - size))
                    }
                } else {
                    None
                }
            } else {
                None
            })
            .next()
            .unwrap_or(ptr::null_mut())
    }

    pub fn reallocate_inplace(&mut self, ptr: *mut u8, size: usize, align: usize) -> usize {
        panic!("Unimplemented method: SlabManager::reallocate_inplace({:?}, {:?}, {:?})", ptr, size, align)
    }

    pub fn reallocate(&mut self, ptr: *mut u8, size: usize, align: usize) -> *mut u8 {
        panic!("Unimplemented method: SlabManager::reallocate({:?}, {:?}, {:?})", ptr, size, align)
    }

    pub fn free(&mut self, ptr: *mut u8, align: usize) {
        panic!("Unimplemented method: SlabManager::free({:?}, {:?})", ptr, align)
    }
}

pub struct SlabAllocator<T: Sized> {
    name: &'static str,
    align: usize,
    // color
    ctor: Option<fn(&mut T) -> ()>,
    slab_size: usize,
    object_size: usize,
    objects_per_slab: usize,
    total_objects: usize,

    partial_list: LinkedList<Slab<T>>,
    full_list: LinkedList<Slab<T>>,
    free_list: LinkedList<Slab<T>>,

    prev: *mut SlabAllocator<()>,
    next: *mut SlabAllocator<()>
}

impl<T: Sized> LinkedNode<SlabAllocator<()>> for SlabAllocator<T> {
    linked_node!(SlabAllocator<()> { prev: prev, next: next });
}

impl<T: Sized> SlabAllocator<T> {
    #[inline(always)]
    const fn check_is_on_slab(object_size: usize) -> bool {
        object_size < MAX_OBJECT_SIZE
    }

    #[inline(always)]
    fn is_on_slab(&self) -> bool {
        Self::check_is_on_slab(self.object_size)
    }

    #[inline(always)]
    fn is_off_slab(&self) -> bool {
        !self.is_on_slab()
    }

    fn init(&mut self, name: &'static str, align: usize, ctor: Option<fn(&mut T) -> ()>, object_size: usize) -> bool {
        let (slab_size, objects_per_slab) = if Self::check_is_on_slab(object_size) {
            let bufctl_size = mem::size_of::<Bufctl>();
            let slab_size = arch::FRAME_SIZE;
            let capacity = rt::align_up(slab_size - mem::size_of::<Slab<T>>() + 1, align) - align;
            let objects_per_slab = capacity / (bufctl_size + object_size);
            (slab_size, objects_per_slab)
        } else {
            // 最低でもarch::FRAME_SIZE、あるいはその倍数
            let slab_size = cmp::max(arch::FRAME_SIZE, (rt::align_up(object_size, align)).next_power_of_two());
            if slab_size > arch::FRAME_SIZE << MAX_ORDER {
                return false;
            }
            let objects_per_slab = slab_size / object_size;
            (slab_size, objects_per_slab)
        };

        *self = SlabAllocator {
            name: name,
            align: align,
            ctor: ctor,
            slab_size: slab_size,
            object_size: object_size,
            objects_per_slab: objects_per_slab,
            total_objects: 0,

            partial_list: LinkedList::new(),
            full_list: LinkedList::new(),
            free_list: LinkedList::new(),

            prev: ptr::null_mut(),
            next: ptr::null_mut()
        };
        true
    }

    pub fn new(name: &'static str, align: usize, ctor: Option<fn(&mut T) -> ()>) -> Option<&'static mut Self> {
        let manager = manager();
        unsafe { manager.allocator.allocate().as_mut() }.and_then(move |allocator_obj| {
            let allocator: &'static mut Self = unsafe { mem::transmute(allocator_obj) };
            if allocator.init(name, align, ctor, mem::size_of::<T>()) {
                manager.add(allocator);
                Some(allocator)
            } else {
                manager.allocator.free(unsafe { mem::transmute(allocator) });
                None
            }
        })
    }

    #[inline(always)]
    pub fn name(&self) -> &str {
        self.name
    }

    #[inline]
    pub fn allocate(&mut self) -> *mut T {
        match self.partial_list.front_mut() {
            Some(slab) => {
                let ptr = slab.allocate(self);
                if !ptr.is_null() {
                    self.partial_list.pop_front();
                    self.full_list.push_back(slab);
                }
                ptr
            },
            None => {
                if self.free_list.is_empty() {
                    self.grow();
                }
                match self.free_list.front_mut() {
                    Some(slab) => {
                        let ptr = slab.allocate(self);
                        if !ptr.is_null() {
                            self.free_list.pop_front();
                            self.partial_list.push_back(slab);
                        }
                        ptr
                    },
                    None => ptr::null_mut()
                }
            }
        }
    }

    pub fn free(&'static mut self, ptr: *mut T) {
        match self.partial_list.iter_mut().find(|slab| slab.matches(self, ptr)) {
            Some(slab) => {
                if slab.free(self, ptr) == self.objects_per_slab {
                    self.partial_list.remove(slab);
                    self.free_list.push_front(slab);
                }
            },
            None => {
                let slab = self.full_list.iter_mut().find(|slab| slab.matches(self, ptr))
                    .expect("Unidentified Freeing Object");
                slab.free(self, ptr);
                self.full_list.remove(slab);
                self.partial_list.push_front(slab);
            }
        }
    }

    fn matches(&self, ptr: *mut T) -> bool {
        self.partial_list.iter_mut().chain(self.full_list.iter_mut())
            .any(|slab| slab.matches(self, ptr))
    }

    fn grow(&mut self) {
        let slab_order = usize::BITS - ((self.slab_size / arch::FRAME_SIZE).leading_zeros() - 1) as usize;

        if self.is_off_slab() {
            let manager = manager();
            let size = mem::size_of::<Slab<T>>() + mem::size_of::<Bufctl>() * self.objects_per_slab;
            let align = mem::align_of::<Slab<T>>();
            let ptr = manager.allocate(size, align);
            unsafe { (ptr as *mut Slab<T>).as_mut() }.and_then(|slab| {
                match super::buddy::manager().allocate(slab_order) {
                    Some(page) => {
                        let data_addr = arch::page::table().map_memory(3, 3, page.addr(), self.slab_size);
                        if data_addr.is_null() {
                            super::buddy::manager().free(page);
                            None
                        } else {
                            Some((slab, data_addr))
                        }
                    },
                    None => {
                        manager.free(ptr, align);
                        None
                    }
                }
            })
        } else {
            super::buddy::manager().allocate(slab_order).and_then(|page| {
                let bufctl_size = mem::size_of::<Bufctl>();
                let slab_addr = arch::page::table().map_memory(3, 3, page.addr(), self.slab_size);
                if slab_addr.is_null() {
                    super::buddy::manager().free(page);
                    None
                } else {
                    let data_addr = slab_addr + mem::size_of::<Slab<T>>() + bufctl_size * self.objects_per_slab;
                    let aligned_addr = rt::align_up(data_addr.value(), self.align);
                    Some((unsafe { &mut *slab_addr.as_mut_ptr() }, VirtAddr::from_raw(aligned_addr)))
                }
            })
        }.map(|(slab, data_addr)| {
            slab.init(self, data_addr.as_mut_ptr());
            self.free_list.push_back(slab);
            self.total_objects += self.objects_per_slab;

            if let Some(ctor) = self.ctor {
                let objects = unsafe { slice::from_raw_parts_mut(data_addr.as_mut_ptr(), self.objects_per_slab) };
                for obj in objects {
                    ctor(obj);
                }
            }
        });
    }
}

impl<T: Sized> fmt::Debug for SlabAllocator<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SlabAllocator")
            .field("name", &self.name)
            .field("align", &self.align)
            .field("slab_size", &self.slab_size)
            .field("object_size", &self.object_size)
            .field("objects_per_slab", &self.objects_per_slab)
            .field("total_objects", &self.total_objects)
            .finish()
    }
}

struct Slab<T> {
    ptr: *mut u8,
    index: Bufctl,
    // color

    prev: *mut Slab<T>,
    next: *mut Slab<T>
}

impl<T> LinkedNode<Slab<T>> for Slab<T> {
    linked_node!(Slab<T> { prev: prev, next: next });
}

impl<T> Slab<T> {
    pub fn init(&mut self, allocator: &SlabAllocator<T>, ptr: *mut u8) {
        *self = Slab {
            ptr: ptr,
            index: 0,

            prev: ptr::null_mut(),
            next: ptr::null_mut()
        };

        let bufctl = self.bufctl(allocator.objects_per_slab);
        let len = bufctl.len();
        for (i, b) in bufctl[..len - 1].iter_mut().enumerate() {
            *b = i + 1;
        }
        bufctl[len - 1] = BUFCTL_END;
    }

    pub fn allocate(&mut self, allocator: &SlabAllocator<T>) -> *mut T {
        if self.index == BUFCTL_END {
            ptr::null_mut()
        } else {
            let bufctl = self.bufctl(allocator.objects_per_slab);
            let index = self.index;
            self.index = mem::replace(&mut bufctl[index], BUFCTL_ALLOCATED);
            let offset = rt::align_up(allocator.object_size, allocator.align);
            unsafe { self.ptr.uoffset(offset * index) as *mut T }
        }
    }

    pub fn matches(&self, allocator: &SlabAllocator<T>, ptr: *mut T) -> bool {
        let ptr_u8 = ptr as *mut u8;
        let last = unsafe { self.ptr.uoffset(allocator.object_size * allocator.objects_per_slab) };
        ptr_u8 >= self.ptr && ptr_u8 < last
    }

    pub fn free(&mut self, allocator: &SlabAllocator<T>, ptr: *mut T) -> usize {
        assert!(self.matches(allocator, ptr));

        let bufctl = self.bufctl(allocator.objects_per_slab);
        let index = ptr as usize - self.ptr as usize;
        assert!(bufctl[index] == BUFCTL_ALLOCATED);

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
    fn bufctl(&mut self, len: usize) -> &'static mut [Bufctl] {
        unsafe {
            let bufctl = (self as *mut Self).uoffset(1) as *mut Bufctl;
            slice::from_raw_parts_mut(bufctl, len)
        }
    }
}

static mut manager_opt: Option<SlabManager> = None;

#[inline]
pub fn init() {
    unsafe {
        manager_opt.into_some().init();
    }
}

#[inline]
pub fn manager() -> &'static mut SlabManager {
    unsafe {
        manager_opt.as_mut().be_some()
    }
}

