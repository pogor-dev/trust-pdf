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
pub(super) struct Arc<T: ?Sized> {
    /// Pointer to the heap-allocated `ArcInner<T>` containing both the reference count and data.
    /// This is wrapped in `NonNull` to enable niche optimization and guarantee non-null.
    pub(super) pointer: ptr::NonNull<ArcInner<T>>,

    /// Zero-sized type marker that tells Rust about our ownership of `T`.
    /// This enables proper drop checking and variance but takes no space.
    pub(super) phantom: PhantomData<T>,
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
    pub(super) unsafe fn from_raw(ptr: *const T) -> Self {
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
    pub(super) fn ptr_eq(this: &Self, other: &Self) -> bool {
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
    pub(super) fn ptr(&self) -> *mut ArcInner<T> {
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
    pub(super) fn get_mut(this: &mut Self) -> Option<&mut T> {
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
    pub(super) fn is_unique(&self) -> bool {
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
