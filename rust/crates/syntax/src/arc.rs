//! # Arc Module: Atomically Reference Counted Smart Pointers
//!
//! This module provides a simplified, memory-efficient implementation of atomic reference counting
//! for shared ownership of data. It's a "vendored and stripped down version of triomphe" - a
//! high-performance Arc library.
//!
//! ## What is Reference Counting?
//!
//! Reference counting is a memory management technique where we keep track of how many "owners"
//! a piece of data has. When the count reaches zero, we know it's safe to free the memory.
//! "Atomic" means this counting is thread-safe - multiple threads can safely share the same data.
//!
//! ## Module Structure
//!
//! This module contains several components:
//!
//! - **`Arc<T>`**: The main atomically reference-counted pointer (like `std::sync::Arc` but simpler)
//! - **`ThinArc<H, T>`**: A memory-optimized version for header + slice data patterns  
//! - **`ArcInner<T>`**: The internal structure that holds the reference count and data
//! - **`HeaderSlice<H, T>`**: A structure that combines header data with a slice
//!
//! ## Key Differences from `std::sync::Arc`
//!
//! - **No weak references**: Simpler implementation, better performance
//! - **Optimized for specific use cases**: Better suited for syntax tree nodes and similar data
//! - **Memory layout control**: `ThinArc` provides better memory efficiency for certain patterns
//!
//! ## Safety and Thread Safety
//!
//! All types in this module are thread-safe when the contained data is `Send + Sync`.
//! The reference counting uses atomic operations to ensure correctness across threads.
//!
//! ## Example Usage
//!
//! ```ignore
//! use crate::arc::{arc::Arc, thin_arc::ThinArc};
//!
//! // Regular Arc usage
//! let data = "Hello, World!";
//! let arc1 = Arc::new(data);  // Reference count = 1
//! let arc2 = arc1.clone();    // Reference count = 2
//! // When arc1 and arc2 are dropped, memory is freed
//!
//! // ThinArc for header + slice patterns
//! let file_data = ThinArc::from_header_and_iter(
//!     "config.txt".to_string(),  // header
//!     vec![1, 2, 3, 4].into_iter()  // slice data
//! );
//! ```

use std::ptr;

use crate::arc::{arc_inner::ArcInner, header_slice::HeaderSlice};

pub(crate) mod arc;
pub(crate) mod arc_inner;
pub(crate) mod header_slice;
pub(crate) mod thin_arc;

#[cfg(test)]
mod arc_inner_tests;
#[cfg(test)]
mod arc_tests;
#[cfg(test)]
mod header_slice_tests;
#[cfg(test)]
mod thin_arc_tests;

/// A soft limit on the amount of references that may be made to an `Arc`.
///
/// This prevents reference count overflow which could lead to use-after-free bugs.
/// Going above this limit will abort your program at exactly `MAX_REFCOUNT + 1` references.
///
/// In practice, this limit (about 2 billion on 64-bit systems) should never be reached
/// in normal programs. If you hit this limit, you likely have a bug like a reference cycle
/// or you're cloning Arc references in a tight loop without dropping them.
const MAX_REFCOUNT: usize = (isize::MAX) as usize;

/// Converts a "thin" pointer to a "thick" (fat) pointer for slice types.
///
/// This is a key function that enables `ThinArc` to work. It reconstructs a proper
/// fat pointer (pointer + length) from a thin pointer by reading the length from
/// the allocation itself.
///
/// # How it works
///
/// 1. Takes a thin pointer to `ArcInner<HeaderSlice<H, [T; 0]>>`
/// 2. Reads the `length` field from the `HeaderSlice`
/// 3. Creates a fat pointer to `ArcInner<HeaderSlice<H, [T]>>` with the correct length
///
/// # Safety
///
/// This function is unsafe because:
/// - It dereferences a raw pointer to read the length
/// - It performs pointer casting that must maintain proper alignment
/// - The caller must ensure the thin pointer is valid and properly initialized
///
/// # Parameters
///
/// - `thin`: A pointer to the thin representation of the data
///
/// # Returns
///
/// A fat pointer that includes both the memory address and the slice length
///
/// # Example Memory Layout
///
/// ```text
/// Before (thin pointer):     After (fat pointer):
/// ┌─────────────┐           ┌─────────────┐
/// │   address   │     →     │   address   │
/// └─────────────┘           │   length    │
///                           └─────────────┘
/// ```
fn thin_to_thick<H, T>(
    thin: *mut ArcInner<HeaderSlice<H, [T; 0]>>,
) -> *mut ArcInner<HeaderSlice<H, [T]>> {
    // SAFETY: The caller guarantees that `thin` points to valid, initialized memory
    // containing an ArcInner with a HeaderSlice that has a valid length field
    let len = unsafe { (*thin).data.length };

    // Create a fat slice pointer from the thin pointer and the length we just read
    let fake_slice: *mut [T] = ptr::slice_from_raw_parts_mut(thin as *mut T, len);

    // Cast the slice pointer to our target type - this "transplants" the metadata
    // (length) from the slice pointer to our ArcInner pointer
    fake_slice as *mut ArcInner<HeaderSlice<H, [T]>>
}
