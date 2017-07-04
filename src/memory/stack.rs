use memory::frame;
use memory::paging::{ActiveTable, Page, PageIter, WRITABLE};

pub struct Stack {
    top: usize,
    bottom: usize,
}

impl Stack {
    pub fn new(top: usize, bottom: usize) -> Stack {
        assert!(top > bottom,
                "Impossible to create stack with top lower than bottom.");
        Stack {
            top: top,
            bottom: bottom,
        }
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn bottom(&self) -> usize {
        self.bottom
    }
}

pub struct StackAllocator {
    range: PageIter,
}

impl StackAllocator {
    pub fn new(range: PageIter) -> StackAllocator {
        StackAllocator { range: range }
    }

    pub fn allocate<A>(&mut self,
                       active_table: &mut ActiveTable,
                       allocator: &mut A,
                       pages: usize)
                       -> Option<Stack>
        where A: frame::Allocator
    {
        if pages == 0 {
            return None;
        }

        let mut range = self.range.clone();

        let guard = range.next();
        let start = range.next();
        let end = if pages == 1 {
            start
        } else {
            range.nth(pages - 2)
        };

        match (guard, start, end) {
            (Some(_), Some(start), Some(end)) => {
                self.range = range;

                for page in Page::range(start, end) {
                    active_table.map(page, WRITABLE, allocator);
                }

                let top = end.base_addr() + Page::SIZE;
                Some(Stack::new(top, start.base_addr()))
            }
            _ => None,
        }
    }
}
