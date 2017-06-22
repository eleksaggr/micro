use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use memory::paging::entry::*;
use memory::paging::Page;

pub trait Level{}

pub struct Table<L: Level> {
    entries: [Entry; Page::ENTRIES],
    level: PhantomData<L>,
}

impl<L> Table<L> where L: Level {
    pub const P4: *mut Table<Level4> = 0xffff_ffff_ffff_f000 as *mut _;

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_free();
        }
    }
}

impl<L> Table<L> where L: HierarchicalLevel {
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let flags = self[index].flags();
        if flags.contains(PRESENT) && !flags.contains(HUGE) {
            let address = self as *const _ as usize;
            Some((address << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index).map(|address| unsafe {
            &*(address as *const _)
        })
    }

    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index).map(|address| unsafe {
            &mut *(address as *mut _)
        })
    }

}

impl<L> Index<usize> for Table<L> where L: HierarchicalLevel {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L> where L: HierarchicalLevel {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

pub trait HierarchicalLevel: Level {
    type NextLevel: Level;
}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}
impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}
impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

impl Level for Level4 {}
impl Level for Level3 {}
impl Level for Level2 {}
impl Level for Level1 {}


