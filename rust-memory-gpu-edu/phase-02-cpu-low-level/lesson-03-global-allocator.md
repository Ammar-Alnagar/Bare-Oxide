# Lesson 3: The `GlobalAlloc` Trait and `#[global_allocator]`

## Learning Goals
- Understand the role of the `GlobalAlloc` trait in the Rust runtime.
- Implement a custom global allocator wrapper that tracks memory statistics.
- Register an allocator globally using the `#[global_allocator]` attribute.
- Analyze thread-safety and reentrancy challenges in memory allocators.

---

## 1. Theory: Rust Global Memory Management

Every allocation produced by `Box`, `Vec`, `String`, or `std::alloc::alloc` routes through the type registered with `#[global_allocator]`.

By default, Rust uses the system allocator (such as `malloc` / `free` in `libc`). Developers can replace this with custom allocators (like `jemalloc`, `mimalloc`, or diagnostic tracking allocators).

### The `GlobalAlloc` Trait
To serve as a global allocator, a type must implement `std::alloc::GlobalAlloc`:
```rust
pub unsafe trait GlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);
    // Optional methods:
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 { ... }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 { ... }
}
```

### Safety Requirements
1. Implementations must be thread-safe (`Sync`). Allocations can occur concurrently across multiple threads.
2. Allocations must not cause infinite recursion. Calling code that allocates memory inside `alloc()` causes a stack overflow crash!

---

## 2. Global Allocation Routing Diagram

```mermaid
graph TD
    subgraph UserSpace["Application Code"]
        VEC["Vec::push(...)"]
        BOX["Box::new(...)"]
    end

    subgraph AllocatorSubsystem["Memory Allocation Boundary"]
        GLOBAL["#[global_allocator]<br/>Custom Tracking Allocator"]
        SYSTEM["std::alloc::System (OS malloc/free)"]
    end

    subgraph TrackingEngine["Telemetry"]
        STATS["Atomic Counters: Total Bytes Allocated"]
    end

    VEC -->|alloc(layout)| GLOBAL
    BOX -->|alloc(layout)| GLOBAL
    GLOBAL -->|Update Metrics| STATS
    GLOBAL -->|Forward Call| SYSTEM
```

---

## 3. Complete Rust Code Example

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TrackingAllocator {
    allocated_bytes: AtomicUsize,
    deallocated_bytes: AtomicUsize,
}

impl TrackingAllocator {
    pub const fn new() -> Self {
        Self {
            allocated_bytes: AtomicUsize::new(0),
            deallocated_bytes: AtomicUsize::new(0),
        }
    }

    pub fn current_usage(&self) -> usize {
        let alloc = self.allocated_bytes.load(Ordering::Relaxed);
        let dealloc = self.deallocated_bytes.load(Ordering::Relaxed);
        alloc.saturating_sub(dealloc)
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Forwarding directly to standard System allocator.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            self.allocated_bytes.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: Forwarding directly to standard System allocator.
        unsafe {
            System.dealloc(ptr, layout);
        }
        self.deallocated_bytes.fetch_add(layout.size(), Ordering::Relaxed);
    }
}

// In a binary crate, you register the global allocator like this:
// #[global_allocator]
// static GLOBAL: TrackingAllocator = TrackingAllocator::new();

fn main() {
    let tracker = TrackingAllocator::new();
    let layout = Layout::from_size_align(128, 8).unwrap();

    // Demonstrate manual invocation of the GlobalAlloc implementation
    // SAFETY: Layout is valid.
    let p = unsafe { tracker.alloc(layout) };
    assert!(!p.is_null());
    assert_eq!(tracker.current_usage(), 128);

    // SAFETY: p was allocated by tracker with identical layout.
    unsafe {
        tracker.dealloc(p, layout);
    }
    assert_eq!(tracker.current_usage(), 0);
    println!("Tracking allocator successfully measured memory lifecycle.");
}
```

---

## 4. Code Breakdown

1. `struct TrackingAllocator`:
   - Holds two `AtomicUsize` counters to track allocated and freed bytes across threads without locks.
2. `unsafe impl GlobalAlloc for TrackingAllocator`:
   - `alloc`: Calls `System.alloc(layout)`. If allocation succeeds, adds `layout.size()` to the counter.
   - `dealloc`: Calls `System.dealloc(ptr, layout)`. Adds `layout.size()` to `deallocated_bytes`.
3. `current_usage()`:
   - Computes live heap footprint: `allocated - deallocated`.
4. `#[global_allocator]`:
   - Attribute placed on a static item to bind the allocator for the entire compiled binary.

---

## 5. Best Practices

1. **Avoid all heap allocation inside `alloc` and `dealloc`**:
   - Do not print with `println!` using string formatting inside an allocator. Standard formatting allocates internally, causing reentrant allocator deadlocks.
2. **Use atomic operations with `Relaxed` ordering for metrics**:
   - Telemetry counters do not synchronize program logic, so `Relaxed` ordering reduces CPU cache overhead.
3. **Use battle-tested allocators in production**:
   - In production microservices or high-throughput batch systems, register `jemallocator` or `mimalloc` to reduce fragmentation.

---

## 6. Common Errors

- **Reentrancy Deadlock**: Formatting strings or acquiring locks that allocate inside `alloc()`.
  - *Fix*: Use lock-free primitives and raw OS calls (or static buffers) within allocator internals.
- **Mismatched size on deallocation**: Subtracting an incorrect size corrupts telemetry.
  - *Fix*: Rely strictly on `layout.size()`.

---

## 7. Exercises

1. Extend `TrackingAllocator` to record the total number of allocation calls (an allocation counter) alongside byte counts.
2. Implement a watermark function `high_watermark(&self) -> usize` that records the maximum concurrent bytes allocated at any time.

---

## 8. Summary & Checklist

- [x] `GlobalAlloc` defines the low-level interface for global memory management.
- [x] `#[global_allocator]` replaces the default allocator across the binary.
- [x] Custom allocators must be thread-safe (`Sync`).
- [x] Allocator functions must avoid reentrant allocations.
