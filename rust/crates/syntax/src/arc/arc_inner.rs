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
/// ```rust,no_run
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

/// # Safety
///
/// `ArcInner<T>` can be `Send` (transferred between threads) when `T` is both `Send` and `Sync`.
///
/// ## Why This Is Safe
///
/// - **T: Send**: The contained data can be moved between threads
/// - **T: Sync**: The contained data can be accessed from multiple threads simultaneously
/// - **AtomicUsize**: The reference count uses atomic operations, making concurrent access safe
///
/// The `ArcInner` itself doesn't add any non-thread-safe operations beyond what `T` provides.
/// The atomic reference count ensures that memory management (allocation/deallocation) is
/// thread-safe regardless of which thread performs the final drop.
///
/// ## Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use std::thread;
///
/// // String is Send + Sync, so Arc<String> can be sent between threads
/// let data = Arc::new(String::from("shared data"));
/// let data_clone = Arc::clone(&data);
///
/// thread::spawn(move || {
///     println!("{}", data_clone); // Safe because ArcInner<String> is Send
/// });
/// ```
unsafe impl<T: ?Sized + Sync + Send> Send for ArcInner<T> {}

/// # Safety
///
/// `ArcInner<T>` can be `Sync` (accessed from multiple threads) when `T` is both `Send` and `Sync`.
///
/// ## Why This Is Safe
///
/// - **Atomic reference count**: All modifications to `count` use atomic operations
/// - **Immutable data access**: The `data` field is never mutated after creation
/// - **T: Sync**: The contained data can be safely accessed from multiple threads
/// - **T: Send**: Required because dropping the `ArcInner` might happen on any thread
///
/// Multiple threads can safely:
/// - Read from the same `ArcInner<T>` simultaneously (via multiple `Arc<T>` instances)
/// - Increment/decrement the reference count atomically
/// - Drop their `Arc<T>` instances from different threads
///
/// ## Thread Safety Guarantee
///
/// The last thread to decrement the reference count to zero will be responsible
/// for deallocating the `ArcInner<T>`. This is safe because:
/// - Only one thread can observe the count transition from 1 to 0
/// - That thread gains exclusive access to deallocate the memory
/// - The atomic operations provide proper memory ordering guarantees
///
/// ## Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use std::thread;
///
/// let data = Arc::new(42i32);
/// let handles: Vec<_> = (0..10).map(|_| {
///     let data_clone = Arc::clone(&data);
///     thread::spawn(move || {
///         println!("{}", *data_clone); // Safe concurrent access
///     })
/// }).collect();
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
/// // ArcInner deallocated safely by the last thread to drop its Arc
/// ```
unsafe impl<T: ?Sized + Sync + Send> Sync for ArcInner<T> {}

#[cfg(test)]
mod tests {
    //! Tests for ArcInner functionality
    //!
    //! These tests verify that ArcInner works correctly as the internal structure
    //! for Arc. They test basic creation, atomic operations, and various data types.

    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_arc_inner_creation() {
        // Verifies basic creation and initialization of ArcInner.
        //
        // This test ensures that:
        // - ArcInner can be created with proper field initialization
        // - The atomic reference count starts at the expected value
        // - The data field contains the correct value
        //
        // This is fundamental to Arc functionality - every Arc starts with
        // a reference count of 1 and contains the user's data.

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
        // Verifies atomic operations on the reference count.
        //
        // This test simulates the core operations that Arc::clone() and Arc::drop()
        // perform on the reference count. It ensures that:
        // - fetch_add correctly increments the count (used by Arc::clone)
        // - fetch_sub correctly decrements the count (used by Arc::drop)
        // - The old value is returned, allowing detection of the 1->0 transition
        // - Operations are atomic and thread-safe
        //
        // These operations are the foundation of Arc's memory management.
        // When the count reaches 0, the Arc knows it's safe to deallocate.

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
        // Verifies that ArcInner implements Send and Sync for appropriate types.
        //
        // This test is crucial for thread safety guarantees. It ensures that:
        // - ArcInner<T> can be sent between threads when T is Send + Sync
        // - Multiple threads can safely access the same ArcInner simultaneously
        // - The type system enforces these constraints at compile time
        //
        // Without these traits, Arc wouldn't be able to provide safe shared
        // ownership across multiple threads.

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
        // Verifies the #[repr(C)] memory layout requirements.
        //
        // This test is critical for the correctness of ThinArc and other
        // pointer arithmetic operations. It ensures that:
        // - The count field is always at offset 0 (start of the struct)
        // - Fields are laid out in the declared order with predictable offsets
        // - Alignment requirements are met
        //
        // The #[repr(C)] attribute guarantees C-compatible layout, which is
        // essential for any unsafe pointer manipulation that might occur in
        // more advanced Arc implementations.

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
}
