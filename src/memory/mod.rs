pub use memory::paging::remap_kernel;

use multiboot2::BootInformation;
use self::frame::BitmapAllocator;

mod frame;
mod paging;

pub fn init(info: &BootInformation) {
    let mmtag = info.memory_map_tag().expect("Memory Map Tag required");
    let elftag = info.elf_sections_tag().expect("ELF Sections Tag required");

    println!("Finding kernel start...");
    let kernel_start = elftag.sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr)
        .min()
        .unwrap();
    println!("Finding kernel end...");
    let kernel_end = elftag.sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr + s.size)
        .max()
        .unwrap();

    let mut allocator = BitmapAllocator::new((info.total_size as usize) * 1024,
                                             mmtag.memory_areas());

    let mut table = paging::remap_kernel(&mut allocator, info);

    use self::paging::Page;
    use buddy::{BASE, SIZE};

    let heap_start_page = Page::containing(BASE);
    let heap_end_page = Page::containing(BASE + SIZE - 1);
    for page in Page::range(heap_start_page, heap_end_page) {
        table.map(page, paging::WRITABLE, &mut allocator);
    }
}
