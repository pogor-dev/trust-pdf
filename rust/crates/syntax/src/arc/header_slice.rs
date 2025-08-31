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

use crate::arc::{arc_inner::ArcInner, arc_main::Arc, thin_arc::ThinArc, thin_to_thick};

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
/// ┌─────────────────┬─────────────┬────────────────────┐
/// │ header: String  │ length: 5   │ slice: [1,2,3,4,5] │
/// └─────────────────┴─────────────┴────────────────────┘
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
    pub(crate) header: H,

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
        assert_eq!(a.length, a.slice.len(), "Length needs to be correct for ThinArc to work");
        let fat_ptr: *mut ArcInner<HeaderSlice<H, [T]>> = a.ptr();
        mem::forget(a);
        let thin_ptr = fat_ptr as *mut [usize] as *mut usize;
        ThinArc {
            pointer: unsafe { ptr::NonNull::new_unchecked(thin_ptr as *mut ArcInner<HeaderSlice<H, [T; 0]>>) },
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test HeaderSlice with ThinArc.
    ///
    /// This demonstrates the typical way HeaderSlice is used - through ThinArc
    /// rather than being constructed directly.
    fn create_test_header_slice(header: &str, data: Vec<i32>) -> ThinArc<String, i32> {
        ThinArc::from_header_and_iter(header.to_string(), data.into_iter())
    }

    #[test]
    fn test_header_slice_structure() {
        // Test that HeaderSlice properly stores header, length, and slice data
        let data = vec![1, 2, 3, 4, 5];
        let thin_arc = create_test_header_slice("test_header", data.clone());

        assert_eq!(thin_arc.header, "test_header");
        assert_eq!(thin_arc.length, 5);
        assert_eq!(thin_arc.slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_header_slice_empty() {
        // Test that HeaderSlice works correctly with empty slices
        let data = vec![];
        let thin_arc = create_test_header_slice("empty", data);

        assert_eq!(thin_arc.header, "empty");
        assert_eq!(thin_arc.length, 0);
        assert_eq!(thin_arc.slice(), &[]);
    }

    #[test]
    fn test_header_slice_different_types() {
        // Test with different header and slice types to verify generic nature
        let header_slice = HeaderSlice {
            header: 42u32,
            length: 3,
            slice: [1.0, 2.0, 3.0],
        };

        assert_eq!(header_slice.header, 42u32);
        assert_eq!(header_slice.length, 3);
        assert_eq!(header_slice.slice, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_header_slice_slice_method() {
        let data = vec![10, 20, 30];
        let thin_arc = create_test_header_slice("slice_test", data);

        let slice_ref = thin_arc.slice();
        assert_eq!(slice_ref.len(), 3);
        assert_eq!(slice_ref[0], 10);
        assert_eq!(slice_ref[1], 20);
        assert_eq!(slice_ref[2], 30);
    }

    #[test]
    fn test_thin_arc_to_arc_conversion() {
        let data = vec![1, 2, 3];
        let thin_arc = create_test_header_slice("conversion_test", data);

        // Convert ThinArc to Arc
        let arc = thin_arc.with_arc(|arc| arc.clone());

        assert_eq!(arc.header, "conversion_test");
        assert_eq!(arc.length, 3);
        assert_eq!(arc.slice(), &[1, 2, 3]);

        // Convert back to ThinArc
        let thin_arc2 = Arc::into_thin(arc);
        assert_eq!(thin_arc2.header, "conversion_test");
        assert_eq!(thin_arc2.slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_arc_from_thin_into_thin() {
        let data = vec![5, 10, 15, 20];
        let thin_arc = create_test_header_slice("roundtrip_test", data);

        // Convert ThinArc -> Arc -> ThinArc
        let arc = Arc::from_thin(thin_arc);
        let thin_arc2 = Arc::into_thin(arc);

        assert_eq!(thin_arc2.header, "roundtrip_test");
        assert_eq!(thin_arc2.length, 4);
        assert_eq!(thin_arc2.slice(), &[5, 10, 15, 20]);
    }

    #[test]
    fn test_header_slice_memory_layout() {
        // Test the #[repr(C)] layout
        let header_slice = HeaderSlice {
            header: "test",
            length: 5,
            slice: [1, 2, 3, 4, 5],
        };

        // Verify that fields are laid out in the expected order
        let base_ptr = &header_slice as *const _ as usize;
        let header_ptr = &header_slice.header as *const _ as usize;
        let length_ptr = &header_slice.length as *const _ as usize;
        let slice_ptr = &header_slice.slice as *const _ as usize;

        assert_eq!(base_ptr, header_ptr);
        assert!(length_ptr > header_ptr);
        assert!(slice_ptr > length_ptr);
    }

    #[test]
    fn test_header_slice_derived_traits() {
        let hs1 = HeaderSlice {
            header: "test",
            length: 2,
            slice: [1, 2],
        };

        let hs2 = HeaderSlice {
            header: "test",
            length: 2,
            slice: [1, 2],
        };

        let hs3 = HeaderSlice {
            header: "different",
            length: 2,
            slice: [1, 2],
        };

        // Test PartialEq
        assert_eq!(hs1, hs2);
        assert_ne!(hs1, hs3);

        // Test Debug (just ensure it doesn't panic)
        let debug_str = format!("{:?}", hs1);
        assert!(debug_str.contains("HeaderSlice"));

        // Test Hash (ensure it's consistent)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        hs1.hash(&mut hasher1);
        hs2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_header_slice_deref_zero_array() {
        // Test the Deref implementation for HeaderSlice<H, [T; 0]>
        let zero_array_hs = HeaderSlice {
            header: "zero",
            length: 3,
            slice: [],
        };

        // This should create a slice of length 3 through the Deref implementation
        let deref_result: &HeaderSlice<&str, [i32]> = &zero_array_hs;
        assert_eq!(deref_result.header, "zero");
        assert_eq!(deref_result.length, 3);
        // Note: We can't safely test the slice content here because it's using
        // raw pointer manipulation, but we can verify the structure exists
    }

    #[test]
    fn test_header_slice_with_large_data() {
        let large_data: Vec<i32> = (0..1000).collect();
        let thin_arc = create_test_header_slice("large", large_data.clone());

        assert_eq!(thin_arc.header, "large");
        assert_eq!(thin_arc.length, 1000);
        assert_eq!(thin_arc.slice().len(), 1000);
        assert_eq!(thin_arc.slice()[0], 0);
        assert_eq!(thin_arc.slice()[999], 999);
    }

    #[test]
    fn test_header_slice_clone() {
        let data = vec![1, 2, 3];
        let thin_arc1 = create_test_header_slice("clone_test", data);
        let thin_arc2 = thin_arc1.clone();

        // Both should have the same data
        assert_eq!(thin_arc1.header, thin_arc2.header);
        assert_eq!(thin_arc1.length, thin_arc2.length);
        assert_eq!(thin_arc1.slice(), thin_arc2.slice());
    }

    #[test]
    fn test_header_slice_send_sync() {
        // Test that ThinArc with HeaderSlice implements Send and Sync for appropriate types
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ThinArc<String, i32>>();
        assert_sync::<ThinArc<String, i32>>();
        assert_send::<ThinArc<i32, String>>();
        assert_sync::<ThinArc<i32, String>>();
    }

    #[test]
    fn test_header_slice_eq_hash() {
        let data1 = vec![1, 2, 3];
        let data2 = vec![1, 2, 3];
        let data3 = vec![1, 2, 4];

        let thin_arc1 = create_test_header_slice("test", data1);
        let thin_arc2 = create_test_header_slice("test", data2);
        let thin_arc3 = create_test_header_slice("test", data3);

        // Test equality by comparing individual fields
        assert_eq!(thin_arc1.header, thin_arc2.header);
        assert_eq!(thin_arc1.slice(), thin_arc2.slice());

        assert_eq!(thin_arc1.header, thin_arc3.header);
        assert_ne!(thin_arc1.slice(), thin_arc3.slice());

        // Test Hash consistency for equal data
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        thin_arc1.hash(&mut hasher1);
        thin_arc2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
