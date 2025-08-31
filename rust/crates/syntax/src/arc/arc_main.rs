//! # Arc: Atomically Reference Counted Smart Pointer
//!
//! This module provides `Arc<T>`, a thread-safe reference-counting smart pointer.
//! It's similar to `std::sync::Arc` but simplified and optimized for specific use cases.
//!
//! ## What is Arc?
//!
//! `Arc` stands for "Atomically Reference Counted". It allows multiple owners to share
//! the same data safely across threads:
//!
//! - **Shared Ownership**: Multiple `Arc` instances can point to the same data
//! - **Thread Safe**: Safe to use across multiple threads simultaneously  
//! - **Automatic Cleanup**: Data is freed when the last `Arc` is dropped
//! - **Zero-Copy Cloning**: `clone()` just increments a counter, doesn't copy data
//!
//! ## Key Differences from `std::sync::Arc`
//!
//! - **No Weak References**: Simpler implementation, better performance
//! - **Abort on Overflow**: Prevents subtle bugs by crashing on reference count overflow
//! - **Optimized Layout**: Better suited for syntax trees and similar data structures
//!
//! ## Memory Management
//!
//! ```text
//! Arc 1 ──┐
//!         ├──► [RefCount: 3][Your Data]
//! Arc 2 ──┤
//!         │
//! Arc 3 ──┘
//! ```
//!
//! When you clone an `Arc`, only the reference count increases. When you drop an `Arc`,
//! the count decreases. When it reaches zero, the data is freed.
//!
//! ## Example Usage
//!
//! ```ignore
//! use crate::arc::arc::Arc;
//!
//! // Create some shared data
//! let data = Arc::new(vec![1, 2, 3, 4, 5]);
//!
//! // Clone creates a new reference (no data copying!)
//! let data_copy = data.clone();
//!
//! // Both variables point to the same data
//! assert_eq!(*data, *data_copy);
//!
//! // Pass to multiple threads
//! let handle = std::thread::spawn(move || {
//!     println!("Thread sees: {:?}", *data_copy);
//! });
//!
//! handle.join().unwrap();
//! // Data is automatically freed when last Arc is dropped
//! ```

use std::cmp::Ordering;
use std::mem::offset_of;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    ptr,
};

use crate::arc::{MAX_REFCOUNT, arc_inner::ArcInner};

/// An atomically reference counted shared pointer
///
/// `Arc<T>` provides shared ownership of a value of type `T`, allocated in the heap.
/// Invoking `clone` on `Arc` produces a new pointer to the same allocation,
/// increasing a reference count. When the last `Arc` pointer to a given allocation
/// is destroyed, the value stored in that allocation is also dropped.
///
/// ## Thread Safety
///
/// `Arc<T>` is thread-safe when `T` is `Send + Sync`. Multiple threads can hold
/// `Arc` instances pointing to the same data and access it simultaneously.
///
/// ## Performance
///
/// - **Clone**: O(1) - just increments an atomic counter
/// - **Drop**: O(1) - decrements counter, O(T) if last reference
/// - **Memory overhead**: One `usize` for the reference count
///
/// ## Example
///
/// ```ignore
/// use crate::arc::arc::Arc;
/// use std::thread;
///
/// let data = Arc::new(42);
/// let mut handles = vec![];
///
/// for i in 0..10 {
///     let data = data.clone(); // Cheap clone - just increment counter
///     let handle = thread::spawn(move || {
///         println!("Thread {}: {}", i, *data);
///     });
///     handles.push(handle);
/// }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
/// ```
///
/// See the documentation for [`Arc`] in the standard library for more details.
/// Unlike the standard library `Arc`, this `Arc` does not support weak reference counting.
///
/// [`Arc`]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html
#[repr(transparent)]
pub(crate) struct Arc<T: ?Sized> {
    /// Pointer to the heap-allocated `ArcInner<T>` containing both the reference count and data.
    /// This is wrapped in `NonNull` to enable niche optimization and guarantee non-null.
    pub(crate) pointer: ptr::NonNull<ArcInner<T>>,

