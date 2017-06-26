use memory::Frame;

pub struct BuddyAllocator {}

impl BuddyAllocator {
    fn allocate(&mut self) -> Option<Frame> {}

    fn deallocate(&mut self, frame: Frame) {
        unimplemented!()
    }
}
