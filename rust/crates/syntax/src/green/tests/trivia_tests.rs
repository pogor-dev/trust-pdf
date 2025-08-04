//! # Tests for PDF Trivia Collection Management
//!
//! Comprehensive tests covering trivia collection creation, access, memory sharing,
//! and PDF-specific semantic requirements per ISO 32000-2.

use std::collections::HashMap;

use crate::{
    SyntaxKind,
    green::{trivia::GreenTrivia, trivia_child::GreenTriviaChild},
};

// Test constants for different PDF trivia types
const NEWLINE_KIND: SyntaxKind = SyntaxKind(1);
const WHITESPACE_KIND: SyntaxKind = SyntaxKind(2);
const COMMENT_KIND: SyntaxKind = SyntaxKind(3);

/// Helper function to create test trivia child with different content types
fn create_trivia_child(kind: SyntaxKind, text: &str) -> GreenTriviaChild {
    GreenTriviaChild::new(kind, text.as_bytes())
}

/// Helper function to create test trivia collection from individual pieces
fn create_trivia_collection(pieces: Vec<(SyntaxKind, &str)>) -> GreenTrivia {
    let children: Vec<GreenTriviaChild> = pieces
        .into_iter()
        .map(|(kind, text)| create_trivia_child(kind, text))
        .collect();

    GreenTrivia::new(children)
}

#[test]
fn test_trivia_new_when_creating_empty_collection_expect_valid_trivia() {
    // Test creating an empty trivia collection
    let empty_trivia = GreenTrivia::new(Vec::<GreenTriviaChild>::new());

    assert_eq!(empty_trivia.children().len(), 0);
}

#[test]
fn test_trivia_new_when_creating_single_child_expect_correct_content() {
    // Test creating trivia with a single child
    let single_comment = create_trivia_collection(vec![(COMMENT_KIND, "%PDF-1.7")]);

    let children = single_comment.children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].kind(), COMMENT_KIND);
    assert_eq!(children[0].text(), b"%PDF-1.7");
}

#[test]
fn test_trivia_new_when_creating_multiple_children_expect_correct_sequence() {
    // Test creating trivia with multiple children in sequence
    let multi_trivia = create_trivia_collection(vec![
        (COMMENT_KIND, "%header comment"),
        (NEWLINE_KIND, "\n"),
        (WHITESPACE_KIND, "  "),
        (COMMENT_KIND, "%another comment"),
        (NEWLINE_KIND, "\n"),
    ]);

    let children = multi_trivia.children();
    assert_eq!(children.len(), 5);

    // Verify sequence is preserved
    assert_eq!(children[0].kind(), COMMENT_KIND);
    assert_eq!(children[0].text(), b"%header comment");
    assert_eq!(children[1].kind(), NEWLINE_KIND);
    assert_eq!(children[1].text(), b"\n");
    assert_eq!(children[2].kind(), WHITESPACE_KIND);
    assert_eq!(children[2].text(), b"  ");
    assert_eq!(children[3].kind(), COMMENT_KIND);
    assert_eq!(children[3].text(), b"%another comment");
    assert_eq!(children[4].kind(), NEWLINE_KIND);
    assert_eq!(children[4].text(), b"\n");
}

#[test]
fn test_trivia_children_when_accessing_expect_slice_access() {
    // Test that children() returns a proper slice with iteration capabilities
    let trivia = create_trivia_collection(vec![
        (WHITESPACE_KIND, " "),
        (WHITESPACE_KIND, "\t"),
        (NEWLINE_KIND, "\n"),
    ]);

    let children = trivia.children();

    // Test slice properties
    assert_eq!(children.len(), 3);
    assert!(!children.is_empty());

    // Test iteration
    let kinds: Vec<SyntaxKind> = children.iter().map(|c| c.kind()).collect();
    assert_eq!(kinds, vec![WHITESPACE_KIND, WHITESPACE_KIND, NEWLINE_KIND]);

    // Test indexing
    assert_eq!(children[0].text(), b" ");
    assert_eq!(children[1].text(), b"\t");
    assert_eq!(children[2].text(), b"\n");
}

#[test]
fn test_trivia_equality_when_identical_collections_expect_equal() {
    // Test equality for identical trivia collections
    let trivia1 = create_trivia_collection(vec![(COMMENT_KIND, "%test"), (NEWLINE_KIND, "\n")]);
    let trivia2 = create_trivia_collection(vec![(COMMENT_KIND, "%test"), (NEWLINE_KIND, "\n")]);

    assert_eq!(trivia1, trivia2);

    // Test dereferenced equality too
    assert_eq!(*trivia1, *trivia2);
}

