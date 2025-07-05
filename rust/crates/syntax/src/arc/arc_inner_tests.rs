//! Tests for ArcInner functionality
//!
//! These tests verify that ArcInner works correctly as the internal structure
//! for Arc. They test basic creation, atomic operations, and various data types.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::arc::arc_inner::ArcInner;

#[test]
fn test_arc_inner_creation() {
    // Test creating ArcInner with a simple type
    let inner = ArcInner {
        count: AtomicUsize::new(1),
        data: 42i32,
    };

    assert_eq!(inner.count.load(Ordering::Relaxed), 1);
    assert_eq!(inner.data, 42);
}

#[test]
fn test_arc_inner_with_string() {
    // Test creating ArcInner with a String (heap-allocated data)
    let test_string = String::from("Hello, World!");
    let inner = ArcInner {
        count: AtomicUsize::new(1),
        data: test_string.clone(),
    };

    assert_eq!(inner.count.load(Ordering::Relaxed), 1);
    assert_eq!(inner.data, test_string);
}

#[test]
fn test_arc_inner_count_operations() {
    // Test atomic operations on the count field (simulating Arc clone/drop)
    let inner = ArcInner {
        count: AtomicUsize::new(1),
        data: "test data",
    };

    // Test increment
    let old_count = inner.count.fetch_add(1, Ordering::Relaxed);
    assert_eq!(old_count, 1);
    assert_eq!(inner.count.load(Ordering::Relaxed), 2);

    // Test decrement
    let old_count = inner.count.fetch_sub(1, Ordering::Relaxed);
    assert_eq!(old_count, 2);
    assert_eq!(inner.count.load(Ordering::Relaxed), 1);
}

#[test]
fn test_arc_inner_send_sync_properties() {
    // Test that ArcInner implements Send and Sync for appropriate types
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    // Test with types that are Send + Sync
    assert_send::<ArcInner<i32>>();
    assert_sync::<ArcInner<i32>>();

    assert_send::<ArcInner<String>>();
    assert_sync::<ArcInner<String>>();
}

#[test]
fn test_arc_inner_memory_layout() {
    // Test the #[repr(C)] layout
    let inner = ArcInner {
        count: AtomicUsize::new(1),
        data: 42i32,
    };

    // Verify that the count field comes first in memory
    let inner_ptr = &inner as *const ArcInner<i32>;
    let count_ptr = &inner.count as *const AtomicUsize;

    assert_eq!(inner_ptr as usize, count_ptr as usize);

    // Verify alignment expectations
    assert_eq!(
        std::mem::align_of::<ArcInner<i32>>(),
        std::mem::align_of::<AtomicUsize>()
    );
}

#[test]
fn test_arc_inner_with_zero_sized_type() {
    // Test ArcInner with zero-sized type
    let inner = ArcInner {
        count: AtomicUsize::new(1),
        data: (),
    };

    assert_eq!(inner.count.load(Ordering::Relaxed), 1);
    assert_eq!(inner.data, ());

    // ZST should not affect the size much
    assert_eq!(
        std::mem::size_of::<ArcInner<()>>(),
        std::mem::size_of::<AtomicUsize>()
    );
}

#[test]
fn test_arc_inner_with_large_data() {
    // Test ArcInner with larger data structures
    let large_vec: Vec<i32> = (0..1000).collect();
    let inner = ArcInner {
        count: AtomicUsize::new(1),
        data: large_vec.clone(),
    };

    assert_eq!(inner.count.load(Ordering::Relaxed), 1);
    assert_eq!(inner.data, large_vec);
    assert_eq!(inner.data.len(), 1000);
}

#[test]
fn test_arc_inner_count_ordering() {
    // Test different atomic orderings
    let inner = ArcInner {
        count: AtomicUsize::new(5),
        data: "ordering test",
    };

    // Test with different orderings
    assert_eq!(inner.count.load(Ordering::Acquire), 5);
    assert_eq!(inner.count.load(Ordering::Relaxed), 5);
    assert_eq!(inner.count.load(Ordering::SeqCst), 5);

    // Test store with different orderings
    inner.count.store(10, Ordering::Release);
    assert_eq!(inner.count.load(Ordering::Acquire), 10);

    inner.count.store(15, Ordering::SeqCst);
    assert_eq!(inner.count.load(Ordering::SeqCst), 15);
}

#[test]
fn test_arc_inner_count_compare_exchange() {
    // Test compare_exchange operations
    let inner = ArcInner {
        count: AtomicUsize::new(1),
        data: "cas test",
    };

    // Successful compare and swap
    let result = inner
        .count
        .compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!(result, Ok(1));
    assert_eq!(inner.count.load(Ordering::Relaxed), 2);

    // Failed compare and swap
    let result = inner
        .count
        .compare_exchange(1, 3, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!(result, Err(2));
    assert_eq!(inner.count.load(Ordering::Relaxed), 2);
}

#[test]
fn test_arc_inner_with_complex_types() {
    // Test with complex nested structures
    #[derive(Debug, PartialEq, Clone)]
    struct ComplexData {
        id: u64,
        name: String,
        values: Vec<f64>,
        metadata: std::collections::HashMap<String, String>,
    }

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("version".to_string(), "1.0".to_string());
    metadata.insert("author".to_string(), "test".to_string());

    let complex = ComplexData {
        id: 12345,
        name: "complex test".to_string(),
        values: vec![1.1, 2.2, 3.3],
        metadata,
    };

    let inner = ArcInner {
        count: AtomicUsize::new(1),
        data: complex.clone(),
    };

    assert_eq!(inner.count.load(Ordering::Relaxed), 1);
    assert_eq!(inner.data, complex);
    assert_eq!(inner.data.id, 12345);
    assert_eq!(inner.data.name, "complex test");
    assert_eq!(inner.data.values.len(), 3);
    assert_eq!(inner.data.metadata.len(), 2);
}

#[test]
fn test_arc_inner_field_offsets() {
    // Test that fields are laid out as expected
    use std::mem;

    let inner = ArcInner {
        count: AtomicUsize::new(42),
        data: 123i32,
    };

    // Calculate expected offsets
    let base_addr = &inner as *const _ as usize;
    let count_addr = &inner.count as *const _ as usize;
    let data_addr = &inner.data as *const _ as usize;

    // count should be at offset 0
    assert_eq!(count_addr - base_addr, 0);

    // data should be after count, properly aligned
    let expected_data_offset = mem::size_of::<AtomicUsize>();
    // Account for padding if needed for alignment
    let data_align = mem::align_of::<i32>();
    let aligned_offset = (expected_data_offset + data_align - 1) & !(data_align - 1);

    assert_eq!(data_addr - base_addr, aligned_offset);
}

#[test]
fn test_arc_inner_with_unsized_type() {
    // Test ArcInner with unsized type (Box<[T]>)
    let data: Box<[i32]> = vec![1, 2, 3, 4, 5].into_boxed_slice();
    let inner = ArcInner {
        count: AtomicUsize::new(1),
        data,
    };

    assert_eq!(inner.count.load(Ordering::Relaxed), 1);
    assert_eq!(&*inner.data, &[1, 2, 3, 4, 5]);
    assert_eq!(inner.data.len(), 5);
}