    /// Zero-sized type marker that tells Rust about our ownership of `T`.
    /// This enables proper drop checking and variance but takes no space.
    pub(crate) phantom: PhantomData<T>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}

impl<T> Arc<T> {
    /// Reconstruct the Arc<T> from a raw pointer obtained from into_raw()
    ///
    /// Note: This raw pointer will be offset in the allocation and must be preceded
    /// by the atomic count.
    ///
    /// It is recommended to use OffsetArc for this
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: *const T) -> Self {
        // To find the corresponding pointer to the `ArcInner` we need
        // to subtract the offset of the `data` field from the pointer.
        unsafe {
            let ptr = (ptr as *const u8).sub(offset_of!(ArcInner<T>, data));
            Arc {
                pointer: ptr::NonNull::new_unchecked(ptr as *mut ArcInner<T>),
                phantom: PhantomData,
            }
        }
    }
}

impl<T: ?Sized> Arc<T> {
    /// Gets a reference to the inner `ArcInner<T>` structure.
    ///
    /// This provides access to both the reference count and the data.
    /// The reference is guaranteed to be valid as long as this `Arc` exists.
    ///
    /// # Safety
    ///
    /// This is safe because while this `Arc` is alive, we're guaranteed that the inner
    /// pointer is valid. The `ArcInner` structure is `Sync` when the inner data is `Sync`,
    /// so we can safely loan out an immutable reference.
    #[inline]
    fn inner(&self) -> &ArcInner<T> {
        // SAFETY:
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { &*self.ptr() }
    }

    /// The slow path for dropping an `Arc` when it's the last reference.
    ///
    /// This function is intentionally not inlined to keep the fast path (just decrementing
    /// the counter) small and fast. It's only called when we need to actually free the memory.
    ///
    /// # Safety
    ///
    /// This must only be called when we know this is the last reference to the data.
    /// The caller must ensure that the reference count has reached zero.
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        // SAFETY: The caller guarantees this is the last reference, so it's safe to
        // convert the raw pointer back to a Box and let it drop, which will free the memory
        unsafe {
            let _ = Box::from_raw(self.ptr());
        }
    }

    /// Tests pointer equality between two `Arc`s.
    ///
    /// Returns `true` if both `Arc`s point to the same allocation (not just equal data).
    /// This is useful for checking if two `Arc`s are literally the same object.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use crate::arc::arc::Arc;
    /// let arc1 = Arc::new(42);
    /// let arc2 = arc1.clone(); // Same allocation
    /// let arc3 = Arc::new(42);  // Different allocation, same data
    ///
    /// assert!(Arc::ptr_eq(&arc1, &arc2)); // Same object
    /// assert!(!Arc::ptr_eq(&arc1, &arc3)); // Different objects
    /// ```
    #[inline]
    pub(crate) fn ptr_eq(this: &Self, other: &Self) -> bool {
        std::ptr::addr_eq(this.ptr(), other.ptr())
    }

    /// Returns a raw pointer to the `ArcInner<T>`.
    ///
    /// This is used internally for various operations but should be used carefully
    /// as it bypasses Rust's safety guarantees.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid as long as at least one `Arc` pointing
    /// to this data exists.
    pub(crate) fn ptr(&self) -> *mut ArcInner<T> {
        self.pointer.as_ptr()
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    /// Creates a new reference to the same data.
    ///
    /// This is very efficient - it only increments the atomic reference count
    /// and returns a new `Arc` pointing to the same heap allocation. No data copying occurs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use crate::arc::arc::Arc;
    /// let original = Arc::new(vec![1, 2, 3]);
    /// let copy = original.clone(); // Fast! No data copying
    ///
    /// // Both point to the same data
    /// assert_eq!(*original, *copy);
    /// ```
    ///
    /// # Panics
    ///
    /// This function will abort the program if the reference count would overflow.
    /// This prevents use-after-free bugs that could occur with reference count wraparound.
    #[inline]
    fn clone(&self) -> Self {
        // Using a relaxed ordering is alright here, as knowledge of the
        // original reference prevents other threads from erroneously deleting
        // the object.
        //
        // As explained in the [Boost documentation][1], Increasing the
        // reference counter can always be done with memory_order_relaxed: New
        // references to an object can only be formed from an existing
        // reference, and passing an existing reference from one thread to
        // another must already provide any required synchronization.
        //
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        let old_size = self.inner().count.fetch_add(1, Relaxed);

        // However we need to guard against massive refcounts in case someone
        // is `mem::forget`ing Arcs. If we don't do this the count can overflow
        // and users will use-after free. We racily saturate to `isize::MAX` on
        // the assumption that there aren't ~2 billion threads incrementing
        // the reference count at once. This branch will never be taken in
        // any realistic program.
        //
        // We abort because such a program is incredibly degenerate, and we
        // don't care to support it.
        if old_size > MAX_REFCOUNT {
            std::process::abort();
        }

        unsafe {
            Arc {
                pointer: ptr::NonNull::new_unchecked(self.ptr()),
                phantom: PhantomData,
            }
        }
    }
}

