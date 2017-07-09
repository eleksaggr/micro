use core::ptr::Unique;
use super::Page;
use super::table::{Flags, Table, Level4, PRESENT, P4};
use memory::frame::{self, Frame};

/// Provides methods that allow mapping a physical frame of the
/// [`Frame`](../../frame/struct.Frame.html) type to a virtual address of the
/// [`Page`](../struct.Page.html) type. It holds the only reference to the P4 table.
pub struct Mapper {
    /// The reference to the current P4 table.
    table: Unique<Table<Level4>>,
}

impl Mapper {
    /// Constructs a new `Mapper`.
    ///
    /// # Safety
    /// The `Mapper` uses an `Unique` pointer to hold the reference to the P4 table,
    /// which means multiple instances of the `Mapper` would break this invariant and result in a
    /// panic.
    ///
    /// # Examples
    ///
    /// ```
    /// let m: Mapper = unsafe {Mapper::new()};
    /// ```
    pub unsafe fn new() -> Mapper {
        Mapper { table: Unique::new(P4) }
    }

    /// Returns a reference to the P4 table the mapper operates on.
    ///
    /// # Examples
    ///
    /// ```
    /// let m = unsafe {Mapper::new()};
    /// let table = m.table();
    /// ```
    pub fn table(&self) -> &Table<Level4> {
        unsafe { self.table.as_ref() }
    }

    /// Returns a mutable reference to the P4 table the mapper operates on.
    ///
    /// # Examples
    ///
    /// ```
    /// let m = unsafe {Mapper::new()};
    /// let mut table = m.table();
    /// ```
    pub fn table_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.table.as_mut() }
    }

    /// Translates a physical address to its corresponding virtual address.
    ///
    /// # Examples
    ///
    /// ```
    /// let v = Mapper::translate(0xFFFF).unwrap();
    /// ```
    pub fn translate(addr: usize) -> Option<usize> {
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

    /// Maps a [`Page`](../struct.Page.html) to a [`Frame`](../../frame/struct.Frame.html).
    ///
    /// The allocator decides which frame the page will be mapped to.
    ///
    /// # Panics
    /// The method may panic if the allocator cannot allocate a physical frame.
    ///
    /// # Examples
    ///
    /// ```
    /// let m = unsafe {Mapper::new()};
    /// m.map(Page::containing(0xFFFFFFFF), PRESENT | WRITABLE, allocator);
    /// ```
    pub fn map<A>(&mut self, page: Page, flags: Flags, allocator: &mut A)
    where
        A: frame::Allocator,
    {
        let frame = allocator.allocate().expect("Out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    /// Maps a [`Page`](../struct.Page.html) to a specific
    /// [`Frame`](../../frame/struct.Frame.html).
    ///
    /// # Panics
    /// The method may panic if the page has already been mapped before.
    ///
    /// # Examples
    ///
    /// ```
    /// let m = unsafe {Mapper:new()};
    /// m.map_to(Page::containing(0xFFFFFFFF), Frame::containing(0xABCDFFFF), PRESENT | WRITABLE,
    /// allocator);
    /// ```
    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: Flags, allocator: &mut A)
    where
        A: frame::Allocator,
    {
        let mut p3 = self.table_mut().next_or_create(page.p4_index(), allocator);
        let mut p2 = p3.next_or_create(page.p3_index(), allocator);
        let mut p1 = p2.next_or_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_free());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }

    /// Identity maps the given [`Frame`](../../frame/struct.Frame.html) to the
    /// [`Page`](../struct.Page.html) with the same base address.
    ///
    /// # Panics
    /// The method may panic if the page the frame corresponds to has been already been mapped.
    ///
    /// # Examples
    ///
    /// ```
    /// let m = unsafe {Mapper::new()};
    /// m.map_id(Frame::containing(0xB8000), WRITABLE, allocator);
    /// ```
    pub fn map_id<A>(&mut self, frame: Frame, flags: Flags, allocator: &mut A)
    where
        A: frame::Allocator,
    {
        let page = Page::containing(frame.base());
        self.map_to(page, frame, flags, allocator)
    }


    /// Unmap the given [`Page`](../struct.Page.html) from the table.
    ///
    /// # Panics
    /// The method may panic if the [`Page`](../struct.Page.html) has the
    /// [`HUGE`](../table/constant.HUGE.html) set, or if the given page has not been mapped yet.
    ///
    /// # Examples
    ///
    /// ```
    /// m.unmap(Page::containing(0xFFFF), allocator);
    /// ```
    pub fn unmap<A>(&mut self, page: Page, _: &mut A)
    where
        A: frame::Allocator,
    {
        assert!(Mapper::translate(page.base()).is_some());

        let p1 = self.table_mut()
            .next_mut(page.p4_index())
            .and_then(|p3| p3.next_mut(page.p3_index()))
            .and_then(|p2| p2.next_mut(page.p2_index()))
            .expect("Mapping code does not support huge pages");

        // let frame = p1[page.p1_index()].frame().unwrap();
        p1[page.p1_index()].free();

        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;
        tlb::flush(VirtualAddress(page.base()));

        // allocator.deallocate(frame);
    }
}
