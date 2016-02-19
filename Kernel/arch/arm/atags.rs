use rt;
use memory;
use memory::kernel::PhysAddr;
use core::mem;
use core::slice;
use core::str;

const ATAG_NONE:      u32 = 0x00000000;
const ATAG_CORE:      u32 = 0x54410001;
const ATAG_MEM:       u32 = 0x54410002;
const ATAG_VIDEOTEXT: u32 = 0x54410003;
const ATAG_RAMDISK:   u32 = 0x54410004;
const ATAG_INITRD2:   u32 = 0x54410005;
const ATAG_SERIAL:    u32 = 0x54410006;
const ATAG_REVISION:  u32 = 0x54410007;
const ATAG_VIDEOLFB:  u32 = 0x54410008;
const ATAG_CMDLINE:   u32 = 0x54410009;

#[allow(improper_ctypes)]
extern {
    static atags_ptr: *const Atag;
}

#[inline(always)]
pub fn atags() -> &'static Atag {
    unsafe { &*atags_ptr }
}

#[inline]
pub fn init_memory() {
    let c = |atag: &Atag| {
        if let Some(AtagData::Memory(memory)) = atag.data() {
            Some(memory)
        } else {
            None
        }
    };
    let mmap = atags().iter().filter_map(&c);
    let size = mmap.clone().fold(0, |size, memory| size + memory.size);
    assert!(size > 0, "Memory map not recognized");

    memory::init_by_iter(
        size,
        mmap.map(|memory| PhysAddr::from_raw(memory.start) .. PhysAddr::from_raw(memory.start + memory.size))
    );
}

#[repr(C)]
pub struct Atag {
    pub size: u32,
    pub tag: u32,
    data: ()
}

impl Atag {
    pub fn iter(&'static self) -> AtagIter {
        AtagIter {
            atag: self as *const Atag
        }
    }

    pub fn data(&self) -> Option<AtagData> {
        unsafe {
            match self.tag {
                ATAG_CORE      => Some(AtagData::Core(mem::transmute(&self.data))),
                ATAG_MEM       => Some(AtagData::Memory(mem::transmute(&self.data))),
                ATAG_VIDEOTEXT => Some(AtagData::VideoText(mem::transmute(&self.data))),
                ATAG_RAMDISK   => Some(AtagData::RamDisk(mem::transmute(&self.data))),
                ATAG_INITRD2   => Some(AtagData::InitRd2(mem::transmute(&self.data))),
                ATAG_SERIAL    => Some(AtagData::Serial(mem::transmute(&self.data))),
                ATAG_REVISION  => Some(AtagData::Revision(mem::transmute(&self.data))),
                ATAG_VIDEOLFB  => Some(AtagData::VideoLfb(mem::transmute(&self.data))),
                ATAG_CMDLINE   => Some(AtagData::CmdLine(mem::transmute(&self.data))),
                _ => None
            }
        }
    }
}

pub enum AtagData {
    Core(&'static AtagCore),
    Memory(&'static AtagMemory),
    VideoText(&'static AtagVideoText),
    RamDisk(&'static AtagRamDisk),
    InitRd2(&'static AtagInitRd2),
    Serial(&'static AtagSerial),
    Revision(&'static AtagRevision),
    VideoLfb(&'static AtagVideoLfb),
    CmdLine(&'static AtagCmdLine)
}

#[repr(C)]
pub struct AtagCore {
    pub flags: u32,
    pub pagesize: u32,
    pub rootdev: u32
}

#[repr(C)]
pub struct AtagMemory {
    pub size: u32,
    pub start: u32
}

#[repr(C)]
pub struct AtagVideoText {
    pub x: u8,
    pub y: u8,
    pub video_page: u16,
    pub video_mode: u8,
    pub video_cols: u8,
    pub video_ega_bx: u16,
    pub video_lines: u8,
    pub video_isvga: u8,
    pub video_points: u16
}

#[repr(C)]
pub struct AtagRamDisk {
    pub flags: u32,
    pub size: u32,
    pub start: u32
}

#[repr(C)]
pub struct AtagInitRd2 {
    pub start: u32,
    pub size: u32
}

#[repr(C)]
pub struct AtagSerial {
    pub low: u32,
    pub high: u32
}

#[repr(C)]
pub struct AtagRevision {
    pub rev: u32
}

#[repr(C)]
pub struct AtagVideoLfb {
    pub lfb_width:      u16,
    pub lfb_height:     u16,
    pub lfb_depth:      u16,
    pub lfb_linelength: u16,
    pub lfb_base:       u32,
    pub lfb_size:       u32,
    pub red_size:       u8,
    pub red_pos:        u8,
    pub green_size:     u8,
    pub green_pos:      u8,
    pub blue_size:      u8,
    pub blue_pos:       u8,
    pub rsvd_size:      u8,
    pub rsvd_pos:       u8
}

#[repr(C)]
pub struct AtagCmdLine;

impl AtagCmdLine {
    pub fn as_str(&self) -> &'static str {
        unsafe {
            let start = self as *const _ as *const u8;
            str::from_utf8_unchecked(slice::from_raw_parts(start, rt::strlen(start)))
        }
    }
}

#[derive(Clone)]
pub struct AtagIter {
    atag: *const Atag
}

impl Iterator for AtagIter {
    type Item = &'static Atag;

    fn next(&mut self) -> Option<&'static Atag> {
        unsafe {
            if (*self.atag).size > 2 && (*self.atag).tag != ATAG_NONE {
                let ret = &*self.atag;
                self.atag = &*((self.atag as *const usize).offset((*self.atag).size as isize) as *const Atag);
                Some(ret)
            } else {
                None
            }
        }
    }
}

