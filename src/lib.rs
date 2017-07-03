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
mod interrupt;
mod memory;

use core::fmt;

#[no_mangle]
pub extern "C" fn kmain(mb_addr: usize) {
    let info = unsafe { multiboot2::load(mb_addr) };
    enable_nxe();
    enable_wp();

    let mut mcon = memory::init(info);
    interrupt::init(&mut mcon);

    unsafe {
        *(0xdeadbeef as *mut u8) = 0xAB;
    }

    println!("Did not crash!");
    loop {}
}

fn enable_nxe() {
    use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

    let nxe = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe);
    }
}

fn enable_wp() {
    use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};

    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
extern "C" fn panic_fmt(fmt: fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("Panicked in {} at line {}: {}", file, line, fmt);
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
