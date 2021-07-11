use super::*;
use crate::memory::physical::{PhysicalPageResource, PHYSICAL_PAGE_RESOURCE};
use crate::utils::address::*;
use crate::utils::page::*;
use core::fmt::Debug;
use core::intrinsics::transmute;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use cortex_a::regs::*;

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct PageTable<L: TableLevel = L4> {
    entries: [PageTableEntry; 512],
    phantom: PhantomData<L>,
}

impl<L: TableLevel> const Index<usize> for PageTable<L> {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L: TableLevel> const IndexMut<usize> for PageTable<L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<L: TableLevel> PageTable<L> {
    const MASK: usize = 0b111111111 << L::SHIFT;

    #[inline]
    pub const fn get_index(a: Address<V>) -> usize {
        (a.as_usize() & Self::MASK) >> L::SHIFT
    }

    fn get_next_table(&mut self, index: usize) -> Option<&'static mut PageTable<L::NextLevel>> {
        if self.entries[index].present() && !self.entries[index].is_block() {
            let addr = self.entries[index].address();
            Some(unsafe { transmute(addr) })
        } else {
            None
        }
    }
}

impl PageTable<L4> {
    pub fn alloc() -> &'static mut Self {
        let frame = Self::alloc_frame4k();
        unsafe {
            frame.zero();
            frame.start().as_mut()
        }
    }

    #[inline]
    pub fn get() -> &'static mut Self {
        unsafe { &mut *(TTBR0_EL1.get() as *mut Self) }
    }

    pub fn identity_map<S: PageSize>(&mut self, frame: Frame<S>, flags: PageFlags) -> Page<S> {
        self.map(Page::new(frame.start().as_usize().into()), frame, flags)
    }

    fn alloc_frame4k() -> Frame<Size4K> {
        let frame = PHYSICAL_PAGE_RESOURCE
            .lock()
            .acquire::<Size4K>(1)
            .unwrap()
            .start;
        unsafe {
            frame.zero();
        }
        frame
    }

    pub fn map<S: PageSize>(
        &mut self,
        page: Page<S>,
        frame: Frame<S>,
        flags: PageFlags,
    ) -> Page<S> {
        // P4
        let table = self;
        // P3
        let index = PageTable::<L4>::get_index(page.start());
        if table[index].is_empty() {
            table[index].set(Self::alloc_frame4k(), PageFlags::page_table_flags());
        }
        let table = table.get_next_table(index).unwrap();
        if S::BYTES == Size1G::BYTES {
            table.entries[PageTable::<L3>::get_index(page.start())].set(frame, flags);
            return page;
        }
        // P2
        let index = PageTable::<L3>::get_index(page.start());
        if table.entries[index].is_empty() {
            table.entries[index].set(Self::alloc_frame4k(), PageFlags::page_table_flags());
        }
        let table = table.get_next_table(index).unwrap();
        if S::BYTES == Size2M::BYTES {
            table.entries[PageTable::<L2>::get_index(page.start())].set(frame, flags);
            return page;
        }
        // P1
        let index = PageTable::<L2>::get_index(page.start());
        if table.entries[index].is_empty() {
            table.entries[index].set(Self::alloc_frame4k(), PageFlags::page_table_flags());
        }
        let table = table.get_next_table(index).unwrap();
        table.entries[PageTable::<L1>::get_index(page.start())]
            .set(frame, flags | PageFlag::SMALL_PAGE);
        page
    }

    pub fn unmap<S: PageSize>(&mut self, _page: Page<S>) {
        unimplemented!()
    }
}
