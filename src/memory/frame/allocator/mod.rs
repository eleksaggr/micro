use super::Frame;

pub mod area;
pub mod bitmap;

pub trait Allocator {
    fn allocate(&mut self) -> Option<Frame>;
    fn deallocate(&mut self, frame: Frame);
}
