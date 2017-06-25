use memory::{Frame, FrameAllocator};
use multiboot2::{MemoryArea, MemoryAreaIter};

pub struct AreaFrameAllocator {
    area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    next: Frame,
    kernel: (Frame, Frame),
    multiboot: (Frame, Frame),
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
            } else if frame <= self.kernel.1 && frame >= self.kernel.0 {
                self.next = Frame { id: self.kernel.1.id + 1 };
            } else if frame <= self.multiboot.1 && frame >= self.multiboot.0 {
                self.next = Frame { id: self.multiboot.1.id + 1 };
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
        kernel_start: usize,
        kernel_end: usize,
        mb_start: usize,
        mb_end: usize,
    ) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next: Frame::containing(0),
            area: None,
            areas: areas,
            kernel: (
                Frame::containing(kernel_start),
                Frame::containing(kernel_end),
            ),
            multiboot: (Frame::containing(mb_start), Frame::containing(mb_end)),
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
