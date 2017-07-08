pub use memory::stack::Stack;
pub use memory::paging::remap_kernel;

use multiboot2::BootInformation;
use self::frame::{Frame, BitmapAllocator, AreaAllocator};
use self::paging::ActiveTable;
use util::log::{Logger, Level};

mod frame;
mod paging;
mod stack;

pub struct MemoryController {
    table: ActiveTable,
    allocator: BitmapAllocator,
    stack_allocator: stack::StackAllocator,
}

impl MemoryController {
    pub fn allocate_stack(&mut self, pages: usize) -> Option<stack::Stack> {
        let &mut MemoryController {
            ref mut table,
            ref mut allocator,
            ref mut stack_allocator,
        } = self;

        stack_allocator.allocate(table, allocator, pages)
    }
}

pub fn init(info: &BootInformation) -> MemoryController {
    let mmtag = info.memory_map_tag().expect("Memory Map Tag required");
    let elftag = info.elf_sections_tag().expect("ELF Sections Tag required");

    let kernel_start = elftag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr)
        .min()
        .unwrap();
    let kernel_end = elftag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr + s.size)
        .max()
        .unwrap();
    let mb_start = info.start_address();
    let mb_end = info.end_address();
    log!(
        Level::Info,
        "Kernel occupies memory from {:#x} to {:#x}",
        kernel_start,
        kernel_end
    );
    log!(
        Level::Info,
        "Multiboot header occupies memory from {:#x} to {:#x}",
        mb_start,
        mb_end
    );

    // Use a simpler AreaAllocator to get the frames the BitmapAllocator needs to store the
    // bitmaps.
    let mut pre_allocator = AreaAllocator::new(
        (kernel_start as usize, kernel_end as usize),
        (mb_start as usize, mb_end as usize),
        mmtag.memory_areas(),
    );

    // Find the total memory size.
    let mut memory_size = 0;
    for area in mmtag.memory_areas() {
        memory_size += area.length;
    }
    log!(
        Level::Info,
        "Total memory size found to be: {} KB",
        memory_size / 1024
    );

    let mut allocator = BitmapAllocator::new((memory_size as usize), &mut pre_allocator);
    let reserved = allocator.used();

    // Remap the kernel.
    let mut table = paging::remap_kernel(&mut allocator, info);

    // Identity map the frames needed by the allocator.
    for frame in Frame::range(
        Frame::containing(reserved.0),
        Frame::containing(reserved.0 + reserved.1),
    ) {
        table.map_id(frame, paging::WRITABLE, &mut allocator);
    }

    use self::paging::Page;
    use buddy::{BASE, SIZE};

    let heap_start_page = Page::containing(BASE);
    let heap_end_page = Page::containing(BASE + SIZE - 1);
    for page in Page::range(heap_start_page, heap_end_page) {
        table.map(page, paging::WRITABLE, &mut allocator);
    }
    log!(
        Level::Info,
        "Heap spans memory region from {:#x} to {:#x}",
        heap_start_page.base_addr(),
        heap_end_page.base_addr() + Page::SIZE - 1
    );

    let stack_allocator = {
        let alloc_start = heap_end_page + 1;
        let alloc_end = alloc_start + 100;
        let alloc_range = Page::range(alloc_start, alloc_end);
        stack::StackAllocator::new(alloc_range)
    };

    MemoryController {
        table: table,
        allocator: allocator,
        stack_allocator: stack_allocator,
    }
}
