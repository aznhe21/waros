use memory;
use arch::{self, multiboot, page};
use arch::x86_io::{inw, outw};
use drivers::display::{Color, DisplaySize, Display};
use core::u32;

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

pub struct Bochs {
    width: DisplaySize,
    height: DisplaySize,
    vram: *mut u32
}

impl Bochs {
    pub fn new(width: DisplaySize, height: DisplaySize) -> Bochs {
        super::set_rgb_palette();

        let minfo = multiboot::info().vbe_mode_info().unwrap();
        unsafe {
            write_reg(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_DISABLED);
            write_reg(VBE_DISPI_INDEX_XRES,   width as u16);
            write_reg(VBE_DISPI_INDEX_YRES,   height as u16);
            write_reg(VBE_DISPI_INDEX_BPP,    u32::BITS as u16);
            write_reg(VBE_DISPI_INDEX_ENABLE, VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED);

            let res = width as usize * height as usize;
            let vram = minfo.vram();
            let vram_end = vram + (res * u32::BYTES) as arch::AddrType;
            page::table().map_direct(page::PageTable::FLAGS_KERNEL, vram .. vram_end);
        }

        Bochs {
            width: width,
            height: height,
            vram: minfo.phys_base_ptr as *mut u32
        }
    }

    pub fn is_available() -> bool {
        unsafe {
            multiboot::info().vbe_mode_info().is_some() && read_reg(VBE_DISPI_INDEX_ID) & 0xFFF0 == VBE_DISPI_ID0
        }
    }
}

impl Display for Bochs {
    fn log(&self) {
        unsafe {
            let width  = read_reg(VBE_DISPI_INDEX_XRES);
            let height = read_reg(VBE_DISPI_INDEX_YRES);
            let depth  = read_reg(VBE_DISPI_INDEX_BPP);
            log!("Display: {}x{}@{}bpp", width, height, depth);
        }
    }

    fn resolution(&self) -> (DisplaySize, DisplaySize) {
        (self.width, self.height)
    }

    fn put(&self, color: Color, x: DisplaySize, y: DisplaySize) {
        let offset = y * self.width + x;
        unsafe {
            *self.vram.offset(offset as isize) = color.as_rgb().as_c32();
        }
    }

    fn clear(&self, color: Color) {
        let size = self.width as usize * self.height as usize;
        unsafe {
            memory::fill32(self.vram, color.as_rgb().as_c32(), size);
        }
    }
}

