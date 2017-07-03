use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::PrivilegeLevel;

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
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
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
            let index = self.next;
            self.table[index] = value;
            self.next += 1;
            index
        } else {
            panic!("Global descriptor table is full.");
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
        use bit_field::BitField;

        let ptr = tss as *const _ as u64;

        let mut low = PRESENT.bits();

        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

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
