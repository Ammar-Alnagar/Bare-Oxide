#![allow(unused_variables, dead_code, unused_mut, unused_imports)]
//! Phase 5: Complete End-to-End Pipeline - Student Challenge
//!
//! Unite the custom host arena with GPU device buffers and an SIMT vector add kernel.
//! Run `cargo test -p phase-05-full-pipeline` to verify your implementation.

use common::{EduContext, EduDeviceBuffer, EduStream};
use phase_02_cpu_low_level::BumpAllocator;
use std::alloc::Layout;
use std::sync::Arc;

/// EXERCISE 1: SIMT Vector Add Kernel
pub fn launch_vector_add_simt(
    stream: &EduStream,
    a: &EduDeviceBuffer<f32>,
    b: &EduDeviceBuffer<f32>,
    c: &mut EduDeviceBuffer<f32>,
) -> Result<(), String> {
    // TODO: Verify equal lengths, synchronize stream, add elements of a and b, write to c.
    todo!("Exercise 1: Implement launch_vector_add_simt")
}

/// EXERCISE 2: Full End-to-End Pipeline
///
/// 1. Initialize custom host BumpAllocator.
/// 2. Allocate and initialize host arrays `host_a` and `host_b`.
/// 3. Create EduContext and EduStream.
/// 4. Upload `host_a` and `host_b` to GPU DeviceBuffers.
/// 5. Allocate zeroed GPU DeviceBuffer `dev_c`.
/// 6. Launch SIMT vector addition kernel.
/// 7. Download result vector to host and return it.
pub fn run_end_to_end_pipeline(n: usize) -> Result<Vec<f32>, String> {
    // TODO: Connect host arena allocation to GPU kernel execution and host result retrieval.
    todo!("Exercise 2: Implement run_end_to_end_pipeline")
}

fn main() {
    println!("=== Phase 5: Student Pipeline Challenge ===");
    println!("Implement run_end_to_end_pipeline and run with `cargo run -p phase-05-full-pipeline`");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Student exercise: run with -- --ignored once implemented"]
    fn test_end_to_end_pipeline_execution() {
        let results = run_end_to_end_pipeline(16).unwrap();
        assert_eq!(results.len(), 16);
        for i in 0..16 {
            let expected = (i as f32 * 1.0) + (i as f32 * 2.0);
            assert_eq!(results[i], expected);
        }
    }
}
