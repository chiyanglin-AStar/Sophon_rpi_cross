#![allow(incomplete_features)]
#![feature(step_trait)]
#![feature(core_intrinsics)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(const_option)]
#![feature(never_type)]
#![feature(format_args_nl)]
#![feature(generic_const_exprs)]
#![no_std]

#[allow(unused)]
#[macro_use]
extern crate log;

pub mod address;
pub mod bitmap_page_allocator;
pub mod cache;
pub mod free_list_allocator;
pub mod page;
pub mod page_table;
pub mod volatile;
