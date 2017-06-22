mod entry;
mod table;

pub use self::entry::*;
use memory::Frame;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub struct Page {
    id: usize,
}

impl Page {
    const ENTRIES: usize = 512;

    pub fn containing(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 ||
                address >= 0xffff_8000_0000_0000,
                "Invalid Address: 0x{:x}", address);
        Page{ id: address / Frame::SIZE }
    }


    fn p4_index(&self) -> usize {
        (self.id >> 27) & 0x1FF
    }

    fn p3_index(&self) -> usize {
        (self.id >> 18) & 0x1FF
    }

    fn p2_index(&self) -> usize {
        (self.id >> 9) & 0x1FF
    }

    fn p1_index(&self) -> usize {
        (self.id >> 0) & 0x1FF     
    }
}

pub fn translate(addr: VirtualAddress) -> Option<PhysicalAddress> {
    let offset = addr % Frame::SIZE;
    translate_page(Page::containing(addr)).map(|frame| frame.id * Frame::SIZE + offset)
}

pub fn translate_page(page: Page) -> Option<Frame> {
    let p3 = unsafe {&*table::Table::P4}.next_table(page.p4_index());

    p3.and_then(|p3| p3.next_table(page.p3_index()))
        .and_then(|p2| p2.next_table(page.p2_index()))
        .and_then(|p1| p1[page.p1_index()].get_frame())
}

