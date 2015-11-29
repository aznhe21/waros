#![allow(dead_code)]

use prelude::*;
use rt;
use arch;
use core::mem;
use core::slice;
use core::str;
use core::fmt;

const MULTIBOOT_HEADER_MAGIC:     u32 = 0x1BADB002;
const MULTIBOOT_HEADER_FLAGS:     u32 = 0x00010003;
const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;

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
    static mut mboot_ptr: *mut MultibootInfo;
}

#[inline(always)]
pub fn magic_valid() -> bool {
    mboot_sig == MULTIBOOT_BOOTLOADER_MAGIC
}

#[inline(always)]
pub fn info() -> &'static MultibootInfo {
    unsafe { &*mboot_ptr }
}

#[inline]
pub fn init() {
    assert!(magic_valid(), "Invalid multiboot magic");

    let info = unsafe {
        mboot_ptr = (mboot_ptr as *mut u8).uoffset(arch::KERNEL_BASE) as *mut MultibootInfo;
        &mut *mboot_ptr
    };
    info.cmdline += arch::KERNEL_BASE as u32;
    info.mods_addr += arch::KERNEL_BASE as u32;
    info.mmap_addr += arch::KERNEL_BASE as u32;
    info.drives_addr += arch::KERNEL_BASE as u32;
    info.config_table += arch::KERNEL_BASE as u32;
    info.boot_loader_name += arch::KERNEL_BASE as u32;
    info.apm_table += arch::KERNEL_BASE as u32;
    info.vbe_controller_info += arch::KERNEL_BASE as u32;
    info.vbe_mode_info += arch::KERNEL_BASE as u32;
}

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
    #[inline(always)]
    pub fn has(&self, bit: u32) -> bool {
        self.flags & 1_u32.wrapping_shl(bit) != 0
    }

    #[inline(always)]
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
            let ptr = self.cmdline as *const u8;
            unsafe {
                str::from_utf8_unchecked(slice::from_raw_parts(ptr, rt::strlen(ptr)))
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
    #[inline(always)]
    pub fn mmap(&self) -> Option<&'static [MemoryMap]> {
        if self.has(6) {
            Some(unsafe {
                let ptr = self.mmap_addr as *mut MemoryMap;
                let size = self.mmap_length as usize / mem::size_of::<MemoryMap>();
                slice::from_raw_parts(ptr, size)
            })
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
            Some(unsafe { &*(self.vbe_controller_info as *mut VbeControllerInfo) })
        } else {
            None
        }
    }

    #[inline]
    pub fn vbe_mode_info(&self) -> Option<&VbeModeInfo> {
        if self.has(11) {
            Some(unsafe { &*(self.vbe_mode_info as *mut VbeModeInfo) })
        } else {
            None
        }
    }
}

#[repr(C, packed)]
pub struct AoutSymbolTable {
    pub tabsize: u32,
    pub strsize: u32,
    pub addr: u32,
    pub reserved: u32
}

#[repr(C, packed)]
pub struct ElfSectionHeaderTable {
    pub num: u32,
    pub size: u32,
    pub addr: u32,
    pub shndx: u32
}

#[repr(u32)]
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

#[repr(C, packed)]
pub struct MemoryMap {
    // この構造体のサイズ(20)
    pub size: u32,
    // 開始アドレス
    pub base_addr: u64,
    // 領域の大きさ
    pub length: u64,
    // メモリの種類
    pub type_: MemoryType
}

#[repr(C, packed)]
pub struct ModList {
    pub start: u32,
    pub end: u32,
    pub cmdline: u32,
    pub pad: u32
}

#[repr(C, packed)]
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

    #[inline(always)]
    pub fn valid(&self) -> bool {
        self.signature == VbeControllerInfo::MAGIC
    }

    pub fn str_vendor_string(&self) -> &'static str {
        let ptr = self.vendor_string as *const u8;
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(ptr, rt::strlen(ptr)))
        }
    }

    pub fn str_vendor_name(&self) -> &'static str {
        let ptr = self.vendor_name as *const u8;
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(ptr, rt::strlen(ptr)))
        }
    }

    pub fn str_product_name(&self) -> &'static str {
        let ptr = self.product_name as *const u8;
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(ptr, rt::strlen(ptr)))
        }
    }

    pub fn str_product_rev(&self) -> &'static str {
        let ptr = self.product_rev as *const u8;
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(ptr, rt::strlen(ptr)))
        }
    }
}

#[repr(C, packed)]
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
    #[inline(always)]
    pub fn vram(&self) -> *mut u8 {
        self.phys_base_ptr as *mut u8
    }
}

