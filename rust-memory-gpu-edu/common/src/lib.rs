//! Shared helper utilities and platform abstraction contracts for rust-memory-gpu-edu.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Marker trait representing plain-old-data safe for host-to-device transfer.
///
/// Types implementing `DeviceCopy` have no pointers, heap allocations, or drop glue.
///
/// # Safety
/// The type must be sound to duplicate with a byte-for-byte copy and must be valid
/// with an all-zero byte pattern.
pub unsafe trait DeviceCopy: Copy + 'static {}

unsafe impl DeviceCopy for u8 {}
unsafe impl DeviceCopy for u16 {}
unsafe impl DeviceCopy for u32 {}
unsafe impl DeviceCopy for u64 {}
unsafe impl DeviceCopy for usize {}
unsafe impl DeviceCopy for i8 {}
unsafe impl DeviceCopy for i16 {}
unsafe impl DeviceCopy for i32 {}
unsafe impl DeviceCopy for i64 {}
unsafe impl DeviceCopy for isize {}
unsafe impl DeviceCopy for f32 {}
unsafe impl DeviceCopy for f64 {}
unsafe impl DeviceCopy for bool {}

unsafe impl<T: DeviceCopy, const N: usize> DeviceCopy for [T; N] {}

/// Simulated or direct CUDA execution context.
#[derive(Debug)]
pub struct EduContext {
    ordinal: usize,
    active_allocations: Arc<AtomicUsize>,
}

impl EduContext {
    /// Create a new educational CUDA context handle for device ordinal.
    pub fn new(ordinal: usize) -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self {
            ordinal,
            active_allocations: Arc::new(AtomicUsize::new(0)),
        }))
    }

    /// Returns the device ordinal.
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Create an educational stream attached to this context.
    pub fn new_stream(self: &Arc<Self>) -> Result<Arc<EduStream>, String> {
        Ok(Arc::new(EduStream {
            context: self.clone(),
        }))
    }

    /// Active allocations tracked on this context.
    pub fn active_allocations(&self) -> usize {
        self.active_allocations.load(Ordering::SeqCst)
    }
}

/// Simulated or direct CUDA stream handle.
#[derive(Debug)]
pub struct EduStream {
    context: Arc<EduContext>,
}

impl EduStream {
    /// Reference to the parent context.
    pub fn context(&self) -> &Arc<EduContext> {
        &self.context
    }

    /// Synchronize the stream.
    pub fn synchronize(&self) -> Result<(), String> {
        // Enforces stream synchronization completion barrier
        Ok(())
    }
}

/// Educational DeviceBuffer demonstrating device memory management and RAII.
pub struct EduDeviceBuffer<T: DeviceCopy> {
    ptr: *mut T,
    len: usize,
    context: Arc<EduContext>,
}

unsafe impl<T: DeviceCopy + Send> Send for EduDeviceBuffer<T> {}
unsafe impl<T: DeviceCopy + Sync> Sync for EduDeviceBuffer<T> {}