#[test]
fn test_trivia_equality_when_different_order_expect_not_equal() {
    // Test inequality for different order of same elements
    let trivia1 = create_trivia_collection(vec![(COMMENT_KIND, "%test"), (NEWLINE_KIND, "\n")]);
    let trivia2 = create_trivia_collection(vec![(NEWLINE_KIND, "\n"), (COMMENT_KIND, "%test")]);

    assert_ne!(trivia1, trivia2);
}

#[test]
fn test_trivia_equality_when_different_content_expect_not_equal() {
    // Test inequality for different content
    let trivia1 = create_trivia_collection(vec![(COMMENT_KIND, "%test1")]);
    let trivia2 = create_trivia_collection(vec![(COMMENT_KIND, "%test2")]);

    assert_ne!(trivia1, trivia2);
}

#[test]
fn test_trivia_equality_when_different_lengths_expect_not_equal() {
    // Test inequality for different lengths
    let trivia1 = create_trivia_collection(vec![(COMMENT_KIND, "%test")]);
    let trivia2 = create_trivia_collection(vec![(COMMENT_KIND, "%test"), (NEWLINE_KIND, "\n")]);

    assert_ne!(trivia1, trivia2);
}

#[test]
fn test_trivia_memory_when_cloning_expect_shared_memory() {
    // Test that cloning shares memory via reference counting
    let original = create_trivia_collection(vec![
        (COMMENT_KIND, "%shared_memory_test"),
        (NEWLINE_KIND, "\n"),
    ]);
    let cloned = original.clone();

    // Both should have same content
    assert_eq!(original, cloned);
    assert_eq!(original.children().len(), cloned.children().len());

    // Memory should be shared (same underlying children array)
    let original_ptr = original.children().as_ptr();
    let cloned_ptr = cloned.children().as_ptr();
    assert_eq!(original_ptr, cloned_ptr);
}

#[test]
fn test_trivia_deref_when_accessing_expect_direct_access() {
    // Test zero-cost deref conversion from owned to borrowed
    let owned = create_trivia_collection(vec![
        (WHITESPACE_KIND, "   "),
        (COMMENT_KIND, "%deref_test"),
    ]);

    // Deref allows accessing methods directly
    assert_eq!(owned.children().len(), 2);
    assert_eq!(owned.children()[0].kind(), WHITESPACE_KIND);
    assert_eq!(owned.children()[1].text(), b"%deref_test");

    // Test that multiple references work
    let owned2 = owned.clone();
    assert_eq!(owned.children().as_ptr(), owned2.children().as_ptr());
}

#[test]
fn test_trivia_hash_map_when_caching_expect_successful_integration() {
    // Test using trivia collections in hash maps for deduplication/caching
    let mut trivia_cache: HashMap<GreenTrivia, u32> = HashMap::new();

    let leading_trivia =
        create_trivia_collection(vec![(WHITESPACE_KIND, "  "), (COMMENT_KIND, "%leading")]);
    let trailing_trivia = create_trivia_collection(vec![(NEWLINE_KIND, "\n")]);
    let empty_trivia = create_trivia_collection(vec![]);

    // Insert trivia with usage counts
    trivia_cache.insert(leading_trivia.clone(), 10);
    trivia_cache.insert(trailing_trivia.clone(), 5);
    trivia_cache.insert(empty_trivia.clone(), 100);

    // Test lookups work correctly
    assert_eq!(trivia_cache.get(&leading_trivia), Some(&10));
    assert_eq!(trivia_cache.get(&trailing_trivia), Some(&5));
    assert_eq!(trivia_cache.get(&empty_trivia), Some(&100));

    // Test that equivalent trivia can be found
    let equivalent_leading =
        create_trivia_collection(vec![(WHITESPACE_KIND, "  "), (COMMENT_KIND, "%leading")]);
    assert_eq!(trivia_cache.get(&equivalent_leading), Some(&10));
}

