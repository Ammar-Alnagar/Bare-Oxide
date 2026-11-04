//! Phase 3: GPU Memory with cuda-rust runtime patterns.

use common::{DeviceCopy, EduContext, EduDeviceBuffer, EduStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Educational demonstration of CudaContext and CudaStream orchestration.
pub struct GpuPipelineHarness {
    pub context: Arc<EduContext>,
    pub stream: Arc<EduStream>,
}

impl GpuPipelineHarness {
    pub fn new(ordinal: usize) -> Result<Self, String> {
        let context = EduContext::new(ordinal)?;
        let stream = context.new_stream()?;
        Ok(Self { context, stream })
    }

    pub fn allocate_and_upload<T: DeviceCopy>(
        &self,
        data: &[T],
    ) -> Result<EduDeviceBuffer<T>, String> {
        EduDeviceBuffer::from_host(&self.stream, data)
    }

    pub fn allocate_zeroed<T: DeviceCopy>(
        &self,
        len: usize,
    ) -> Result<EduDeviceBuffer<T>, String> {
        EduDeviceBuffer::zeroed(&self.stream, len)
    }
}

/// Simulated DeviceBox demonstrating stream-ordered asynchronous deallocation.
pub struct EduDeviceBox<T: DeviceCopy> {
    buffer: Option<EduDeviceBuffer<T>>,
    stream: Arc<EduStream>,
    async_freed: Arc<AtomicBool>,
}

impl<T: DeviceCopy> EduDeviceBox<T> {
    pub fn new(stream: Arc<EduStream>, value: T) -> Result<Self, String> {
        let slice = [value];
        let buffer = EduDeviceBuffer::from_host(&stream, &slice)?;
        Ok(Self {
            buffer: Some(buffer),
            stream,
            async_freed: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Free asynchronously on the stream.
    pub fn free_async(mut self) -> Result<(), String> {
        if let Some(buf) = self.buffer.take() {
            // Enqueue asynchronous deallocation on the stream
            self.stream.synchronize()?;
            drop(buf);
            self.async_freed.store(true, Ordering::SeqCst);
        }
        Ok(())
    }

    pub fn is_async_freed(&self) -> bool {
        self.async_freed.load(Ordering::SeqCst)
    }
}

impl<T: DeviceCopy> Drop for EduDeviceBox<T> {
    fn drop(&mut self) {
        if self.buffer.is_some() {
            // Synchronous fallback drop
            let _ = self.stream.synchronize();
            self.buffer.take();
        }
    }
}

/// Simulated kernel launch boundary with Rust ownership guarantees.
///
/// Immutable inputs (&DeviceBuffer) can be shared across multiple kernels.
/// Mutable inputs (&mut DeviceBuffer) guarantee exclusive access, preventing GPU data races.
pub fn launch_vector_scale_kernel(
    stream: &EduStream,
    input: &EduDeviceBuffer<f32>,
    output: &mut EduDeviceBuffer<f32>,
    scale: f32,
) -> Result<(), String> {
    assert_eq!(input.len(), output.len(), "Buffer sizes must match");
    stream.synchronize()?;

    // In native cuda-oxide, this emits cuLaunchKernel on the stream.
    // In our educational CPU runner, we perform the exact element-wise operation.
    // SAFETY: Slices are valid, non-overlapping because output is &mut.
    unsafe {
        let in_ptr = input.as_device_ptr();
        let out_ptr = output.as_device_ptr();
        for i in 0..input.len() {
            let val = *in_ptr.add(i);
            *out_ptr.add(i) = val * scale;
        }
    }

    stream.synchronize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_harness() {
        let harness = GpuPipelineHarness::new(0).unwrap();
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let dev_buf = harness.allocate_and_upload(&data).unwrap();
        assert_eq!(dev_buf.len(), 4);

        let downloaded = dev_buf.to_host_vec(&harness.stream).unwrap();
        assert_eq!(downloaded, data);
    }

    #[test]
    fn test_device_box_async_free() {
        let harness = GpuPipelineHarness::new(0).unwrap();
        let dev_box = EduDeviceBox::new(harness.stream.clone(), 42u64).unwrap();
        assert!(!dev_box.is_async_freed());

        dev_box.free_async().unwrap();
    }

    #[test]
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
