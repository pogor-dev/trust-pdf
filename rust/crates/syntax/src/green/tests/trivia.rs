use std::collections::HashMap;

use crate::{
    SyntaxKind,
    green::{trivia::GreenTrivia, trivia::GreenTriviaChild},
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

// =============================================================================
// GreenTriviaChild Tests
// =============================================================================

#[test]
fn test_trivia_child_properties_when_accessing_expect_correct_values() {
    // Combine multiple property tests into one to reduce redundancy
    let test_cases = [
        (NEWLINE_KIND, "\n", 1usize),
        (WHITESPACE_KIND, " ", 1usize),
        (COMMENT_KIND, "%PDF-1.7", 8usize),
    ];

    for (kind, text, expected_width) in test_cases {
        let trivia = create_trivia_child(kind, text);
        assert_eq!(trivia.kind(), kind);
        assert_eq!(trivia.text(), text.as_bytes());
        assert_eq!(trivia.width(), expected_width);
    }
}

#[test]
fn test_trivia_child_new_when_creating_with_unicode_expect_correct_byte_count() {
    // Test Unicode handling in trivia - important for PDF comments that might contain non-ASCII
    let unicode_comment = create_trivia_child(COMMENT_KIND, "%café");

    // "café" in UTF-8 is 5 bytes (é = 2 bytes) + '%' = 6 bytes total
    assert_eq!(unicode_comment.text(), "%café".as_bytes());
    assert_eq!(unicode_comment.width(), 6);
}

#[test]
fn test_trivia_child_new_when_creating_with_binary_data_expect_exact_preservation() {
    // Test that trivia preserves exact bytes, even non-UTF8 sequences
    let binary_data = &[0xFF, 0xFE, 0x00, 0x01];
    let binary_trivia = GreenTriviaChild::new(COMMENT_KIND, binary_data);

    assert_eq!(binary_trivia.text(), binary_data);
    assert_eq!(binary_trivia.width(), 4);
}

#[test]
fn test_trivia_child_equality_when_identical_content_expect_equal() {
    let newline1 = create_trivia_child(NEWLINE_KIND, "\n");
    let newline2 = create_trivia_child(NEWLINE_KIND, "\n");
    assert_eq!(newline1, newline2);
}

#[test]
fn test_trivia_child_equality_when_different_kinds_same_content_expect_not_equal() {
    let newline = create_trivia_child(NEWLINE_KIND, "\n");
    let whitespace_newline = create_trivia_child(WHITESPACE_KIND, "\n");
    assert_ne!(newline, whitespace_newline);
}

#[test]
fn test_trivia_child_equality_when_same_kind_different_content_expect_not_equal() {
    let space1 = create_trivia_child(WHITESPACE_KIND, " ");
    let space2 = create_trivia_child(WHITESPACE_KIND, "  ");
    assert_ne!(space1, space2);
}

#[test]
fn test_trivia_child_memory_when_cloning_expect_shared_memory() {
    let original = create_trivia_child(COMMENT_KIND, "%shared");
    let cloned = original.clone();

    // Both should have same content
    assert_eq!(original.kind(), cloned.kind());
    assert_eq!(original.text(), cloned.text());
    assert_eq!(original.width(), cloned.width());

    // Memory should be shared (reference counted)
    assert_eq!(original, cloned);
}

#[test]
fn test_trivia_child_hash_when_using_in_collections_expect_consistent_behavior() {
    let mut map = HashMap::new();
    let trivia = create_trivia_child(COMMENT_KIND, "%test");

    map.insert(trivia.clone(), "test_value");

    // Should be able to retrieve using equivalent trivia
    let lookup_trivia = create_trivia_child(COMMENT_KIND, "%test");
    assert_eq!(map.get(&lookup_trivia), Some(&"test_value"));
}

// =============================================================================
// GreenTrivia Collection Tests
// =============================================================================

#[test]
fn test_trivia_new_when_creating_empty_collection_expect_valid_trivia() {
    let empty_trivia = GreenTrivia::new(Vec::<GreenTriviaChild>::new());

    assert_eq!(empty_trivia.children().len(), 0);
    assert_eq!(empty_trivia.width(), 0);
    assert_eq!(empty_trivia.text(), b"");
}

#[test]
fn test_trivia_new_when_creating_single_child_expect_correct_content() {
    let single_comment = create_trivia_collection(vec![(COMMENT_KIND, "%PDF-1.7")]);

    let children = single_comment.children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].kind(), COMMENT_KIND);
    assert_eq!(children[0].text(), b"%PDF-1.7");
    assert_eq!(single_comment.width(), 8);
    assert_eq!(single_comment.text(), b"%PDF-1.7");
}

