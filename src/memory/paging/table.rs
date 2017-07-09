use core::marker::PhantomData;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use memory::frame::{self, Frame};
use memory::paging::Page;
use memory::paging::mapper::Mapper;
use multiboot2::ElfSection;

/// A mutable reference to the current P4 table.
pub const P4: *mut Table<Level4> = 0xffff_ffff_ffff_f000 as *mut _;

// This should be an associated const, but cannot be used in the array defintion.
/// The amount of entries one table has.
const ENTRIES: usize = 512;

/// A representation of a paging table that holds [`ENTRIES`](constant.ENTRIES.html) amount of
/// [`Entry`](struct.Entry.html) objects.
pub struct Table<L: Level> {
    entries: [Entry; ENTRIES],
    level: PhantomData<L>,
}

impl<L> Table<L>
where
    L: Level,
{
    /// Resets the table by putting all entries to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// t.reset();
    /// ```
    pub fn reset(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.free();
        }
    }
}

impl<L> Index<usize> for Table<L>
where
    L: Level,
{
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L>
where
    L: Level,
{
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

impl<L> Table<L>
where
    L: IterableLevel,
{
    pub fn next(&self, index: usize) -> Option<&Table<L::Next>> {
        self.next_addr(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    pub fn next_mut(&mut self, index: usize) -> Option<&mut Table<L::Next>> {
        self.next_addr(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    fn next_addr(&self, index: usize) -> Option<usize> {
        let flags = self[index].flags();
        if flags.contains(PRESENT) && !(flags.contains(HUGE)) {
            let addr = self as *const _ as usize;
            Some((addr << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_or_create<A>(&mut self, index: usize, allocator: &mut A) -> &mut Table<L::Next>
    where
        A: frame::Allocator,
    {
        if self.next(index).is_none() {
            assert!(
                !self.entries[index].flags().contains(HUGE),
                "Mapping code does not support huge pages"
            );
            let frame = allocator.allocate().expect("No frames available");
            self.entries[index].set(frame, PRESENT | WRITABLE);
            self.next_mut(index).unwrap().reset();
        }
        self.next_mut(index).unwrap()
    }
}

pub trait Level {}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl Level for Level4 {}
impl Level for Level3 {}
impl Level for Level2 {}
impl Level for Level1 {}

pub trait IterableLevel: Level {
    type Next: Level;
}

impl IterableLevel for Level4 {
    type Next = Level3;
}

impl IterableLevel for Level3 {
    type Next = Level2;
}

impl IterableLevel for Level2 {
    type Next = Level1;
}

bitflags! {
    pub flags Flags: u64 {
        const PRESENT =         1 << 0,
        const WRITABLE =        1 << 1,
        const SUPERVISOR =      1 << 2,
        const WRITE_THROUGH =   1 << 3,
        const NO_CACHE =        1 << 4,
        const ACCESSED =        1 << 5,
        const DIRTY =           1 << 6,
        const HUGE =            1 << 7,
        const GLOBAL =          1 << 8,
        const NO_EXEC =         1 << 63,
    }
}

impl Flags {
    pub fn from_elf(section: &ElfSection) -> Flags {
        use multiboot2::{ELF_SECTION_ALLOCATED, ELF_SECTION_WRITABLE, ELF_SECTION_EXECUTABLE};

        let mut flags = Flags::empty();

        if section.flags().contains(ELF_SECTION_ALLOCATED) {
            flags = flags | PRESENT;
        }
        if section.flags().contains(ELF_SECTION_WRITABLE) {
            flags = flags | WRITABLE;
        }
        if !section.flags().contains(ELF_SECTION_EXECUTABLE) {
            flags = flags | NO_EXEC;
        }

        flags
    }
}

/// An entry in a page table.
pub struct Entry(u64);

impl Entry {
    /// Sets the [`Entry`](struct.Entry.html) to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// e.free();
    /// ```
    pub fn free(&mut self) {
        self.0 = 0;
    }

    /// Returns whether an [`Entry`](struct.Entry.html) is free or used.
    ///
    /// # Examples
    ///
    /// ```
    /// let free = e.is_free();
    /// assert_eq!(free, false);
    /// ```
    pub fn is_free(&self) -> bool {
        self.0 == 0
    }

    /// Returns the flags that are set for the [`Entry`](struct.Entry.html).
    ///
    /// # Examples
    ///
    /// ```
    /// e.flags();
    /// ```
    pub fn flags(&self) -> Flags {
        Flags::from_bits_truncate(self.0)
    }

    /// Sets an entry to the given [`Frame`](../../frame/struct.Frame.html)
    pub fn set(&mut self, frame: Frame, flags: Flags) {
        assert!(frame.base() & !0x000f_ffff_ffff_f000 == 0);
        self.0 = (frame.base() as u64) | flags.bits();
    }

    /// Returns the [`Frame`](../../frame/struct.Frame.html) the [`Entry`](struct.Entry.html)
    /// points to.
    pub fn frame(&self) -> Option<Frame> {
        if self.flags().contains(PRESENT) {
            Some(Frame::containing(self.0 as usize & 0x000f_ffff_ffff_f000))
        } else {
            None
        }
    }
}

/// The currently active table.
pub struct ActiveTable {
    /// The `Mapper` the table uses.
    mapper: Mapper,
}

impl Deref for ActiveTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActiveTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActiveTable {
    /// Constructs a new `ActiveTable`.
    ///
    /// # Safety
    ///
    /// See [`Mapper::new()`](../mapper/struct.Mapper.html#method.new).
    pub unsafe fn new() -> ActiveTable {
        ActiveTable { mapper: Mapper::new() }
    }

    /// Execute the given closure of the form `FnOnce(&mut Mapper)` on the given `InactiveTable`.
    ///
    /// # Examples
    ///
    /// ```
    /// ```
    pub fn with<F>(&mut self, table: &mut InactiveTable, page: &mut TempPage, f: F)
    where
        F: FnOnce(&mut Mapper),
    {
        use x86_64::instructions::tlb;
        use x86_64::registers::control_regs;

        {
            let backup = Frame::containing(control_regs::cr3().0 as usize);

            let p4 = page.map_table(backup.clone(), self);

            self.table_mut()[511].set(table.frame.clone(), PRESENT | WRITABLE);
            tlb::flush_all();

            f(self);

            p4[511].set(backup, PRESENT | WRITABLE);
            tlb::flush_all();
        }

        page.unmap(self);
    }

    /// Switches the active table to the given `InactiveTable` and returns the currently active
    /// table as an `InactiveTable`.
    pub fn switch(&mut self, table: InactiveTable) -> InactiveTable {
        use x86_64::PhysicalAddress;
        use x86_64::registers::control_regs;

        let old = InactiveTable { frame: Frame::containing(control_regs::cr3().0 as usize) };

        unsafe {
            control_regs::cr3_write(PhysicalAddress(table.frame.base() as u64));
        }
        old
    }
}

pub struct InactiveTable {
    pub frame: Frame,
}

impl InactiveTable {
    pub fn new(frame: Frame, table: &mut ActiveTable, page: &mut TempPage) -> InactiveTable {
        {
            let table = page.map_table(frame.clone(), table);
            table.reset();

            table[511].set(frame.clone(), PRESENT | WRITABLE);
        }
        page.unmap(table);

        InactiveTable { frame: frame }
    }
}

pub struct TempPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TempPage {
    pub fn new<A>(page: Page, allocator: &mut A) -> TempPage
    where
        A: frame::Allocator,
    {
        TempPage {
            page: page,
            allocator: TinyAllocator::new(allocator),
        }
    }

    pub fn map(&mut self, frame: Frame, table: &mut ActiveTable) -> usize {
        table.map_to(self.page, frame, WRITABLE, &mut self.allocator);
        self.page.base()
    }

    pub fn map_table(&mut self, frame: Frame, table: &mut ActiveTable) -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, table) as *mut Table<Level1>) }
    }

    pub fn unmap(&mut self, table: &mut ActiveTable) {
        table.unmap(self.page, &mut self.allocator);
    }
}

struct TinyAllocator([Option<Frame>; 3]);

impl TinyAllocator {
    fn new<A>(allocator: &mut A) -> TinyAllocator
    where
        A: frame::Allocator,
    {
        let mut f = || allocator.allocate();

        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }
}

impl frame::Allocator for TinyAllocator {
    fn allocate(&mut self) -> Option<Frame> {
        for option in &mut self.0 {
            if option.is_some() {
                return option.take();
            }
        }
        None
    }

    fn deallocate(&mut self, frame: Frame) {
        for option in &mut self.0 {
            if option.is_none() {
                *option = Some(frame);
                return;
            }
        }
        panic!("Allocator can only hold 3 frames.");
    }
}
