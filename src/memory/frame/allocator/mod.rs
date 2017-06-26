use super::Frame;

mod bitmap;

pub trait Allocator {
    fn allocate(&mut self) -> Option<Frame>;
    fn deallocate(&mut self, frame: Frame);
}
