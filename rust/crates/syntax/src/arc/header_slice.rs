//! # HeaderSlice: Combining Header Data with Slice Data
//!
//! This module provides `HeaderSlice<H, T>`, a data structure that efficiently combines
//! a header value with a dynamically-sized slice. This is particularly useful for
//! representing data like "a string filename with file contents" or "metadata with a
//! list of items".
//!
//! ## What Problem Does This Solve?
//!
//! Often you want to store some metadata alongside a collection of items:
//! - File name + file contents  
//! - Node type + child nodes (in a syntax tree)
//! - Table name + table rows
//! - Function name + parameters
//!
//! You could use a tuple like `(Header, Vec<T>)`, but this has downsides:
//! - Two separate allocations (header and vector data)
//! - Extra indirection when accessing the slice
//! - Less cache-friendly memory layout
//!
//! ## How HeaderSlice Solves This
//!
//! `HeaderSlice` stores everything in a single allocation:
//!
//! ```text
//! Memory layout:
//! ┌─────────────┬─────────────┬─────────────┐
//! │   header    │   length    │    slice    │
//! │     H       │   usize     │   [T; n]    │
//! └─────────────┴─────────────┴─────────────┘
//! ```
//!
//! ## Key Features
//!
//! - **Single allocation**: Header and slice data stored together
//! - **Cache efficient**: Related data is close in memory
//! - **Zero-copy conversions**: Efficient conversion between `Arc` and `ThinArc`
//! - **Type safe**: Full Rust type checking and borrowing rules
//!
//! ## Example Usage
//!
//! ```ignore
//! use crate::arc::{thin_arc::ThinArc, header_slice::HeaderSlice};
//!
//! // Create a file representation
//! let file_data = ThinArc::from_header_and_iter(
//!     "config.txt".to_string(),           // Header: filename
//!     b"port=8080\ndebug=true".iter().cloned()  // Slice: file contents
//! );
//!
//! println!("File: {}", file_data.header);
//! println!("Size: {} bytes", file_data.length);
//! println!("Content: {:?}", file_data.slice());
//! ```

use std::{marker::PhantomData, mem, ops::Deref, ptr};

use crate::arc::{arc::Arc, arc_inner::ArcInner, thin_arc::ThinArc, thin_to_thick};

/// A data structure that combines a header value with a dynamically-sized slice.
///
/// `HeaderSlice<H, T>` efficiently stores a header of type `H` alongside a slice `[T]`
/// in a single memory allocation. This is particularly useful for syntax trees, file
/// representations, and other data structures where you need metadata with a collection.
///
/// ## Memory Layout
///
/// ```text
/// HeaderSlice<String, u8> example:
/// ┌─────────────────┬─────────────┬─────────────────┐
/// │ header: String  │ length: 5   │ slice: [1,2,3,4,5] │
/// └─────────────────┴─────────────┴─────────────────┘
/// ```
///
/// ## Usage with Arc Types
///
/// `HeaderSlice` is designed to work with `Arc` and `ThinArc`:
/// - `Arc<HeaderSlice<H, [T]>>` - Fat pointer (pointer + length on stack)
/// - `ThinArc<H, T>` - Thin pointer (only pointer on stack, length in heap)
///
/// ## Derive Traits
///
/// - **Debug**: For debugging output
/// - **Eq, PartialEq**: For equality comparisons
/// - **Hash**: For use in hash maps and sets
/// - **PartialOrd**: For ordering comparisons
///
/// ## Example
///
/// ```ignore
/// # use crate::arc::{thin_arc::ThinArc, header_slice::HeaderSlice};
/// // Syntax tree node: node type + children
/// let node = ThinArc::from_header_and_iter(
///     "BinaryOp".to_string(),
///     vec!["left_child", "right_child"].into_iter()
/// );
///
/// assert_eq!(node.header, "BinaryOp");
/// assert_eq!(node.slice().len(), 2);
/// ```
#[derive(Debug, Eq, PartialEq, Hash, PartialOrd)]
#[repr(C)]
pub(crate) struct HeaderSlice<H, T: ?Sized> {
    /// The header data of type `H`.
    ///
    /// This can be any type - commonly strings, enums, or structs containing metadata.
    /// It's stored first in the layout, immediately followed by the length and slice data.
    pub(super) header: H,

    /// The length of the slice.
    ///
    /// This stores how many elements are in the slice portion. For `ThinArc`,
    /// this length is read from here to reconstruct fat pointers when needed.
    pub(super) length: usize,

    /// The slice data.
    ///
    /// This is where the actual slice elements are stored. The `T: ?Sized` allows
    /// this to be either `[T; 0]` (for thin pointers) or `[T]` (for fat pointers).
    pub(super) slice: T,
}

impl<H, T> HeaderSlice<H, [T]> {
    /// Returns a reference to the slice portion of the `HeaderSlice`.
    ///
    /// This provides convenient access to just the slice data, ignoring the header.
    /// It's equivalent to accessing the `slice` field directly but more readable.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use crate::arc::thin_arc::ThinArc;
    /// let data = ThinArc::from_header_and_iter("header".to_string(), vec![1, 2, 3].into_iter());
    ///
    /// let slice = data.slice();
    /// assert_eq!(slice.len(), 3);
    /// assert_eq!(slice[0], 1);
    /// ```
    pub(crate) fn slice(&self) -> &[T] {
        &self.slice
    }
}