impl<T: ?Sized> Arc<T> {
    /// Provides mutable access to the contents if the `Arc` is uniquely owned.
    ///
    /// Returns `Some(&mut T)` if this is the only `Arc` pointing to the data
    /// (reference count is 1), allowing safe mutation. Returns `None` if there
    /// are other references, preventing data races.
    ///
    /// This is useful for copy-on-write patterns where you want to modify data
    /// only if no one else is using it.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use crate::arc::arc::Arc;
    /// let mut arc = Arc::new(42);
    ///
    /// // We have the only reference, so we can get mutable access
    /// if let Some(data) = Arc::get_mut(&mut arc) {
    ///     *data = 84;
    /// }
    ///
    /// assert_eq!(*arc, 84);
    ///
    /// let arc2 = arc.clone(); // Now there are 2 references
    /// assert!(Arc::get_mut(&mut arc).is_none()); // Can't get mutable access
    /// ```
    #[inline]
    pub(crate) fn get_mut(this: &mut Self) -> Option<&mut T> {
        if this.is_unique() {
            unsafe {
                // SAFETY: We verified that the reference count is 1, so no other
                // references exist. It's safe to return a mutable reference.
                Some(&mut (*this.ptr()).data)
            }
        } else {
            None
        }
    }

    /// Returns `true` if this is the only `Arc` pointing to the data.
    ///
    /// This checks if the reference count is exactly 1, meaning this `Arc`
    /// is the sole owner of the data. This is useful for optimizations like
    /// avoiding unnecessary clones when you can modify in place.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use crate::arc::arc::Arc;
    /// let arc = Arc::new(42);
    /// assert!(arc.is_unique()); // Only one reference
    ///
    /// let arc2 = arc.clone();
    /// assert!(!arc.is_unique()); // Now there are two references
    /// assert!(!arc2.is_unique());
    /// ```
    ///
    /// # Memory Ordering
    ///
    /// Uses `Acquire` ordering to ensure proper synchronization. This is necessary
    /// to prevent race conditions where another thread might be in the process
    /// of dropping their reference.
    pub(crate) fn is_unique(&self) -> bool {
        // See the extensive discussion in [1] for why this needs to be Acquire.
        //
        // [1] https://github.com/servo/servo/issues/21186
        self.inner().count.load(Acquire) == 1
    }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;

