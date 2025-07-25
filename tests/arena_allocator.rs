#![feature(allocator_api, slice_ptr_get)]

mod common;

#[test]
#[cfg(feature = "arena_allocator")]
fn arena_allocator() {
    use common::test_allocator;
    use mem_allocs::ArenaAllocator;

    test_allocator(ArenaAllocator::new(800)).unwrap();
}
