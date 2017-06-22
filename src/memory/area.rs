use memory::{Frame, FrameAllocator};
use multiboot2::{MemoryArea, MemoryAreaIter};

pub struct AreaFrameAllocator {
    area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    next: Frame,
    reserved: (Frame, Frame),
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate(&mut self) -> Option<Frame> {
        if let Some(area) = self.area {
            let frame = Frame { id: self.next.id };

            let last = {
                let address = area.base_addr + area.length - 1;
                Frame::containing(address as usize)
            };


            if frame > last {
                self.next_area();
            } else if frame <= self.reserved.1 && frame >= self.reserved.0 {
                self.next = Frame { id: self.reserved.1.id + 1 };
            } else {
                self.next.id += 1;
                return Some(frame);
            }
            self.allocate()
        } else {
            None
        }
    }

    fn deallocate(&mut self, frame: Frame) {
        unimplemented!()
    }
}

impl AreaFrameAllocator {
    pub fn new(
        areas: MemoryAreaIter,
        reserved_min: usize,
        reserved_max: usize,
    ) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next: Frame::containing(0),
            area: None,
            areas: areas,
            reserved: (Frame { id: reserved_min }, Frame { id: reserved_max }),
        };
        allocator.next_area();
        allocator
    }

    fn next_area(&mut self) {
        self.area = self.areas
            .clone()
            .filter(|area| {
                let address = area.base_addr + area.length - 1;
                Frame::containing(address as usize) >= self.next
            })
            .min_by_key(|area| area.base_addr);

        if let Some(area) = self.area {
            let start = Frame::containing(area.base_addr as usize);
            if self.next < start {
                self.next = start;
            }
        }
    }
}
