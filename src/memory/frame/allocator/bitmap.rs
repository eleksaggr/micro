use core::mem;
use memory::frame::{Allocator, Frame};

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
        for _ in 1..n {
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
        let mut bitmap = Bitmap::containing(self.base, frame.base());
        bitmap.unset(Bitmap::offset(frame.base()));
    }
}
