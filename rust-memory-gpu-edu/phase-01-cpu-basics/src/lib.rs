#![allow(unused_variables, dead_code, unused_mut, unused_imports)]
//! Phase 1: CPU Memory Basics - Student Exercises
//!
//! Complete the functions and structures below according to the lesson specifications.
//! Run `cargo test -p phase-01-cpu-basics` to verify your implementations.

use std::rc::Rc;
use std::sync::Arc;

/// EXERCISE 1: Stack Allocation Mechanics
///
/// Compute `(x + 10) * 2` using only stack-allocated local variables.
pub fn stack_allocation_example(x: i32) -> i32 {
    // TODO: Allocate local variables on the stack frame and return the result.
    todo!("Exercise 1: Implement stack_allocation_example using local stack variables")
}

/// EXERCISE 2: Ownership Move Semantics
///
/// Take ownership of `val` by value and return its byte length (`val.len()`).
pub fn ownership_move_example(val: String) -> usize {
    // TODO: Accept `val` by value (transferring ownership) and return its length.
    todo!("Exercise 2: Implement ownership_move_example")
}

/// EXERCISE 3: Mutable Borrowing
///
/// Append the byte `100` to the borrowed vector `buf` and return its new length.
pub fn borrowing_example(buf: &mut Vec<u8>) -> usize {
    // TODO: Borrow `buf` mutably, push byte 100, and return its length.
    todo!("Exercise 3: Implement borrowing_example with mutable reference")
}

/// EXERCISE 4: Lifetime Annotations
///
/// Compare string slices `x` and `y` and return the longer slice.
/// If equal in length, return `x`.
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    // TODO: Return the longer string slice with lifetime `'a`.
    todo!("Exercise 4: Implement longest with lifetime annotations")
}

/// EXERCISE 5: Smart Pointers
///
/// Construct a tuple containing:
/// 1. A `Box<i32>` storing 42.
/// 2. An `Rc<String>` storing "cpu_rc".
/// 3. An `Arc<Vec<u8>>` storing `vec![1, 2, 3, 4]`.
pub fn smart_pointers_example() -> (Box<i32>, Rc<String>, Arc<Vec<u8>>) {
    // TODO: Create and return the three smart pointer types.
    todo!("Exercise 5: Implement smart_pointers_example")
}

/// EXERCISE 6: Struct Memory Layout and Alignment
///
/// Define a C-compatible struct `MixedLayout` with fields:
/// - `a: u8`
/// - `b: u32`
/// - `c: u16`
/// Annotate with `#[repr(C)]` so that padding aligns with C layout rules (total size 12 bytes).
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MixedLayout {
    pub a: u8,
    pub b: u32,
    pub c: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Student exercise: run with -- --ignored to test once implemented"]
    fn test_stack_allocation() {
        assert_eq!(stack_allocation_example(5), 30);
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored to test once implemented"]
    fn test_ownership_move() {
        let s = String::from("Rust Memory");
        let len = ownership_move_example(s);
        assert_eq!(len, 11);
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored to test once implemented"]
    fn test_borrowing() {
        let mut v = vec![1, 2, 3];
        let len = borrowing_example(&mut v);
        assert_eq!(len, 4);
        assert_eq!(v[3], 100);
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored to test once implemented"]
    fn test_lifetimes() {
        let s1 = "short";
        let s2 = "longer_string";
        assert_eq!(longest(s1, s2), "longer_string");
    }

    #[test]
    #[ignore = "Student exercise: run with -- --ignored to test once implemented"]
    fn test_smart_pointers() {
        let (b, r, a) = smart_pointers_example();
        assert_eq!(*b, 42);
        let r_clone = Rc::clone(&r);
        assert_eq!(Rc::strong_count(&r), 2);
        drop(r_clone);
        assert_eq!(Rc::strong_count(&r), 1);
        assert_eq!(a.len(), 4);
    }

    #[test]
    fn test_layout_and_alignment() {
        assert_eq!(std::mem::size_of::<MixedLayout>(), 12);
        assert_eq!(std::mem::align_of::<MixedLayout>(), 4);
    }
}