#[test]
fn test_trivia_borrow_trait_when_used_expect_successful_integration() {
    // Test that trivia can be used in collections that rely on Borrow trait
    use std::collections::HashSet;

    let trivia =
        create_trivia_collection(vec![(COMMENT_KIND, "%borrow_test"), (NEWLINE_KIND, "\n")]);

    // Test that Borrow trait works for HashSet lookups
    let mut set = HashSet::new();
    set.insert(trivia.clone());

    // HashSet uses Borrow trait internally for lookups
    assert!(set.contains(&trivia));

    // Test with equivalent trivia (should find it due to Borrow implementation)
    let equivalent =
        create_trivia_collection(vec![(COMMENT_KIND, "%borrow_test"), (NEWLINE_KIND, "\n")]);
    assert!(set.contains(&equivalent));
}

#[test]
fn test_trivia_to_owned_conversion_when_borrowed_expect_new_owned_instance() {
    // Test ToOwned trait implementation for converting borrowed to owned
    let original = create_trivia_collection(vec![
        (WHITESPACE_KIND, "  "),
        (COMMENT_KIND, "%to_owned_test"),
    ]);
    let borrowed: &_ = &*original; // Get borrowed reference via deref

    // Use to_owned to convert back to owned
    let owned_again = borrowed.to_owned();

    // Should be equal but separate instances
    assert_eq!(original, owned_again);
    assert_eq!(original.children().len(), owned_again.children().len());

    // Memory should still be shared (same underlying data)
    assert_eq!(
        original.children().as_ptr(),
        owned_again.children().as_ptr()
    );
}

#[test]
fn test_trivia_raw_pointer_when_converting_expect_preserved_data() {
    // Test the unsafe FFI operations for raw pointer conversion
    let original = create_trivia_collection(vec![
        (COMMENT_KIND, "%raw_test"),
        (NEWLINE_KIND, "\n"),
        (WHITESPACE_KIND, "  "),
    ]);

    // Verify original data before conversion
    assert_eq!(original.children().len(), 3);
    assert_eq!(original.children()[0].text(), b"%raw_test");

    // Convert to raw pointer (this should exercise into_raw)
    let original_clone = original.clone(); // Keep one reference alive
    let raw_ptr = GreenTrivia::into_raw(original);

    // Convert back from raw pointer (this should exercise from_raw)
    let recovered = unsafe { GreenTrivia::from_raw(raw_ptr) };

    // Verify recovered trivia has identical properties
    assert_eq!(recovered.children().len(), 3);
    assert_eq!(recovered.children()[0].text(), b"%raw_test");
    assert_eq!(recovered.children()[1].text(), b"\n");
    assert_eq!(recovered.children()[2].text(), b"  ");

    // Should be equal to original
    assert_eq!(original_clone, recovered);
    assert_eq!(
        original_clone.children().as_ptr(),
        recovered.children().as_ptr()
    );
}

#[test]
fn test_trivia_raw_pointer_when_memory_safety_expect_preserved_integrity() {
    // Test that raw pointer operations maintain memory safety
    let trivias: Vec<_> = (0..5)
        .map(|i| {
            create_trivia_collection(vec![
                (COMMENT_KIND, &format!("%test_{}", i)),
                (NEWLINE_KIND, "\n"),
            ])
        })
        .collect();

    // Convert to raw pointers
    let raw_ptrs: Vec<_> = trivias
        .into_iter()
        .map(|trivia| {
            let len = trivia.children().len();
            let first_comment = trivia.children()[0].text().to_vec();
            let ptr = GreenTrivia::into_raw(trivia);
            (ptr, len, first_comment)
        })
        .collect();

    // Convert back and verify all data is intact
    for (raw_ptr, expected_len, expected_comment) in raw_ptrs {
        let recovered = unsafe { GreenTrivia::from_raw(raw_ptr) };
        assert_eq!(recovered.children().len(), expected_len);
        assert_eq!(recovered.children()[0].text(), expected_comment);
        assert_eq!(recovered.children()[1].text(), b"\n");
    }
}

#[test]
fn test_trivia_debug_formatting_when_multiple_children_expect_list_format() {
    // Test debug formatting contains expected structure and content
    let trivia = create_trivia_collection(vec![(COMMENT_KIND, "%debug"), (NEWLINE_KIND, "\n")]);

    let debug_str = format!("{:?}", trivia);
    // Should format as a debug list of children
    assert!(debug_str.contains("["));
    assert!(debug_str.contains("]"));
}

#[test]
fn test_trivia_display_formatting_when_multiple_children_expect_concatenated_text() {
    // Test display formatting concatenates all child text
    let trivia = create_trivia_collection(vec![
        (COMMENT_KIND, "%header"),
        (NEWLINE_KIND, "\n"),
        (WHITESPACE_KIND, "  "),
    ]);

    let display_str = format!("{}", &*trivia);
    assert_eq!(display_str, "%header\n  ");
}

