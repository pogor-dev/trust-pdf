//! # ArcInner: The Internal Structure of Arc
//!
//! This module defines `ArcInner<T>`, the internal structure that `Arc<T>` points to
//! in heap memory. It contains both the atomic reference count and the actual data.
//!
//! ## Memory Layout
//!
//! ```text
//! ArcInner<T> in heap memory:
//! ┌────────────────────┐
//! │ count: AtomicUsize │ ← Reference count (how many Arcs point here)
//! ├────────────────────┤
//! │ data: T            │ ← The actual data being shared
//! └────────────────────┘
//! ```
//!
//! ## Why This Structure?
//!
//! By keeping the reference count and data together in a single allocation:
//! - **Cache efficiency**: Count and data are close in memory
//! - **Single allocation**: Only one heap allocation needed per Arc
//! - **Atomic operations**: Reference counting can use CPU-level atomic operations
//!
//! ## Thread Safety
//!
//! `ArcInner<T>` is `Send + Sync` when `T` is `Send + Sync`. The reference count
//! uses atomic operations, making it safe for multiple threads to increment/decrement
//! the count simultaneously.

use std::sync::atomic;

/// The actual object allocated in the heap by an `Arc<T>`.
///
/// This structure combines the atomic reference count with the user's data
/// in a single heap allocation. Multiple `Arc<T>` instances point to the same
/// `ArcInner<T>` and coordinate through the atomic reference count.
///
/// ## Memory Layout
///
/// The `#[repr(C)]` ensures a predictable memory layout where `count` comes
/// first, followed immediately by `data`. This layout is important for
/// the pointer arithmetic used in `ThinArc`.
///
/// ## Reference Counting
///
/// - Starts at 1 when first created
/// - Incremented when `Arc::clone()` is called  
/// - Decremented when `Arc::drop()` is called
/// - Memory is freed when count reaches 0
///
/// ## Example
///
/// ```ignore
/// use std::sync::atomic::AtomicUsize;
/// use crate::arc::arc_inner::ArcInner;
///
/// // This is what gets allocated when you create Arc::new(42)
/// let inner = ArcInner {
///     count: AtomicUsize::new(1),  // One Arc pointing to this
///     data: 42,                    // The actual data
/// };
/// ```
#[repr(C)]
pub(crate) struct ArcInner<T: ?Sized> {
    /// The atomic reference count.
    ///
    /// This tracks how many `Arc<T>` instances are currently pointing to this allocation.
    /// It uses atomic operations to ensure thread-safety when multiple threads are
    /// cloning or dropping `Arc` instances simultaneously.
    ///
    /// - Initial value: 1 (when the first `Arc` is created)
    /// - Incremented: When `Arc::clone()` is called
    /// - Decremented: When `Arc::drop()` is called  
    /// - Memory freed: When this reaches 0
    pub(crate) count: atomic::AtomicUsize,

    /// The actual data being shared.
    ///
    /// This is the data that all `Arc<T>` instances pointing to this allocation
    /// will provide access to. It's the last field so that it can be a dynamically
    /// sized type (DST) like slices or trait objects.
    pub(crate) data: T,
}

unsafe impl<T: ?Sized + Sync + Send> Send for ArcInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for ArcInner<T> {}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

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
        assert_eq!(std::mem::align_of::<ArcInner<i32>>(), std::mem::align_of::<AtomicUsize>());
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
        assert_eq!(std::mem::size_of::<ArcInner<()>>(), std::mem::size_of::<AtomicUsize>());
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
        let result = inner.count.compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst);
        assert_eq!(result, Ok(1));
        assert_eq!(inner.count.load(Ordering::Relaxed), 2);

        // Failed compare and swap
        let result = inner.count.compare_exchange(1, 3, Ordering::SeqCst, Ordering::SeqCst);
        assert_eq!(result, Err(2));
        assert_eq!(inner.count.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_arc_inner_with_complex_types() {
        // Test with complex nested structures
        #[derive(Debug, PartialEq, Clone)]
        struct ComplexData {
            id: u32,
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
}
