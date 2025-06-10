#![no_std]
#![feature(allocator_api, cfg_select, slice_ptr_get)]

#[cfg(feature = "c_allocator")]
pub mod c_allocator;
