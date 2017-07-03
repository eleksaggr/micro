use core::mem;
use memory::frame::{Allocator, Frame};
use multiboot2::{MemoryArea, MemoryAreaIter};

pub struct BitmapAllocator {
    amount: usize,
    base: usize,
    last: usize,
}

impl BitmapAllocator {
    pub fn new<A>(total_size: usize, allocator: &mut A) -> BitmapAllocator
    where
        A: Allocator,
    {
        // First determine the amount of bitmaps needed, to address the whole memory.
        // Each Bitmap can hold mem::size_of::<usize> * 8 frames, since a frame is simply
        // represented by a bit.
        let amount = total_size / (Frame::SIZE * mem::size_of::<usize>() * 8);

        // Allocate a frame to save the bitmaps in memory.
        let n = (amount * mem::size_of::<usize>()) / Frame::SIZE + 1;
        let frame = allocator
            .allocate()
            .expect("Could not allocate frame for bitmaps.");
        for i in 1..n {
            allocator
                .allocate()
                .expect("Could not allocate frame for bitmaps.");
        }

        // Now zero the memory space the bitmaps will occupy.
        for i in 0..amount {
            Bitmap::from(frame.base(), i).zero();
        }

        let allocator = BitmapAllocator {
            amount: amount,
            base: frame.base(),
            last: 0,
        };

        allocator.mark(allocator.base, amount * mem::size_of::<usize>());

        // Mark the whole lower part of memory as used, so we won't write into something important.
        allocator.mark(0x0, 0x130000);

        allocator
    }

    fn mark_area(&self, area: &MemoryArea) {
        self.mark(area.base_addr as usize, area.length as usize);
    }

    fn mark(&self, addr: usize, length: usize) {
        let n = length / Frame::SIZE + 1;
        for i in 0..n {
            let p = addr + i * Frame::SIZE;
            Bitmap::containing(self.base, p).set(Bitmap::offset(p));
        }
    }

    pub fn used(&self) -> (usize, usize) {
        (
            self.base,
            (self.amount * mem::size_of::<usize>()) / Frame::SIZE + 1,
        )
    }
}

struct Bitmap {
    ptr: *mut usize,
}

impl Bitmap {
    fn from(base: usize, index: usize) -> Bitmap {
        Bitmap { ptr: (base + index * mem::size_of::<usize>()) as *mut usize }
    }

    fn containing(base: usize, addr: usize) -> Bitmap {
        let frame = Frame::containing(addr);
        let index = frame.id() / (mem::size_of::<usize>() * 8);
        Bitmap::from(base, index)
    }

    fn offset(addr: usize) -> usize {
        let frame = Frame::containing(addr);
        frame.id() % (mem::size_of::<usize>() * 8)
    }

    fn zero(&mut self) {
        unsafe {
            *self.ptr = 0x0;
        }
    }

    fn set(&mut self, offset: usize) {
        unsafe {
            *self.ptr = *self.ptr | (1 << offset);
        }
    }

    fn unset(&mut self, offset: usize) {
        unsafe {
            *self.ptr = *self.ptr & !(1 << offset);
        }
    }

    fn next(&mut self) -> Option<usize> {
        for i in 0..(mem::size_of::<usize>() * 8) {
            unsafe {
                if *self.ptr & (1 << i) == 0 {
                    self.set(i);
                    return Some(i);
                }
            }
        }
        None
    }
}

impl Allocator for BitmapAllocator {
    fn allocate(&mut self) -> Option<Frame> {
        for i in self.last..self.amount {
            if i > self.last {
                self.last = i;
            }
            let offset = Bitmap::from(self.base, i).next();
            if offset.is_some() {
                let frame = Frame { id: i * mem::size_of::<usize>() * 8 + offset.unwrap() };
                return Some(frame);
            }
        }
        None
    }

    fn deallocate(&mut self, frame: Frame) {
        unimplemented!()
    }
}

// pub struct BitmapAllocator {
//     amount: usize,
//     length: usize,
// }

// impl BitmapAllocator {
//     pub const BASE: usize = 0x2000000;

//     pub fn new(size: usize, areas: MemoryAreaIter) -> BitmapAllocator {
//         let length = mem::size_of::<usize>();

//         // Figure out how many bitmaps we need to store, to address the whole memory.
//         let n = size / (Frame::SIZE * (8 * length));
//         for i in 0..(n + 1) {
//             // Mark all frames in the i-th bitmap as unused.
//             unsafe {
//                 *((BitmapAllocator::BASE + i * 8) as *mut usize) = 0x0;
//             }
//         }

//         let allocator = BitmapAllocator {
//             amount: n * length,
//             length: length,
//         };

//         // Set the bitmaps according to which frames are already used by the kernel and the
//         // multiboot header.
//         for area in areas {
//             allocator.mark(area.base_addr as usize, area.length as usize);
//         }

//         // Now mark the memory used by the bitmaps as used.
//         allocator.mark(BitmapAllocator::BASE, n * length);

//         allocator

//     }

//     fn mark(&self, addr: usize, size: usize) {
//         // Figure out how many frames the area needs.
//         let range = size / Frame::SIZE;
//         for i in 0..range {
//             let p = addr + i * Frame::SIZE;
//             let index = self.containing(p);
//             // Find the offset of the frame in the bitmap.
//             let offset = (p % (Frame::SIZE * (8 * self.length))) / Frame::SIZE;
//             // If this assert fails, the above offset code is probably broken.
//             assert!(
//                 (p % (Frame::SIZE * (8 * self.length))) % Frame::SIZE == 0,
//                 "Address was not aligned to frame."
//             );

//             self.set(index, offset);
//         }
//     }

//     fn set(&self, index: usize, offset: usize) {
//         let bitmap = (BitmapAllocator::BASE + index * self.length) as *mut usize;

//         unsafe {
//             *bitmap = *bitmap | (1 << offset);
//         }
//     }

//     fn unset(&self, index: usize, offset: usize) {
//         let bitmap = (BitmapAllocator::BASE + index * self.length) as *mut usize;

//         unsafe {
//             *bitmap = *bitmap & !(1 << offset);
//         }
//     }

//     fn containing(&self, addr: usize) -> usize {
//         addr / (Frame::SIZE * (8 * self.length))
//     }

//     fn next(&self) -> Option<Frame> {
//         for i in 0..self.amount {
//             unsafe {
//                 let bitmap = *((BitmapAllocator::BASE + i * self.length) as *const usize);
//                 for j in 0..(8 * self.length) {
//                     if bitmap & (1 << j) == 0 {
//                         return Some(Frame { id: i * (8 * self.length) + j });
//                     }
//                 }
//             }
//         }
//         None
//     }
// }

// impl Allocator for BitmapAllocator {
//     fn allocate(&mut self) -> Option<Frame> {
//         self.next()
//     }

//     fn deallocate(&mut self, frame: Frame) {
//         unimplemented!()
//     }
// }
