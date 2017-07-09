//! The kernel of the Zinc operating system.

#![feature(abi_x86_interrupt)]
#![feature(alloc)]
#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(unique)]
#![no_std]

#![deny(missing_docs)]

extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate buddy;
#[macro_use]
extern crate lazy_static;
extern crate multiboot2;
extern crate spin;
extern crate rlibc;
extern crate volatile;
extern crate x86_64;

#[macro_use]
mod vga;
#[macro_use]
mod util;
mod sync;
mod memory;
mod interrupt;
mod error;

use core::fmt;
use util::log::{Level, Logger};

#[no_mangle]
/// The kernel entry point.
pub extern "C" fn kmain(mb_addr: usize) {
    log!(Level::Info, "Starting execution...");
    let info = unsafe { multiboot2::load(mb_addr) };

    log!(Level::Info, "Initializing memory...");
    let mut mcon = memory::init(&info);

    log!(Level::Info, "Enabling interrupt handlers...");
    interrupt::init(&mut mcon);

    panic!("Did not crash!");
}

#[lang = "eh_personality"]
/// Not too sure.
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
/// Prints information about a panic that occured, including the filename, line number and a
/// message.
extern "C" fn panic_fmt(fmt: fmt::Arguments, file: &'static str, line: u32) -> ! {
    log!(
        util::log::Level::Error,
        "Panicked in {} at line {}: {}",
        file,
        line,
        fmt
    );
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
/// Not too sure.
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