impl<H, T> Arc<HeaderSlice<H, [T]>> {
    /// Converts an `Arc<HeaderSlice<H, [T]>>` into a `ThinArc<H, T>`.
    ///
    /// This consumes the `Arc` and returns a `ThinArc` pointing to the same data.
    /// The refcount is not modified - we're just changing the pointer representation
    /// from fat (pointer + length) to thin (just pointer).
    ///
    /// # How It Works
    ///
    /// 1. Verifies that the stored length matches the slice length
    /// 2. Extracts the fat pointer from the Arc
    /// 3. Converts it to a thin pointer by discarding the length metadata
    /// 4. Returns a ThinArc wrapping the thin pointer
    ///
    /// # Panics
    ///
    /// Panics if the stored length doesn't match the actual slice length.
    /// This would indicate a bug in the data structure construction.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use crate::arc::{arc::Arc, header_slice::HeaderSlice, thin_arc::ThinArc};
    /// // This is typically done internally, not by user code
    /// // let arc: Arc<HeaderSlice<String, [i32]>> = /* ... */;
    /// // let thin_arc: ThinArc<String, i32> = Arc::into_thin(arc);
    /// ```
    #[inline]
    pub(crate) fn into_thin(a: Self) -> ThinArc<H, T> {
        assert_eq!(
            a.length,
            a.slice.len(),
            "Length needs to be correct for ThinArc to work"
        );
        let fat_ptr: *mut ArcInner<HeaderSlice<H, [T]>> = a.ptr();
        mem::forget(a);
        let thin_ptr = fat_ptr as *mut [usize] as *mut usize;
        ThinArc {
            pointer: unsafe {
                ptr::NonNull::new_unchecked(thin_ptr as *mut ArcInner<HeaderSlice<H, [T; 0]>>)
            },
            phantom: PhantomData,
        }
    }

    /// Converts a `ThinArc<H, T>` into an `Arc<HeaderSlice<H, [T]>>`.
    ///
    /// This consumes the `ThinArc` and returns an `Arc` pointing to the same data.
    /// The refcount is not modified - we're just changing the pointer representation
    /// from thin (just pointer) to fat (pointer + length).
    ///
    /// # How It Works
    ///
    /// 1. Uses `thin_to_thick` to convert the thin pointer to a fat pointer
    /// 2. Wraps the fat pointer in an Arc
    /// 3. The length is read from the heap allocation to reconstruct the fat pointer
    ///
    /// # Safety
    ///
    /// This function is safe because:
    /// - The `ThinArc` guarantees the pointer is valid
    /// - `thin_to_thick` properly reconstructs the fat pointer
    /// - The refcount transfer maintains memory safety
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use crate::arc::{arc::Arc, header_slice::HeaderSlice, thin_arc::ThinArc};
    /// // let thin_arc: ThinArc<String, i32> = /* ... */;
    /// // let arc: Arc<HeaderSlice<String, [i32]>> = Arc::from_thin(thin_arc);
    /// ```
    #[inline]
    pub(crate) fn from_thin(a: ThinArc<H, T>) -> Self {
        let ptr = thin_to_thick(a.pointer.as_ptr());
        mem::forget(a);
        unsafe {
            Arc {
                pointer: ptr::NonNull::new_unchecked(ptr),
                phantom: PhantomData,
            }
        }
    }
}

impl<H, T> Deref for HeaderSlice<H, [T; 0]> {
    type Target = HeaderSlice<H, [T]>;

    /// Converts a thin `HeaderSlice<H, [T; 0]>` to a fat `HeaderSlice<H, [T]>`.
    ///
    /// This is a crucial piece of the `ThinArc` implementation. It allows us to take
    /// a `HeaderSlice` with a zero-sized array `[T; 0]` (which doesn't store length
    /// in the type) and convert it to a proper slice `[T]` by reading the length
    /// from the `length` field.
    ///
    /// # How It Works
    ///
    /// 1. Read the stored `length` field  
    /// 2. Create a slice pointer using the length and the address of our zero-sized array
    /// 3. Cast this slice pointer to a `HeaderSlice<H, [T]>` pointer
    /// 4. Dereference to get a reference to the fat `HeaderSlice`
    ///
    /// # Safety
    ///
    /// This is safe because:
    /// - The memory layout of `HeaderSlice<H, [T; 0]>` and `HeaderSlice<H, [T]>` is compatible
    /// - The `length` field accurately represents how many `T` elements follow
    /// - The slice elements were properly initialized when the structure was created
    ///
    /// # Memory Layout
    ///
    /// ```text
    /// From: HeaderSlice<H, [T; 0]>     To: HeaderSlice<H, [T]>
    /// ┌─────────┬────────┬─────┐     ┌─────────┬────────┬─────────────┐
    /// │ header  │ length │ []  │ →   │ header  │ length │ [T; length] │
    /// └─────────┴────────┴─────┘     └─────────┴────────┴─────────────┘
    /// ```
    fn deref(&self) -> &Self::Target {
        let len = self.length;
        let fake_slice: *const [T] = ptr::slice_from_raw_parts(self as *const _ as *const T, len);
        unsafe { &*(fake_slice as *const HeaderSlice<H, [T]>) }
    }
}
