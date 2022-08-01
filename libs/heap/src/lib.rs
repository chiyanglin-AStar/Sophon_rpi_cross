#![no_std]
#![feature(core_intrinsics)]
#![feature(step_trait)]

use core::{
    alloc::{GlobalAlloc, Layout},
    iter::Step,
    ops::Range,
};

use memory::{
    address::V,
    free_list_allocator::FreeListAllocator,
    page::{Page, PageResource, PageSize, Size1G},
};
use spin::Mutex;

pub struct NoAlloc;

unsafe impl GlobalAlloc for NoAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unreachable!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unreachable!()
    }
}

/// The kernel heap memory manager.
pub struct UserHeap {
    fa: Mutex<FreeListAllocator<V, Self, { Size1G::LOG_BYTES + 1 }, false>>,
}

impl UserHeap {
    pub const fn new() -> Self {
        Self {
            fa: Mutex::new(FreeListAllocator::new()),
        }
    }

    pub fn init(&'static self) {
        self.fa.lock().init(self)
    }
}

impl PageResource<V> for UserHeap {
    /// Allocate virtual pages that are backed by physical memory.
    fn acquire_pages<S: PageSize>(&self, pages: usize) -> Option<Range<Page<S>>> {
        let addr = memory::sbrk(pages << S::LOG_BYTES)?;
        assert!(addr.is_aligned_to(S::BYTES));
        let start_page = Page::new(addr);
        let end_page = Page::forward(start_page, pages);
        Some(start_page..end_page)
    }

    /// Release and unmap virtual pages.
    fn release_pages<S: PageSize>(&self, _pages: Range<Page<S>>) {
        unimplemented!()
    }
}

unsafe impl GlobalAlloc for UserHeap {
    #[inline(always)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        assert!(layout.pad_to_align().size() <= Size1G::BYTES);
        self.fa.lock().alloc(&layout).as_mut_ptr()
    }

    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.fa.lock().free(ptr.into(), &layout)
    }
}
