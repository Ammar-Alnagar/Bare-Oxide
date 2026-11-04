//! Phase 4: Advanced GPU Memory Management patterns.

use common::{DeviceCopy, EduContext};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Educational DisjointSlice representation for data-race-free parallel writes.
pub struct DisjointSlice<'a, T> {
    ptr: *mut T,
    len: usize,
    _marker: std::marker::PhantomData<&'a mut [T]>,
}

unsafe impl<'a, T: Send> Send for DisjointSlice<'a, T> {}
unsafe impl<'a, T: Sync> Sync for DisjointSlice<'a, T> {}

impl<'a, T> DisjointSlice<'a, T> {
    /// Create a disjoint slice from an exclusive mutable slice.
    pub fn from_slice(slice: &'a mut [T]) -> Self {
        let len = slice.len();
        let ptr = slice.as_mut_ptr();
        Self {
            ptr,
            len,
            _marker: std::marker::PhantomData,
        }
    }

    /// Number of elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Split slice into two disjoint non-overlapping mutable slices.
    pub fn split_at_mut(self, mid: usize) -> (Self, Self) {
        assert!(mid <= self.len, "Split index out of bounds");
        let left_ptr = self.ptr;
        // SAFETY: mid <= len, so adding mid is within allocation bounds.
        let right_ptr = unsafe { self.ptr.add(mid) };
        let left_len = mid;
        let right_len = self.len - mid;

        (
            DisjointSlice {
                ptr: left_ptr,
                len: left_len,
                _marker: std::marker::PhantomData,
            },
            DisjointSlice {
                ptr: right_ptr,
                len: right_len,
                _marker: std::marker::PhantomData,
            },
        )
    }

    /// Write value at index safely within bounds.
    pub fn write_at(&mut self, index: usize, value: T) {
        assert!(index < self.len, "Index out of bounds");
        // SAFETY: index is verified to be within bounds of this disjoint partition.
        unsafe {
            std::ptr::write(self.ptr.add(index), value);
        }
    }

    /// Read value at index safely within bounds.
    pub fn read_at(&self, index: usize) -> T
    where
        T: Copy,
    {
        assert!(index < self.len, "Index out of bounds");
        // SAFETY: index is verified to be within bounds of this disjoint partition.
        unsafe { *self.ptr.add(index) }
    }
}

/// Educational Pinned Host Buffer (page-locked memory).
pub struct EduPinnedHostBuffer<T: DeviceCopy> {
    data: Vec<T>,
    context: Arc<EduContext>,
}

impl<T: DeviceCopy> EduPinnedHostBuffer<T> {
    pub fn new(context: Arc<EduContext>, capacity: usize) -> Self {
        // In native cuda-core, this calls cuMemAllocHost.
        // In educational CPU mode, we wrap a contiguous vector with context tracking.
        Self {
            data: Vec::with_capacity(capacity),
            context,
        }
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn context(&self) -> &Arc<EduContext> {
        &self.context
    }
}

/// Educational GPU Memory Pool stats and tracking.
#[derive(Debug, Default)]
pub struct EduMemPool {
    used_bytes: AtomicUsize,
    reserved_bytes: AtomicUsize,
    high_watermark: AtomicUsize,
}

impl EduMemPool {
    pub fn new(initial_reserved: usize) -> Self {
        Self {
            used_bytes: AtomicUsize::new(0),
            reserved_bytes: AtomicUsize::new(initial_reserved),
            high_watermark: AtomicUsize::new(0),
        }
    }

    pub fn allocate(&self, bytes: usize) -> Result<(), String> {
        let current = self.used_bytes.fetch_add(bytes, Ordering::SeqCst) + bytes;
        let reserved = self.reserved_bytes.load(Ordering::SeqCst);
        if current > reserved {
            self.reserved_bytes.store(current, Ordering::SeqCst);
        }
        let hw = self.high_watermark.load(Ordering::SeqCst);
        if current > hw {
            self.high_watermark.store(current, Ordering::SeqCst);
        }
        Ok(())
    }

    pub fn deallocate(&self, bytes: usize) {
        self.used_bytes.fetch_sub(bytes, Ordering::SeqCst);
    }

    pub fn used_current(&self) -> usize {
        self.used_bytes.load(Ordering::SeqCst)
    }

    pub fn high_watermark(&self) -> usize {
        self.high_watermark.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disjoint_slice_parallel_writes() {
        let mut raw_data = vec![0u32; 10];
        let slice = DisjointSlice::from_slice(&mut raw_data);

        let (mut left, mut right) = slice.split_at_mut(5);
        assert_eq!(left.len(), 5);
        assert_eq!(right.len(), 5);

        // Mutate left and right partitions independently with zero aliasing
        for i in 0..5 {
            left.write_at(i, (i + 1) as u32 * 10);
            right.write_at(i, (i + 1) as u32 * 100);
        }

        assert_eq!(left.read_at(0), 10);
        assert_eq!(right.read_at(0), 100);

        // Verify backing data
        assert_eq!(raw_data, vec![10, 20, 30, 40, 50, 100, 200, 300, 400, 500]);
    }

    #[test]
    fn test_pinned_host_buffer() {
        let ctx = EduContext::new(0).unwrap();
        let mut pinned = EduPinnedHostBuffer::new(ctx, 4);
        pinned.push(1.5f32);
        pinned.push(2.5f32);
        assert_eq!(pinned.len(), 2);
        assert_eq!(pinned.as_slice(), &[1.5f32, 2.5f32]);
    }

    #[test]
    fn test_mem_pool_tracking() {
        let pool = EduMemPool::new(1024);
        pool.allocate(256).unwrap();
        assert_eq!(pool.used_current(), 256);
        assert_eq!(pool.high_watermark(), 256);

        pool.allocate(512).unwrap();
        assert_eq!(pool.used_current(), 768);
        assert_eq!(pool.high_watermark(), 768);

        pool.deallocate(256);
        assert_eq!(pool.used_current(), 512);
        // High watermark remains peak usage
        assert_eq!(pool.high_watermark(), 768);
    }
}
