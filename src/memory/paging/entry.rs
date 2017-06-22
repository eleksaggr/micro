use memory::Frame;

bitflags! {
    pub flags EntryFlags: u64 {
        const PRESENT = 1 << 0,
        const WRITABLE = 1 << 1,
        const USER_ACCESSIBLE = 1 << 2,
        const WRITE_THROUGH = 1 << 3,
        const NO_CACHE = 1 << 4,
        const ACCESSED = 1 << 5,
        const DIRTY = 1 << 6,
        const HUGE = 1 << 7,
        const GLOBAL = 1 << 8,
        const NO_EXECUTE = 1 << 63,
    }
}

pub struct Entry(u64);

impl Entry {

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn is_free(&self) -> bool {
        self.0 == 0
    } 

    pub fn set_free(&mut self) {
        self.0 = 0;
    }

    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        assert!(frame.base_addr() & !0x000ffffffffff000  == 0);
        self.0 = (frame.base_addr() as u64) | flags.bits();
    }

    pub fn get_frame(&self) -> Option<Frame> {
        if self.flags().contains(PRESENT) {
            Some(Frame::containing(self.0 as usize & 0x000ffffffffff000))
        } else {
            None
        }
    }
}

