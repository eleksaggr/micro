pub use memory::area::AreaFrameAllocator;
pub use memory::paging::remap_kernel;

use multiboot2::BootInformation;

mod area;
mod paging;

pub trait FrameAllocator {
    fn allocate(&mut self) -> Option<Frame>;
    fn deallocate(&mut self, frame: Frame);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    id: usize,
}

impl Frame {
    const SIZE: usize = 4096;

    fn clone(&self) -> Frame {
        Frame { id: self.id }
    }

    fn containing(address: usize) -> Self {
        Frame { id: address / Frame::SIZE }
    }

    fn base_addr(&self) -> usize {
        self.id * Frame::SIZE
    }

    fn range(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }
}

struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.id += 1;
            Some(frame)
        } else {
            None
        }
    }
}

pub fn init(info: &BootInformation) {
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

    println!(
        "Kernel Start: {:#x}, Kernel End: {:#x}",
        kernel_start,
        kernel_end
    );
    println!(
        "Multiboot Start: {:#x}, Multiboot End: {:#x}",
        info.start_address(),
        info.end_address()
    );

    let mut allocator = AreaFrameAllocator::new(
        mmtag.memory_areas(),
        kernel_start as usize,
        kernel_end as usize,
        info.start_address(),
        info.end_address(),
    );

    let mut table = paging::remap_kernel(&mut allocator, info);

    use self::paging::Page;
    use buddy::{BASE, SIZE};

    let heap_start_page = Page::containing(BASE);
    println!(
        "Page containing heap base at {:#x}",
        heap_start_page.base_addr()
    );
    let heap_end_page = Page::containing(BASE + SIZE - 1);
    println!("Page containg heap end at {:#x}", heap_end_page.base_addr());

    for page in Page::range(heap_start_page, heap_end_page) {
        table.map(page, paging::WRITABLE, &mut allocator);
    }
}