#[test]
fn test_trivia_display_formatting_when_non_utf8_content_expect_debug_fallback() {
    // Test display formatting handles non-UTF8 content gracefully
    let non_utf8_child = GreenTriviaChild::new(COMMENT_KIND, &[0xFF, 0xFE, 0xFD]);
    let trivia = GreenTrivia::new(vec![non_utf8_child]);

    let display_str = format!("{}", &*trivia);
    // Should fall back to debug representation for invalid UTF-8
    assert!(display_str.contains("[") && display_str.contains("]"));
}

#[test]
fn test_trivia_concurrent_access_when_multithreaded_expect_safe_access() {
    // Test that trivia can be safely shared across threads
    use std::sync::Arc as StdArc;
    use std::thread;

    let trivia =
        create_trivia_collection(vec![(COMMENT_KIND, "%concurrent"), (NEWLINE_KIND, "\n")]);
    let shared_trivia = StdArc::new(trivia);

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let trivia_clone = StdArc::clone(&shared_trivia);
            thread::spawn(move || {
                // Each thread should see the same data
                assert_eq!(trivia_clone.children().len(), 2);
                assert_eq!(trivia_clone.children()[0].text(), b"%concurrent");
                assert_eq!(trivia_clone.children()[1].text(), b"\n");
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
    // Test that equal trivia collections have the same hash
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let trivia1 = create_trivia_collection(vec![(COMMENT_KIND, "%same"), (NEWLINE_KIND, "\n")]);
    let trivia2 = create_trivia_collection(vec![(COMMENT_KIND, "%same"), (NEWLINE_KIND, "\n")]);

    assert_eq!(trivia1, trivia2);

    // Calculate hashes
    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    trivia1.hash(&mut hasher1);
    trivia2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_trivia_pdf_specific_when_stream_boundary_expect_precise_newlines() {
    // Test PDF-specific trivia: stream boundary requirements (ISO 32000-2 §7.3.8)
    let stream_boundary = create_trivia_collection(vec![
        (NEWLINE_KIND, "\n"), // Required newline after 'stream' keyword
    ]);

    let children = stream_boundary.children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].kind(), NEWLINE_KIND);
    assert_eq!(children[0].text(), b"\n");
}

#[test]
fn test_trivia_pdf_specific_when_content_stream_spacing_expect_space_separation() {
    // Test PDF-specific trivia: content stream spacing (ISO 32000-2 §8.1.1)
    let content_spacing = create_trivia_collection(vec![
        (WHITESPACE_KIND, " "), // Space between content stream tokens
        (WHITESPACE_KIND, " "),
        (WHITESPACE_KIND, " "),
    ]);

    let children = content_spacing.children();
    assert_eq!(children.len(), 3);

    // All should be single spaces
    for child in children {
        assert_eq!(child.kind(), WHITESPACE_KIND);
        assert_eq!(child.text(), b" ");
    }
}

#[test]
fn test_trivia_text_when_multiple_children_expect_concatenated_string() {
    // Test the collection-level text() method that returns a String
    let mixed_trivia = create_trivia_collection(vec![
        (COMMENT_KIND, "%PDF-1.7"),
        (NEWLINE_KIND, "\n"),
        (WHITESPACE_KIND, "  "),
        (COMMENT_KIND, "%header"),
        (NEWLINE_KIND, "\n"),
    ]);

    let concatenated_text = mixed_trivia.text();

    assert_eq!(concatenated_text, "%PDF-1.7\n  %header\n");
    assert_eq!(concatenated_text.len(), 19); // Verify expected total length (8+1+2+7+1)
}

#[test]
fn test_trivia_text_when_empty_collection_expect_empty_string() {
    // Test text() method with empty trivia collection
    let empty_trivia = GreenTrivia::new(Vec::<GreenTriviaChild>::new());

    let text = empty_trivia.text();

    assert_eq!(text, "");
    assert_eq!(text.len(), 0);
}

#[test]
fn test_trivia_text_when_single_child_expect_same_content() {
    // Test text() method with single trivia child
    let single_comment = create_trivia_collection(vec![(COMMENT_KIND, "%single comment")]);

    let text = single_comment.text();

    assert_eq!(text, "%single comment");
    assert_eq!(text.len(), 15);
}
