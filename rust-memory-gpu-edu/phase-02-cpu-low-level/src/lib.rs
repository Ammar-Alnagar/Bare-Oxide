//! Phase 2: Low-Level CPU Memory, Custom Allocators, and Safe Wrappers.

use std::alloc::{alloc, dealloc, Layout};
use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

/// A simple high-performance Bump Allocator.
///
/// Allocates memory linearly from a contiguous pre-allocated byte buffer.
pub struct BumpAllocator {
    buffer: *mut u8,
    capacity: usize,
    offset: UnsafeCell<usize>,
}

unsafe impl Send for BumpAllocator {}
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    /// Creates a new bump allocator with fixed capacity in bytes.
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 16).expect("Valid layout");
        // SAFETY: Allocating raw uninitialized byte buffer with 16-byte alignment.
        let buffer = unsafe { alloc(layout) };
        assert!(!buffer.is_null(), "Allocation failed");
        Self {
            buffer,
            capacity,
            offset: UnsafeCell::new(0),
        }
    }

    /// Allocates `size` bytes aligned to `align`.
    pub fn alloc(&self, layout: Layout) -> Option<*mut u8> {
        let current_offset = unsafe { *self.offset.get() };
        let align = layout.align();
        let size = layout.size();

        // Calculate aligned starting offset
        let aligned_offset = (current_offset + (align - 1)) & !(align - 1);
        let next_offset = aligned_offset.checked_add(size)?;

        if next_offset > self.capacity {
            return None; // Out of memory in this bump chunk
        }

        // SAFETY: Updating bump pointer offset.
        unsafe {
            *self.offset.get() = next_offset;
            Some(self.buffer.add(aligned_offset))
        }
    }

    /// Reset allocator offset to zero, reclaiming all memory at once.
    pub fn reset(&self) {
        // SAFETY: Resets the offset. The caller must ensure no existing references remain active.
        unsafe {
            *self.offset.get() = 0;
        }
    }

    /// Current allocated bytes.
    pub fn allocated_bytes(&self) -> usize {
        unsafe { *self.offset.get() }
    }
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 16).expect("Valid layout");
        // SAFETY: Buffer was allocated with identical layout in BumpAllocator::new.
        unsafe {
            dealloc(self.buffer, layout);
        }
    }
}

/// Safe educational RAII wrapper around manual raw allocation.
pub struct SafeRawBuffer<T> {
    ptr: NonNull<T>,
    capacity: usize,
    len: usize,
}

impl<T> SafeRawBuffer<T> {
    /// Allocate uninitialized buffer capable of holding `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than zero");
        let layout = Layout::array::<T>(capacity).expect("Valid array layout");
        // SAFETY: Requesting aligned memory from the global allocator.
        let raw = unsafe { alloc(layout) as *mut T };
        let ptr = NonNull::new(raw).expect("Allocation out of memory");
        Self {
            ptr,
            capacity,
            len: 0,
        }
    }

    /// Push an element into the buffer.
    pub fn push(&mut self, value: T) {
        assert!(self.len < self.capacity, "Buffer overflow");
        // SAFETY: self.ptr is non-null, self.len < self.capacity, offset is within bounds.
        unsafe {
            let target = self.ptr.as_ptr().add(self.len);
            std::ptr::write(target, value);
        }
        self.len += 1;
    }

    /// Returns a slice of initialized elements.
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: Elements 0..self.len have been properly initialized with std::ptr::write.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns number of initialized elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T> Drop for SafeRawBuffer<T> {
    fn drop(&mut self) {
        // SAFETY: Drop each initialized element in place.
        for i in 0..self.len {
            unsafe {
                std::ptr::drop_in_place(self.ptr.as_ptr().add(i));
            }
        }
        let layout = Layout::array::<T>(self.capacity).expect("Valid array layout");
        // SAFETY: Free the underlying raw buffer using the original array layout.
        unsafe {
            dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}

/// Demonstrates ManuallyDrop and std::mem::forget.
pub fn demonstrate_leak_and_forget() -> usize {
    let original = String::from("Do Not Drop Automatically");
    let manual = ManuallyDrop::new(original);
    let len = manual.len();

    // Take out inner value or bypass drop
    let _forgotten = ManuallyDrop::into_inner(manual);
    // Suppress drop
    std::mem::forget(_forgotten);

    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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
    fn test_manual_drop() {
        let len = demonstrate_leak_and_forget();
        assert_eq!(len, 25);
    }
}
