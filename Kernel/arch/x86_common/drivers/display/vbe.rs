use prelude::*;
use multiboot;
use memory;
use arch::page;
use memory::kernel::PhysAddr;
use num_traits::Unsigned;
use super::{Color, DisplaySize, Display};
use core::{u8, u16, u32};
use core::ops::Range;

pub struct Vbe {
    cinfo: &'static multiboot::VbeControllerInfo,
    minfo: &'static multiboot::VbeModeInfo
}

impl Vbe {
    pub fn new() -> Vbe {
        /*unsafe {
            let ginfo = multiboot::info().vbe_controller_info().expect("VBE should be supported");

            if !ginfo.valid() || ginfo.version < 0x0102 {
                panic!("VESA is not supported");
            }
        }*/

        super::set_rgb_palette();

        let vbe = Vbe {
            cinfo: multiboot::info().vbe_controller_info().unwrap(),
            minfo: multiboot::info().vbe_mode_info().unwrap()
        };

        assert_eq!(match (vbe.minfo.rmask, vbe.minfo.gmask, vbe.minfo.bmask) {
            (8, 8, 8) => u32::BITS,
            (5, 6, 5) => u16::BITS,
            (5, 5, 5) => u16::BITS,
            _         => u8::BITS
        }, vbe.minfo.bpp as usize);

        let res = vbe.minfo.h_res as usize * vbe.minfo.v_res as usize;
        unsafe {
            let vram = vbe.vram::<u8>();
            let vram_end = vram.uoffset(res * vbe.minfo.bpp as usize);
            page::kernel_pt.map_direct(3, 3, PhysAddr::from_mut_ptr(vram) .. PhysAddr::from_mut_ptr(vram_end));
        }

        vbe
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
        self.minfo.vram() as *mut T
    }

    #[inline]
    fn put_by_uint<T: Unsigned + Copy>(&self, color: T, x: DisplaySize, y: DisplaySize) {
        debug_assert!(x >= 0 && x < self.width());
        debug_assert!(y >= 0 && y < self.height());

        let offset = y * self.width() + x;
        let vram = self.vram::<T>();
        unsafe {
            *vram.uoffset(offset as usize) = color;
        }
    }

    #[inline]
    fn horizontal_line_by_uint<T : Unsigned + Copy>(&self, color: T, range: Range<DisplaySize>, y: DisplaySize) {
        debug_assert!(range.start >= 0 && range.end < self.width());
        debug_assert!(y >= 0 && y < self.height());

        unsafe {
            let offset = y * self.width();
            let vram = self.vram::<T>().uoffset(offset as usize);
            for i in range {
                *vram.uoffset(i as usize) = color;
            }
        }
    }

    /*#[inline]
    fn clear_by_uint<T : Unsigned + Copy>(&self, color: T) {
        let vram = self.vram::<T>();
        for i in 0 .. self.width() as usize * self.height() as usize {
            unsafe {
                *vram.uoffset(i) = color;
            }
        }
    }*/
}

macro_rules! delegate {
    ($name:ident($($arg_name:ident : $arg_type:ty),*) => $to:ident) => {
        fn $name(&self, color: Color $(, $arg_name: $arg_type)*) {
            let rgb = color.as_rgb();
            match (self.minfo.rmask, self.minfo.gmask, self.minfo.bmask) {
                (8, 8, 8) => self.$to(rgb.as_c32() $(, $arg_name)*),
                (5, 6, 5) => self.$to(rgb.as_c16() $(, $arg_name)*),
                (5, 5, 5) => self.$to(rgb.as_c15() $(, $arg_name)*),
                _         => self.$to(rgb.as_c8() $(, $arg_name)*),
            }
        }
    }
}

impl Display for Vbe {
    fn available() -> bool {
        multiboot::info().vbe_controller_info().map_or(false, |cinfo| cinfo.valid() && cinfo.version >= 0x0102)
    }

    fn log(&self) {
        log!("Display: {}x{}@{}bpp", self.minfo.h_res, self.minfo.v_res, self.minfo.bpp);
        log!("Mask: Red={}, Green={}, Blue={}", self.minfo.rmask, self.minfo.gmask, self.minfo.bmask);
    }

    fn resolution(&self) -> (DisplaySize, DisplaySize) {
        (self.width(), self.height())
    }

    delegate!(put(x: DisplaySize, y: DisplaySize) => put_by_uint);
    delegate!(horizontal_line(range: Range<DisplaySize>, y: DisplaySize) => horizontal_line_by_uint);
    //delegate!(clear() => clear_by_uint);

    fn clear(&self, color: Color) {
        let rgb = color.as_rgb();
        let size = self.width() as usize * self.height() as usize;
        match (self.minfo.rmask, self.minfo.gmask, self.minfo.bmask) {
            (8, 8, 8) => unsafe { memory::fill32(self.vram(), rgb.as_c32(), size) },
            (5, 6, 5) => unsafe { memory::fill16(self.vram(), rgb.as_c16(), size) },
            (5, 5, 5) => unsafe { memory::fill16(self.vram(), rgb.as_c15(), size) },
            _         => unsafe { memory::fill8(self.vram(), rgb.as_c8(), size) }
        }
    }
}

