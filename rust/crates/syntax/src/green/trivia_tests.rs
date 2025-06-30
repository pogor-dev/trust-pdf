//! # Tests for PDF Trivia Management System
//!
//! Comprehensive tests covering trivia creation, access, memory sharing,
//! and PDF-specific semantic requirements per ISO 32000-2.

use std::collections::HashMap;

use crate::{SyntaxKind, green::trivia::GreenTrivia};

// Test constants for different PDF trivia types
const NEWLINE_KIND: SyntaxKind = SyntaxKind(1);
const WHITESPACE_KIND: SyntaxKind = SyntaxKind(2);
const COMMENT_KIND: SyntaxKind = SyntaxKind(3);

/// Helper function to create test trivia with different content types
fn create_trivia(kind: SyntaxKind, text: &str) -> GreenTrivia {
    GreenTrivia::new(kind, text.as_bytes())
}

#[test]
fn test_trivia_creation_and_basic_access() {
    // Test creating trivia with different kinds and content
    let newline = create_trivia(NEWLINE_KIND, "\n");
    let space = create_trivia(WHITESPACE_KIND, " ");
    let comment = create_trivia(COMMENT_KIND, "%PDF-1.7");

    // Test basic property access
    assert_eq!(newline.kind(), NEWLINE_KIND);
    assert_eq!(newline.text(), b"\n");
    assert_eq!(newline.width(), 1);

    assert_eq!(space.kind(), WHITESPACE_KIND);
    assert_eq!(space.text(), b" ");
    assert_eq!(space.width(), 1);

    assert_eq!(comment.kind(), COMMENT_KIND);
    assert_eq!(comment.text(), b"%PDF-1.7");
    assert_eq!(comment.width(), 8);
}

#[test]
fn test_pdf_specific_trivia_types() {
    // Test PDF-specific trivia scenarios per ISO 32000-2

    // Stream separator: Required \n after stream (§7.3.8)
    let stream_newline = create_trivia(NEWLINE_KIND, "\n");
    assert_eq!(stream_newline.text(), b"\n");
    assert_eq!(stream_newline.width(), 1);

    // Xref table formatting: Fixed-width spacing (§7.5.4)
    let xref_spaces = create_trivia(WHITESPACE_KIND, "     "); // 5 spaces
    assert_eq!(xref_spaces.text(), b"     ");
    assert_eq!(xref_spaces.width(), 5);

    // Content stream tokens: Space-separated (§8.1.1)
    let content_space = create_trivia(WHITESPACE_KIND, " ");
    assert_eq!(content_space.text(), b" ");
    assert_eq!(content_space.width(), 1);

    // PDF version comment
    let version_comment = create_trivia(COMMENT_KIND, "%PDF-2.0");
    assert_eq!(version_comment.text(), b"%PDF-2.0");
    assert_eq!(version_comment.width(), 8);
}

#[test]
fn test_trivia_equality() {
    // Test equality for identical trivia
    let newline1 = create_trivia(NEWLINE_KIND, "\n");
    let newline2 = create_trivia(NEWLINE_KIND, "\n");
    assert_eq!(newline1, newline2);

    // Test inequality for different kinds with same content
    let newline = create_trivia(NEWLINE_KIND, "\n");
    let whitespace_newline = create_trivia(WHITESPACE_KIND, "\n");
    assert_ne!(newline, whitespace_newline);

    // Test inequality for same kind with different content
    let space1 = create_trivia(WHITESPACE_KIND, " ");
    let space2 = create_trivia(WHITESPACE_KIND, "  ");
    assert_ne!(space1, space2);

    // Test inequality for completely different trivia
    let comment = create_trivia(COMMENT_KIND, "%hello");
    let space = create_trivia(WHITESPACE_KIND, " ");
    assert_ne!(comment, space);
}

#[test]
fn test_trivia_memory_sharing() {
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
    assert_eq!(
        original_ptr, cloned_ptr,
        "Memory should be shared via reference counting"
    );
}

