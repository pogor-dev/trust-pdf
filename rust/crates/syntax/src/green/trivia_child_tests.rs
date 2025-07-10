//! # Tests for PDF Trivia Management System
//!
//! Comprehensive tests covering trivia creation, access, memory sharing,
//! and PDF-specific semantic requirements per ISO 32000-2.

use std::collections::HashMap;

use crate::{SyntaxKind, green::trivia_child::GreenTriviaChild};

// Test constants for different PDF trivia types
const NEWLINE_KIND: SyntaxKind = SyntaxKind(1);
const WHITESPACE_KIND: SyntaxKind = SyntaxKind(2);
const COMMENT_KIND: SyntaxKind = SyntaxKind(3);

/// Helper function to create test trivia with different content types
fn create_trivia(kind: SyntaxKind, text: &str) -> GreenTriviaChild {
    GreenTriviaChild::new(kind, text.as_bytes())
}

#[test]
fn test_trivia_kind_when_accessing_expect_correct_syntax_kind() {
    // Test accessing trivia kinds
    let newline = create_trivia(NEWLINE_KIND, "\n");
    let space = create_trivia(WHITESPACE_KIND, " ");
    let comment = create_trivia(COMMENT_KIND, "%PDF-1.7");

    // Verify kind access
    assert_eq!(newline.kind(), NEWLINE_KIND);
    assert_eq!(space.kind(), WHITESPACE_KIND);
    assert_eq!(comment.kind(), COMMENT_KIND);
}

#[test]
fn test_trivia_text_when_accessing_expect_correct_byte_content() {
    // Test accessing trivia text content
    let newline = create_trivia(NEWLINE_KIND, "\n");
    let space = create_trivia(WHITESPACE_KIND, " ");
    let comment = create_trivia(COMMENT_KIND, "%PDF-1.7");

    // Verify text access
    assert_eq!(newline.text(), b"\n");
    assert_eq!(space.text(), b" ");
    assert_eq!(comment.text(), b"%PDF-1.7");
}

#[test]
fn test_trivia_width_when_accessing_expect_correct_byte_length() {
    // Test accessing trivia width (byte length)
    let newline = create_trivia(NEWLINE_KIND, "\n");
    let space = create_trivia(WHITESPACE_KIND, " ");
    let comment = create_trivia(COMMENT_KIND, "%PDF-1.7");

    // Verify width access
    assert_eq!(newline.width(), 1);
    assert_eq!(space.width(), 1);
    assert_eq!(comment.width(), 8);
}

#[test]
fn test_trivia_equality_when_identical_content_expect_equal() {
    // Test equality for identical trivia
    let newline1 = create_trivia(NEWLINE_KIND, "\n");
    let newline2 = create_trivia(NEWLINE_KIND, "\n");
    assert_eq!(newline1, newline2);
}

#[test]
fn test_trivia_equality_when_different_kinds_same_content_expect_not_equal() {
    // Test inequality for different kinds with same content
    let newline = create_trivia(NEWLINE_KIND, "\n");
    let whitespace_newline = create_trivia(WHITESPACE_KIND, "\n");
    assert_ne!(newline, whitespace_newline);
}

#[test]
fn test_trivia_equality_when_same_kind_different_content_expect_not_equal() {
    // Test inequality for same kind with different content
    let space1 = create_trivia(WHITESPACE_KIND, " ");
    let space2 = create_trivia(WHITESPACE_KIND, "  ");
    assert_ne!(space1, space2);
}

#[test]
fn test_trivia_equality_when_completely_different_expect_not_equal() {
    // Test inequality for completely different trivia
    let comment = create_trivia(COMMENT_KIND, "%hello");
    let space = create_trivia(WHITESPACE_KIND, " ");
    assert_ne!(comment, space);
}

#[test]
fn test_trivia_memory_when_cloning_expect_shared_memory() {
    // Test that cloning shares memory via reference counting
    let original = create_trivia(COMMENT_KIND, "%shared");
    let cloned = original.clone();

    // Both should have same content
    assert_eq!(original.kind(), cloned.kind());
    assert_eq!(original.text(), cloned.text());
    assert_eq!(original.width(), cloned.width());

    // Memory should be shared (same underlying data)
    let original_ptr = original.text().as_ptr();
    let cloned_ptr = cloned.text().as_ptr();
    assert_eq!(original_ptr, cloned_ptr);
}

#[test]
fn test_trivia_zero_cost_conversions_when_deref_expect_direct_access() {
    // Test zero-cost deref conversion from owned to borrowed
    let owned = create_trivia(WHITESPACE_KIND, "   ");

    // Deref allows accessing methods directly
    assert_eq!(owned.kind(), WHITESPACE_KIND);
    assert_eq!(owned.text(), b"   ");
    assert_eq!(owned.width(), 3);

    // Test that multiple references work
    let owned2 = owned.clone();
    assert_eq!(owned.text(), owned2.text());
}

