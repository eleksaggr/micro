use memory::frame::{Allocator, Frame};
use multiboot2::{MemoryArea, MemoryAreaIter};

pub struct BitmapAllocator {
    amount: usize,
}

impl BitmapAllocator {
    pub const BASE: usize = 0x2000000;
    pub fn new(size: usize, areas: MemoryAreaIter) -> BitmapAllocator {
        // Figure out how many bitmaps we need to store, to address the whole memory.
        let n = size / (Frame::SIZE * 64);
        for i in 0..(n + 1) {
            // Mark all frames in the i-th bitmap as unused.
            unsafe {
                *((BitmapAllocator::BASE + i * 8) as *mut u64) = 0x0;
            }
        }

        // Set the bitmaps according to which frames are already used by the kernel and the
        // multiboot header.
        for area in areas {
            BitmapAllocator::mark(area.base_addr as usize, area.length as usize);
        }

        // Now mark the memory used by the bitmaps as used.
        BitmapAllocator::mark(BitmapAllocator::BASE, n * 8);

        BitmapAllocator { amount: n * 8 }
    }

    fn mark(addr: usize, size: usize) {
        // Figure out how many frames the area needs.
        let range = size / Frame::SIZE;
        for i in 0..range {
            let p = addr + i * Frame::SIZE;
            let index = BitmapAllocator::containing(p);
            // Find the offset of the frame in the bitmap.
            let offset = (p % (Frame::SIZE * 64)) / Frame::SIZE;
            // If this assert fails, the above offset code is probably broken.
            assert!((p % (Frame::SIZE * 64)) % Frame::SIZE == 0,
                    "Address was not aligned to frame.");

            BitmapAllocator::set(index, offset);
        }
    }

    fn set(index: usize, offset: usize) {
        let bitmap = (BitmapAllocator::BASE + index * 8) as *mut u64;

        unsafe {
            *bitmap = *bitmap | (1 << offset);
        }
    }

    fn unset(index: usize, offset: usize) {
        let bitmap = (BitmapAllocator::BASE + index * 8) as *mut u64;

        unsafe {
            *bitmap = *bitmap & !(1 << offset);
        }
    }

    fn containing(addr: usize) -> usize {
        addr / (Frame::SIZE * 64)
    }

    fn next(&self) -> Option<Frame> {
        for i in 0..self.amount {
            unsafe {
                let bitmap = *((BitmapAllocator::BASE + i * 8) as *mut u64);
                for j in 0..64 {
                    if bitmap & (1 << j) == 0 {
                        return Some(Frame { id: i * 64 + j });
                    }
                }
            }
        }
        None
    }
}

impl Allocator for BitmapAllocator {
    fn allocate(&mut self) -> Option<Frame> {
        self.next()
    }

    fn deallocate(&mut self, frame: Frame) {
        unimplemented!()
    }
}
