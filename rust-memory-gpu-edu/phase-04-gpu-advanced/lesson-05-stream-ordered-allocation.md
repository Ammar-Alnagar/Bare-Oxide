# Lesson 5: Stream-Ordered Allocation and Memory Pools (`MemPool`)

## Learning Goals
- Understand GPU memory fragmentation and allocation latency.
- Learn the CUDA Virtual Memory Management (VMM) and Memory Pool (`cuMemPool`) architecture.
- Master `MemPool` watermarks and release threshold tuning.
- Track memory metrics using `PoolMemStats`.

---

## 1. Theory: The Driver Memory Pool Architecture

Frequent calls to `cuMemAlloc` and `cuMemFree` cause severe performance problems:
- Each call requires an OS driver context switch and acquires global synchronization locks.
- Interleaved allocations and frees cause device VRAM fragmentation.

### CUDA Memory Pools (`cuMemPool`)
CUDA 11.2+ introduced driver-managed memory pools:
- When a stream frees memory (`cuMemFreeAsync`), the memory is **not returned to the OS**.
- The freed block is retained inside an on-device memory pool (`MemPool`).
- Subsequent `cuMemAllocAsync` calls on that stream reuse cached blocks instantly in $O(1)$ time with **zero driver lock contention**.

### Release Threshold Tuning
The `release_threshold` controls how aggressively the pool returns memory to the OS:
- Setting `threshold = 0`: Memory is returned to the OS as soon as the pool becomes idle.
- Setting `threshold = u64::MAX`: Memory remains reserved indefinitely in the pool. This eliminates all allocation overhead for stable production workloads (such as neural network inference).

---

## 2. Memory Pool Lifecycle Diagram

```mermaid
graph TD
    subgraph HostApp["Application Layer"]
        A1["Alloc Request 1 (128 MB)"]
        F1["Free Request 1 (128 MB)"]
        A2["Alloc Request 2 (128 MB)"]
    end

    subgraph DriverPool["MemPool (Driver Memory Pool)"]
        RESERVED["Reserved Memory Block (128 MB)<br/>Retained on GPU VRAM"]
    end

    subgraph OSVRAM["Operating System / GPU Driver"]
        OS_ALLOC["cuMemAlloc (Heavyweight OS Call)"]
    end

    A1 -->|Initial Miss: Calls OS| OS_ALLOC
    OS_ALLOC --> RESERVED
    F1 -->|Recycled into Pool (0 latency)| RESERVED
    A2 -->|Instant Cache Hit! (No OS Call)| RESERVED
```

---

## 3. Complete Rust Code Example

```rust
use common::{EduContext, EduStream};
use phase_04_gpu_advanced::EduMemPool;
use std::sync::Arc;

fn main() -> Result<(), String> {
    let context = EduContext::new(0)?;
    let stream = context.new_stream()?;

    // 1. Initialize Educational Memory Pool with 1 GB reserved capacity
    let pool = EduMemPool::new(1024 * 1024 * 1024);

    // 2. Simulate streaming allocations and deallocations
    println!("Step 1: Allocate 256 MB tensor buffer");
    pool.allocate(256 * 1024 * 1024)?;
    println!("Used: {} MB, High Watermark: {} MB",
        pool.used_current() / (1024 * 1024),
        pool.high_watermark() / (1024 * 1024));

    println!("Step 2: Allocate another 512 MB tensor buffer");
    pool.allocate(512 * 1024 * 1024)?;
    println!("Used: {} MB, High Watermark: {} MB",
        pool.used_current() / (1024 * 1024),
        pool.high_watermark() / (1024 * 1024));

    println!("Step 3: Free the 256 MB buffer into the pool");
    pool.deallocate(256 * 1024 * 1024);
    println!("Used: {} MB, High Watermark: {} MB",
        pool.used_current() / (1024 * 1024),
        pool.high_watermark() / (1024 * 1024));

    // High watermark accurately preserves peak memory usage
    assert_eq!(pool.high_watermark() / (1024 * 1024), 768);

    stream.synchronize()?;
    Ok(())
}
```

---

## 4. Code Breakdown

1. `EduMemPool::new(...)`:
   - Wraps CUDA `cuMemPoolCreate` logic.
   - Pre-reserves address space to prevent OS allocation churn.
2. `pool.allocate(bytes)`:
   - Uses `cuMemAllocFromPoolAsync`.
   - Serviced from previously freed pool blocks if available.
3. `pool.high_watermark()`:
   - Tracks peak memory usage across the application lifecycle.
4. `release_threshold` (in native `cuda-core`):
   - Configured via `pool.set_release_threshold(u64::MAX)` to prevent the driver from releasing memory during steady-state execution.

---

## 5. Best Practices

1. **Set `release_threshold` to `u64::MAX` for iterative pipelines**: Eliminates OS allocation overhead in long-running services.
2. **Synchronize streams before reading pool memory stats**: Memory stats are updated asynchronously by GPU stream workers.
3. **Use the default pool when custom isolation is not required**: Access the default pool via `context.default_mem_pool()`.
4. **Reset watermarks between test benchmarks**: Use `reset_used_high()` to isolate peak memory measurements for individual workload phases.

---

## 6. Common Errors

- **Assuming pool stats are synchronously consistent without stream sync**:
  - Reading `mem_stats` while kernels are actively allocating yields stale or partial snapshots.
  - *Fix*: Call `stream.synchronize()` before inspecting metrics.
- **Leaking pooled memory across independent tasks**:
  - Failing to free stream-ordered buffers prevents the pool from recycling memory chunks.
  - *Fix*: Ensure all `DeviceBuffer` instances drop cleanly.

---

## 7. Exercises

1. Write a test that simulates 1,000 rapid allocate/free cycles and verify that peak memory never exceeds the size of a single allocation.
2. Explain the difference between `used_current` (checked out to buffers) and `reserved_current` (held by the pool from the OS).

---

## 8. Summary & Checklist

- [x] `cuMemPool` caches freed memory blocks to eliminate driver allocation latency.
- [x] Stream-ordered allocation recycles memory in $O(1)$ time.
- [x] Release thresholds control when pool memory is returned to the OS.
- [x] High watermarks track peak memory consumption accurately.
