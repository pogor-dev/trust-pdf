use crate::{
    NodeOrToken, SyntaxKind,
    green::{
        element::GreenElement, node::GreenNode, node_child::GreenChild, token::GreenToken,
        trivia::GreenTrivia,
    },
};

// Test constants for different PDF syntax kinds
const OBJ_KIND: SyntaxKind = SyntaxKind(1);
const STRING_KIND: SyntaxKind = SyntaxKind(2);
const NUMBER_KIND: SyntaxKind = SyntaxKind(3);
const DICT_KIND: SyntaxKind = SyntaxKind(4);
const NAME_KIND: SyntaxKind = SyntaxKind(5);

/// Helper function to create test tokens with different content types
fn create_token(kind: SyntaxKind, text: &str) -> GreenToken {
    let empty_trivia = GreenTrivia::new([]);
    GreenToken::new(kind, text.as_bytes(), empty_trivia.clone(), empty_trivia)
}

/// Helper function to create test nodes with given children
fn create_node(kind: SyntaxKind, children: Vec<GreenElement>) -> GreenNode {
    GreenNode::new(kind, children)
}

/// Helper function to create a simple dictionary node for testing
fn create_dict_node() -> GreenNode {
    let type_name = create_token(NAME_KIND, "/Type");
    let catalog_name = create_token(NAME_KIND, "/Catalog");
    create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(type_name),
            NodeOrToken::Token(catalog_name),
        ],
    )
}

#[test]
fn test_new_node_child_when_creating_with_offset_expect_correct_values() {
    let node = create_dict_node();
    let rel_offset = 42;

    let child = GreenChild::Node {
        rel_offset,
        node: node.clone(),
    };

    // Test that we can access the node and offset correctly
    match child {
        GreenChild::Node {
            rel_offset: offset,
            node: n,
        } => {
            assert_eq!(offset, 42);
            assert_eq!(n, node);
        }
        _ => panic!("Expected Node variant"),
    }
}

#[test]
fn test_new_token_child_when_creating_with_offset_expect_correct_values() {
    let token = create_token(STRING_KIND, "(Hello PDF)");
    let rel_offset = 24;

    let child = GreenChild::Token {
        rel_offset,
        token: token.clone(),
    };

    // Test that we can access the token and offset correctly
    match child {
        GreenChild::Token {
            rel_offset: offset,
            token: t,
        } => {
            assert_eq!(offset, 24);
            assert_eq!(t, token);
        }
        _ => panic!("Expected Token variant"),
    }
}

#[test]
fn test_as_ref_when_node_child_expect_node_element_ref() {
    let node = create_dict_node();
    let child = GreenChild::Node {
        rel_offset: 0,
        node: node.clone(),
    };

    let element_ref = child.as_ref();

    match element_ref {
        NodeOrToken::Node(node_data) => {
            // Verify we get a reference to the node's data
            assert_eq!(node_data.kind(), node.kind());
            assert_eq!(node_data.full_width(), node.full_width());
        }
        NodeOrToken::Token(_) => panic!("Expected Node element reference"),
    }
}

#[test]
fn test_as_ref_when_token_child_expect_token_element_ref() {
    let token = create_token(NUMBER_KIND, "123.45");
    let child = GreenChild::Token {
        rel_offset: 10,
        token: token.clone(),
    };

    let element_ref = child.as_ref();

    match element_ref {
        NodeOrToken::Token(token_data) => {
            // Verify we get a reference to the token's data
            assert_eq!(token_data.kind(), token.kind());
            assert_eq!(token_data.text(), token.text());
        }
        NodeOrToken::Node(_) => panic!("Expected Token element reference"),
    }
}

#[test]
fn test_rel_offset_when_node_child_expect_correct_offset() {
    let node = create_dict_node();
    let expected_offset = 100;
    let child = GreenChild::Node {
        rel_offset: expected_offset,
        node,
    };

    assert_eq!(child.rel_offset(), expected_offset);
}

