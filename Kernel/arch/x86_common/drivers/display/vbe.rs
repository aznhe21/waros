use memory;
use arch::{multiboot, page};
use drivers::display::{Color, DisplaySize, Display};
use core::cmp;
use core::mem;
use core::{u8, u16, u32};
use core::ops::Range;

#[allow(dead_code)]
pub struct Vbe {
    cinfo: &'static multiboot::VbeControllerInfo,
    minfo: &'static multiboot::VbeModeInfo
}

impl Vbe {
    pub fn new() -> Vbe {
        let vbe = Vbe {
            cinfo: multiboot::info().vbe_controller_info().unwrap(),
            minfo: multiboot::info().vbe_mode_info().unwrap()
        };

        assert!(
            match (vbe.minfo.rmask, vbe.minfo.gmask, vbe.minfo.bmask, vbe.minfo.resv_mask) {
                (8, 8, 8, 8) => u32::BITS,
                (8, 8, 8, 0) => 24,
                (5, 6, 5, 0) => u16::BITS,
                (5, 5, 5, 0) => u16::BITS,
                _            => u8::BITS
            } == vbe.minfo.bpp as usize,
            "assertion failed: VBE Mask: Red={}, Green={}, Blue={}, Reserve={}, Bpp={}",
            vbe.minfo.rmask, vbe.minfo.gmask, vbe.minfo.bmask, vbe.minfo.resv_mask, vbe.minfo.bpp
        );

        let res = vbe.minfo.h_res as usize * vbe.minfo.v_res as usize;
        page::table().map_direct(page::PageTable::FLAGS_KERNEL, vbe.minfo.vram(), res * vbe.minfo.bpp as usize);

        vbe
    }

    pub fn is_available() -> bool {
        multiboot::info().vbe_controller_info().map_or(false, |cinfo| cinfo.is_valid() && cinfo.version >= 0x0102)
    }

    #[inline(always)]
    fn width(&self) -> DisplaySize {
        self.minfo.h_res as DisplaySize
    }

    #[inline(always)]
    fn height(&self) -> DisplaySize {
        self.minfo.v_res as DisplaySize
    }

    #[inline(always)]
    fn vram<T>(&self) -> *mut T {
        self.minfo.phys_base_ptr as *mut T
    }

    #[inline(always)]
    fn bpl<T>(&self) -> usize {
        self.minfo.logical_scan as usize / mem::size_of::<T>()
    }

    #[inline]
    fn clamp_x(&self, x: DisplaySize) -> DisplaySize {
        cmp::min(cmp::max(x, 0), self.width() - 1)
    }

    #[inline]
    fn clamp_y(&self, y: DisplaySize) -> DisplaySize {
        cmp::min(cmp::max(y, 0), self.height() - 1)
    }

    #[inline]
    fn put_by_uint<T: Copy>(&self, color: T, x: DisplaySize, y: DisplaySize) {
        let x = self.clamp_x(x);
        let y = self.clamp_y(y);

        let offset = y as isize * self.bpl::<T>() as isize + x as isize;
        let vram = self.vram::<T>();
        unsafe {
            *vram.offset(offset) = color;
        }
    }

    #[inline]
    fn horizontal_line_by_uint<T : Copy>(&self, color: T, range: Range<DisplaySize>, y: DisplaySize) {
        let range = self.clamp_x(range.start) .. self.clamp_x(range.end);
        let y = self.clamp_y(y);

        unsafe {
            let offset = y as isize * self.bpl::<T>() as isize;
            let vram = self.vram::<T>().offset(offset);
            for i in range {
                *vram.offset(i as isize) = color;
            }
        }
    }

    #[inline]
    fn clear_by_uint<T : Copy>(&self, color: T) {
        let vram = self.vram::<T>();
        for i in 0 .. self.bpl::<u8>() as isize * self.height() as isize {
            unsafe {
                *vram.offset(i) = color;
            }
        }
    }
}

macro_rules! delegate {
    ($name:ident($($arg_name:ident : $arg_type:ty),*) => $to:ident) => {
        fn $name(&self, color: Color $(, $arg_name: $arg_type)*) {
            match (self.minfo.rmask, self.minfo.gmask, self.minfo.bmask, self.minfo.resv_mask) {
                (8, 8, 8, 8) => self.$to(color.as_c32() $(, $arg_name)*),
                (8, 8, 8, 0) => self.$to(color.as_c24() $(, $arg_name)*),
                (5, 6, 5, 0) => self.$to(color.as_c16() $(, $arg_name)*),
                (5, 5, 5, 0) => self.$to(color.as_c15() $(, $arg_name)*),
                _            => self.$to(color.as_c8()  $(, $arg_name)*),
            }
        }
    }
}

impl Display for Vbe {
    fn log(&self) {
        log!("Display: {}x{}@{}bpp", self.minfo.h_res, self.minfo.v_res, self.minfo.bpp);
        log!("Mask: Red={}, Green={}, Blue={}, Reserve={}", self.minfo.rmask, self.minfo.gmask, self.minfo.bmask,
             self.minfo.resv_mask);
    }

    #[inline]
    fn resolution(&self) -> (DisplaySize, DisplaySize) {
        (self.width(), self.height())
    }

    delegate!(put(x: DisplaySize, y: DisplaySize) => put_by_uint);
    delegate!(horizontal_line(range: Range<DisplaySize>, y: DisplaySize) => horizontal_line_by_uint);

    fn clear(&self, color: Color) {
        let size = self.bpl::<u8>() * self.height() as usize;
        match (self.minfo.rmask, self.minfo.gmask, self.minfo.bmask, self.minfo.resv_mask) {
            (8, 8, 8, 8) => unsafe { memory::fill32(self.vram(), color.as_c32(), size) },
            (8, 8, 8, 0) => self.clear_by_uint(color.as_c24()),
            (5, 6, 5, 0) => unsafe { memory::fill16(self.vram(), color.as_c16(), size) },
            (5, 5, 5, 0) => unsafe { memory::fill16(self.vram(), color.as_c15(), size) },
            _            => unsafe { memory::fill8(self.vram(), color.as_c8(), size) }
        }
    }
}

