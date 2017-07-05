use x86_64::PrivilegeLevel;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::tss::TaskStateSegment;

pub struct GlobalDescriptorTable {
    table: [u64; 8],
    next: usize,
}

impl GlobalDescriptorTable {
    pub fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            next: 1,
        }
    }

    pub fn load(&'static self) {
        use x86_64::instructions::tables::{DescriptorTablePointer, lgdt};

        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.table.len() * 8 - 1) as u16,
        };

        unsafe { lgdt(&ptr) };
    }

    pub fn add(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(low, high) => {
                let index = self.push(low);
                self.push(high);
                index
            }
        };

        SegmentSelector::new(index as u16, PrivilegeLevel::Ring0)
    }

    fn push(&mut self, value: u64) -> usize {
        if self.next < self.table.len() {
            self.table[self.next] = value;
            self.next += 1;
            self.next - 1

        } else {
            panic!("GDT full");
        }
    }
}

pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    pub fn code() -> Descriptor {
        let flags = USER | PRESENT | EXECUTABLE | LONG;
        Descriptor::UserSegment(flags.bits())
    }

    pub fn tss(tss: &'static TaskStateSegment) -> Descriptor {
        use core::mem::size_of;

        let ptr = tss as *const _ as u64;

        let mut low = PRESENT.bits();
        low = low | ((ptr & 0xFFFFFF) << 16);
        low = low | ((ptr & 0xFF000000) << 56);
        low = low | ((size_of::<TaskStateSegment>() - 1) as u64);
        low = low | (0x9 << 40);

        let mut high = 0;
        high = high | ((ptr & 0xFFFFFFFF00000000) >> 32);

        Descriptor::SystemSegment(low, high)
    }
}

bitflags! {
    flags DescriptorFlags: u64 {
        const CONFORMING    = 1 << 42,
        const EXECUTABLE    = 1 << 43,
        const USER          = 1 << 44,
        const PRESENT       = 1 << 47,
        const LONG          = 1 << 53,
    }
}
