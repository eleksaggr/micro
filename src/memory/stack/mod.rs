use memory::frame;
use memory::paging::{self, ActiveTable, Page, PageIter};

#[derive(Debug)]
pub struct Stack {
    top: usize,
    bottom: usize,
}

impl Stack {
    fn new(top: usize, bottom: usize) -> Stack {
        assert!(top > bottom);
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
                       table: &mut ActiveTable,
                       allocator: &mut A,
                       size: usize)
                       -> Option<Stack>
        where A: frame::Allocator
    {
        if size == 0 {
            return None;
        }

        let mut range = self.range.clone();

        let guard = range.next();
        let start = range.next();
        let end = if size == 1 {
            start
        } else {
            range.nth(size - 2)
        };

        match (guard, start, end) {
            (Some(_), Some(start), Some(end)) => {
                self.range = range;

                for page in Page::range(start, end) {
                    table.map(page, paging::WRITABLE, allocator);
                }

                let top = end.base_addr() + Page::SIZE;
                Some(Stack::new(top, start.base_addr()))
            }
            _ => None,
        }
    }
}