impl<T: DeviceCopy> EduDeviceBuffer<T> {
    /// Allocate device buffer and copy data from host slice.
    pub fn from_host(stream: &EduStream, data: &[T]) -> Result<Self, String> {
        let len = data.len();
        let layout = std::alloc::Layout::array::<T>(len).map_err(|e| e.to_string())?;

        let ptr = if layout.size() == 0 {
            std::ptr::NonNull::dangling().as_ptr()
        } else {
            // SAFETY: In educational CPU mode, we simulate GPU memory with aligned host allocation.
            let raw = unsafe { std::alloc::alloc(layout) as *mut T };
            if raw.is_null() {
                return Err("Failed to allocate device memory".to_string());
            }
            // SAFETY: Memory is valid, source slice has len elements, types are DeviceCopy.
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), raw, len);
            }
            raw
        };

        stream.context().active_allocations.fetch_add(1, Ordering::SeqCst);
        stream.synchronize()?;

        Ok(Self {
            ptr,
            len,
            context: stream.context().clone(),
        })
    }

    /// Allocate zero-initialized device buffer of given length.
    pub fn zeroed(stream: &EduStream, len: usize) -> Result<Self, String> {
        let layout = std::alloc::Layout::array::<T>(len).map_err(|e| e.to_string())?;
        let ptr = if layout.size() == 0 {
            std::ptr::NonNull::dangling().as_ptr()
        } else {
            // SAFETY: Allocate zeroed memory to simulate cuMemAlloc + cuMemsetD8.
            let raw = unsafe { std::alloc::alloc_zeroed(layout) as *mut T };
            if raw.is_null() {
                return Err("Failed to allocate zeroed device memory".to_string());
            }
            raw
        };

        stream.context().active_allocations.fetch_add(1, Ordering::SeqCst);
        stream.synchronize()?;

        Ok(Self {
            ptr,
            len,
            context: stream.context().clone(),
        })
    }

    /// Returns the number of elements in the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the buffer has zero elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total size in bytes.
    pub fn num_bytes(&self) -> usize {
        self.len * std::mem::size_of::<T>()
    }

    /// Raw pointer representing the device address.
    pub fn as_device_ptr(&self) -> *mut T {
        self.ptr
    }

    /// Copies data back to host vector and synchronizes stream.
    pub fn to_host_vec(&self, stream: &EduStream) -> Result<Vec<T>, String> {
        stream.synchronize()?;
        if self.len == 0 {
            return Ok(Vec::new());
        }
        let mut host = Vec::with_capacity(self.len);
        // SAFETY: The device buffer is valid and initialized, host buffer has allocated capacity.
        unsafe {
            std::ptr::copy_nonoverlapping(self.ptr, host.as_mut_ptr(), self.len);
            host.set_len(self.len);
        }
        Ok(host)
    }

    /// Borrow disjoint mutable sub-slices for data-race-free parallel kernels.
    pub fn split_at_mut(&mut self, mid: usize) -> (&mut [T], &mut [T]) {
        assert!(mid <= self.len, "Split index out of bounds");
        // SAFETY: Slices are disjoint and originate from the same contiguous allocation.
        unsafe {
            (
                std::slice::from_raw_parts_mut(self.ptr, mid),
                std::slice::from_raw_parts_mut(self.ptr.add(mid), self.len - mid),
            )
        }
    }
}

impl<T: DeviceCopy> Drop for EduDeviceBuffer<T> {
    fn drop(&mut self) {
        if self.len > 0 {
            if let Ok(layout) = std::alloc::Layout::array::<T>(self.len) {
                if layout.size() > 0 && !self.ptr.is_null() {
                    // SAFETY: Free the allocated device memory matching its original layout.
                    unsafe {
                        std::alloc::dealloc(self.ptr as *mut u8, layout);
                    }
                    self.context.active_allocations.fetch_sub(1, Ordering::SeqCst);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_buffer_roundtrip() {
        let ctx = EduContext::new(0).unwrap();
        let stream = ctx.new_stream().unwrap();
        let host_data = vec![1.0f32, 2.5, 3.75, 4.125];

        let dev_buf = EduDeviceBuffer::from_host(&stream, &host_data).unwrap();
        assert_eq!(dev_buf.len(), 4);
        assert_eq!(ctx.active_allocations(), 1);

        let retrieved = dev_buf.to_host_vec(&stream).unwrap();
        assert_eq!(retrieved, host_data);

        drop(dev_buf);
        assert_eq!(ctx.active_allocations(), 0);
    }

    #[test]
    fn test_disjoint_split() {
        let ctx = EduContext::new(0).unwrap();
        let stream = ctx.new_stream().unwrap();
        let mut dev_buf = EduDeviceBuffer::zeroed(&stream, 8).unwrap();

        let (left, right) = dev_buf.split_at_mut(4);
        left.fill(42);
        right.fill(99);

        let retrieved = dev_buf.to_host_vec(&stream).unwrap();
        assert_eq!(retrieved, vec![42, 42, 42, 42, 99, 99, 99, 99]);
    }
}
