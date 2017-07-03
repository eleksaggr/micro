pub use memory::paging::remap_kernel;

use multiboot2::BootInformation;
use self::frame::{BitmapAllocator, AreaAllocator};

mod frame;
mod paging;

pub fn init(info: &BootInformation) {
    let mmtag = info.memory_map_tag().expect("Memory Map Tag required");
    let elftag = info.elf_sections_tag().expect("ELF Sections Tag required");

    println!("Finding kernel start...");
    let kernel_start = elftag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr)
        .min()
        .unwrap();
    println!("Finding kernel end...");
    let kernel_end = elftag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr + s.size)
        .max()
        .unwrap();
    println!(
        "Kernel located at: {:#x} until {:#x}",
        (kernel_start as usize),
        (kernel_end as usize)
    );

    let mb_start = info.start_address();
    let mb_end = info.end_address();
    println!(
        "Multiboot located at: {:#x} until {:#x}",
        (mb_start as usize),
        (mb_end as usize)
    );


    let mut pre_allocator = AreaAllocator::new(
        (kernel_start as usize, kernel_end as usize),
        (mb_start as usize, mb_end as usize),
        mmtag.memory_areas(),
    );


    let mut memory_size = 0;
    for area in mmtag.memory_areas() {
        memory_size += area.length;
    }

    let mut allocator = BitmapAllocator::new((memory_size as usize), &mut pre_allocator);
    let reserved = allocator.used();

    let mut table = paging::remap_kernel(&mut allocator, info, reserved);

    use self::paging::Page;
    use buddy::{BASE, SIZE};

    let heap_start_page = Page::containing(BASE);
    let heap_end_page = Page::containing(BASE + SIZE - 1);
    for page in Page::range(heap_start_page, heap_end_page) {
        table.map(page, paging::WRITABLE, &mut allocator);
    }
}
