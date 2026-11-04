//! Phase 1: CPU Memory Basics implementation and educational examples.

use std::rc::Rc;
use std::sync::Arc;

/// Demonstrates stack frame mechanics.
pub fn stack_allocation_example(x: i32) -> i32 {
    let y = x + 10;
    let z = y * 2;
    z
}

/// Demonstrates ownership transfer (move semantics).
pub fn ownership_move_example(val: String) -> usize {
    // val moved into this function
    val.len()
}

/// Demonstrates immutable and mutable borrowing.
pub fn borrowing_example(buf: &mut Vec<u8>) -> usize {
    buf.push(100);
    buf.len()
}

/// Demonstrates lifetime annotations on references.
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

/// Demonstrates smart pointer ownership and reference counts.
pub fn smart_pointers_example() -> (Box<i32>, Rc<String>, Arc<Vec<u8>>) {
    let b = Box::new(42);
    let r = Rc::new(String::from("cpu_rc"));
    let a = Arc::new(vec![1, 2, 3, 4]);
    (b, r, a)
}

/// Representation of memory layout and padding.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MixedLayout {
    pub a: u8,   // 1 byte
    // 3 bytes padding
    pub b: u32,  // 4 bytes
    pub c: u16,  // 2 bytes
    // 2 bytes padding to align struct to 4 bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_allocation() {
        assert_eq!(stack_allocation_example(5), 30);
    }

    #[test]
    fn test_ownership_move() {
        let s = String::from("Rust Memory");
        let len = ownership_move_example(s);
        assert_eq!(len, 11);
    }

    #[test]
    fn test_borrowing() {
        let mut v = vec![1, 2, 3];
        let len = borrowing_example(&mut v);
        assert_eq!(len, 4);
        assert_eq!(v[3], 100);
    }

    #[test]
    fn test_lifetimes() {
        let s1 = "short";
        let s2 = "longer_string";
        assert_eq!(longest(s1, s2), "longer_string");
    }

    #[test]
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
