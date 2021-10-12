use super::{KERNEL_HEAP, KERNEL_HEAP_SIZE};
use core::cmp::{max, min};
use core::{alloc::Layout, iter::Step};
use memory::{address::Address, page::*};

pub struct FreeListAllocator {
    cells: [Address; KERNEL_HEAP_SIZE.trailing_zeros() as usize + 1],
    retry: bool,
}

impl FreeListAllocator {
    const MIN_SIZE: usize = 1 << 4;

    pub const fn new() -> Self {
        Self {
            cells: [Address::ZERO; KERNEL_HEAP_SIZE.trailing_zeros() as usize + 1],
            retry: false,
        }
    }

    pub fn init(&mut self) {}

    #[inline(always)]
    fn pop_cell(&mut self, size_class: usize) -> Option<Address> {
        let cell = self.cells[size_class];
        if cell.is_zero() {
            None
        } else {
            self.cells[size_class] = unsafe { cell.load() };
            Some(cell)
        }
    }

    #[inline(always)]
    fn push_cell(&mut self, size_class: usize, cell: Address) {
        unsafe {
            cell.store(self.cells[size_class]);
        }
        self.cells[size_class] = cell;
    }

    const fn size_class(block_size: usize) -> usize {
        block_size.next_power_of_two().trailing_zeros() as _
    }

    #[inline(always)]
    fn cell_size(layout: &Layout) -> usize {
        max(layout.pad_to_align().size(), Self::MIN_SIZE)
    }

    fn release_large_pages(&mut self) {
        let start_sc = Self::size_class(Size2M::BYTES);
        let mut sc = start_sc;
        while sc < self.cells.len() {
            while let Some(cell) = self.pop_cell(sc) {
                let pages = 1usize << sc >> Size2M::LOG_BYTES;
                let start = Page::<Size2M>::new(cell);
                KERNEL_HEAP.release_pages(start..Page::forward(start, pages));
            }
            sc += 1;
        }
    }

    fn alloc_cell(&mut self, size_class: usize) -> Option<Address> {
        if size_class >= self.cells.len() {
            None
        } else if let Some(cell) = self.pop_cell(size_class) {
            Some(cell)
        } else {
            let next_level_cell = self.alloc_cell(size_class + 1)?;
            let (cell0, cell1) = (next_level_cell, next_level_cell + (1 << size_class));
            self.push_cell(size_class, cell1);
            Some(cell0)
        }
    }

    #[inline(always)]
    fn alloc_cell_fast(&mut self, size_class: usize) -> Option<Address> {
        if let Some(cell) = self.pop_cell(size_class) {
            Some(cell)
        } else {
            None
        }
    }

    fn alloc_cell_slow(&mut self, size_class: usize) -> Address {
        match self.alloc_cell(size_class) {
            Some(cell) => cell,
            None => {
                assert!(!self.retry, "OutOfMemory");
                let pages = (((1 << size_class) + Size2M::MASK) >> Size2M::LOG_BYTES) << 1;
                let vs = KERNEL_HEAP.allocate_pages::<Size2M>(pages);
                let mut cursor = vs.start.start();
                let end = vs.end.start();
                while cursor < end {
                    let align = cursor.as_usize().trailing_zeros();
                    let size = min(1 << align, end - cursor);
                    assert!(size > 0);
                    let size_class = if size.is_power_of_two() {
                        Self::size_class(size)
                    } else {
                        Self::size_class(size) - 1
                    };
                    assert!(cursor.as_usize() & ((1 << size_class) - 1) == 0);
                    self.push_cell(size_class, cursor);
                    cursor += size;
                }
                self.retry = true;
                let x = self.alloc_cell_slow(size_class);
                self.retry = false;
                x
            }
        }
    }

    #[inline(always)]
    pub fn alloc(&mut self, layout: &Layout) -> Address {
        let cell_size = Self::cell_size(&layout);
        let size_class = Self::size_class(cell_size);
        if let Some(cell) = self.alloc_cell_fast(size_class) {
            return cell;
        }
        self.alloc_cell_slow(size_class)
    }

    #[inline(always)]
    pub fn free(&mut self, start: Address, layout: &Layout) {
        let cell_size = Self::cell_size(&layout);
        let size_class = Self::size_class(cell_size);
        self.push_cell(size_class, start);
        self.release_large_pages();
    }
}
