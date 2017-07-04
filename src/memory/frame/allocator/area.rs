use memory::frame::{Allocator, Frame};
use multiboot2::{MemoryArea, MemoryAreaIter};

pub struct AreaAllocator {
    area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    kernel: (Frame, Frame),
    multiboot: (Frame, Frame),
    next: Frame,
}

impl AreaAllocator {
    pub fn new(
        kernel: (usize, usize),
        multiboot: (usize, usize),
        areas: MemoryAreaIter,
    ) -> AreaAllocator {
        let mut allocator = AreaAllocator {
            next: Frame { id: Frame::containing(kernel.1).id() + 1 },
            area: None,
            areas: areas,
            kernel: (Frame::containing(kernel.0), Frame::containing(kernel.1)),
            multiboot: (
                Frame::containing(multiboot.0),
                Frame::containing(multiboot.1),
            ),
        };
        allocator.next();
        allocator
    }

    fn next(&mut self) {
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

impl Allocator for AreaAllocator {
    fn allocate(&mut self) -> Option<Frame> {
        if let Some(area) = self.area {
            let frame = self.next.clone();

            let last = {
                let address = area.base_addr + area.length - 1;
                Frame::containing(address as usize)
            };

            if frame > last {
                self.next();
            } else if frame >= self.kernel.0 && frame <= self.kernel.1 {
                self.next = Frame { id: self.kernel.1.id + 1 };
            } else if frame >= self.multiboot.0 && frame <= self.multiboot.1 {
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

    fn deallocate(&mut self, _: Frame) {
        unimplemented!()
    }
}
