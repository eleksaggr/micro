#![feature(abi_x86_interrupt)]
#![feature(alloc)]
#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(unique)]
#![no_std]

extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate bit_field;
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
mod memory;
mod interrupt;

use core::fmt;
use util::log::Logger;

#[no_mangle]
pub extern "C" fn kmain(mb_addr: usize) {
    log!(util::log::Level::Info, "Starting execution...");
    let info = unsafe { multiboot2::load(mb_addr) };

    let mut mcon = memory::init(&info);
    interrupt::init(&mut mcon);

    unsafe {
        *(0xdeadbeef as *mut u8) = 0x88;
    }

    panic!("Did not crash!");
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
extern "C" fn panic_fmt(fmt: fmt::Arguments, file: &'static str, line: u32) -> ! {
    log!(util::log::Level::Error,
         "Panicked in {} at line {}: {}",
         file,
         line,
         fmt);
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
