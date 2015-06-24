#![allow(dead_code)]

use prelude::*;
use rt;
use core::mem;
use core::slice;
use core::fmt;

const MULTIBOOT_HEADER_MAGIC:     u32 = 0x1BADB002;
const MULTIBOOT_HEADER_FLAGS:     u32 = 0x00010003;
const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;

const STACK_SIZE: u16 = 0x4000;

const MB_INFO_FLAG_MEM_SIZE:         u32 = 0x00000001;
const MB_INFO_FLAG_BOOT_DEVICE:      u32 = 0x00000002;
const MB_INFO_FLAG_COMMAND_LINE:     u32 = 0x00000004;
const MB_INFO_FLAG_MODULES:          u32 = 0x00000008;
const MB_INFO_FLAG_AOUT_SYMS:        u32 = 0x00000010;
const MB_INFO_FLAG_ELF_SYMS:         u32 = 0x00000020;
const MB_INFO_FLAG_MEMORY_MAP:       u32 = 0x00000040;
const MB_INFO_FLAG_DRIVES:           u32 = 0x00000080;
const MB_INFO_FLAG_CONFIG_TABLE:     u32 = 0x00000100;
const MB_INFO_FLAG_BOOT_LOADER_NAME: u32 = 0x00000200;
const MB_INFO_FLAG_APM_TABLE:        u32 = 0x00000400;
const MB_INFO_FLAG_GRAPHICS_TABLE:   u32 = 0x00000800;

extern {
    static mboot_sig: u32;
    static mboot_ptr: *mut MultibootInfo;
}

pub fn magic_valid() -> bool {
    mboot_sig == MULTIBOOT_BOOTLOADER_MAGIC
}

pub fn info() -> &'static MultibootInfo {
    unsafe { &*::arch::indirect_pointer(mboot_ptr) }
}

/*#[cfg(target_arch="x86")]
pub fn bootloader_memory_size() -> usize {
    unsafe {
        log!("{:x}", mboot_ptr as usize);
        let p: *mut multiboot_info = mem::transmute(mboot_ptr as usize + 0xC0000000);
        log!("{:x}", (*mboot_ptr).mem_lower);
        log!("{:x}", (*mboot_ptr).mem_upper);
        0
        //(*mboot_ptr).mem_lower as usize
    }
}

#[cfg(target_arch="x86_64")]
pub fn bootloader_memory_size() -> usize {
    unsafe {
        log!("{}", (*mboot_ptr).mem_lower);
        (*mboot_ptr).mem_lower as usize | ((*mboot_ptr).mem_upper as usize) << 32
    }
}*/

#[repr(C, packed)]
pub struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_count: u32,
    mods_addr: u32,
    _aout: AoutSymbolTable,
    mmap_length: u32,
    mmap_addr: u32,
    drives_length: u32,
    drives_addr: u32,
    config_table: u32,
    boot_loader_name: u32,
    apm_table: u32,
    vbe_controller_info: u32,
    vbe_mode_info: u32,
    pub vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16
}

impl MultibootInfo {
    #[inline]
    pub fn has(&self, bit: u8) -> bool {
        self.flags & (1 << bit) != 0
    }

    #[inline]
    pub fn mem_size(&self) -> Option<u32> {
        if self.has(0) {
            Some((self.mem_lower + self.mem_upper + 1024) * 1024)
        } else {
            None
        }
    }

    // 1: boot_device

