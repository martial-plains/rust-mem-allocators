#![no_std]
#![feature(allocator_api, cfg_select, slice_ptr_get)]

extern crate alloc;

cfg_select! {
    feature = "c_allocator" => {
        mod c_allocator;
        pub use c_allocator::*;
    }
}

cfg_select! {
    feature = "arena_allocator" => {
        mod arena_allocator;
        pub use arena_allocator::*;
    }
}
