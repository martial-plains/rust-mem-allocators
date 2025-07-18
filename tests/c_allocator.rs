#![cfg(feature = "c_allocator")]
#![feature(allocator_api, slice_ptr_get)]

mod common;

#[test]
fn c_allocator() {
    use common::test_allocator;
    use mem_allocs::CAllocator;

    test_allocator(CAllocator).unwrap();
}

#[test]
fn raw_c_allocator() {
    use common::test_allocator;
    use mem_allocs::RawCAllocator;

    test_allocator(RawCAllocator).unwrap();
}
