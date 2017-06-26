#![feature(const_fn)]
#![feature(associated_consts)]
#![feature(allocator)]

#![allocator]
#![no_std]

extern crate spin;
#[macro_use]
extern crate lazy_static;

use spin::Mutex;

pub const BASE: usize = 0x40000000;
pub const SIZE: usize = Block::SIZE * (1 << BuddyAllocator::ORDER);

lazy_static! {
    static ref ALLOCATOR: Mutex<BuddyAllocator> = Mutex::new(BuddyAllocator::new());
}

pub struct BuddyAllocator {
    blocks: [Block; SIZE / Block::SIZE],
}

impl BuddyAllocator {
    const ORDER: u8 = 9;

    pub fn new() -> Self {
        BuddyAllocator {
            blocks: [Block {
                order: BuddyAllocator::ORDER,
                used: false,
            }; SIZE / Block::SIZE],
        }
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        // Check if we can even fit the request on the heap.
        assert!(size <= SIZE, "Request exceeds maximum allocation unit.");
        assert!(align & (align - 1) == 0, "Alignment is off boundary.");

        // Calculate the order of blocks we need and try to find a fit.
        let order = BuddyAllocator::order(size);
        let index = self.fit(order).unwrap_or_else(|| {
            // Search for a higher order block and split it if necessary.
            for i in order..(BuddyAllocator::ORDER + 1) {
                let opt = self.fit(i);
                match opt {
                    Some(index) => {
                        for _ in 0..(i - order) {
                            self.split(index);
                        }
                        return index;
                    }
                    None => continue,
                }
            }
            //TODO: This should not panic, but rather return an Err.
            panic!("Could not fit request into current memory scheme.");
        });

        // Mark all of the now reserved blocks as used.
        self.set(index, order, true);

        // Return a pointer to the first of the reversed blocks.
        unsafe { (BASE as *mut u8).offset((index * Block::SIZE) as isize) }
    }

    pub fn deallocate(&mut self, ptr: *mut u8, size: usize, align: usize) {
        assert!(
            (ptr as usize) < BASE + SIZE && (ptr as usize) >= BASE,
            "Could not deallocate pointer outside of the heap."
        );
        let index = ((ptr as usize) - BASE) / Block::SIZE;
        let order = self.blocks[index].order;

        // Mark the blocks as now unused.
        self.set(index, order, false);

        if self.is_left(index, order) {
            // Check if the right block is used.
            if !self.blocks[index + (1 << order)].used {
                self.merge(index);
            }
        } else {
            // Check if the left block is used.
            if !self.blocks[index - (1 << order)].used {
                self.merge(index - (1 << order));
            }
        }
    }

    pub fn zero(ptr: *mut u8, size: usize) {
        let mut p = ptr;
        for _ in 0..size {
            unsafe {
                *p = 0;
                p = ptr.offset(1);
            }
        }
    }

    fn order(size: usize) -> u8 {
        let mut i = 0;
        while size > (Block::SIZE * (1 << i)) {
            i += 1;
        }
        i
    }

    fn fit(&self, order: u8) -> Option<usize> {
        assert!(
            order <= BuddyAllocator::ORDER,
            "Order exceeds the maximum order of the allocator."
        );
        for (i, block) in self.blocks.iter().enumerate() {
            if order == block.order && !block.used {
                return Some(i);
            }
        }
        None
    }

    fn split(&mut self, index: usize) {
        let order = self.blocks[index].order;
        assert!(
            index % (1 << order) == 0,
            "Index is not properly aligned for its order."
        );

        for i in 0..(1 << order) {
            assert!(
                !self.blocks[index + i].used,
                "Unable to split a block currently in use."
            );
            self.blocks[index + i].order -= 1;
        }
    }

    fn merge(&mut self, index: usize) {
        let order = self.blocks[index].order;
        assert!(
            index % (1 << order) == 0,
            "Index is not properly aligned for its order."
        );

        for i in 0..(1 << (order + 1)) {
            assert!(
                !self.blocks[index + i].used,
                "Unable to merge two blocks currently in use."
            );
            self.blocks[index + i].order += 1;
        }

        if order + 1 != BuddyAllocator::ORDER {
            // We are left per definition.
            if !self.blocks[index + (1 << (order + 1))].used {
                self.merge(index);
            }

        }
    }

    fn is_left(&self, index: usize, order: u8) -> bool {
        assert!(
            order <= BuddyAllocator::ORDER,
            "Order exceeds the maximum order of the allocator."
        );

        assert!(
            index % (1 << order) == 0,
            "Index is not properly aligned for its order."
        );

        index % (2 * (1 << order)) == 0
    }

    fn set(&mut self, index: usize, order: u8, used: bool) {
        assert!(
            order <= BuddyAllocator::ORDER,
            "Order exceeds the maximum order of the allocator."
        );

        for i in 0..(1 << order) {
            self.blocks[index + i].used = used;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    order: u8,
    used: bool,
}

impl Block {
    pub const SIZE: usize = 4000; // 4KiB
}

#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    ALLOCATOR.lock().allocate(size, align)
}

#[no_mangle]
pub extern "C" fn __rust_allocate_zeroed(size: usize, align: usize) -> *mut u8 {
    let ptr = ALLOCATOR.lock().allocate(size, align);
    BuddyAllocator::zero(ptr, size);
    ptr
}

#[no_mangle]
pub extern "C" fn __rust_reallocate(
    ptr: *mut u8,
    old_size: usize,
    size: usize,
    align: usize,
) -> *mut u8 {
    let new_ptr = ALLOCATOR.lock().allocate(size, align);
    ALLOCATOR.lock().deallocate(ptr, old_size, align);
    new_ptr
}

#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(
    ptr: *mut u8,
    old_size: usize,
    size: usize,
    align: usize,
) -> usize {
    size
}

#[no_mangle]
pub extern "C" fn __rust_deallocate(ptr: *mut u8, size: usize, align: usize) {
    ALLOCATOR.lock().deallocate(ptr, size, align);
}

#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, align: usize) -> usize {
    size
}