#[test]
fn test_trivia_hash_map_when_caching_expect_successful_integration() {
    // Test using trivia in hash maps for deduplication/caching
    let mut trivia_cache: HashMap<GreenTriviaChild, u32> = HashMap::new();

    let space = create_trivia(WHITESPACE_KIND, " ");
    let newline = create_trivia(NEWLINE_KIND, "\n");
    let comment = create_trivia(COMMENT_KIND, "%test");

    // Insert trivia with counts
    trivia_cache.insert(space.clone(), 5);
    trivia_cache.insert(newline.clone(), 3);
    trivia_cache.insert(comment.clone(), 1);

    // Test lookups work correctly
    assert_eq!(trivia_cache.get(&space), Some(&5));
    assert_eq!(trivia_cache.get(&newline), Some(&3));
    assert_eq!(trivia_cache.get(&comment), Some(&1));

    // Test that equivalent trivia can be found
    let space2 = create_trivia(WHITESPACE_KIND, " ");
    assert_eq!(trivia_cache.get(&space2), Some(&5));
}

#[test]
fn test_trivia_borrow_trait_when_used_expect_successful_deref() {
    // Test that trivia can be used in collections that rely on Borrow trait
    let trivia = create_trivia(COMMENT_KIND, "%borrow");

    // Test that we can use borrow methods
    assert_eq!(trivia.kind(), COMMENT_KIND);
    assert_eq!(trivia.text(), b"%borrow");
    assert_eq!(trivia.width(), 7);

    // Test borrow in context (the trait is implemented, even if we can't access the type directly)
    let _borrowed_ref = &*trivia; // This uses Deref
}

#[test]
fn test_trivia_borrow_trait_when_hashset_lookups_expect_successful_integration() {
    // Test that trivia works with collections that use Borrow trait internally
    use std::collections::HashSet;

    let trivia = create_trivia(COMMENT_KIND, "%borrow_test");

    // Test that Borrow trait works for HashSet lookups
    let mut set = HashSet::new();
    set.insert(trivia.clone());

    // HashSet uses Borrow trait internally for lookups
    assert!(set.contains(&trivia));

    // Test with equivalent trivia (should find it due to Borrow implementation)
    let equivalent = create_trivia(COMMENT_KIND, "%borrow_test");
    assert!(set.contains(&equivalent));

    // Verify the data through deref
    let borrowed_data = &*trivia; // Use deref instead of borrow() for direct access
    assert_eq!(borrowed_data.kind(), COMMENT_KIND);
    assert_eq!(borrowed_data.text(), b"%borrow_test");
    assert_eq!(borrowed_data.width(), 12);
}

#[test]
fn test_trivia_to_owned_conversion_when_borrowed_expect_new_owned_instance() {
    // Test ToOwned trait implementation for converting borrowed to owned
    let original = create_trivia(WHITESPACE_KIND, "  ");
    let borrowed: &_ = &*original; // Get borrowed reference via deref

    // Use to_owned to convert back to owned
    let owned_again = borrowed.to_owned();

    // Should be equal but separate instances
    assert_eq!(original, owned_again);
    assert_eq!(original.kind(), owned_again.kind());
    assert_eq!(original.text(), owned_again.text());
    assert_eq!(original.width(), owned_again.width());

    // Memory should still be shared (same underlying data)
    assert_eq!(original.text().as_ptr(), owned_again.text().as_ptr());
}

#[test]
fn test_trivia_partial_eq_when_explicit_usage_expect_correct_comparison() {
    // Test explicit PartialEq usage to ensure the trait is exercised
    use std::cmp::PartialEq;

    let trivia1 = create_trivia(NEWLINE_KIND, "\n");
    let trivia2 = create_trivia(NEWLINE_KIND, "\n");
    let trivia3 = create_trivia(WHITESPACE_KIND, "\n");

    // Use PartialEq::eq explicitly
    assert!(PartialEq::eq(&trivia1, &trivia2));
    assert!(!PartialEq::eq(&trivia1, &trivia3));

    // Test via deref to GreenTriviaData
    let data1: &_ = &*trivia1;
    let data2: &_ = &*trivia2;
    let data3: &_ = &*trivia3;

    assert!(PartialEq::eq(data1, data2));
    assert!(!PartialEq::eq(data1, data3));
}

