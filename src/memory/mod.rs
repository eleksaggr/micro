mod area;
mod paging;

pub use memory::area::AreaFrameAllocator;

pub trait FrameAllocator {
    fn allocate(&mut self, size: u64) -> Option<Frame>;
    fn deallocate(&mut self, frame: Frame);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    id: usize,
}

impl Frame {
    const SIZE: usize = 4096;

    fn containing(address: usize) -> Self {
        Frame { id: address / Frame::SIZE}
    }

    fn base_addr(&self) -> usize {
        self.id * Frame::SIZE
    }
}
