#![allow(unused_variables, dead_code, unused_mut, unused_imports)]
//! Phase 3: GPU Memory with cuda-rust - Student Exercises
//!
//! Implement GPU pipeline harnesses, DeviceBox asynchronous frees, and safe launch boundaries.
//! Run `cargo test -p phase-03-gpu-memory-basics` to verify your implementations.

use common::{DeviceCopy, EduContext, EduDeviceBuffer, EduStream};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// EXERCISE 1: GPU Pipeline Harness
pub struct GpuPipelineHarness {
    pub context: Arc<EduContext>,
    pub stream: Arc<EduStream>,
}

impl GpuPipelineHarness {
    pub fn new(ordinal: usize) -> Result<Self, String> {
        // TODO: Create EduContext for device ordinal, then create a new stream.
        todo!("Exercise 1A: Implement GpuPipelineHarness::new")
    }

    pub fn allocate_and_upload<T: DeviceCopy>(
        &self,
        data: &[T],
    ) -> Result<EduDeviceBuffer<T>, String> {
        // TODO: Call EduDeviceBuffer::from_host using self.stream.
        todo!("Exercise 1B: Implement GpuPipelineHarness::allocate_and_upload")
    }

    pub fn allocate_zeroed<T: DeviceCopy>(
        &self,
        len: usize,
    ) -> Result<EduDeviceBuffer<T>, String> {
        // TODO: Call EduDeviceBuffer::zeroed using self.stream.
        todo!("Exercise 1C: Implement GpuPipelineHarness::allocate_zeroed")
    }
}

/// EXERCISE 2: DeviceBox with Stream-Ordered Asynchronous Free
pub struct EduDeviceBox<T: DeviceCopy> {
    buffer: Option<EduDeviceBuffer<T>>,
    stream: Arc<EduStream>,
    async_freed: Arc<AtomicBool>,
}

impl<T: DeviceCopy> EduDeviceBox<T> {
    pub fn new(stream: Arc<EduStream>, value: T) -> Result<Self, String> {
        // TODO: Allocate single-element device buffer and initialize with value.
        todo!("Exercise 2A: Implement EduDeviceBox::new")
    }

    pub fn free_async(mut self) -> Result<(), String> {
        // TODO: Synchronize stream, take and drop the buffer, and set async_freed flag.
        todo!("Exercise 2B: Implement EduDeviceBox::free_async")
    }

    pub fn is_async_freed(&self) -> bool {
        todo!("Exercise 2C: Return whether the buffer has been asynchronously freed")
    }
}

impl<T: DeviceCopy> Drop for EduDeviceBox<T> {
    fn drop(&mut self) {
        // Synchronous fallback deallocation
    }
}

/// EXERCISE 3: Kernel Launch Boundary with Rust Ownership
pub fn launch_vector_scale_kernel(
    stream: &EduStream,
    input: &EduDeviceBuffer<f32>,
    output: &mut EduDeviceBuffer<f32>,
    scale: f32,
) -> Result<(), String> {
    // TODO: Verify lengths match, synchronize stream, multiply input by scale, write to output.
    todo!("Exercise 3: Implement launch_vector_scale_kernel")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
    fn test_pipeline_harness() {
        let harness = GpuPipelineHarness::new(0).unwrap();
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let dev_buf = harness.allocate_and_upload(&data).unwrap();
        assert_eq!(dev_buf.len(), 4);

        let downloaded = dev_buf.to_host_vec(&harness.stream).unwrap();
        assert_eq!(downloaded, data);
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
    fn test_device_box_async_free() {
        let harness = GpuPipelineHarness::new(0).unwrap();
        let dev_box = EduDeviceBox::new(harness.stream.clone(), 42u64).unwrap();
        assert!(!dev_box.is_async_freed());

        dev_box.free_async().unwrap();
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
    fn test_kernel_launch_safety() {
        let harness = GpuPipelineHarness::new(0).unwrap();
        let input_data = vec![2.0f32, 4.0, 8.0, 16.0];
        let in_buf = harness.allocate_and_upload(&input_data).unwrap();
        let mut out_buf = harness.allocate_zeroed::<f32>(4).unwrap();

        launch_vector_scale_kernel(&harness.stream, &in_buf, &mut out_buf, 2.5).unwrap();

        let result = out_buf.to_host_vec(&harness.stream).unwrap();
        assert_eq!(result, vec![5.0f32, 10.0, 20.0, 40.0]);
    }
}
