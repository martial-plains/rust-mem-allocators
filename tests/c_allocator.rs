#![feature(allocator_api, slice_ptr_get)]

mod common;

#[test]
#[cfg(feature = "c_allocator")]
fn c_allocator() {
    use common::test_allocator;
    use mem_allocs::c_allocator::CAllocator;

    test_allocator(CAllocator).unwrap();
}

#[test]
#[cfg(feature = "c_allocator")]
fn raw_c_allocator() {
    use common::test_allocator;
    use mem_allocs::c_allocator::RawCAllocator;

    test_allocator(RawCAllocator).unwrap();
}