#[test]
fn test_rel_offset_when_token_child_expect_correct_offset() {
    let token = create_token(NAME_KIND, "/Producer");
    let expected_offset = 50;
    let child = GreenChild::Token {
        rel_offset: expected_offset,
        token,
    };

    assert_eq!(child.rel_offset(), expected_offset);
}

#[test]
fn test_full_width_calculation_when_node_child_expect_correct_width() {
    // Create a node with known content to calculate expected width
    let type_token = create_token(NAME_KIND, "/Type"); // 5 bytes
    let catalog_token = create_token(NAME_KIND, "/Catalog"); // 8 bytes
    let node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(type_token),
            NodeOrToken::Token(catalog_token),
        ],
    );

    let rel_offset = 20;
    let child = GreenChild::Node { rel_offset, node };

    let element_ref = child.as_ref();
    let full_width = element_ref.full_width();

    // The full width should include all child tokens
    assert!(full_width >= 13); // At least the sum of token lengths
}

#[test]
fn test_full_width_calculation_when_token_child_expect_token_width() {
    let token = create_token(STRING_KIND, "(Hello World)"); // 13 bytes
    let rel_offset = 15;
    let child = GreenChild::Token { rel_offset, token };

    let element_ref = child.as_ref();
    let full_width = element_ref.full_width();

    // Should match the token's content length
    assert_eq!(full_width, 13);
}

#[test]
fn test_range_calculation_when_combining_offset_and_width_expect_correct_bounds() {
    let token = create_token(NUMBER_KIND, "42");
    let rel_offset = 100;
    let child = GreenChild::Token { rel_offset, token };

    let start = child.rel_offset();
    let width = child.as_ref().full_width();
    let end = start + width;

    assert_eq!(start, 100);
    assert_eq!(end, 100 + width);
    assert!(width > 0);
}

#[test]
fn test_clone_when_node_child_expect_identical_copy() {
    let node = create_dict_node();
    let original = GreenChild::Node {
        rel_offset: 30,
        node,
    };

    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.rel_offset(), cloned.rel_offset());

    // Verify both references point to the same underlying data
    match (&original, &cloned) {
        (GreenChild::Node { node: n1, .. }, GreenChild::Node { node: n2, .. }) => {
            assert_eq!(n1, n2);
        }
        _ => panic!("Both should be Node variants"),
    }
}

#[test]
fn test_clone_when_token_child_expect_identical_copy() {
    let token = create_token(NAME_KIND, "/Pages");
    let original = GreenChild::Token {
        rel_offset: 75,
        token,
    };

    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.rel_offset(), cloned.rel_offset());

    // Verify both references point to the same underlying data
    match (&original, &cloned) {
        (GreenChild::Token { token: t1, .. }, GreenChild::Token { token: t2, .. }) => {
            assert_eq!(t1, t2);
        }
        _ => panic!("Both should be Token variants"),
    }
}

#[test]
fn test_debug_when_node_child_expect_readable_output() {
    let node = create_dict_node();
    let child = GreenChild::Node {
        rel_offset: 12,
        node,
    };

    let debug_str = format!("{:?}", child);

    // Should contain variant name and key information
    assert!(debug_str.contains("Node"));
    assert!(debug_str.contains("rel_offset"));
    assert!(debug_str.contains("12"));
}

#[test]
fn test_debug_when_token_child_expect_readable_output() {
    let token = create_token(STRING_KIND, "(Debug Test)");
    let child = GreenChild::Token {
        rel_offset: 88,
        token,
    };

    let debug_str = format!("{:?}", child);

    // Should contain variant name and key information
    assert!(debug_str.contains("Token"));
    assert!(debug_str.contains("rel_offset"));
    assert!(debug_str.contains("88"));
}

#[test]
fn test_partial_eq_when_identical_nodes_expect_equal() {
    let node1 = create_dict_node();
    let node2 = node1.clone(); // Same underlying data due to Arc sharing

    let child1 = GreenChild::Node {
        rel_offset: 100,
        node: node1,
    };
    let child2 = GreenChild::Node {
        rel_offset: 100,
        node: node2,
    };

    assert_eq!(child1, child2);
}