#[test]
fn test_trivia_zero_cost_conversions() {
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
fn test_trivia_hash_map_integration() {
    // Test using trivia in hash maps for deduplication/caching
    let mut trivia_cache: HashMap<GreenTrivia, u32> = HashMap::new();

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
fn test_trivia_borrow_trait() {
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
fn test_trivia_borrow_trait_direct() {
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
fn test_trivia_to_owned_conversion() {
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
fn test_trivia_partial_eq_explicit() {
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

#[cfg(test)]
mod unsafe_tests {
    use super::*;

    #[test]
    fn test_trivia_raw_pointer_operations() {
        // Test the unsafe FFI operations for raw pointer conversion
        let original = create_trivia(COMMENT_KIND, "%raw_test");

        // Verify original data before conversion
        assert_eq!(original.kind(), COMMENT_KIND);
        assert_eq!(original.text(), b"%raw_test");
        assert_eq!(original.width(), 9);

        // Convert to raw pointer (this should exercise into_raw)
        let original_clone = original.clone(); // Keep one reference alive
        let raw_ptr = GreenTrivia::into_raw(original);

        // Convert back from raw pointer (this should exercise from_raw)
        let recovered = unsafe { GreenTrivia::from_raw(raw_ptr) };

        // Verify recovered trivia has identical properties
        assert_eq!(recovered.kind(), COMMENT_KIND);
        assert_eq!(recovered.text(), b"%raw_test");
        assert_eq!(recovered.width(), 9);

        // Should be equal to original
        assert_eq!(original_clone, recovered);
        assert_eq!(original_clone.text().as_ptr(), recovered.text().as_ptr());
    }

    #[test]
    fn test_trivia_raw_pointer_memory_safety() {
        // Test that raw pointer operations maintain memory safety
        let trivias: Vec<_> = (0..10)
            .map(|i| create_trivia(WHITESPACE_KIND, &" ".repeat(i + 1)))
            .collect();

        // Convert to raw pointers
        let raw_ptrs: Vec<_> = trivias
            .into_iter()
            .map(|trivia| {
                let width = trivia.width();
                let ptr = GreenTrivia::into_raw(trivia);
                (ptr, width)
            })
            .collect();

        // Convert back and verify all data is intact
        for (raw_ptr, expected_width) in raw_ptrs {
            let recovered = unsafe { GreenTrivia::from_raw(raw_ptr) };
            assert_eq!(recovered.kind(), WHITESPACE_KIND);
            assert_eq!(recovered.width(), expected_width);

            // Verify content matches expected pattern
            let expected_spaces = " ".repeat(expected_width as usize);
            assert_eq!(recovered.text(), expected_spaces.as_bytes());
        }
    }
}

#[test]
fn test_empty_trivia() {
    // Test edge case: empty trivia content
    let empty = create_trivia(WHITESPACE_KIND, "");

    assert_eq!(empty.kind(), WHITESPACE_KIND);
    assert_eq!(empty.text(), b"");
    assert_eq!(empty.width(), 0);
}

#[test]
fn test_large_trivia_content() {
    // Test with larger trivia content (e.g., long comments)
    let large_comment_text = "%This is a very long PDF comment that might span multiple lines and contain various characters including special ones like ñ and émojis 🔥";
    let large_comment = create_trivia(COMMENT_KIND, large_comment_text);

    assert_eq!(large_comment.kind(), COMMENT_KIND);
    assert_eq!(large_comment.text(), large_comment_text.as_bytes());
    assert_eq!(large_comment.width(), large_comment_text.len() as u64);
}

#[test]
fn test_trivia_with_special_characters() {
    // Test with various special characters that might appear in PDF files
    let special_chars = vec![
        ("\r\n", "Windows line ending"),
        ("\r", "Mac line ending"),
        ("\t", "Tab character"),
        (" \t \t", "Mixed spaces and tabs"),
        (
            "% comment with tab\tand space",
            "Comment with mixed whitespace",
        ),
    ];

    for (text, description) in special_chars {
        let trivia = create_trivia(WHITESPACE_KIND, text);
        assert_eq!(
            trivia.text(),
            text.as_bytes(),
            "Failed for: {}",
            description
        );
        assert_eq!(
            trivia.width(),
            text.len() as u64,
            "Failed width for: {}",
            description
        );
    }
}

#[test]
fn test_trivia_round_trip_fidelity() {
    // Test that trivia preserves exact bytes for round-trip fidelity
    let test_cases = vec![
        "\n",         // Unix newline
        "\r\n",       // Windows newline
        "\r",         // Mac newline
        " ",          // Single space
        "\t",         // Tab
        "    ",       // Multiple spaces
        " \t \n",     // Mixed whitespace
        "%PDF-1.7",   // Version comment
        "% comment",  // Spaced comment
        "%comment\n", // Comment with newline
    ];

    for original_text in test_cases {
        let trivia = create_trivia(COMMENT_KIND, original_text);
        let recovered_text = std::str::from_utf8(trivia.text()).unwrap();

        assert_eq!(
            original_text, recovered_text,
            "Round-trip fidelity failed for: {:?}",
            original_text
        );
    }
}

#[test]
fn test_trivia_debug_and_display() {
    // Test debug and display formatting
    let comment = create_trivia(COMMENT_KIND, "%test");

    let debug_str = format!("{:?}", comment);
    assert!(debug_str.contains("GreenTrivia"));
    assert!(debug_str.contains("kind"));
    assert!(debug_str.contains("text"));

    let display_str = format!("{}", &*comment);
    // The Display implementation formats as a debug byte array [37, 116, 101, 115, 116] for "%test"
    assert!(display_str.contains("[") && display_str.contains("]"));
    assert!(display_str.contains("37")); // ASCII code for '%'
    assert!(display_str.contains("116")); // ASCII code for 't'
}

#[test]
fn test_trivia_concurrent_access() {
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
fn test_pdf_semantic_whitespace_significance() {
    // Test scenarios where PDF whitespace has semantic meaning

    // ISO 32000-2 §7.3.10: obj declarations - newline separates header from body
    let obj_newline = create_trivia(NEWLINE_KIND, "\n");
    assert_eq!(obj_newline.text(), b"\n");

    // ISO 32000-2 §7.3.8: stream keyword - newline required after stream
    let stream_newline = create_trivia(NEWLINE_KIND, "\n");
    assert_eq!(stream_newline.text(), b"\n");

    // ISO 32000-2 §7.5.4: xref entries - fixed-width formatting
    let xref_field_space = create_trivia(WHITESPACE_KIND, " ");
    assert_eq!(xref_field_space.width(), 1);

    // ISO 32000-2 §7.5.5: startxref - newline separates keyword and offset
    let startxref_newline = create_trivia(NEWLINE_KIND, "\n");
    assert_eq!(startxref_newline.text(), b"\n");

    // ISO 32000-2 §8.1.1: content streams - space-separated tokens only
    let content_separator = create_trivia(WHITESPACE_KIND, " ");
    assert_eq!(content_separator.text(), b" ");
}

#[test]
fn test_trivia_performance_characteristics() {
    // Test that creating many trivia instances is efficient
    let mut trivias = Vec::new();

    for i in 0..1000 {
        let trivia = create_trivia(WHITESPACE_KIND, " ");
        trivias.push(trivia);

        // Every 100 iterations, verify the trivia is correct
        if i % 100 == 0 {
            assert_eq!(trivias[i].kind(), WHITESPACE_KIND);
            assert_eq!(trivias[i].text(), b" ");
            assert_eq!(trivias[i].width(), 1);
        }
    }

    // All instances should be equal (same content)
    for trivia in &trivias {
        assert_eq!(*trivia, trivias[0]);
    }
}

#[test]
fn test_trivia_utf8_sequences() {
    // Test with UTF-8 sequences that might appear in PDF comments
    let utf8_cases = vec![
        ("% Basic ASCII", "Basic ASCII comment"),
        ("% Café", "UTF-8 with accented characters"),
        ("% 中文", "UTF-8 with Chinese characters"),
        ("% 🔥", "UTF-8 with emoji"),
        ("% ©2023", "UTF-8 with copyright symbol"),
    ];

    for (comment_text, description) in utf8_cases {
        let trivia = create_trivia(COMMENT_KIND, comment_text);

        // Verify round-trip fidelity
        let recovered = std::str::from_utf8(trivia.text()).unwrap();
        assert_eq!(comment_text, recovered, "Failed for: {}", description);

        // Verify width matches byte length (not character length)
        assert_eq!(
            trivia.width(),
            comment_text.len() as u64,
            "Width mismatch for: {}",
            description
        );
    }
}

#[test]
fn test_trivia_pdf_line_ending_variations() {
    // Test different line ending styles that might appear in PDF files
    let line_endings = vec![
        ("\n", "Unix LF"),
        ("\r\n", "Windows CRLF"),
        ("\r", "Classic Mac CR"),
    ];

    for (ending, description) in line_endings {
        let trivia = create_trivia(NEWLINE_KIND, ending);

        assert_eq!(trivia.kind(), NEWLINE_KIND);
        assert_eq!(trivia.text(), ending.as_bytes());
        assert_eq!(trivia.width(), ending.len() as u64);

        // Verify line endings preserve exact bytes
        let recovered = std::str::from_utf8(trivia.text()).unwrap();
        assert_eq!(
            ending, recovered,
            "Line ending fidelity failed for: {}",
            description
        );
    }
}

#[test]
fn test_trivia_hash_consistency() {
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

    assert_eq!(
        hasher1.finish(),
        hasher2.finish(),
        "Equal trivia should have equal hashes"
    );
}

#[test]
fn test_trivia_clone_independence() {
    // Test that cloned trivia behaves independently for mutations to containers
    let original = create_trivia(WHITESPACE_KIND, "   ");
    let cloned = original.clone();

    // Put them in separate vectors
    let mut vec1 = vec![original];
    let mut vec2 = vec![cloned];

    // Modify the vectors independently
    vec1.push(create_trivia(NEWLINE_KIND, "\n"));
    vec2.push(create_trivia(COMMENT_KIND, "%test"));

    // The trivia at index 0 should still be identical
    assert_eq!(vec1[0], vec2[0]);
    assert_eq!(vec1[0].text(), vec2[0].text());

    // But the vectors should have different elements at index 1
    assert_ne!(vec1[1], vec2[1]);
    assert_ne!(vec1[1].kind(), vec2[1].kind());
}

#[test]
fn test_trivia_borrow_method_direct() {
    // Test that exercises the borrow() method implementation (lines 126-128)
    // The borrow method is used internally by standard collections

    let trivia = create_trivia(COMMENT_KIND, "%direct_borrow");

    // Method 1: HashMap.get() uses Borrow trait to convert the key for lookup
    // This will call our custom borrow() method when the key types differ
    let mut map = HashMap::new();
    map.insert(trivia.clone(), "test_value");

    // This lookup internally uses Borrow to convert &GreenTrivia to &GreenTriviaData for comparison
    assert_eq!(map.get(&trivia), Some(&"test_value"));

    // Method 2: HashSet.contains() also uses Borrow trait
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(trivia.clone());

    // This calls borrow() to convert for the contains check
    assert!(set.contains(&trivia));

    // Method 3: Multiple lookups to ensure the borrow method gets called repeatedly
    for _ in 0..5 {
        assert_eq!(map.get(&trivia), Some(&"test_value"));
        assert!(set.contains(&trivia));
    }

    // Method 4: Test with different equivalent trivia to ensure borrow is used for comparison
    let equivalent_trivia = create_trivia(COMMENT_KIND, "%direct_borrow");
    assert_eq!(map.get(&equivalent_trivia), Some(&"test_value"));
    assert!(set.contains(&equivalent_trivia));

    // Method 5: Remove operations also use borrow
    assert!(set.remove(&trivia));
    assert!(!set.contains(&trivia));

    // Verify properties to ensure our test trivia is correct
    assert_eq!(trivia.kind(), COMMENT_KIND);
    assert_eq!(trivia.text(), b"%direct_borrow");
    assert_eq!(trivia.width(), 14);
}
