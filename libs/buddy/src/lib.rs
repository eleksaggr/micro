#![feature(const_fn)]
#![feature(allocator)]

#![allocator]
#![no_std]

extern crate spin;

use spin::Mutex;

// #[no_mangle]
// pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {}

// #[no_mangle]
// pub extern "C" fn __rust_allocate_zeroed(size: usize, align: usize) -> *mut u8 {}

// #[no_mangle]
// pub extern "C" fn __rust_reallocate(
//     ptr: *mut u8,
//     old_size: usize,
//     size: usize,
//     align: usize,
// ) -> *mut u8 {
// }

// #[no_mangle]
// pub extern "C" fn __rust_reallocate_inplace(
//     ptr: *mut u8,
//     old_size: usize,
//     size: usize,
//     align: usize,
// ) -> usize {
// }

// #[no_mangle]
// pub extern "C" fn __rust_deallocate(ptr: *mut u8, size: usize, align: usize) {}

// #[no_mangle]
// pub extern "C" fn __rust_usable_size(size: usize, align: usize) -> usize {}

pub const BASE: usize = 0x40000000;
pub const SIZE: usize = 512 * 4000; // 4 * 2^7 KiB

const BLOCK_SIZE: usize = 4000; // KiB
const MAX_ORDER: u8 = 7;

static ALLOCATOR: Mutex<BuddyAllocator> = Mutex::new(BuddyAllocator::new());

pub struct BuddyAllocator {
    entries: [Entry; SIZE / BLOCK_SIZE],
}

impl BuddyAllocator {
    pub const fn new() -> BuddyAllocator {
        BuddyAllocator {
            entries: [Entry {
                order: MAX_ORDER,
                left: false,
                used: false,
            }; SIZE / BLOCK_SIZE],
        }
    }

    pub fn allocate(&mut self, size: usize) -> *mut u32 {
        assert!(
            size <= SIZE,
            "Size exceeds maximum allocation unit of {}",
            SIZE
        );

        let order = BuddyAllocator::order(size);
        let next = self.next(order, 0).unwrap_or_else(|| {
            for i in order..(MAX_ORDER + 1) {
                let next = self.next(i, 0);
                match next {
                    Some(index) => {
                        for _ in 0..(i - order) {
                            self.split(index);
                        }
                        return index;
                    }
                    None => continue,
                }
            }
            panic!("Could not create a block for the allocation");
        });

        for i in 0..(1 << order) {
            self.entries[next + i].used = true;
        }

        unsafe {
            let base = BASE as *mut u32;
            base.offset((next * BLOCK_SIZE) as isize)
        }
    }

    pub fn deallocate(&mut self, ptr: *mut u32) {
        assert!(
            (ptr as usize) >= BASE && (ptr as usize) <= BASE + SIZE,
            "Pointer with address {:#x} does not point to the Heap",
            ptr as usize
        );

        let index = ((ptr as usize) - BASE) / BLOCK_SIZE;
        let order = self.entries[index].order;
        let left = self.entries[index].left;

        for i in 0..(1 << order) {
            self.entries[index + i].used = false;
        }

        if left {
            if !self.entries[index + (1 << (order + 1))].used {
                self.merge(index);
            }
        } else {
            if !self.entries[index - (1 << (order + 1))].used {
                self.merge(index - (1 << (order + 1)));
            }
        }
    }

    fn split(&mut self, pos: usize) {
        let order = self.entries[pos].order;

        for i in 0..(1 << order) {
            if i < (1 << order) / 2 || ((order == 0) && i == 0) {
                self.entries[pos + i].left = true;
            } else {
                self.entries[pos + i].left = false;
            }
            assert!(!self.entries[pos + i].used);
            self.entries[pos + i].order -= 1;
        }
    }

    fn merge(&mut self, pos: usize) {
        let order = self.entries[pos].order;

        for i in 0..(1 << (order + 1)) {
            assert!(!self.entries[pos + i].used);
            self.entries[pos + i].order += 1;
            self.entries[pos + i].left = false;
        }
    }

    fn next(&self, order: u8, pos: usize) -> Option<usize> {
        for (i, entry) in self.entries.iter().enumerate().skip(pos) {
            if entry.order == order && !entry.used {
                return Some(i);
            }
        }
        None
    }

    fn order(size: usize) -> u8 {
        let mut i = 0;
        while size > (BLOCK_SIZE * (1 << i)) {
            i += 1;
        }
        i
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    order: u8,
    left: bool,
    used: bool,
}

#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, _: usize) -> *mut u32 {
    ALLOCATOR.lock().allocate(size)
}

#[no_mangle]
pub extern "C" fn __rust_allocate_zeroed(size: usize, _: usize) -> *mut u32 {
    //TODO: This lies to the user.
    ALLOCATOR.lock().allocate(size)
}

#[no_mangle]
pub extern "C" fn __rust_deallocate(ptr: *mut u32, size: usize, _: usize) {
    ALLOCATOR.lock().deallocate(ptr);
}

#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _: usize) -> usize {
    size
}

#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(
    ptr: *mut u32,
    size: usize,
    new: usize,
    align: usize,
) -> usize {
    size
}

#[no_mangle]
pub extern "C" fn __rust_reallocate(ptr: *mut u32, size: usize, new: usize, _: usize) -> *mut u32 {
    use core::{cmp, ptr};

    let new_ptr = __rust_allocate(new, 0);
    unsafe { ptr::copy(ptr, new_ptr, cmp::min(size, new)) };
    __rust_deallocate(ptr, size, 0);
    new_ptr
}
