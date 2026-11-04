//! Phase 5: Complete End-to-End CPU Arena to GPU Pipeline.

use common::{EduContext, EduDeviceBuffer, EduStream};
use phase_02_cpu_low_level::BumpAllocator;
use std::alloc::Layout;
use std::sync::Arc;

/// Vector Add SIMT Kernel simulation.
///
/// In native cuda-oxide, this represents an AOT-compiled SIMT kernel:
/// ```ignore
/// #[cuda_kernel]
/// pub fn vector_add(a: &[f32], b: &[f32], c: &mut [f32]) {
///     let idx = thread_idx().x + block_idx().x * block_dim().x;
///     if idx < c.len() {
///         c[idx] = a[idx] + b[idx];
///     }
/// }
/// ```
pub fn launch_vector_add_simt(
    stream: &EduStream,
    a: &EduDeviceBuffer<f32>,
    b: &EduDeviceBuffer<f32>,
    c: &mut EduDeviceBuffer<f32>,
) -> Result<(), String> {
    assert_eq!(a.len(), b.len(), "Buffer lengths must match");
    assert_eq!(a.len(), c.len(), "Buffer lengths must match");

    stream.synchronize()?;

    // SAFETY: Input buffers are shared reads, output buffer is exclusive write.
    // In native cuda-oxide, this executes cuLaunchKernel on the stream.
    unsafe {
        let ptr_a = a.as_device_ptr();
        let ptr_b = b.as_device_ptr();
        let ptr_c = c.as_device_ptr();

        for i in 0..c.len() {
            *ptr_c.add(i) = *ptr_a.add(i) + *ptr_b.add(i);
        }
    }

    stream.synchronize()?;
    Ok(())
}

/// Runs the complete end-to-end educational memory pipeline:
/// 1. Allocate host input memory using a custom Bump Allocator.
/// 2. Populate host data and upload to GPU `DeviceBuffer` instances over PCIe.
/// 3. Execute SIMT vector addition kernel.
/// 4. Download output vector back to host.
/// 5. Automatically deallocate all host and GPU memory via RAII.
pub fn run_end_to_end_pipeline(n: usize) -> Result<Vec<f32>, String> {
    println!("=== Phase 5: Executing Complete End-to-End Pipeline ===");

    // Step 1: Initialize Custom Host Arena / Bump Allocator
    let byte_size = n * std::mem::size_of::<f32>();
    let arena = BumpAllocator::new(byte_size * 2 + 1024);
    println!("1. Host custom bump arena initialized (capacity: {} bytes)", byte_size * 2 + 1024);

    let layout = Layout::array::<f32>(n).map_err(|e| e.to_string())?;

    // SAFETY: Memory allocated within arena capacity; populated with valid f32 values.
    let host_a = unsafe {
        let p = arena.alloc(layout).expect("Arena alloc A failed") as *mut f32;
        for i in 0..n {
            std::ptr::write(p.add(i), i as f32 * 1.0);
        }
        std::slice::from_raw_parts(p, n)
    };

    // SAFETY: Memory allocated within arena capacity; populated with valid f32 values.
    let host_b = unsafe {
        let p = arena.alloc(layout).expect("Arena alloc B failed") as *mut f32;
        for i in 0..n {
            std::ptr::write(p.add(i), i as f32 * 2.0);
        }
        std::slice::from_raw_parts(p, n)
    };
    println!("2. Host memory populated with {} elements via custom allocator", n);

    // Step 2: Initialize CUDA Context and Stream
    let context: Arc<EduContext> = EduContext::new(0)?;
    let stream: Arc<EduStream> = context.new_stream()?;
    println!("3. CUDA context and non-blocking stream initialized on device 0");

    // Step 3: Upload host memory to GPU DeviceBuffer
    let dev_a = EduDeviceBuffer::from_host(&stream, host_a)?;
    let dev_b = EduDeviceBuffer::from_host(&stream, host_b)?;
    let mut dev_c = EduDeviceBuffer::<f32>::zeroed(&stream, n)?;
    println!("4. Data uploaded to GPU DeviceBuffers across PCIe bus");

    // Step 4: Launch SIMT Vector Add Kernel
    println!("5. Launching SIMT vector addition kernel on GPU stream");
    launch_vector_add_simt(&stream, &dev_a, &dev_b, &mut dev_c)?;

    // Step 5: Download results back to host
    let results = dev_c.to_host_vec(&stream)?;
    println!("6. Results successfully copied back to host vector");

    // Step 6: Memory Cleanup
    // dev_a, dev_b, dev_c drop here, issuing cuMemFree
    // arena drops here, deallocating the backing chunk cleanly
    println!("7. All host and GPU resources reclaimed cleanly via RAII");
    println!("=== Pipeline Execution Succeeded ===");

    Ok(results)
}

fn main() {
    let count = 8;
    match run_end_to_end_pipeline(count) {
        Ok(results) => {
            println!("Output vector: {:?}", results);
            for (i, val) in results.iter().enumerate() {
                let expected = (i as f32 * 1.0) + (i as f32 * 2.0);
                assert!((val - expected).abs() < f32::EPSILON);
            }
            println!("Verification check: ALL OUTPUT VALUES MATCH EXPECTATIONS!");
        }
        Err(e) => {
            eprintln!("Pipeline execution failed: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_to_end_pipeline_execution() {
        let results = run_end_to_end_pipeline(16).unwrap();
        assert_eq!(results.len(), 16);
        for i in 0..16 {
            let expected = (i as f32 * 1.0) + (i as f32 * 2.0);
            assert_eq!(results[i], expected);
        }
    }
}
