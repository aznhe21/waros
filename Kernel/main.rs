/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * main.rs
 * - Top-level file for kernel
 *
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */
#![feature(no_std)]	//< unwind needs to define lang items
#![feature(lang_items)]	//< unwind needs to define lang items
#![feature(asm)]	//< As a kernel, we need inline assembly
#![feature(core)]	//< libcore (see below) is not yet stablized
#![feature(alloc)]	//< liballoc (see below) is not yet stablized
#![feature(associated_consts)]
#![feature(zero_one)]
#![feature(concat_idents)]
#![no_std]	//< Kernels can't use std
#![no_builtins]

use prelude::*;

// Load libcore (it's nice and freestanding)
// - We want the macros from libcore
#[macro_use]
extern crate core;
extern crate alloc;

// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

// Runtime functions
pub mod rt;

// Achitecture-specific modules
#[cfg(target_arch="x86_64")] #[path="arch/amd64/mod.rs"]
pub mod arch;
#[cfg(target_arch="x86")] #[path="arch/x86/mod.rs"]
pub mod arch;

// Prelude
mod prelude;

mod num_traits;

// Collections library
#[macro_use]
mod collections;

// Exception handling (panic)
pub mod unwind;

// Logging code
mod logging;

// Multiboot data
mod multiboot;

// Memory management
pub mod memory;

// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kmain() {
    use core::cmp;
    use arch::interrupt::input::{Event, Key};

    use arch::drivers::display;
    use arch::drivers::display::{Color, Display, DisplaySize};

    log!("WARos: Switched to protected mode");

    if !multiboot::magic_valid() {
        panic!("Multiboot magic is invalid");
    }

    if !multiboot::info().vbe_controller_info().expect("VBE not supported").valid() {
        panic!("VBE signature is invalid");
    }

    //memory::init(0x00400000, 0xC0000000);
    arch::interrupt::init();
    arch::drivers::init();

    let mut msize = 1024 * 1024_u32;
    for mmap in multiboot::info().mmap().expect("There is no memory map information") {
        log!("Base: {:08X}, Length: {:08X}, Type: {}", mmap.base_addr, mmap.length, mmap.type_);
        log!("End: {:08X}", mmap.base_addr + mmap.length);
        if mmap.type_ == multiboot::MemoryType::Usable {
            msize += mmap.length as u32;
        }
    }
    log!("MMap Size: {} MB", msize / 1024 / 1024);

    log!("Memory: {} MB", memory::size() / 1024 / 1024);
    //log!("CmdLine: {}", multiboot::info().str_cmdline());

    //let display = display::vga::Vga::new();
    //let display = display::bochs::Bochs::new(1024, 768);
    let display = display::vbe::Vbe::new();
    display.log();
    display.clear(Color::White);

    let mut mouse_pos = (0 as DisplaySize, 0 as DisplaySize);
    let mut clicking = false;
    let mut color: u8 = 1;

    arch::interrupt::sti();
    arch::interrupt::hlt();

    loop {
        arch::interrupt::cli();

        match arch::interrupt::input::get() {
            Event::Keyboard(Key::Down(code)) => {
                log!("Key down: {:02x}", code);
            },
            Event::Keyboard(Key::Up(code)) => {
                // keyup
                log!("Key up: {:02x}", code);

                match code {
                    0x01 => {// Esc
                        display.clear(Color::White);
                    },
                    0x1C => {// Enter
                        let color = Color::from_u8(color).unwrap();
                        display.clear(color);
                    }
                    _ => {}
                }
            },
            Event::Mouse(mouse) => {
                if clicking {
                    clicking = mouse.left;
                } else if mouse.left {
                    clicking = true;
                }

                let res = display.resolution();
                let prev_mouse = mouse_pos;

                mouse_pos.0 += mouse.x;
                if mouse_pos.0 < 0 {
                    mouse_pos.0 = 0;
                } else if mouse_pos.0 >= res.0 as i32 {
                    mouse_pos.0 = res.0 as i32 - 1;
                }

                mouse_pos.1 += mouse.y;
                if mouse_pos.1 < 0 {
                    mouse_pos.1 = 0;
                } else if mouse_pos.1 >= res.1 as i32 {
                    mouse_pos.1 = res.1 as i32 - 1;
                }
                log!("Mouse: {} {}", mouse_pos.0, mouse_pos.1);

                let pcolor = Color::from_u8(color).unwrap();
                if clicking {
                    let l = cmp::max(0, mouse_pos.0 - 4);        // 左端を切り上げ
                    //let r = cmp::min(mouse_pos.0 + 4, res.0 - 1);// 右端を切り捨て
                    let r = mouse_pos.0 + 4;
                    let t = cmp::max(0, mouse_pos.1 - 4);        // 上端を切り上げ
                    let b = cmp::min(mouse_pos.1 + 4, res.1 - 1);// 下端を切り捨て
                    display.fill(pcolor, (l, t, r - l, b - t));
                } else {
                    //display.put(pcolor, mouse_pos.0, mouse_pos.1);
                    display.line(pcolor, (prev_mouse.0, prev_mouse.1), (mouse_pos.0, mouse_pos.1));
                }

                color += 1;
                if color > 15 {
                    color = 1;
                }
            },
            Event::None => {
                arch::interrupt::sti();
                arch::interrupt::hlt();
            }
        }
    }
}
