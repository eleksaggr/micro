#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(unique)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate multiboot2;
extern crate spin;
#[macro_use]
extern crate bitflags;

#[macro_use]
mod vga;
mod memory;

use core::fmt;
use memory::FrameAllocator;

#[no_mangle]
pub extern "C" fn kmain(mb_addr: usize) {
    let info = unsafe { multiboot2::load(mb_addr) };
    let mm_tag = info.memory_map_tag().expect("Memory map tag required");
    let elf_sections_tag = info.elf_sections_tag().expect("Elf sections tag required");

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_stop = elf_sections_tag
        .sections()
        .map(|s| s.addr + s.size)
        .max()
        .unwrap();

    let mb_end = mb_addr + (info.total_size as usize);
    println!("Boundaries: ");
    println!("  Kernel: 0x{:x} until 0x{:x}", kernel_start, kernel_stop);
    println!("  Multiboot: 0x{:x} until 0x{:x}", mb_addr, mb_end);

    println!("Sections: ");
    for section in elf_sections_tag.sections() {
        println!(
            "  Addr: 0x{:x}, Size: 0x{:x}, Flags: 0x{:x}",
            section.addr,
            section.size,
            section.flags
        );
    }

    let mut allocator = memory::AreaFrameAllocator::new(
        mm_tag.memory_areas(),
        kernel_start as usize,
        mb_end as usize,
    );
    println!("{:?}", allocator.allocate());

    panic!("Execution ended.");
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
extern "C" fn panic_fmt(fmt: fmt::Arguments, file: &'static str, line: u32) -> ! {
    vga::set_color(vga::Color::Red, vga::Color::Black);
    println!("Panicked in {} at line {}: {}", file, line, fmt);
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
