use arch::x86_io::{inw, outw};
use super::{Color, DisplaySize, Display};
use multiboot;

const VBE_DISPI_IOPORT_INDEX: u16 = 0x01CE;
const VBE_DISPI_IOPORT_DATA:  u16 = 0x01CF;

const VBE_DISPI_INDEX_ID:               u16 = 0x0;
const VBE_DISPI_INDEX_XRES:             u16 = 0x1;
const VBE_DISPI_INDEX_YRES:             u16 = 0x2;
const VBE_DISPI_INDEX_BPP:              u16 = 0x3;
const VBE_DISPI_INDEX_ENABLE:           u16 = 0x4;
const VBE_DISPI_INDEX_BANK:             u16 = 0x5;
const VBE_DISPI_INDEX_VIRT_WIDTH:       u16 = 0x6;
const VBE_DISPI_INDEX_VIRT_HEIGHT:      u16 = 0x7;
const VBE_DISPI_INDEX_X_OFFSET:         u16 = 0x8;
const VBE_DISPI_INDEX_Y_OFFSET:         u16 = 0x9;
const VBE_DISPI_INDEX_VIDEO_MEMORY_64K: u16 = 0xa;

const VBE_DISPI_ID0: u16 = 0xB0C0;
const VBE_DISPI_ID1: u16 = 0xB0C1;
const VBE_DISPI_ID2: u16 = 0xB0C2;
const VBE_DISPI_ID3: u16 = 0xB0C3;
const VBE_DISPI_ID4: u16 = 0xB0C4;
const VBE_DISPI_ID5: u16 = 0xB0C5;

const VBE_DISPI_DISABLED:    u16 = 0x00;
const VBE_DISPI_ENABLED:     u16 = 0x01;
const VBE_DISPI_GETCAPS:     u16 = 0x02;
const VBE_DISPI_8BIT_DAC:    u16 = 0x20;
const VBE_DISPI_LFB_ENABLED: u16 = 0x40;
const VBE_DISPI_NOCLEARMEM:  u16 = 0x80;

#[inline]
unsafe fn write_reg(index: u16, data: u16) {
    outw(VBE_DISPI_IOPORT_INDEX, index);
    outw(VBE_DISPI_IOPORT_DATA, data);
}

#[inline]
unsafe fn read_reg(index: u16) -> u16 {
    outw(VBE_DISPI_IOPORT_INDEX, index);
    inw(VBE_DISPI_IOPORT_DATA)
}

//static mut vram: *mut u8 = 0 as *mut u8;

#[inline(always)]
pub fn available() -> bool {
    unsafe { read_reg(VBE_DISPI_INDEX_ID) & 0xFFF0 == VBE_DISPI_ID0 }
}

#[inline(always)]
pub fn pre_init() {
}

/*#[inline(always)]
fn configure(width: u16, height: u16, depth: u16, use_lfb: bool, clear_memory: bool) -> *mut u8 {
    unsafe {
        /*if !available() {
            panic!("Bochs not available");
        }

        let mem = read_reg(VBE_DISPI_INDEX_VIDEO_MEMORY_64K) * 64 * 1024;
        let yres_virt = mem / (width * (depth / 8));*/

        write_reg(VBE_DISPI_INDEX_ENABLE,      VBE_DISPI_DISABLED);
        write_reg(VBE_DISPI_INDEX_XRES,        width);
        write_reg(VBE_DISPI_INDEX_YRES,        height);
        write_reg(VBE_DISPI_INDEX_BPP,         depth);
        //write_reg(VBE_DISPI_INDEX_BANK,        0);
        //write_reg(VBE_DISPI_INDEX_VIRT_WIDTH,  width);
        //write_reg(VBE_DISPI_INDEX_VIRT_HEIGHT, yres_virt);
        //write_reg(VBE_DISPI_INDEX_X_OFFSET,    0);
        //write_reg(VBE_DISPI_INDEX_Y_OFFSET,    0);
        write_reg(VBE_DISPI_INDEX_ENABLE,      VBE_DISPI_ENABLED |
          if use_lfb      { VBE_DISPI_LFB_ENABLED } else { 0 } |
          if clear_memory { 0 }                     else { VBE_DISPI_NOCLEARMEM }
        );

        //log!("{}x{} @ {} bpp", width, height, depth);

        (if use_lfb { 0xFD000000_usize } else { 0x000A0000 }) as *mut u8
    }
}*/

pub struct Bochs {
    width: DisplaySize,
    height: DisplaySize,
    vram: *mut u8
}

impl Bochs {
    pub fn new(width: DisplaySize, height: DisplaySize) -> Bochs {
        super::set_rgb_palette();
        unsafe {
            write_reg(VBE_DISPI_INDEX_ENABLE,      VBE_DISPI_DISABLED);
            write_reg(VBE_DISPI_INDEX_XRES,        width as u16);
            write_reg(VBE_DISPI_INDEX_YRES,        height as u16);
            write_reg(VBE_DISPI_INDEX_BPP,         8);
            write_reg(VBE_DISPI_INDEX_ENABLE,      VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED);
        }

        Bochs {
            width: width,
            height: height,
            vram: multiboot::info().vbe_mode_info().unwrap().vram() as *mut u8
        }
    }
}

impl Display for Bochs {
    fn available() -> bool { available() }

    fn resolution(&self) -> (DisplaySize, DisplaySize) {
        (self.width, self.height)
    }

    fn put(&self, color: Color, x: DisplaySize, y: DisplaySize) {
        let offset = y * self.width + x;
        unsafe {
            *self.vram.offset(offset as isize) = color as u8;
        }
    }
}

/*#[inline(always)]
pub fn init() {
    unsafe {
        common::set_rgb();
        configure(320, 200, 8, true, true);
        /*for i in 0_isize..322+1 {
            *vram.offset(i + 0) = 0xFF;
            // *vram.offset(i * 3 + 1) = 0xFF;
            // *vram.offset(i * 3 + 2) = 0x00;
        }*/
        //vram = 0x000A0000 as *mut u8;

        fill(Color::Yellow, 0, 0, 320, 200);
        /*for i in 0..1024 {
            *vram.offset(i) = Color::Yellow as u8;
        }*/
    }
}

pub fn fill(color: Color, x: u16, y: u16, width: u16, height: u16) {
    unsafe {
        common::fill(vram, color as u8, 320, x, y, width, height);
    }
}*/

