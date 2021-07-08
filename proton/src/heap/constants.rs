use core::iter::Step;
use core::ops::Range;

use crate::utils::address::*;
use crate::utils::page::*;

// pub const KERNEL_CORE0_STACK_START: usize = 0xffff0000_0007c000;
// pub const KERNEL_CORE0_STACK_END:   usize = 0xffff0000_00080000;

/// Kernel process stack
pub const KERNEL_STACK_PAGES: usize = 8; // Too many???
pub const KERNEL_STACK_SIZE: usize = KERNEL_STACK_PAGES * Size4K::BYTES;

// /// User heap layout
// pub const USER_STACK_START: Address<V> = Address::new(0x111900000);
// pub const USER_STACK_PAGES: usize = 4; // Too many???
// pub const USER_STACK_SIZE: usize = USER_STACK_PAGES * Size4K::SIZE;
// pub const USER_STACK_END: Address<V> = Address::new(USER_STACK_START.as_usize() + USER_STACK_SIZE);
// pub const USER_CODE_START: Page = Page::of(USER_STACK_END);

pub const LOG_KERNEL_HEAP_PAGES: usize = 12; // 16M
pub const KERNEL_HEAP_PAGES: usize = 1 << LOG_KERNEL_HEAP_PAGES;
// pub const KERNEL_HEAP_SIZE: usize = KERNEL_HEAP_PAGES * Size4K::SIZE;

// pub const KERNEL_START: usize = 0x80000; // 16M

pub const KERNEL_HEAP_LARGE_PAGES: usize = 128;

// #[inline]
// pub fn kernel_memory() -> Range<Address> {
//     let start = unsafe { Address::from(&__kernel_start as *const _ as usize) };
//     let end = unsafe { Address::from(&__kernel_end as *const _) };
//     start..end
// }

#[inline]
pub fn kernel_memory() -> Range<Page<Size4K>> {
    let start = unsafe { Address::<V>::from(&__kernel_start as *const _ as usize) };
    let end = unsafe { Address::<V>::from(&__kernel_end as *const _) };
    let start = Page::<Size4K>::new(start);
    let end = Page::<Size4K>::new(end.align_up(Size2M::BYTES));
    start..end
}

#[inline]
pub fn kernel_heap_memory() -> Range<Page<Size4K>> {
    let kernel = kernel_memory();
    let start = kernel.start;
    let end = Page::forward(start, KERNEL_HEAP_PAGES);
    start..end
}

pub const LOG_MAX_HEAP_SIZE: usize = 32; // 4G
pub const MAX_HEAP_SIZE: usize = 1 << LOG_MAX_HEAP_SIZE; // 4G

extern "C" {
    static __kernel_start: usize;
    static __kernel_end: usize;
}
