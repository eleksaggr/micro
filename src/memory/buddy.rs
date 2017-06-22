use memory::Frame;

pub struct BuddyAllocator {

}

impl BuddyAllocator {

    fn allocate(&mut self, size: u64) -> Option<Frame> {

    }

    fn deallocate(&mut self, frame: Frame) {
        unimplemented!()
    }
}
