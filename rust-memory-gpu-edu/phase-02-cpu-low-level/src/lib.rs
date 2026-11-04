#![allow(unused_variables, dead_code, unused_mut, unused_imports)]
//! Phase 2: CPU Low-Level Memory - Student Exercises
//!
//! Implement custom memory allocation, raw pointer handling, and safe RAII wrappers.
//! Run `cargo test -p phase-02-cpu-low-level` to verify your implementations.

use std::alloc::Layout;
use std::cell::UnsafeCell;
use std::ptr::NonNull;

/// EXERCISE 1: Custom Bump Allocator
///
/// Implement a high-performance linear bump allocator.
pub struct BumpAllocator {
    buffer: *mut u8,
    capacity: usize,
    offset: UnsafeCell<usize>,
}

unsafe impl Send for BumpAllocator {}
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    /// Creates a new bump allocator with fixed capacity in bytes.
    /// Allocate backing memory using `std::alloc::alloc` with 16-byte alignment.
    pub fn new(capacity: usize) -> Self {
        // TODO: Allocate uninitialized memory buffer of `capacity` bytes aligned to 16.
        todo!("Exercise 1A: Implement BumpAllocator::new")
    }

    /// Allocates `size` bytes aligned to `align` using bitwise alignment arithmetic:
    /// `aligned_offset = (current_offset + (align - 1)) & !(align - 1)`
    pub fn alloc(&self, layout: Layout) -> Option<*mut u8> {
        // TODO: Calculate aligned offset and advance bump pointer if within capacity.
        todo!("Exercise 1B: Implement BumpAllocator::alloc")
    }

    /// Reclaim all memory in bulk by resetting the offset to zero.
    pub fn reset(&self) {
        // TODO: Reset the internal offset to 0.
        todo!("Exercise 1C: Implement BumpAllocator::reset")
    }

    /// Return the currently allocated bytes.
    pub fn allocated_bytes(&self) -> usize {
        // TODO: Read the current offset.
        todo!("Exercise 1D: Implement BumpAllocator::allocated_bytes")
    }
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        // TODO: Deallocate backing buffer matching its creation layout using `std::alloc::dealloc`.
    }
}

/// EXERCISE 2: Safe RAII Buffer Wrapper around Raw Allocations
pub struct SafeRawBuffer<T> {
    ptr: NonNull<T>,
    capacity: usize,
    len: usize,
}

impl<T> SafeRawBuffer<T> {
    /// Allocate uninitialized buffer capable of holding `capacity` elements of type `T`.
    pub fn with_capacity(capacity: usize) -> Self {
        // TODO: Allocate memory for `capacity` elements using Layout::array::<T>.
        todo!("Exercise 2A: Implement SafeRawBuffer::with_capacity")
    }

    /// Push an element into the buffer using `std::ptr::write`.
    pub fn push(&mut self, value: T) {
        // TODO: Check bounds and write `value` to `self.ptr.as_ptr().add(self.len)`.
        todo!("Exercise 2B: Implement SafeRawBuffer::push")
    }

    /// Return slice of initialized elements.
    pub fn as_slice(&self) -> &[T] {
        // TODO: Convert raw parts into a safe slice `std::slice::from_raw_parts`.
        todo!("Exercise 2C: Implement SafeRawBuffer::as_slice")
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T> Drop for SafeRawBuffer<T> {
    fn drop(&mut self) {
        // TODO: Drop initialized elements with `std::ptr::drop_in_place`, then `dealloc` buffer.
    }
}

/// EXERCISE 3: Destructor Suppression with ManuallyDrop and forget
pub fn demonstrate_leak_and_forget() -> usize {
    // TODO: Create a String, wrap it in ManuallyDrop, extract its length, and suppress drop.
    todo!("Exercise 3: Implement demonstrate_leak_and_forget")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
    fn test_bump_allocator() {
        let bump = BumpAllocator::new(1024);
        let layout1 = Layout::from_size_align(64, 8).unwrap();
        let p1 = bump.alloc(layout1);
        assert!(p1.is_some());
        assert_eq!(bump.allocated_bytes(), 64);

        let layout2 = Layout::from_size_align(128, 16).unwrap();
        let p2 = bump.alloc(layout2);
        assert!(p2.is_some());

        bump.reset();
        assert_eq!(bump.allocated_bytes(), 0);
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
    fn test_safe_raw_buffer() {
        let mut buf = SafeRawBuffer::<String>::with_capacity(3);
        buf.push(String::from("alloc_1"));
        buf.push(String::from("alloc_2"));
        buf.push(String::from("alloc_3"));

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.as_slice()[0], "alloc_1");
        assert_eq!(buf.as_slice()[1], "alloc_2");
        assert_eq!(buf.as_slice()[2], "alloc_3");
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
    fn test_manual_drop() {
        let len = demonstrate_leak_and_forget();
        assert_eq!(len, 25);
    }
}
