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
pub(super) struct ArcInner<T: ?Sized> {
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
    pub(super) count: atomic::AtomicUsize,

    /// The actual data being shared.
    ///
    /// This is the data that all `Arc<T>` instances pointing to this allocation
    /// will provide access to. It's the last field so that it can be a dynamically
    /// sized type (DST) like slices or trait objects.
    pub(super) data: T,
}

unsafe impl<T: ?Sized + Sync + Send> Send for ArcInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for ArcInner<T> {}