#[test]
fn test_partial_eq_when_different_offsets_expect_not_equal() {
    let node = create_dict_node();

    let child1 = GreenChild::Node {
        rel_offset: 50,
        node: node.clone(),
    };
    let child2 = GreenChild::Node {
        rel_offset: 60,
        node,
    };

    assert_ne!(child1, child2);
}

#[test]
fn test_partial_eq_when_different_variants_expect_not_equal() {
    let node = create_dict_node();
    let token = create_token(STRING_KIND, "test");

    let node_child = GreenChild::Node {
        rel_offset: 0,
        node,
    };
    let token_child = GreenChild::Token {
        rel_offset: 0,
        token,
    };

    assert_ne!(node_child, token_child);
}

#[test]
fn test_hash_when_identical_children_expect_same_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let node = create_dict_node();
    let child1 = GreenChild::Node {
        rel_offset: 25,
        node: node.clone(),
    };
    let child2 = GreenChild::Node {
        rel_offset: 25,
        node,
    };

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    child1.hash(&mut hasher1);
    child2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_hash_when_different_offsets_expect_different_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let node = create_dict_node();
    let child1 = GreenChild::Node {
        rel_offset: 25,
        node: node.clone(),
    };
    let child2 = GreenChild::Node {
        rel_offset: 26,
        node,
    };

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    child1.hash(&mut hasher1);
    child2.hash(&mut hasher2);

    // Different offsets should produce different hashes
    assert_ne!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_memory_efficiency_when_multiple_references_expect_shared_data() {
    // Test that cloning doesn't duplicate the underlying node/token data
    let node = create_dict_node();
    let original = GreenChild::Node {
        rel_offset: 0,
        node,
    };

    // Create multiple references
    let children: Vec<_> = (0..100)
        .map(|i| GreenChild::Node {
            rel_offset: i * 10,
            node: match &original {
                GreenChild::Node { node, .. } => node.clone(),
                _ => unreachable!(),
            },
        })
        .collect();

    // All should reference the same underlying data (verified by equality)
    for child in &children {
        match (&original, child) {
            (GreenChild::Node { node: n1, .. }, GreenChild::Node { node: n2, .. }) => {
                assert_eq!(n1, n2); // Same underlying Arc data
            }
            _ => panic!("All should be Node variants"),
        }
    }
}

#[test]
fn test_pdf_specific_when_obj_structure_expect_correct_hierarchy() {
    // Test a realistic PDF object structure:
    // 1 0 obj
    // <<
    //   /Type /Catalog
    //   /Pages 2 0 R
    // >>
    // endobj

    let obj_num = create_token(NUMBER_KIND, "1");
    let gen_num = create_token(NUMBER_KIND, "0");
    let obj_keyword = create_token(OBJ_KIND, "obj");

    let type_name = create_token(NAME_KIND, "/Type");
    let catalog_name = create_token(NAME_KIND, "/Catalog");
    let pages_name = create_token(NAME_KIND, "/Pages");
    let pages_ref_obj = create_token(NUMBER_KIND, "2");
    let pages_ref_gen = create_token(NUMBER_KIND, "0");

    // Build dictionary
    let dict = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(type_name),
            NodeOrToken::Token(catalog_name),
            NodeOrToken::Token(pages_name),
            NodeOrToken::Token(pages_ref_obj),
            NodeOrToken::Token(pages_ref_gen),
        ],
    );

    // Build complete object
    let obj_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Token(obj_num),
            NodeOrToken::Token(gen_num),
            NodeOrToken::Token(obj_keyword),
            NodeOrToken::Node(dict),
        ],
    );

    let obj_child = GreenChild::Node {
        rel_offset: 0,
        node: obj_node,
    };

    // Verify structure
    assert_eq!(obj_child.rel_offset(), 0);

    match obj_child.as_ref() {
        NodeOrToken::Node(node) => {
            // Should have the expected structure and content
            assert!(node.full_width() > 0);
        }
        _ => panic!("Expected node reference"),
    }
}

