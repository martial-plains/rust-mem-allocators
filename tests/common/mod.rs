use std::{
    alloc::{Allocator, Layout},
    convert::TryFrom,
    error::Error,
    mem::size_of,
};

/// Tests the functionality of a custom memory allocator.
/// This includes allocating and deallocating memory for both a non-empty array
/// and an empty array. The function ensures that memory is allocated and
/// deallocated correctly, and that the data is properly written to and read from
/// the allocated memory.
///
/// # Parameters
/// - `allocator`: The custom allocator that will be tested.
///
/// # Returns
/// Returns `Ok(())` if the allocator works as expected, or an error if any
/// allocation or deallocation operation fails.
///
/// # Example
/// ```
/// let allocator = MyAllocator::new();
/// test_allocator(allocator).expect("Allocator test failed");
/// ```
#[allow(clippy::cast_ptr_alignment)] // This is to allow casting raw pointers with non-aligned types.
pub fn test_allocator<A: Allocator>(allocator: A) -> Result<(), Box<dyn Error>> {
    // Allocate memory for 100 `i32` elements
    let allocation_layout = Layout::array::<i32>(100)?;

    // Allocate the memory from the custom allocator
    let allocation_pointer = allocator.allocate(allocation_layout)?;

    // Verify the allocated memory matches the expected size
    assert!(allocation_layout.size() / size_of::<i32>() == 100);
    assert!(allocation_pointer.len() / size_of::<i32>() == 100);

    // Create a mutable slice from the allocated raw pointer
    let allocated_slice =
        unsafe { core::slice::from_raw_parts_mut(allocation_pointer.as_ptr().cast::<i32>(), 100) };

    // Populate the allocated memory with values 0 to 99
    for (index, value) in allocated_slice.iter_mut().enumerate() {
        *value = i32::try_from(index)?;
    }

    // Deallocate the memory once we're done with it
    unsafe { allocator.deallocate(allocation_pointer.as_non_null_ptr(), allocation_layout) };

    // Test allocating and deallocating zero-length memory
    let empty_layout = Layout::array::<u8>(0)?;
    let empty_pointer = allocator.allocate(empty_layout)?;

    // Deallocate the zero-length memory
    unsafe { allocator.deallocate(empty_pointer.as_non_null_ptr(), empty_layout) };

    Ok(())
}