#[test]
fn test_trivia_new_when_creating_multiple_children_expect_correct_sequence() {
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
fn test_trivia_width_when_calculating_total_expect_sum_of_children() {
    let trivia = create_trivia_collection(vec![
        (COMMENT_KIND, "%PDF-1.7"),     // 8 bytes
        (NEWLINE_KIND, "\n"),           // 1 byte
        (WHITESPACE_KIND, "    "),      // 4 bytes
        (COMMENT_KIND, "%End of file"), // 12 bytes
    ]);

    assert_eq!(trivia.width(), 25); // 8 + 1 + 4 + 12 = 25
}

#[test]
fn test_trivia_text_when_concatenating_expect_combined_content() {
    let trivia = create_trivia_collection(vec![
        (COMMENT_KIND, "%header"),
        (NEWLINE_KIND, "\n"),
        (WHITESPACE_KIND, "  "),
        (COMMENT_KIND, "%comment"),
    ]);

    assert_eq!(trivia.text(), b"%header\n  %comment");
}

#[test]
fn test_trivia_text_when_empty_collection_expect_empty_string() {
    let empty_trivia = GreenTrivia::new(Vec::<GreenTriviaChild>::new());
    assert_eq!(empty_trivia.text(), b"");
}

#[test]
fn test_trivia_equality_when_identical_collections_expect_equal() {
    let trivia1 = create_trivia_collection(vec![(COMMENT_KIND, "%test"), (NEWLINE_KIND, "\n")]);
    let trivia2 = create_trivia_collection(vec![(COMMENT_KIND, "%test"), (NEWLINE_KIND, "\n")]);

    assert_eq!(trivia1, trivia2);
}

#[test]
fn test_trivia_equality_when_different_collections_expect_not_equal() {
    let trivia1 = create_trivia_collection(vec![(COMMENT_KIND, "%test1")]);
    let trivia2 = create_trivia_collection(vec![(COMMENT_KIND, "%test2")]);

    assert_ne!(trivia1, trivia2);
}

#[test]
fn test_trivia_hash_when_using_in_collections_expect_consistent_behavior() {
    let mut map = HashMap::new();
    let trivia = create_trivia_collection(vec![(COMMENT_KIND, "%key")]);

    map.insert(trivia.clone(), "test_value");

    // Should be able to retrieve using equivalent trivia
    let lookup_trivia = create_trivia_collection(vec![(COMMENT_KIND, "%key")]);
    assert_eq!(map.get(&lookup_trivia), Some(&"test_value"));
}

// =============================================================================
// PDF-Specific Trivia Tests
// =============================================================================

#[test]
fn test_pdf_header_trivia_when_creating_expect_correct_structure() {
    // Test PDF header comment structure: "%PDF-1.7"
    let pdf_header =
        create_trivia_collection(vec![(COMMENT_KIND, "%PDF-1.7"), (NEWLINE_KIND, "\n")]);

    assert_eq!(pdf_header.text(), b"%PDF-1.7\n");
    assert_eq!(pdf_header.width(), 9);
}

#[test]
fn test_xref_spacing_trivia_when_creating_expect_fixed_width() {
    // Test xref table spacing - ISO 32000-2 §7.5.4 requires specific formatting
    let xref_spacing = create_trivia_collection(vec![
        (WHITESPACE_KIND, "0000000000"), // 10-digit offset
        (WHITESPACE_KIND, " "),          // single space
        (WHITESPACE_KIND, "65535"),      // 5-digit generation
        (WHITESPACE_KIND, " "),          // single space
    ]);

    assert_eq!(xref_spacing.text(), b"0000000000 65535 ");
    assert_eq!(xref_spacing.width(), 17);
}

#[test]
fn test_stream_boundary_trivia_when_creating_expect_precise_newlines() {
    // Test stream keyword newline requirements - ISO 32000-2 §7.3.8
    let stream_boundary = create_trivia_collection(vec![
        (NEWLINE_KIND, "\n"), // Required newline after "stream"
    ]);

    assert_eq!(stream_boundary.text(), b"\n");
    assert_eq!(stream_boundary.width(), 1);
}

#[test]
fn test_obj_declaration_trivia_when_creating_expect_header_separation() {
    // Test obj declaration formatting - ISO 32000-2 §7.3.10
    let obj_trivia = create_trivia_collection(vec![
        (WHITESPACE_KIND, " "), // Space between number and generation
        (WHITESPACE_KIND, " "), // Space between generation and "obj"
        (NEWLINE_KIND, "\n"),   // Newline separating header from body
    ]);

    assert_eq!(obj_trivia.text(), b"  \n");
    assert_eq!(obj_trivia.width(), 3);
}

// =============================================================================
// Memory Management and Raw Pointer Tests
// =============================================================================

#[test]
fn test_trivia_into_raw_when_converting_to_pointer_expect_valid_operation() {
    let trivia = create_trivia_collection(vec![(COMMENT_KIND, "%test")]);
    let original_text = trivia.text();

    // Convert to raw pointer
    let raw_ptr = GreenTrivia::into_raw(trivia);

    // Convert back and verify integrity
    let recovered_trivia = unsafe { GreenTrivia::from_raw(raw_ptr) };
    assert_eq!(recovered_trivia.text(), original_text);
}

#[test]
fn test_trivia_child_into_raw_when_converting_to_pointer_expect_valid_operation() {
    let trivia_child = create_trivia_child(COMMENT_KIND, "%test");
    let original_text = trivia_child.text().to_vec();

    // Convert to raw pointer
    let raw_ptr = GreenTriviaChild::into_raw(trivia_child);

    // Convert back and verify integrity
    let recovered_child = unsafe { GreenTriviaChild::from_raw(raw_ptr) };
    assert_eq!(recovered_child.text(), &original_text);
}

#[test]
fn test_trivia_borrow_when_using_as_trivia_data_expect_same_content() {
    use std::borrow::Borrow;

    let trivia = create_trivia_collection(vec![(COMMENT_KIND, "%test")]);
    let trivia_data: &crate::green::trivia::GreenTriviaData = trivia.borrow();

    // Verify that borrowing gives access to the same data
    assert_eq!(trivia.children().len(), trivia_data.children().len());
    assert_eq!(trivia.width(), trivia_data.width());
    assert_eq!(trivia.text(), trivia_data.text());
}

#[test]
fn test_trivia_to_owned_when_converting_trivia_data_expect_equivalent_trivia() {
    use std::borrow::ToOwned;

    let original_trivia = create_trivia_collection(vec![(COMMENT_KIND, "%test")]);
    let trivia_data = &*original_trivia;
    let owned_trivia = trivia_data.to_owned();

    assert_eq!(
        original_trivia.children().len(),
        owned_trivia.children().len()
    );
    assert_eq!(original_trivia.width(), owned_trivia.width());
    assert_eq!(original_trivia.text(), owned_trivia.text());
}

// =============================================================================
// Edge Cases and Error Conditions
// =============================================================================

#[test]
fn test_trivia_when_large_collection_expect_efficient_handling() {
    // Test with a large number of trivia elements
    let mut pieces = Vec::new();
    for i in 0..1000 {
        pieces.push((WHITESPACE_KIND, " "));
        if i % 10 == 0 {
            pieces.push((NEWLINE_KIND, "\n"));
        }
    }

    let large_trivia = create_trivia_collection(pieces);

    // Should handle large collections efficiently
    assert_eq!(large_trivia.children().len(), 1100); // 1000 spaces + 100 newlines
    assert!(large_trivia.width() > 1000);
}

#[test]
fn test_trivia_when_zero_length_children_expect_valid_handling() {
    // Test with zero-length trivia elements (edge case)
    let empty_comment = create_trivia_child(COMMENT_KIND, "");
    let trivia = GreenTrivia::new(vec![empty_comment]);

    assert_eq!(trivia.children().len(), 1);
    assert_eq!(trivia.width(), 0);
    assert_eq!(trivia.text(), b"");
}

// =============================================================================
// Debug and Display Tests
// =============================================================================

#[test]
fn test_trivia_debug_when_formatting_expect_readable_output() {
    let trivia = create_trivia_collection(vec![(COMMENT_KIND, "%test")]);
    let debug_output = format!("{:?}", trivia);

    // Debug output should be non-empty and contain useful information
    assert!(!debug_output.is_empty());
}

#[test]
fn test_trivia_display_when_formatting_expect_text_content() {
    let trivia = create_trivia_collection(vec![
        (COMMENT_KIND, "%header"),
        (NEWLINE_KIND, "\n"),
        (WHITESPACE_KIND, "  "),
    ]);

    let display_output = format!("{}", &*trivia);
    assert_eq!(display_output, "%header\n  ");
}

#[test]
fn test_trivia_child_display_when_formatting_expect_content_representation() {
    let comment = create_trivia_child(COMMENT_KIND, "%test");
    let display_output = format!("{}", &*comment);

    // Should represent the content in some form
    assert!(!display_output.is_empty());
}

#[test]
fn test_trivia_data_equality_when_same_children_expect_equal() {
    let trivia1 = create_trivia_collection(vec![(COMMENT_KIND, "%test"), (NEWLINE_KIND, "\n")]);
    let trivia2 = create_trivia_collection(vec![(COMMENT_KIND, "%test"), (NEWLINE_KIND, "\n")]);

    let trivia_data1: &crate::green::trivia::GreenTriviaData = &*trivia1;
    let trivia_data2: &crate::green::trivia::GreenTriviaData = &*trivia2;

    assert_eq!(trivia_data1, trivia_data2);
}

#[test]
fn test_trivia_data_equality_when_different_children_expect_not_equal() {
    let trivia1 = create_trivia_collection(vec![(COMMENT_KIND, "%test1"), (NEWLINE_KIND, "\n")]);
    let trivia2 = create_trivia_collection(vec![(COMMENT_KIND, "%test2"), (NEWLINE_KIND, "\n")]);

    let trivia_data1: &crate::green::trivia::GreenTriviaData = &*trivia1;
    let trivia_data2: &crate::green::trivia::GreenTriviaData = &*trivia2;

    assert_ne!(trivia_data1, trivia_data2);
}

#[test]
fn test_trivia_child_borrow_when_using_as_trivia_child_data_expect_same_content() {
    use std::borrow::Borrow;

    let trivia_child = create_trivia_child(COMMENT_KIND, "%test");
    let trivia_child_data: &crate::green::trivia::GreenTriviaChildData = trivia_child.borrow();

    // Verify that borrowing gives access to the same data
    assert_eq!(trivia_child.kind(), trivia_child_data.kind());
    assert_eq!(trivia_child.text(), trivia_child_data.text());
    assert_eq!(trivia_child.width(), trivia_child_data.width());
}

#[test]
fn test_trivia_child_data_equality_when_same_content_expect_equal() {
    let trivia_child1 = create_trivia_child(COMMENT_KIND, "%test");
    let trivia_child2 = create_trivia_child(COMMENT_KIND, "%test");

    let trivia_child_data1: &crate::green::trivia::GreenTriviaChildData = &*trivia_child1;
    let trivia_child_data2: &crate::green::trivia::GreenTriviaChildData = &*trivia_child2;

    assert_eq!(trivia_child_data1, trivia_child_data2);
}

#[test]
fn test_trivia_child_data_equality_when_different_content_expect_not_equal() {
    let trivia_child1 = create_trivia_child(COMMENT_KIND, "%test1");
    let trivia_child2 = create_trivia_child(COMMENT_KIND, "%test2");

    let trivia_child_data1: &crate::green::trivia::GreenTriviaChildData = &*trivia_child1;
    let trivia_child_data2: &crate::green::trivia::GreenTriviaChildData = &*trivia_child2;

    assert_ne!(trivia_child_data1, trivia_child_data2);
}

#[test]
fn test_trivia_child_data_equality_when_different_kinds_expect_not_equal() {
    let trivia_child1 = create_trivia_child(COMMENT_KIND, "%test");
    let trivia_child2 = create_trivia_child(WHITESPACE_KIND, "%test");

    let trivia_child_data1: &crate::green::trivia::GreenTriviaChildData = &*trivia_child1;
    let trivia_child_data2: &crate::green::trivia::GreenTriviaChildData = &*trivia_child2;

    assert_ne!(trivia_child_data1, trivia_child_data2);
}

#[test]
fn test_trivia_child_data_to_owned_when_converting_expect_equivalent_child() {
    use std::borrow::ToOwned;

    let original_child = create_trivia_child(COMMENT_KIND, "%test");
    let trivia_child_data = &*original_child;
    let owned_child = trivia_child_data.to_owned();

    assert_eq!(original_child.kind(), owned_child.kind());
    assert_eq!(original_child.text(), owned_child.text());
    assert_eq!(original_child.width(), owned_child.width());
}

#[test]
fn test_trivia_child_display_when_invalid_utf8_expect_debug_format() {
    // Create trivia with invalid UTF-8 bytes
    let invalid_utf8_bytes = &[0xFF, 0xFE, 0x00, 0x01];
    let trivia_child = GreenTriviaChild::new(COMMENT_KIND, invalid_utf8_bytes);

    let display_output = format!("{}", &*trivia_child);

    // Should display as debug format when UTF-8 conversion fails
    assert!(display_output.contains("[255, 254, 0, 1]"));
}
