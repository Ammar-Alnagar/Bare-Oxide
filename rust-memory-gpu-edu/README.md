# Rust Memory & GPU Education Platform (`rust-memory-gpu-edu`)

Welcome to the comprehensive, low-level Rust systems programming course.
This project teaches CPU and GPU memory mechanics, custom allocators, safe abstractions over unsafe memory operations, and GPU computing using the official **NVIDIA `cuda-rust`** platform.

All modules follow **ASD-STE100** simplified technical English rules:
- Short, direct sentences.
- Active voice.
- Explicit safety contracts (`// SAFETY:`).
- Detailed Mermaid architectural diagrams.
- Line-by-line code breakdowns, best practices, common pitfalls, and hands-on exercises.

---

## 1. Project Structure

```
rust-memory-gpu-edu/
├── README.md                      # Project overview, hardware/software prerequisites, build guide
├── Cargo.toml                     # Cargo workspace definition
├── common/                        # Shared abstractions, traits, and educational runtime harness
│   ├── Cargo.toml
│   └── src/lib.rs
├── phase-01-cpu-basics/           # Phase 1: CPU Memory Basics
│   ├── Cargo.toml
│   ├── src/lib.rs
│   ├── lesson-01-stack-heap.md
│   ├── lesson-02-ownership-borrowing.md
│   ├── lesson-03-lifetimes.md
│   ├── lesson-04-smart-pointers.md
│   └── lesson-05-layout-and-alignment.md
├── phase-02-cpu-low-level/        # Phase 2: CPU Low-Level Memory & Custom Allocators
│   ├── Cargo.toml
│   ├── src/lib.rs
│   ├── lesson-01-raw-pointers-unsafe.md
│   ├── lesson-02-std-alloc-layout.md
│   ├── lesson-03-global-allocator.md
│   ├── lesson-04-custom-allocators.md
│   └── lesson-05-manual-drop-leak-safe-wrappers.md
├── phase-03-gpu-memory-basics/    # Phase 3: GPU Memory with cuda-rust
│   ├── Cargo.toml
│   ├── src/lib.rs
│   ├── lesson-01-cuda-context-stream.md
│   ├── lesson-02-devicebuffer.md
│   ├── lesson-03-host-device-copy.md
│   ├── lesson-04-devicebox-async-free.md
│   └── lesson-05-ownership-and-launch-boundary.md
├── phase-04-gpu-advanced/         # Phase 4: Advanced GPU Memory Management
│   ├── Cargo.toml
│   ├── src/lib.rs
│   ├── lesson-01-shared-memory.md
│   ├── lesson-02-constant-memory.md
│   ├── lesson-03-disjoint-slice.md
│   ├── lesson-04-pinned-and-unified-memory.md
│   ├── lesson-05-stream-ordered-allocation.md
│   └── lesson-06-kernel-models-simt-and-tile.md
└── phase-05-full-pipeline/        # Phase 5: Complete End-to-End Pipeline
    ├── Cargo.toml
    ├── src/main.rs
    └── lesson-01-end-to-end-example.md
```

---

## 2. Hardware and Software Requirements

### Hardware Requirements
- **Host CPU**: x86_64 or aarch64 CPU with at least 8 GB RAM.
- **GPU (for live GPU execution)**:
  - NVIDIA GPU with Compute Capability 7.0 or newer (Volta, Turing, Ampere, Ada Lovelace, Hopper, Blackwell).
  - Recommended Compute Capability: 8.0+ (Ampere / Ada Lovelace / Hopper) for full support of asynchronous allocations (`cuMemAllocAsync`), thread block clusters, and asynchronous barriers.
- **CPU-Only / Local CI Testing**:
  - All lessons contain runnable code and unit tests.
  - In environments without an NVIDIA GPU or CUDA driver installed, the included workspace compiles and tests successfully using the educational runtime abstraction.

### Software Requirements
- **Rust Toolchain**:
  - Stable Rust 1.89 or newer (tested with Rust 1.94+).
  - For `cuda-oxide` SIMT compilation: pinned toolchain Rust `1.99.1` with `rustc-dev` and `llvm-tools` (managed via `cargo oxide`).
- **CUDA Toolkit**:
  - CUDA Toolkit 12.0 or newer (recommended 12.4+).
  - CUDA Driver matching or exceeding the toolkit version.
- **Git**:
  - Required to pull repository dependencies and submodules.

---

## 3. How to Build and Run Each Lesson

From the `rust-memory-gpu-edu` directory:

```bash
# Build the entire workspace
cargo build

# Run all automated tests across all 5 phases
cargo test

# Run tests for a specific phase
cargo test -p phase-01-cpu-basics
cargo test -p phase-02-cpu-low-level
cargo test -p phase-03-gpu-memory-basics
cargo test -p phase-04-gpu-advanced
cargo test -p phase-05-full-pipeline

# Run the complete end-to-end Phase 5 pipeline
cargo run -p phase-05-full-pipeline
```

---

## 4. Known Limitations of the Current `cuda-rust` Release

As documented in the official [NVIDIA cuda-rust repository](https://github.com/NVIDIA/cuda-rust):
1. **Early Research Release**: `cuda-rust` is an active research project. Kernel compiler interfaces and runtime driver bindings may undergo breaking changes across releases.
2. **Pinned Toolchains**: `cuda-oxide` SIMT model requires a pinned compiler version (Rust 1.99.1) because it relies on private `rustc_codegen_ssa` internal compiler APIs.
3. **Synchronous Drop vs Asynchronous Free**: `DeviceBuffer<T>` drop executes synchronous `cuMemFree` unless stream-ordered completion leases or asynchronous drop wrappers are used. Dropping buffers while asynchronous stream work is in-flight without synchronization will result in driver error or undefined behavior.
4. **Hardware-Specific Features**: Advanced capabilities such as Thread Block Clusters require Compute Capability 9.0+ (Hopper). Dynamic shared memory opt-in sizes exceeding 48 KiB require explicit device attribute queries.
