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
#![feature(associated_consts, const_fn)]
#![feature(core_intrinsics)]
#![feature(zero_one, num_bits_bytes, step_by, ptr_as_ref, iter_arith)]
#![feature(unicode, alloc, collections)]

#![no_std]	//< Kernels can't use std
#![no_builtins]

extern crate rustc_unicode;
extern crate alloc_system;
extern crate alloc;
extern crate collections;

// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

// Runtime functions
pub mod rt;

// Prelude
mod prelude;

// Lists library
#[macro_use]
mod lists;

// Achitecture-specific modules
#[cfg(target_arch="x86_64")] #[path="arch/amd64/mod.rs"]
pub mod arch;
#[cfg(target_arch="x86")] #[path="arch/x86/mod.rs"]
pub mod arch;

mod num_traits;

// Exception handling (panic)
pub mod unwind;

// Logging code
mod logging;

// Multiboot data
mod multiboot;

// Memory management
pub mod memory;

mod event;

mod timer;

// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kmain() -> ! {
    use core::cmp;
    use event::Event;
    use arch::interrupt::device::Device;
    use arch::drivers::display::{self, Color, Display, DisplaySize};

    log!("WARos: Switched to protected mode");

    multiboot::init();
    memory::init_by_multiboot(multiboot::info().mmap().expect("Memory map not provided"));
    timer::init();
    arch::interrupt::init();
    arch::task::init();

    log!("Total: {} MB Free: {} MB", memory::buddy::manager().total_size() / 1024 / 1024,
        memory::buddy::manager().free_size() / 1024 / 1024);

    let display = display::vbe::Vbe::new();
    display.log();
    display.clear(Color::White);

    let mut mouse_pos = (0 as DisplaySize, 0 as DisplaySize);
    let mut clicking = false;
    let mut color: u8 = 1;

    let (mut pri_count, mut a_count) = ((0usize, 0usize), (0usize, 0usize));

    arch::task::manager().add(task_a, &mut a_count.1);
    arch::task::manager().reset_timer();

    let disp_timer = timer::manager().by_queue(event::queue());
    disp_timer.reset(1000);

    loop {
        arch::interrupt::cli();
        pri_count.1 += 1;

        match event::Event::get() {
            Event::Device(Device::KeyDown(code)) => {
                log!("Key down: {:02X}", code);
            },
            Event::Device(Device::KeyUp(code)) => {
                // keyup
                log!("Key up: {:02X}", code);

                match code {
                    0x01 => {// Esc
                        display.clear(Color::White);
                    },
                    0x1C => {// Enter
                        let color = Color::from_u8(color).unwrap();
                        display.clear(color);
                    },
                    0x39 => {// Space
                        log!("Counter: {}", timer::manager().counter());
                    },
                    _ => {}
                }
            },
            Event::Device(Device::Mouse(mouse)) => {
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
                    let r = cmp::min(mouse_pos.0 + 4, res.0 - 1);// 右端を切り捨て
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
            Event::Timer(timer_id) => {
                match timer_id {
                    _ if timer_id == disp_timer.id() => {
                        log!("Primary: {}, A: {}", pri_count.1 - pri_count.0, a_count.1 - a_count.0);
                        pri_count.0 = pri_count.1;
                        a_count.0 = a_count.1;
                        disp_timer.reset(1000);
                    },
                    _ => log!("Timer {}", timer_id)
                }
            },
            _ => {
                //arch::interrupt::sti_hlt();
                arch::interrupt::sti();
            }
        }
    }
}

extern "C" fn task_a(a_count: &mut usize) {
    use core::intrinsics;
    log!("Task-A has launched");
    loop {
        unsafe {
            intrinsics::volatile_store(a_count, intrinsics::overflowing_add(intrinsics::volatile_load(a_count), 1));
        }
    }
}