    // 2: cmdline
    pub fn str_cmdline(&self) -> &'static str {
        if self.has(2) {
            unsafe {
                let ptr = ::arch::indirect_pointer(self.cmdline as *const u8);
                mem::transmute(slice::from_raw_parts(ptr, rt::strlen(ptr)))
            }
        } else {
            ""
        }
    }

    // 3: mods

    // 4: aout symbol table
    #[inline]
    pub fn aout(&self) -> Option<&AoutSymbolTable> {
        if self.has(4) {
            Some(&self._aout)
        } else {
            None
        }
    }

    // 5: elf section header table
    #[inline]
    pub fn elf(&self) -> Option<&ElfSectionHeaderTable> {
        if self.has(5) {
            Some(unsafe { mem::transmute(&self._aout) })
        } else {
            None
        }
    }

    // 6: mmap
    pub fn mmap(&self) -> Option<&'static [MemoryMap]> {
        if self.has(6) {
            let ptr = ::arch::indirect_pointer(self.mmap_addr as *mut MemoryMap);
            let size = self.mmap_length as usize / mem::size_of::<MemoryMap>();
            Some(unsafe { slice::from_raw_parts(ptr, size) })
        } else {
            None
        }
    }

    // 7: drives
    // 8: config_table
    // 9: boot_loader_name
    // 10: apm_table

    // 11: vbe
    #[inline]
    pub fn vbe_controller_info(&self) -> Option<&VbeControllerInfo> {
        if self.has(11) {
            Some(unsafe { &*::arch::indirect_pointer(self.vbe_controller_info as *mut VbeControllerInfo) })
        } else {
            None
        }
    }

    #[inline]
    pub fn vbe_mode_info(&self) -> Option<&VbeModeInfo> {
        if self.has(11) {
            Some(unsafe { &*::arch::indirect_pointer(self.vbe_mode_info as *mut VbeModeInfo) })
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct AoutSymbolTable {
    pub tabsize: u32,
    pub strsize: u32,
    pub addr: u32,
    pub reserved: u32
}

#[repr(C)]
pub struct ElfSectionHeaderTable {
    pub num: u32,
    pub size: u32,
    pub addr: u32,
    pub shndx: u32
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum MemoryType {
    Usable = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    AcpiNvs = 4,
    Bad = 5
}

impl MemoryType {
    pub fn description(&self) -> &str {
        match *self {
            MemoryType::Usable => "Usable RAM",
            MemoryType::Reserved => "Reserved",
            MemoryType::AcpiReclaimable => "ACPI reclaimable memory",
            MemoryType::AcpiNvs => "ACPI NVS memory",
            MemoryType::Bad => "Area containing bad memory"
        }
    }
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

#[repr(C)]
pub struct MemoryMap {
    pub size: u32,
    pub base_addr: u64,
    pub length: u64,
    pub type_: MemoryType
}

#[repr(C)]
pub struct ModList {
    pub start: u32,
    pub end: u32,
    pub cmdline: u32,
    pub pad: u32
}

#[repr(C)]
pub struct VbeControllerInfo {
    pub signature: u32,
    pub version: u16,
    pub vendor_string: u32,
    pub capabilities: u32,
    pub video_mode_ptr: u32,
    pub total_memory: u16,
    pub software_rev: u16,
    pub vendor_name: u32,
    pub product_name: u32,
    pub product_rev: u32,

    pub reserved: [u8; 222]
}

impl VbeControllerInfo {
    const MAGIC: u32 = 0x41534556; // 'VESA'

    #[inline]
    pub fn valid(&self) -> bool {
        self.signature == VbeControllerInfo::MAGIC
    }

    pub fn str_vendor_string(&self) -> &'static str {
        unsafe {
            let ptr = ::arch::indirect_pointer(self.vendor_string as *const u8);
            mem::transmute(slice::from_raw_parts(ptr, rt::strlen(ptr)))
        }
    }

    pub fn str_vendor_name(&self) -> &'static str {
        unsafe {
            let ptr = ::arch::indirect_pointer(self.vendor_name as *const u8);
            mem::transmute(slice::from_raw_parts(ptr, rt::strlen(ptr)))
        }
    }

    pub fn str_product_name(&self) -> &'static str {
        unsafe {
            let ptr = ::arch::indirect_pointer(self.product_name as *const u8);
            mem::transmute(slice::from_raw_parts(ptr, rt::strlen(ptr)))
        }
    }

    pub fn str_product_rev(&self) -> &'static str {
        unsafe {
            let ptr = ::arch::indirect_pointer(self.product_rev as *const u8);
            mem::transmute(slice::from_raw_parts(ptr, rt::strlen(ptr)))
        }
    }
}

#[repr(C)]
pub struct VbeModeInfo {
    pub mode_attr: u16,
    pub win_attr: [u8; 2],
    pub win_grain: u16,
    pub win_size: u16,
    pub win_seg: [u16; 2],
    pub win_scheme: u32,
    pub logical_scan: u16,

    pub h_res: u16,
    pub v_res: u16,
    pub char_width: u8,
    pub char_height: u8,
    pub memory_planes: u8,
    pub bpp: u8,
    pub banks: u8,
    pub memory_layout: u8,
    pub bank_size: u8,
    pub image_planes: u8,
    pub page_function: u8,

    pub rmask: u8,
    pub rpos: u8,
    pub gmask: u8,
    pub gpos: u8,
    pub bmask: u8,
    pub bpos: u8,
    pub resv_mask: u8,
    pub resv_pos: u8,
    pub dcm_info: u8,

    pub phys_base_ptr: u32,
    pub offscreen_ptr: u32,
    pub offscreen_size: u16,

    pub reserved: [u8; 206]
}

impl VbeModeInfo {
    #[inline]
    pub fn vram(&self) -> *mut u8 {
        ::arch::indirect_pointer_mut(self.phys_base_ptr as *mut u8)
    }
}

