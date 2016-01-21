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
#![crate_name = "kernel"]

#![feature(lang_items, no_std, asm)]
// Crates
#![feature(unicode, alloc, collections)]
// Unstable language features
#![feature(associated_consts, const_fn, concat_idents, augmented_assignments)]
// Unstable library features
#![feature(core_intrinsics, zero_one, num_bits_bytes, op_assign_traits, drop_in_place, fnbox)]
#![feature(unique, shared)]

#![cfg_attr(any(target_arch="x86_64", target_arch="x86"), feature(step_by))]

#![no_std]
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

// Lists library
#[macro_use]
pub mod lists;

pub mod sync;

// Achitecture-specific modules
#[cfg(target_arch="x86_64")] #[path="arch/amd64/mod.rs"]
pub mod arch;
#[cfg(target_arch="x86")] #[path="arch/x86/mod.rs"]
pub mod arch;

// Exception handling (panic)
pub mod unwind;

// Logging code
pub mod logging;

// Memory management
pub mod memory;

pub mod task;

pub mod event;

pub mod timer;

pub mod drivers;

// Kernel entrypoint
#[lang="start"]
#[no_mangle]
pub fn kmain() -> ! {
    use core::cmp;
    use event::Event;
    use drivers::Device;
    use drivers::display::{Color, Display, DisplaySize};
    use drivers::keyboard::Keyboard;

    arch::init_memory();
    //memory::init_by_manual(arch::kernel_end().as_phys_addr() .. arch::kernel_end().as_phys_addr() + 0x1000000);// 16MB
    //memory::init_by_detection(memory::MAX_ADDR);

    event::init();
    timer::init();
    arch::interrupt::init();
    task::init();

    log!("Total: {} MB Free: {} MB", memory::buddy::manager().total_size() / 1024 / 1024,
        memory::buddy::manager().free_size() / 1024 / 1024);

    let display = arch::drivers::display::vbe::Vbe::new();
    display.log();
    display.clear(Color::White);

    let mut mouse_pos = (0 as DisplaySize, 0 as DisplaySize);
    let mut clicking = false;
    let mut color: u8 = 1;

    let (mut pri_count, mut a_count) = ((0usize, 0usize), (0usize, 0usize));

    let a_count_addr = &mut a_count.1 as *mut usize as usize;
    task::spawn(move || {
        use core::intrinsics;

        let a_count = a_count_addr as *mut usize;

        log!("Task-A has launched");
        loop {
            unsafe {
                intrinsics::volatile_store(a_count, intrinsics::overflowing_add(intrinsics::volatile_load(a_count), 1));
            }
        }
    });

    let disp_timer = timer::Timer::with_queue(event::queue());
    disp_timer.reset(1000);

    //arch::interrupt::wait();

    loop {
        arch::interrupt::disable();
        pri_count.1 += 1;

        match event::queue().pop() {
            Some(Event::Device(Device::Keyboard(Keyboard::Down(state)))) => {
                log!("Key down: {:02X}", state.code);
            },
            Some(Event::Device(Device::Keyboard(Keyboard::Press(state)))) => {
                log!("Key press: {:02X}", state.code);
            },
            Some(Event::Device(Device::Keyboard(Keyboard::Up(state)))) => {
                log!("Key up: {:02X}", state.code);

                match state.code {
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
            Some(Event::Device(Device::Mouse(mouse))) => {
                if clicking {
                    clicking = mouse.left();
                } else if mouse.left() {
                    clicking = true;
                }

                let res = display.resolution();
                let prev_mouse = mouse_pos;

                mouse_pos.0 += mouse.x as i32;
                if mouse_pos.0 < 0 {
                    mouse_pos.0 = 0;
                } else if mouse_pos.0 >= res.0 as i32 {
                    mouse_pos.0 = res.0 as i32 - 1;
                }

                mouse_pos.1 += mouse.y as i32;
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
            Some(Event::Timer(timer_id)) => {
                match timer_id {
                    _ if timer_id == disp_timer.id() => {
                        log!("Primary: {}, A: {}", pri_count.1.wrapping_sub(pri_count.0),
                            a_count.1.wrapping_sub(a_count.0));
                        pri_count.0 = pri_count.1;
                        a_count.0 = a_count.1;

                        disp_timer.reset(1000);
                    },
                    _ => log!("Timer {}", timer_id)
                }
            },
            None => {
                //arch::interrupt::enable_wait();
                arch::interrupt::enable();
            }
        }
    }
}