#[test]
fn test_trivia_raw_pointer_when_converting_expect_preserved_data() {
    // Test the unsafe FFI operations for raw pointer conversion
    let original = create_trivia(COMMENT_KIND, "%raw_test");

    // Verify original data before conversion
    assert_eq!(original.kind(), COMMENT_KIND);
    assert_eq!(original.text(), b"%raw_test");
    assert_eq!(original.width(), 9);

    // Convert to raw pointer (this should exercise into_raw)
    let original_clone = original.clone(); // Keep one reference alive
    let raw_ptr = GreenTriviaChild::into_raw(original);

    // Convert back from raw pointer (this should exercise from_raw)
    let recovered = unsafe { GreenTriviaChild::from_raw(raw_ptr) };

    // Verify recovered trivia has identical properties
    assert_eq!(recovered.kind(), COMMENT_KIND);
    assert_eq!(recovered.text(), b"%raw_test");
    assert_eq!(recovered.width(), 9);

    // Should be equal to original
    assert_eq!(original_clone, recovered);
    assert_eq!(original_clone.text().as_ptr(), recovered.text().as_ptr());
}

#[test]
fn test_trivia_raw_pointer_when_memory_safety_expect_preserved_integrity() {
    // Test that raw pointer operations maintain memory safety
    let trivias: Vec<_> = (0..10)
        .map(|i| create_trivia(WHITESPACE_KIND, &" ".repeat(i + 1)))
        .collect();

    // Convert to raw pointers
    let raw_ptrs: Vec<_> = trivias
        .into_iter()
        .map(|trivia| {
            let width = trivia.width();
            let ptr = GreenTriviaChild::into_raw(trivia);
            (ptr, width)
        })
        .collect();

    // Convert back and verify all data is intact
    for (raw_ptr, expected_width) in raw_ptrs {
        let recovered = unsafe { GreenTriviaChild::from_raw(raw_ptr) };
        assert_eq!(recovered.kind(), WHITESPACE_KIND);
        assert_eq!(recovered.width(), expected_width);

        // Verify content matches expected pattern
        let expected_spaces = " ".repeat(expected_width as usize);
        assert_eq!(recovered.text(), expected_spaces.as_bytes());
    }
}

#[test]
fn test_empty_trivia_when_empty_content_expect_zero_width() {
    // Test edge case: empty trivia content
    let empty = create_trivia(WHITESPACE_KIND, "");

    assert_eq!(empty.kind(), WHITESPACE_KIND);
    assert_eq!(empty.text(), b"");
    assert_eq!(empty.width(), 0);
}

#[test]
fn test_trivia_debug_formatting_when_comment_expect_structured_output() {
    // Test debug formatting contains expected structure and content
    let comment = create_trivia(COMMENT_KIND, "%test");

    let debug_str = format!("{:?}", comment);
    assert!(debug_str.contains("GreenTrivia"));
    assert!(debug_str.contains("kind"));
    assert!(debug_str.contains("text"));
}

#[test]
fn test_trivia_display_formatting_when_comment_expect_byte_array() {
    // Test display formatting shows byte array representation
    let comment = create_trivia(COMMENT_KIND, "%test");

    let display_str = format!("{}", &*comment);
    // The Display implementation formats as a debug byte array [37, 116, 101, 115, 116] for "%test"
    assert!(display_str.contains("[") && display_str.contains("]"));
    assert!(display_str.contains("37")); // ASCII code for '%'
    assert!(display_str.contains("116")); // ASCII code for 't'
}

#[test]
fn test_trivia_concurrent_access_when_multithreaded_expect_safe_access() {
    // Test that trivia can be safely shared across threads
    use std::sync::Arc as StdArc;
    use std::thread;

    let trivia = create_trivia(COMMENT_KIND, "%concurrent");
    let shared_trivia = StdArc::new(trivia);

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let trivia_clone = StdArc::clone(&shared_trivia);
            thread::spawn(move || {
                // Each thread should see the same data
                assert_eq!(trivia_clone.kind(), COMMENT_KIND);
                assert_eq!(trivia_clone.text(), b"%concurrent");
                assert_eq!(trivia_clone.width(), 11);
                i // Return thread ID for verification
            })
        })
        .collect();

    // Wait for all threads and verify they completed
    for (i, handle) in handles.into_iter().enumerate() {
        let thread_id = handle.join().unwrap();
        assert_eq!(thread_id, i);
    }
}

#[test]
fn test_trivia_hash_consistency_when_equal_expect_same_hash() {
    // Test that equal trivia have the same hash
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let trivia1 = create_trivia(COMMENT_KIND, "%same");
    let trivia2 = create_trivia(COMMENT_KIND, "%same");

    assert_eq!(trivia1, trivia2);

    // Calculate hashes
    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    trivia1.hash(&mut hasher1);
    trivia2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}
