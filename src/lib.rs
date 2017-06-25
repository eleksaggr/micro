#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(unique)]
#![feature(alloc)]
#![no_std]

extern crate rlibc;
extern crate volatile;
extern crate multiboot2;
extern crate spin;
extern crate x86_64;
#[macro_use]
extern crate bitflags;
extern crate buddy;
#[macro_use]
extern crate alloc;

#[macro_use]
mod vga;
mod memory;

use core::fmt;
use alloc::boxed::Box;
use alloc::vec::Vec;

#[no_mangle]
pub extern "C" fn kmain(mb_addr: usize) {
    let info = unsafe { multiboot2::load(mb_addr) };
    // let mm_tag = info.memory_map_tag().expect("Memory map tag required");
    // let elf_sections_tag = info.elf_sections_tag().expect("Elf sections tag required");

    // let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    // let kernel_stop = elf_sections_tag
    //     .sections()
    //     .map(|s| s.addr + s.size)
    //     .max()
    //     .unwrap();

    // let mb_end = mb_addr + (info.total_size as usize);
    // println!("Boundaries: ");
    // println!("  Kernel: 0x{:x} until 0x{:x}", kernel_start, kernel_stop);
    // println!("  Multiboot: 0x{:x} until 0x{:x}", mb_addr, mb_end);

    // println!("Sections: ");
    // for section in elf_sections_tag.sections() {
    //     println!(
    //         "  Addr: 0x{:x}, Size: 0x{:x}, Flags: 0x{:x}",
    //         section.addr,
    //         section.size,
    //         section.flags
    //     );
    // }

    // let mut allocator = memory::AreaFrameAllocator::new(
    //     mm_tag.memory_areas(),
    //     kernel_start as usize,
    //     kernel_end as usize,
    //     mb_start as usize,
    //     mb_end as usize,
    // );

    enable_nxe();
    enable_wp();
    memory::init(info);
    // memory::remap_kernel(&mut allocator, info);

    let mut test = Box::new(42);
    *test -= 15;
    let test2 = Box::new("Hello");
    println!("{:?} {:?}", test, test2);

    let mut vec = vec![1, 2, 3, 4, 5, 6, 7];
    vec[3] = 42;
    for i in &vec {
        print!("{}", i);
    }

    let mut test3 = Vec::with_capacity(10000);
    test3.push(100);

    panic!("Execution ended.");
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
    vga::set_color(vga::Color::Red, vga::Color::Black);
    println!("Panicked in {} at line {}: {}", file, line, fmt);
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