#[test]
fn test_rel_range_when_node_child_expect_correct_range() {
    let node = create_dict_node();
    let rel_offset = 100;
    let child = GreenChild::Node { rel_offset, node };

    let range = child.rel_range();
    let expected_end = rel_offset + child.as_ref().full_width();

    assert_eq!(range.start, rel_offset);
    assert_eq!(range.end, expected_end);
    assert!(range.end > range.start); // Range should be non-empty
}

#[test]
fn test_rel_range_when_token_child_expect_correct_range() {
    let token = create_token(STRING_KIND, "(Hello World)"); // 13 bytes
    let rel_offset = 50;
    let child = GreenChild::Token { rel_offset, token };

    let range = child.rel_range();

    assert_eq!(range.start, 50);
    assert_eq!(range.end, 50 + 13); // Should be rel_offset + token width
    assert_eq!(range.len(), 13);
}

#[test]
fn test_rel_range_when_zero_offset_expect_range_from_zero() {
    let token = create_token(NUMBER_KIND, "42");
    let child = GreenChild::Token {
        rel_offset: 0,
        token,
    };

    let range = child.rel_range();

    assert_eq!(range.start, 0);
    assert_eq!(range.end, 2); // "42" is 2 bytes
    assert!(range.contains(&0));
    assert!(range.contains(&1));
    assert!(!range.contains(&2)); // Range is exclusive of end
}

#[test]
fn test_rel_range_when_large_offset_expect_correct_calculation() {
    let token = create_token(NAME_KIND, "/Producer");
    let large_offset = 1_000_000;
    let child = GreenChild::Token {
        rel_offset: large_offset,
        token,
    };

    let range = child.rel_range();
    let token_width = 9u32; // "/Producer" is 9 bytes

    assert_eq!(range.start, large_offset);
    assert_eq!(range.end, large_offset + token_width);
    assert_eq!(range.len(), token_width as usize);
}

#[test]
fn test_rel_range_when_empty_token_expect_empty_range() {
    let token = create_token(STRING_KIND, ""); // Empty token
    let rel_offset = 25;
    let child = GreenChild::Token { rel_offset, token };

    let range = child.rel_range();

    assert_eq!(range.start, rel_offset);
    assert_eq!(range.end, rel_offset); // Start equals end for empty range
    assert!(range.is_empty());
    assert_eq!(range.len(), 0);
}

#[test]
fn test_rel_range_when_complex_node_expect_full_subtree_range() {
    // Create a complex node with multiple children
    let type_token = create_token(NAME_KIND, "/Type"); // 5 bytes
    let catalog_token = create_token(NAME_KIND, "/Catalog"); // 8 bytes
    let pages_token = create_token(NAME_KIND, "/Pages"); // 6 bytes

    let complex_node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(type_token),
            NodeOrToken::Token(catalog_token),
            NodeOrToken::Token(pages_token),
        ],
    );

    let rel_offset = 200;
    let child = GreenChild::Node {
        rel_offset,
        node: complex_node,
    };

    let range = child.rel_range();
    let expected_width = child.as_ref().full_width();

    assert_eq!(range.start, rel_offset);
    assert_eq!(range.end, rel_offset + expected_width);
    assert!(range.len() >= 19); // At least the sum of token lengths
}

#[test]
fn test_rel_range_when_comparing_ranges_expect_correct_ordering() {
    let token1 = create_token(NUMBER_KIND, "1");
    let token2 = create_token(NUMBER_KIND, "2");

    let child1 = GreenChild::Token {
        rel_offset: 10,
        token: token1,
    };
    let child2 = GreenChild::Token {
        rel_offset: 20,
        token: token2,
    };

    let range1 = child1.rel_range();
    let range2 = child2.rel_range();

    // First range should come before second range
    assert!(range1.end <= range2.start);
    assert!(range1.start < range2.start);
    assert!(!range1.contains(&range2.start));
}

