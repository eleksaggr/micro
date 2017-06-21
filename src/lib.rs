#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate multiboot2;
extern crate spin;

use core::fmt;

#[macro_use]
mod vga;

#[no_mangle]
pub extern "C" fn kmain(mb_addr: usize) {
    vga::clear_screen();
    println!("Hello, World!aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    panic!("Execution ended.");
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
extern "C" fn panic_fmt(fmt: fmt::Arguments, file_line: (&(&'static str, u32))) -> ! {
    vga::set_color(vga::Color::Red, vga::Color::Black);
    println!("Panicked in X at line Y: {}", fmt);
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
