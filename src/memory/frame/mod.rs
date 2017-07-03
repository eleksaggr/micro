pub use self::allocator::Allocator;
pub use self::allocator::area::AreaAllocator;
pub use self::allocator::bitmap::BitmapAllocator;

use core::iter::Iterator;

mod allocator;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    id: usize,
}

impl Frame {
    pub const SIZE: usize = 4096; //4KB

    /// Returns the Frame containing the address given.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address the Frame shall contain.
    pub fn containing(addr: usize) -> Frame {
        Frame { id: addr / Frame::SIZE }
    }

    /// Returns the first address of the memory space the Frame represents.
    pub fn base(&self) -> usize {
        self.id * Frame::SIZE
    }

    pub fn clone(&self) -> Frame {
        Frame { id: self.id }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn range(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }
}

pub struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.id += 1;
            Some(frame)
        } else {
            None
        }
    }
}
