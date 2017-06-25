#![feature(allocator)]
#![feature(const_fn)]

#![allocator]
#![no_std]

extern crate spin;

use spin::Mutex;

pub const START: usize = 0x40000000;
pub const SIZE: usize = 100 * 1024; // 100 KiB

static ALLOCATOR: Mutex<BumpAllocator> = Mutex::new(BumpAllocator::new(START, SIZE));

#[derive(Debug)]
struct BumpAllocator {
    start: usize,
    size: usize,
    next: usize,
}

impl BumpAllocator {
    const fn new(start: usize, size: usize) -> BumpAllocator {
        BumpAllocator {
            start: start,
            size: size,
            next: start,
        }
    }

    fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let start = align_up(self.next, align);
        let end = start.saturating_add(size);

        if end <= self.start + self.size {
            self.next = end;
            Some(start as *mut u8)
        } else {
            None
        }
    }
}

pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("'align' must be power of 2");
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    ALLOCATOR
        .lock()
        .allocate(size, align)
        .expect("Out of memory")
}

#[no_mangle]
pub extern "C" fn __rust_deallocate(_ptr: *mut u8, _size: usize, _align: usize) {}

#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}

#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(
    _ptr: *mut u8,
    size: usize,
    _new_size: usize,
    _align: usize,
) -> usize {
    size
}

#[no_mangle]
pub extern "C" fn __rust_reallocate(
    ptr: *mut u8,
    size: usize,
    new_size: usize,
    align: usize,
) -> *mut u8 {
    use core::{ptr, cmp};

    let new_ptr = __rust_allocate(new_size, align);
    unsafe { ptr::copy(ptr, new_ptr, cmp::min(size, new_size)) };
    __rust_deallocate(ptr, size, align);
    new_ptr
}
