use core::alloc::{AllocError, Allocator, GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::{NonNull, null_mut, slice_from_raw_parts_mut};

use alloc::{vec, vec::Vec};

/// A simple bump allocator using a pre-allocated `Vec` as backing storage.
///
/// This allocator does not support individual deallocation. Instead,
/// memory can be released all at once using [`reset`].
///
/// # Example
///
/// ```
/// #![feature(allocator_api)]
///
/// use mem_allocs::ArenaAllocator;
/// use core::alloc::{Allocator, Layout};
///
/// let arena = ArenaAllocator::new(1024);
///
/// let layout = Layout::from_size_align(16, 8).unwrap();
/// let ptr = arena.allocate(layout).unwrap();
///
/// // Use the pointer...
///
/// // Free all allocations
/// arena.reset();
///
/// // You can allocate again now from the beginning of the buffer
/// let ptr2 = arena.allocate(layout).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct ArenaAllocator {
    buffer: UnsafeCell<Vec<MaybeUninit<u8>>>,
    offset: UnsafeCell<usize>,
}

impl ArenaAllocator {
    /// Creates a new arena allocator with a given capacity in bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The number of bytes to reserve in the arena.
    ///
    /// # Example
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use mem_allocs::ArenaAllocator;
    ///
    /// let arena = ArenaAllocator::new(1024);
    /// ```
    #[must_use]
    pub fn new(bytes: usize) -> Self {
        let vec = vec![MaybeUninit::<u8>::uninit(); bytes];
        Self {
            buffer: UnsafeCell::new(vec),
            offset: UnsafeCell::new(0),
        }
    }

    /// Resets the arena, making all previously allocated memory available again.
    ///
    /// This **does not** call destructors or deallocate memory — use only
    /// when you know the memory is no longer in use.
    ///
    /// # Safety
    ///
    /// - It is undefined behavior to use any pointer returned from `allocate`
    ///   after calling `reset`.
    ///
    /// # Example
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use mem_allocs::ArenaAllocator;
    /// use core::alloc::{Allocator, Layout};
    ///
    /// let arena = ArenaAllocator::new(1024);
    /// let layout = Layout::from_size_align(32, 8).unwrap();
    /// let _ptr = arena.allocate(layout).unwrap();
    ///
    /// arena.reset(); // All memory is now reused
    /// ```
    pub fn reset(&self) {
        unsafe {
            *self.offset.get() = 0;
        }
    }

    /// Aligns `offset` upwards to the next multiple of `align`.
    const fn align_up(offset: usize, align: usize) -> usize {
        (offset + align - 1) & !(align - 1)
    }

    /// Returns the total capacity in bytes.
    pub fn capacity(&self) -> usize {
        unsafe { (*self.buffer.get()).len() }
    }
}

unsafe impl Allocator for ArenaAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        let buffer = unsafe { &mut *self.buffer.get() };
        let offset = unsafe { &mut *self.offset.get() };

        let base_ptr = buffer.as_mut_ptr().cast::<u8>();
        let aligned_offset = Self::align_up(*offset, layout.align());
        let end = aligned_offset
            .checked_add(layout.size())
            .ok_or(AllocError)?;

        if end > buffer.len() {
            return Err(AllocError);
        }

        *offset = end;

        let ptr = unsafe { base_ptr.add(aligned_offset) };
        let slice = slice_from_raw_parts_mut(ptr, layout.size());

        Ok(unsafe { NonNull::new_unchecked(slice) })
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

unsafe impl GlobalAlloc for ArenaAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocate(layout).map_or_else(
            |_| null_mut(),
            |non_null_slice| non_null_slice.as_non_null_ptr().cast().as_ptr(),
        )
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::vec::Vec;

    #[test]
    /// Tests the `ArenaAllocator` with a generic vector.
    fn test_generic_vector_with_arena_allocator() {
        let capacity = 800;
        let allocator = ArenaAllocator::new(capacity);
        let mut vector: Vec<usize, ArenaAllocator> = Vec::with_capacity_in(100, allocator);

        for index in 0..100 {
            vector.push(index);
        }

        assert_eq!(vector.len(), 100);
        for (expected_index, actual_value) in vector.into_iter().enumerate().take(100) {
            assert_eq!(actual_value, expected_index);
        }
    }

    #[test]
    fn allocate_and_reset() {
        let arena = ArenaAllocator::new(100);

        let layout = Layout::from_size_align(10, 4).unwrap();
        let ptr1 = arena.allocate(layout).expect("allocation 1 failed");

        let offset_after_first_alloc = unsafe { *arena.offset.get() };

        let layout2 = Layout::from_size_align(20, 8).unwrap();
        let ptr2 = arena.allocate(layout2).expect("allocation 2 failed");

        let offset_after_second_alloc = unsafe { *arena.offset.get() };
        assert!(offset_after_second_alloc > offset_after_first_alloc);

        arena.reset();

        let offset_after_reset = unsafe { *arena.offset.get() };
        assert_eq!(offset_after_reset, 0);

        let ptr3 = arena.allocate(layout).expect("allocation 3 failed");
        assert_eq!(ptr1.as_ptr(), ptr3.as_ptr());
        assert_ne!(ptr2.as_ptr(), ptr3.as_ptr());
    }

    #[test]
    fn allocation_fails_when_out_of_space() {
        let arena = ArenaAllocator::new(128);

        let layout = Layout::from_size_align(128, 1).unwrap();
        assert!(arena.allocate(layout).is_ok());

        let layout2 = Layout::from_size_align(1, 1).unwrap();
        assert!(arena.allocate(layout2).is_err());

        arena.reset();
        assert!(arena.allocate(layout2).is_ok());
    }
}
