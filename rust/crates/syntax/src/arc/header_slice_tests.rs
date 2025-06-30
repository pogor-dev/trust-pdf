//! Tests for HeaderSlice functionality
//!
//! These tests verify that HeaderSlice works correctly in combination with ThinArc
//! and Arc. They test the basic structure, conversions, and various data types.

use crate::arc::{arc::Arc, header_slice::HeaderSlice, thin_arc::ThinArc};

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
