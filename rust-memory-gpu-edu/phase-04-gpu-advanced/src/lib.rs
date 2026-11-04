#![allow(unused_variables, dead_code, unused_mut, unused_imports)]
//! Phase 4: Advanced GPU Memory Management - Student Exercises
//!
//! Implement disjoint slice partitioning, pinned memory buffers, and memory pool tracking.
//! Run `cargo test -p phase-04-gpu-advanced` to verify your implementations.

use common::{DeviceCopy, EduContext};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// EXERCISE 1: DisjointSlice for Safe Parallel Writes
pub struct DisjointSlice<'a, T> {
    ptr: *mut T,
    len: usize,
    _marker: std::marker::PhantomData<&'a mut [T]>,
}

unsafe impl<'a, T: Send> Send for DisjointSlice<'a, T> {}
unsafe impl<'a, T: Sync> Sync for DisjointSlice<'a, T> {}

impl<'a, T> DisjointSlice<'a, T> {
    pub fn from_slice(slice: &'a mut [T]) -> Self {
        // TODO: Construct DisjointSlice from mutable slice.
        todo!("Exercise 1A: Implement DisjointSlice::from_slice")
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Split slice into two disjoint non-overlapping mutable slices.
    pub fn split_at_mut(self, mid: usize) -> (Self, Self) {
        // TODO: Assert mid <= self.len and return two disjoint sub-slices.
        todo!("Exercise 1B: Implement DisjointSlice::split_at_mut")
    }

    pub fn write_at(&mut self, index: usize, value: T) {
        // TODO: Bounds check index and write value to self.ptr.add(index).
        todo!("Exercise 1C: Implement DisjointSlice::write_at")
    }

    pub fn read_at(&self, index: usize) -> T
    where
        T: Copy,
    {
        // TODO: Bounds check index and read value from self.ptr.add(index).
        todo!("Exercise 1D: Implement DisjointSlice::read_at")
    }
}

/// EXERCISE 2: Pinned Host Buffer (Page-Locked Host Memory)
pub struct EduPinnedHostBuffer<T: DeviceCopy> {
    data: Vec<T>,
    context: Arc<EduContext>,
}

impl<T: DeviceCopy> EduPinnedHostBuffer<T> {
    pub fn new(context: Arc<EduContext>, capacity: usize) -> Self {
        // TODO: Create pinned host buffer with capacity.
        todo!("Exercise 2: Implement EduPinnedHostBuffer::new")
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

/// EXERCISE 3: Memory Pool Accounting and Watermarks
#[derive(Debug, Default)]
pub struct EduMemPool {
    used_bytes: AtomicUsize,
    reserved_bytes: AtomicUsize,
    high_watermark: AtomicUsize,
}

impl EduMemPool {
    pub fn new(initial_reserved: usize) -> Self {
        // TODO: Initialize pool with reserved bytes.
        todo!("Exercise 3A: Implement EduMemPool::new")
    }

    pub fn allocate(&self, bytes: usize) -> Result<(), String> {
        // TODO: Increment used_bytes, update high_watermark if needed.
        todo!("Exercise 3B: Implement EduMemPool::allocate")
    }

    pub fn deallocate(&self, bytes: usize) {
        // TODO: Decrement used_bytes.
        todo!("Exercise 3C: Implement EduMemPool::deallocate")
    }

    pub fn used_current(&self) -> usize {
        todo!("Exercise 3D: Return used bytes")
    }

    pub fn high_watermark(&self) -> usize {
        todo!("Exercise 3E: Return high watermark")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
    fn test_disjoint_slice_parallel_writes() {
        let mut raw_data = vec![0u32; 10];
        let slice = DisjointSlice::from_slice(&mut raw_data);

        let (mut left, mut right) = slice.split_at_mut(5);
        assert_eq!(left.len(), 5);
        assert_eq!(right.len(), 5);

        for i in 0..5 {
            left.write_at(i, (i + 1) as u32 * 10);
            right.write_at(i, (i + 1) as u32 * 100);
        }

        assert_eq!(left.read_at(0), 10);
        assert_eq!(right.read_at(0), 100);

        assert_eq!(raw_data, vec![10, 20, 30, 40, 50, 100, 200, 300, 400, 500]);
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
    fn test_pinned_host_buffer() {
        let ctx = EduContext::new(0).unwrap();
        let mut pinned = EduPinnedHostBuffer::new(ctx, 4);
        pinned.push(1.5f32);
        pinned.push(2.5f32);
        assert_eq!(pinned.len(), 2);
        assert_eq!(pinned.as_slice(), &[1.5f32, 2.5f32]);
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
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
        assert_eq!(pool.high_watermark(), 768);
    }
}
