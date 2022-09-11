mod physical_page_resource;

use self::physical_page_resource::PHYSICAL_PAGE_RESOURCE;
use super::kernel::KERNEL_MEMORY_MAPPER;
use core::ops::Range;
use interrupt::UninterruptibleMutex;
use memory::{address::P, page::*};

pub struct PhysicalMemory {
    _private: (),
}

impl PhysicalMemory {
    pub const fn new() -> Self {
        Self { _private: () }
    }

    pub fn init(&self, frames: &'static [Range<Frame>]) {
        PHYSICAL_PAGE_RESOURCE.lock().init(frames);
        KERNEL_MEMORY_MAPPER.init();
    }

    pub fn acquire<S: PageSize>(&self) -> Option<Frame<S>> {
        let _guard = KERNEL_MEMORY_MAPPER.with_kernel_address_space();
        PHYSICAL_PAGE_RESOURCE.lock_uninterruptible().acquire()
    }

    pub fn release<S: PageSize>(&self, frame: Frame<S>) {
        let _guard = KERNEL_MEMORY_MAPPER.with_kernel_address_space();
        PHYSICAL_PAGE_RESOURCE.lock_uninterruptible().release(frame)
    }
}

pub static PHYSICAL_MEMORY: PhysicalMemory = PhysicalMemory::new();

impl PageAllocator<P> for PhysicalMemory {
    #[inline(always)]
    fn alloc<S: PageSize>(&self) -> Option<Frame<S>> {
        self.acquire()
    }

    #[inline(always)]
    fn dealloc<S: PageSize>(&self, frame: Frame<S>) {
        self.release(frame)
    }
}

pub struct SharedPhysicalPage<S: PageSize> {
    frame: Frame<S>,
}

impl<S: PageSize> SharedPhysicalPage<S> {
    pub fn new(frame: Frame<S>) -> Self {
        Self { frame }
    }
}

impl<S: PageSize> Drop for SharedPhysicalPage<S> {
    fn drop(&mut self) {
        PHYSICAL_MEMORY.release(self.frame);
    }
}
