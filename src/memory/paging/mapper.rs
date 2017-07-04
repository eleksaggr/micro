use core::ptr::Unique;
use super::Page;
use super::table::{Flags, Table, Level4, PRESENT, P4};
use memory::frame::{self, Frame};

pub struct Mapper {
    p4: Unique<Table<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper { p4: Unique::new(P4) }
    }

    // pub fn get(&self) -> &Table<Level4> {
    //     unsafe { self.p4.as_ref() }
    // }

    pub fn get_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn translate(&self, addr: usize) -> Option<usize> {
        let offset = addr % Page::SIZE;

        let page = Page::containing(addr);
        let frame = {
            let p3 = unsafe { &*P4 }.next(page.p4_index());

            // TODO: Handle huge pages.

            p3.and_then(|p3| p3.next(page.p3_index()))
                .and_then(|p2| p2.next(page.p2_index()))
                .and_then(|p1| p1[page.p1_index()].frame())
        };
        frame.map(|f| f.id() * Page::SIZE + offset)
    }


    pub fn map<A>(&mut self, page: Page, flags: Flags, allocator: &mut A)
    where
        A: frame::Allocator,
    {
        let frame = allocator.allocate().expect("Out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: Flags, allocator: &mut A)
    where
        A: frame::Allocator,
    {
        let mut p3 = self.get_mut().next_or_create(page.p4_index(), allocator);
        let mut p2 = p3.next_or_create(page.p3_index(), allocator);
        let mut p1 = p2.next_or_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_free());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }

    pub fn map_id<A>(&mut self, frame: Frame, flags: Flags, allocator: &mut A)
    where
        A: frame::Allocator,
    {
        let page = Page::containing(frame.base());
        self.map_to(page, frame, flags, allocator)
    }


    pub fn unmap<A>(&mut self, page: Page, _: &mut A)
    where
        A: frame::Allocator,
    {
        assert!(self.translate(page.base_addr()).is_some());

        let p1 = self.get_mut()
            .next_mut(page.p4_index())
            .and_then(|p3| p3.next_mut(page.p3_index()))
            .and_then(|p2| p2.next_mut(page.p2_index()))
            .expect("Mapping code does not support huge pages");

        // let frame = p1[page.p1_index()].frame().unwrap();
        p1[page.p1_index()].free();

        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;
        tlb::flush(VirtualAddress(page.base_addr()));

        // allocator.deallocate(frame);
    }
}
