#![no_std]
#![feature(allocator_api, cfg_select)]

cfg_select! {
    feature = "c_allocator" => {
        mod c_allocator;
        pub use c_allocator::*;
    }
}
