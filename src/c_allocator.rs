use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    cmp,
    ffi::c_void,
    mem,
    ptr::{self, NonNull},
};

use libc::{free, malloc};

/// A custom memory allocator that interfaces with the C standard library's allocation functions.
///
/// ```
/// use mem_allocs::CAllocator;
///
/// #[global_allocator]
/// static GLOBAL_ALLOCATOR: CAllocator = CAllocator;
///
/// fn main() {
///     let mut vector: Vec<usize> = Vec::with_capacity(100);
///
///     for index in 0..100 {
///         vector.push(index);
///     }
///
///     assert_eq!(vector.len(), 100);
///     for (expected_index, actual_value) in vector.into_iter().enumerate().take(100) {
///         assert_eq!(actual_value, expected_index);
///     }
/// }
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct CAllocator;

unsafe impl Allocator for CAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let alignment = layout.align().max(mem::size_of::<usize>());
        let size = layout.size();
        let ptr = allocate_memory(size, alignment)?;

        NonNull::new(ptr::slice_from_raw_parts_mut(ptr, size)).ok_or(AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _: Layout) {
        unsafe { free(ptr.as_ptr().cast::<c_void>()) };
    }
}

unsafe impl GlobalAlloc for CAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        allocate_memory(layout.size(), layout.align()).unwrap_or(ptr::null_mut())
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.alloc(layout) };
        if !ptr.is_null() {
            unsafe { ptr::write_bytes(ptr, 0, layout.size()) };
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        unsafe { free(ptr.cast::<c_void>()) };
    }

    unsafe fn realloc(&self, old_ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
        let new_layout = Layout::from_size_align(new_size, old_layout.align()).unwrap();
        let new_ptr = unsafe { self.alloc(new_layout) };
        if !new_ptr.is_null() {
            let copy_size = cmp::min(old_layout.size(), new_size);
            unsafe { ptr::copy_nonoverlapping(old_ptr, new_ptr, copy_size) };
            unsafe { self.dealloc(old_ptr, old_layout) };
        }
        new_ptr
    }
}

/// Allocates memory with the specified size and alignment.
///
/// # Errors
///
/// Returns an `AllocError` if the allocation fails.
fn allocate_memory(size: usize, alignment: usize) -> Result<*mut u8, AllocError> {
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "solaris",
        target_os = "openbsd",
        target_os = "linux",
        target_os = "macos",
    ))]
    {
        let mut ptr: *mut c_void = core::ptr::null_mut();
        let result = unsafe { libc::posix_memalign(&raw mut ptr, alignment, size) };
        if result != 0 {
            Err(AllocError)
        } else {
            Ok(ptr.cast::<u8>())
        }
    }

    #[cfg(not(any(
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "solaris",
        target_os = "openbsd",
        target_os = "linux",
        target_os = "macos",
    )))]
    {
        let ptr = unsafe { libc::memalign(alignment, size) as *mut u8 };
        if ptr.is_null() {
            Err(AllocError)
        } else {
            Ok(ptr)
        }
    }
}

/// A custom memory allocator that interfaces with the C standard library's basic allocation functions (`malloc`/`free`).
///
/// ```
/// use mem_allocs::RawCAllocator;
///
/// #[global_allocator]
/// static GLOBAL_ALLOCATOR: RawCAllocator = RawCAllocator;
///
/// fn main() {
///     let mut vector: Vec<usize> = Vec::with_capacity(100);
///
///     for index in 0..100 {
///         vector.push(index);
///     }
///
///     assert_eq!(vector.len(), 100);
///     for (expected_index, actual_value) in vector.into_iter().enumerate().take(100) {
///         assert_eq!(actual_value, expected_index);
///     }
/// }
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct RawCAllocator;

unsafe impl Allocator for RawCAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let size = layout.size();
        NonNull::new(ptr::slice_from_raw_parts_mut(
            unsafe { malloc(size).cast::<u8>() },
            size,
        ))
        .ok_or(AllocError)
    }

    unsafe fn deallocate(&self, allocated_ptr: NonNull<u8>, _: Layout) {
        unsafe { free(allocated_ptr.as_ptr().cast::<c_void>()) };
    }
}

unsafe impl GlobalAlloc for RawCAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { malloc(layout.size()).cast::<u8>() }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let allocated_ptr = unsafe { self.alloc(layout) };
        if !allocated_ptr.is_null() {
            unsafe { ptr::write_bytes(allocated_ptr, 0, layout.size()) };
        }
        allocated_ptr
    }

    unsafe fn dealloc(&self, allocated_ptr: *mut u8, _: Layout) {
        unsafe { free(allocated_ptr.cast::<c_void>()) };
    }

    unsafe fn realloc(&self, old_ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, old_layout.align()) };
        let new_ptr = unsafe { self.alloc(new_layout) };
        if !new_ptr.is_null() {
            let copy_size = cmp::min(old_layout.size(), new_size);
            unsafe { ptr::copy_nonoverlapping(old_ptr, new_ptr, copy_size) };
            unsafe { self.dealloc(old_ptr, old_layout) };
        }
        new_ptr
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_generic_vector_with_c_allocator() {
        let allocator = CAllocator;
        let mut vector: Vec<usize, CAllocator> = Vec::with_capacity_in(100, allocator);

        for index in 0..100 {
            vector.push(index);
        }

        assert_eq!(vector.len(), 100);
        for (expected_index, actual_value) in vector.into_iter().enumerate().take(100) {
            assert_eq!(actual_value, expected_index);
        }
    }

    #[test]
    fn test_generic_vector_with_raw_c_allocator() {
        let allocator = RawCAllocator;
        let mut vector: Vec<usize, RawCAllocator> = Vec::with_capacity_in(100, allocator);

        for index in 0..100 {
            vector.push(index);
        }

        assert_eq!(vector.len(), 100);
        for (expected_index, actual_value) in vector.into_iter().enumerate().take(100) {
            assert_eq!(actual_value, expected_index);
        }
    }
}