    /// Provides immutable access to the inner data.
    ///
    /// This allows you to use `Arc<T>` as if it were a `&T`. You can call methods
    /// on the inner data directly through the `Arc`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use crate::arc::arc::Arc;
    /// let arc = Arc::new(String::from("hello"));
    ///
    /// // Can call String methods directly
    /// assert_eq!(arc.len(), 5);
    /// assert!(arc.starts_with("he"));
    /// ```
    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    /// Decrements the reference count and frees memory if this was the last reference.
    ///
    /// This is called automatically when an `Arc` goes out of scope. It decrements
    /// the atomic reference count, and if the count reaches zero, it frees the
    /// underlying memory and drops the contained data.
    ///
    /// # Memory Ordering
    ///
    /// Uses `Release` ordering for the decrement and `Acquire` ordering for the
    /// final check. This ensures proper synchronization between threads - any
    /// operations on the data happen-before the memory is freed.
    #[inline]
    fn drop(&mut self) {
        // Because `fetch_sub` is already atomic, we do not need to synchronize
        // with other threads unless we are going to delete the object.
        if self.inner().count.fetch_sub(1, Release) != 1 {
            return;
        }

        // FIXME(bholley): Use the updated comment when [2] is merged.
        //
        // This load is needed to prevent reordering of use of the data and
        // deletion of the data.  Because it is marked `Release`, the decreasing
        // of the reference count synchronizes with this `Acquire` load. This
        // means that use of the data happens before decreasing the reference
        // count, which happens before this load, which happens before the
        // deletion of the data.
        //
        // As explained in the [Boost documentation][1],
        //
        // > It is important to enforce any possible access to the object in one
        // > thread (through an existing reference) to *happen before* deleting
        // > the object in a different thread. This is achieved by a "release"
        // > operation after dropping a reference (any access to the object
        // > through this reference must obviously happened before), and an
        // > "acquire" operation before deleting the object.
        //
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        // [2]: https://github.com/rust-lang/rust/pull/41714
        self.inner().count.load(Acquire);

        unsafe {
            self.drop_slow();
        }
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Arc<T> {
    /// Compares two `Arc`s for equality.
    ///
    /// Two `Arc`s are considered equal if they point to the same allocation
    /// (same object) OR if their contents are equal. This means that even
    /// different allocations with the same data will compare as equal.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use crate::arc::arc::Arc;
    /// let arc1 = Arc::new(42);
    /// let arc2 = arc1.clone(); // Same allocation
    /// let arc3 = Arc::new(42);  // Different allocation, same data
    ///
    /// assert_eq!(arc1, arc2); // Same object
    /// assert_eq!(arc1, arc3); // Same data, different objects
    /// ```
    fn eq(&self, other: &Arc<T>) -> bool {
        Self::ptr_eq(self, other) || *(*self) == *(*other)
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd for Arc<T> {
    /// Compares two `Arc`s by comparing their contents.
    ///
    /// The comparison is done on the data itself, not the pointers.
    /// This allows `Arc<T>` to be used in sorted collections.
    fn partial_cmp(&self, other: &Arc<T>) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }

    /// Returns `true` if the contents of this `Arc` are less than the other.
    fn lt(&self, other: &Arc<T>) -> bool {
        *(*self) < *(*other)
    }

    /// Returns `true` if the contents of this `Arc` are less than or equal to the other.
    fn le(&self, other: &Arc<T>) -> bool {
        *(*self) <= *(*other)
    }

    /// Returns `true` if the contents of this `Arc` are greater than the other.
    fn gt(&self, other: &Arc<T>) -> bool {
        *(*self) > *(*other)
    }

    /// Returns `true` if the contents of this `Arc` are greater than or equal to the other.
    fn ge(&self, other: &Arc<T>) -> bool {
        *(*self) >= *(*other)
    }
}

impl<T: ?Sized + Ord> Ord for Arc<T> {
    /// Compares two `Arc`s by comparing their contents.
    ///
    /// This allows `Arc<T>` to be used as keys in `BTreeMap` and similar
    /// ordered collections.
    fn cmp(&self, other: &Arc<T>) -> Ordering {
        (**self).cmp(&**other)
    }
}

/// `Arc<T>` implements `Eq` when `T` implements `Eq`.
///
/// This is automatically derived and ensures that equality is reflexive,
/// symmetric, and transitive.
impl<T: ?Sized + Eq> Eq for Arc<T> {}

impl<T: ?Sized + Hash> Hash for Arc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ptr::NonNull,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    // Helper function to create an Arc for testing
    // Note: This creates proper Arc instances that manage their own memory
    fn create_test_arc<T>(data: T) -> Arc<T> {
        let inner = Box::new(ArcInner {
            count: AtomicUsize::new(1),
            data,
        });
        let ptr = Box::into_raw(inner);
        Arc {
            pointer: unsafe { NonNull::new_unchecked(ptr) },
            phantom: PhantomData,
        }
    }
    #[test]
    fn test_arc_structure_creation() {
        let arc = create_test_arc(42i32);

        // Verify we can access the inner data
        unsafe {
            let inner = arc.pointer.as_ref();
            assert_eq!(inner.count.load(Ordering::Relaxed), 1);
            assert_eq!(inner.data, 42);
        }
        // Arc will automatically clean itself up when dropped
    }

    #[test]
    fn test_arc_with_string() {
        let test_string = String::from("Hello, Arc!");
        let arc = create_test_arc(test_string.clone());

        unsafe {
            let inner = arc.pointer.as_ref();
            assert_eq!(inner.count.load(Ordering::Relaxed), 1);
            assert_eq!(inner.data, test_string);

            // Arc automatically cleans up memory when dropped
        }
    }

    #[test]
    fn test_arc_with_vec() {
        let test_vec = vec![1, 2, 3, 4, 5];
        let arc = create_test_arc(test_vec.clone());

        unsafe {
            let inner = arc.pointer.as_ref();
            assert_eq!(inner.count.load(Ordering::Relaxed), 1);
            assert_eq!(inner.data, test_vec);

            // Arc automatically cleans up memory when dropped
        }
    }

    #[test]
    fn test_arc_memory_layout() {
        // Test that Arc<T> has the expected memory layout
        use std::mem;

        // Arc should be the size of a pointer due to #[repr(transparent)]
        assert_eq!(mem::size_of::<Arc<i32>>(), mem::size_of::<*const ()>());
        assert_eq!(mem::size_of::<Arc<String>>(), mem::size_of::<*const ()>());

        // Arc should have the same alignment as a pointer
        assert_eq!(mem::align_of::<Arc<i32>>(), mem::align_of::<*const ()>());
        assert_eq!(mem::align_of::<Arc<String>>(), mem::align_of::<*const ()>());
    }
    #[test]
    fn test_arc_pointer_operations() {
        let arc = create_test_arc(100u32);

        // Test that we can get raw pointer
        let raw_ptr = arc.pointer.as_ptr();

        // Test that we can access data through pointer
        unsafe {
            let inner_ref = &*raw_ptr;
            assert_eq!(inner_ref.data, 100u32);
            assert_eq!(inner_ref.count.load(Ordering::Relaxed), 1);

            // Arc automatically cleans up memory when dropped
        }
    }

    #[test]
    fn test_arc_send_sync_properties() {
        // Test that Arc implements Send and Sync for appropriate types
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        // Test with types that are Send + Sync
        assert_send::<Arc<i32>>();
        assert_sync::<Arc<i32>>();

        assert_send::<Arc<String>>();
        assert_sync::<Arc<String>>();

        assert_send::<Arc<Vec<i32>>>();
        assert_sync::<Arc<Vec<i32>>>();
    }

    #[test]
    fn test_arc_phantom_data() {
        let arc = create_test_arc("phantom test");

        // PhantomData should be zero-sized
        assert_eq!(std::mem::size_of_val(&arc.phantom), 0);

        // PhantomData should not affect the total size due to #[repr(transparent)]
        assert_eq!(std::mem::size_of::<Arc<&str>>(), std::mem::size_of::<NonNull<ArcInner<&str>>>());
    }

    #[test]
    fn test_arc_with_unsized_type() {
        // Test with boxed slice (unsized type)
        let data: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
        let arc = create_test_arc(data);

        unsafe {
            let inner = arc.pointer.as_ref();
            assert_eq!(inner.count.load(Ordering::Relaxed), 1);
            assert_eq!(&*inner.data, &[1, 2, 3][..]);

            // Arc automatically cleans up memory when dropped
        }
    }
    #[test]
    fn test_arc_nonnull_properties() {
        let arc = create_test_arc(42i32);

        // Test that we can create a reference safely
        // NonNull guarantees the pointer is never null
        unsafe {
            let _inner_ref = arc.pointer.as_ref();
            // If we get here without panic, NonNull is working correctly

            // Arc automatically cleans up memory when dropped
        }
    }

    #[test]
    fn test_arc_multiple_instances() {
        // Test creating multiple Arc instances
        let arc1 = create_test_arc(1i32);
        let arc2 = create_test_arc(2i32);
        let arc3 = create_test_arc(3i32);

        unsafe {
            // Verify each has correct data
            assert_eq!(arc1.pointer.as_ref().data, 1);
            assert_eq!(arc2.pointer.as_ref().data, 2);
            assert_eq!(arc3.pointer.as_ref().data, 3);

            // Verify they point to different memory locations
            assert_ne!(arc1.pointer.as_ptr(), arc2.pointer.as_ptr());
            assert_ne!(arc2.pointer.as_ptr(), arc3.pointer.as_ptr());
            assert_ne!(arc1.pointer.as_ptr(), arc3.pointer.as_ptr());

            // Arc automatically cleans up memory when dropped
            // Arc automatically cleans up memory when dropped
            // Arc automatically cleans up memory when dropped
        }
    }

    #[test]
    fn test_arc_clone_behavior() {
        let arc1 = create_test_arc(42i32);
        let arc2 = arc1.clone();

        unsafe {
            // Verify both point to the same memory
            assert_eq!(arc1.pointer.as_ptr(), arc2.pointer.as_ptr());

            // Verify reference count increased
            let count = arc1.pointer.as_ref().count.load(Ordering::Relaxed);
            assert_eq!(count, 2);

            // Arc automatically cleans up memory when dropped
            // Arc automatically cleans up memory when dropped
        }
    }

    #[test]
    fn test_arc_ptr_eq() {
        let arc1 = create_test_arc(42i32);
        let arc2 = arc1.clone();
        let arc3 = create_test_arc(42i32);

        // arc1 and arc2 should point to the same allocation
        assert!(Arc::ptr_eq(&arc1, &arc2));

        // arc1 and arc3 should point to different allocations
        assert!(!Arc::ptr_eq(&arc1, &arc3));
    }

    #[test]
    fn test_arc_is_unique() {
        let arc1 = create_test_arc(42i32);

        // Should be unique initially
        assert!(arc1.is_unique());

        let arc2 = arc1.clone();

        // Should not be unique after clone
        assert!(!arc1.is_unique());
        assert!(!arc2.is_unique());

        drop(arc2);

        // Should be unique again after dropping the clone
        assert!(arc1.is_unique());
    }

    #[test]
    fn test_arc_get_mut() {
        let mut arc1 = create_test_arc(42i32);

        // Should be able to get mutable reference when unique
        let data_ref = Arc::get_mut(&mut arc1);
        assert!(data_ref.is_some());
        *data_ref.unwrap() = 100;

        let _arc2 = arc1.clone();

        // Should not be able to get mutable reference when not unique
        let data_ref = Arc::get_mut(&mut arc1);
        assert!(data_ref.is_none());
    }

    #[test]
    fn test_arc_deref() {
        let arc = create_test_arc("hello".to_string());

        // Test Deref functionality
        assert_eq!(*arc, "hello".to_string());
        assert_eq!(arc.len(), 5);
        assert_eq!(arc.chars().count(), 5);
    }

    #[test]
    fn test_arc_drop_behavior() {
        let arc1 = create_test_arc(vec![1, 2, 3]);
        let pointer = arc1.pointer.as_ptr();

        unsafe {
            // Initial reference count should be 1
            assert_eq!((*pointer).count.load(Ordering::Relaxed), 1);
        }

        let _arc2 = arc1.clone();

        unsafe {
            // Reference count should be 2 after clone
            assert_eq!((*pointer).count.load(Ordering::Relaxed), 2);
        }

        drop(arc1);

        unsafe {
            // Reference count should be 1 after dropping one Arc
            assert_eq!((*pointer).count.load(Ordering::Relaxed), 1);

            // Arc automatically cleans up memory when dropped
        }
    }

    #[test]
    fn test_arc_partial_eq() {
        let arc1 = create_test_arc(42i32);
        let arc2 = arc1.clone();
        let arc3 = create_test_arc(42i32);
        let arc4 = create_test_arc(100i32);

        // Same pointer should be equal
        assert!(arc1 == arc2);

        // Different pointers with same value should be equal
        assert!(arc1 == arc3);

        // Different values should not be equal
        assert!(arc1 != arc4);
    }

    #[test]
    fn test_arc_partial_ord() {
        let arc1 = create_test_arc(10i32);
        let arc2 = create_test_arc(20i32);
        let arc3 = create_test_arc(10i32);

        // Test ordering
        assert!(arc1 < arc2);
        assert!(arc2 > arc1);
        assert!(arc1 <= arc2);
        assert!(arc2 >= arc1);
        assert!(arc1 <= arc3);
        assert!(arc1 >= arc3);

        use std::cmp::Ordering;
        assert_eq!(arc1.partial_cmp(&arc2), Some(Ordering::Less));
        assert_eq!(arc2.partial_cmp(&arc1), Some(Ordering::Greater));
        assert_eq!(arc1.partial_cmp(&arc3), Some(Ordering::Equal));
    }

    #[test]
    fn test_arc_ord() {
        let arc1 = create_test_arc(10i32);
        let arc2 = create_test_arc(20i32);
        let arc3 = create_test_arc(10i32);

        use std::cmp::Ordering;
        assert_eq!(arc1.cmp(&arc2), Ordering::Less);
        assert_eq!(arc2.cmp(&arc1), Ordering::Greater);
        assert_eq!(arc1.cmp(&arc3), Ordering::Equal);
    }

    #[test]
    fn test_arc_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let arc1 = create_test_arc("test".to_string());
        let arc2 = create_test_arc("test".to_string());
        let arc3 = create_test_arc("different".to_string());

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        arc1.hash(&mut hasher1);
        arc2.hash(&mut hasher2);
        arc3.hash(&mut hasher3);

        // Equal values should have equal hashes
        assert_eq!(hasher1.finish(), hasher2.finish());

        // Different values should (usually) have different hashes
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    #[test]
    fn test_arc_with_zero_sized_type() {
        let arc = create_test_arc(());

        // Verify we can access the ZST
        assert_eq!(*arc, ());
    }

    #[test]
    fn test_arc_complex_type() {
        #[derive(Debug, PartialEq, Clone)]
        struct ComplexType {
            id: u32,
            name: String,
            data: Vec<i32>,
        }

        let complex = ComplexType {
            id: 123,
            name: "test".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let arc = create_test_arc(complex.clone());

        // Verify all fields are accessible
        assert_eq!(arc.id, 123);
        assert_eq!(arc.name, "test");
        assert_eq!(arc.data, vec![1, 2, 3, 4, 5]);
    }
}
