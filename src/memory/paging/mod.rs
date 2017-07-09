pub use self::mapper::Mapper;
pub use self::table::{ActiveTable, PRESENT, WRITABLE};

use core::ops::Add;
use memory::frame::{self, Frame};
use memory::paging::table::{Flags, InactiveTable, TempPage};
use multiboot2::BootInformation;

mod mapper;
mod table;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    id: usize,
}

impl Page {
    pub const SIZE: usize = 4096;

    pub fn containing(addr: usize) -> Page {
        assert!(
            addr < 0x0000_8000_0000_0000 || addr >= 0xffff_8000_0000_0000,
            "Invalid Address: 0x{:x}",
            addr
        );
        Page { id: addr / Page::SIZE }
    }

    pub fn base(&self) -> usize {
        self.id * Page::SIZE
    }

    pub fn p4_index(&self) -> usize {
        (self.id >> 27) & 0x1FF
    }
    pub fn p3_index(&self) -> usize {
        (self.id >> 18) & 0x1FF
    }
    pub fn p2_index(&self) -> usize {
        (self.id >> 9) & 0x1FF
    }
    pub fn p1_index(&self) -> usize {
        self.id & 0x1FF
    }

    pub fn range(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
    }
}

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs: usize) -> Page {
        Page { id: self.id + rhs }
    }
}

#[derive(Debug, Clone)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let page = self.start;
            self.start.id += 1;
            Some(page)
        } else {
            None
        }
    }
}

pub fn remap_kernel<A>(allocator: &mut A, info: &BootInformation) -> ActiveTable
where
    A: frame::Allocator,
{
    let mut temp = TempPage::new(Page { id: 0xdeadaffe }, allocator);

    let mut table = unsafe { ActiveTable::new() };
    let mut new = {
        let frame = allocator.allocate().expect("No more frames");
        InactiveTable::new(frame, &mut table, &mut temp)
    };

    table.with(&mut new, &mut temp, |mapper| {
        let tag = info.elf_sections_tag().expect("Memory Map Tag not valid");

        for section in tag.sections() {
            if !section.is_allocated() {
                continue;
            }

            assert!(
                section.start_address() % Page::SIZE == 0,
                "Sections need to be aligned to the page size."
            );

            let flags = Flags::from_elf(section);
            let start = Frame::containing(section.start_address());
            let end = Frame::containing(section.end_address() - 1);

            for frame in Frame::range(start, end) {
                mapper.map_id(frame, flags, allocator);
            }
        }

        // Identity map the VGA buffer.
        mapper.map_id(Frame::containing(0xb8000), WRITABLE, allocator);

        // Identity map the Multiboot info structure.
        let mb_start = Frame::containing(info.start_address());
        let mb_end = Frame::containing(info.end_address() - 1);
        for frame in Frame::range(mb_start, mb_end) {
            mapper.map_id(frame, PRESENT, allocator);
        }
    });

    let old = table.switch(new);

    let old_p4 = Page::containing(old.frame.base());
    table.unmap(old_p4, allocator);

    table
}