#[test]
fn test_rel_range_when_adjacent_children_expect_non_overlapping() {
    let token1 = create_token(STRING_KIND, "abc"); // 3 bytes
    let token2 = create_token(STRING_KIND, "def"); // 3 bytes

    let child1 = GreenChild::Token {
        rel_offset: 100,
        token: token1,
    };
    let child2 = GreenChild::Token {
        rel_offset: 103, // Right after first child
        token: token2,
    };

    let range1 = child1.rel_range();
    let range2 = child2.rel_range();

    // Ranges should be adjacent but not overlapping
    assert_eq!(range1.end, range2.start);
    assert_eq!(range1, 100..103);
    assert_eq!(range2, 103..106);
}

#[test]
fn test_rel_range_when_pdf_structure_expect_realistic_ranges() {
    // Test realistic PDF structure ranges:
    // "1 0 obj\n<<\n/Type /Catalog\n>>\nendobj"

    let obj_num = create_token(NUMBER_KIND, "1"); // 1 byte
    let gen_num = create_token(NUMBER_KIND, "0"); // 1 byte  
    let obj_keyword = create_token(OBJ_KIND, "obj"); // 3 bytes

    let child1 = GreenChild::Token {
        rel_offset: 0,
        token: obj_num,
    };
    let child2 = GreenChild::Token {
        rel_offset: 2, // After "1 "
        token: gen_num,
    };
    let child3 = GreenChild::Token {
        rel_offset: 4, // After "1 0 "
        token: obj_keyword,
    };

    let range1 = child1.rel_range();
    let range2 = child2.rel_range();
    let range3 = child3.rel_range();

    // Verify realistic PDF token ranges
    assert_eq!(range1, 0..1);
    assert_eq!(range2, 2..3);
    assert_eq!(range3, 4..7);

    // Verify no overlaps
    assert!(range1.end <= range2.start);
    assert!(range2.end <= range3.start);
}

#[test]
fn test_rel_range_when_nested_structure_expect_parent_encompasses_children() {
    // Create nested structure: dict containing tokens
    let inner_token = create_token(NAME_KIND, "/Type"); // 5 bytes
    let dict_node = create_node(DICT_KIND, vec![NodeOrToken::Token(inner_token)]);

    let dict_child = GreenChild::Node {
        rel_offset: 50,
        node: dict_node,
    };

    let dict_range = dict_child.rel_range();

    // The parent node range should encompass all its content
    assert!(dict_range.len() >= 5); // At least as wide as its child token
    assert_eq!(dict_range.start, 50);
    assert!(dict_range.end > dict_range.start);
}

#[test]
fn test_rel_range_when_different_token_types_expect_appropriate_widths() {
    // Test different PDF token types and their typical ranges
    let name_token = create_token(NAME_KIND, "/Producer");
    let string_token = create_token(STRING_KIND, "(Hello PDF)");
    let number_token = create_token(NUMBER_KIND, "123.456");

    let name_child = GreenChild::Token {
        rel_offset: 0,
        token: name_token,
    };
    let string_child = GreenChild::Token {
        rel_offset: 100,
        token: string_token,
    };
    let number_child = GreenChild::Token {
        rel_offset: 200,
        token: number_token,
    };

    let name_range = name_child.rel_range();
    let string_range = string_child.rel_range();
    let number_range = number_child.rel_range();

    // Verify each range matches expected token content
    assert_eq!(name_range.len(), 9); // "/Producer"
    assert_eq!(string_range.len(), 11); // "(Hello PDF)"
    assert_eq!(number_range.len(), 7); // "123.456"

    // Verify ranges start at expected offsets
    assert_eq!(name_range.start, 0);
    assert_eq!(string_range.start, 100);
    assert_eq!(number_range.start, 200);
}
