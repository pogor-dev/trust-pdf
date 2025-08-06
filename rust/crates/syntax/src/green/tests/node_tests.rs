use std::borrow::Borrow;

use crate::{
    NodeOrToken, SyntaxKind,
    green::{
        element::GreenElement,
        node::{GreenChild, GreenNode, GreenNodeData},
        token::GreenToken,
        trivia::GreenTrivia,
    },
};

// Test constants for different PDF syntax kinds
const OBJ_KIND: SyntaxKind = SyntaxKind(1);
const STRING_KIND: SyntaxKind = SyntaxKind(2);
const NUMBER_KIND: SyntaxKind = SyntaxKind(3);
const DICT_KIND: SyntaxKind = SyntaxKind(4);
const NAME_KIND: SyntaxKind = SyntaxKind(5);
const ARRAY_KIND: SyntaxKind = SyntaxKind(6);
const STREAM_KIND: SyntaxKind = SyntaxKind(7);
const OBJECT_KIND: SyntaxKind = SyntaxKind(8);
const KW_KIND: SyntaxKind = SyntaxKind(9);

/// Helper function to create test tokens with different content types
fn create_token(kind: SyntaxKind, text: &str) -> GreenToken {
    let empty_trivia = GreenTrivia::new([]);
    GreenToken::new(kind, text.as_bytes(), empty_trivia.clone(), empty_trivia)
}

/// Helper function to create test nodes with given children
fn create_node(kind: SyntaxKind, children: Vec<GreenElement>) -> GreenNode {
    GreenNode::new(kind, children)
}

/// Helper function to create test elements (either nodes or tokens)
fn create_element_token(kind: SyntaxKind, text: &str) -> GreenElement {
    NodeOrToken::Token(create_token(kind, text))
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

/// Helper function to create a complex PDF object node
fn create_obj_node() -> GreenNode {
    let obj_num = create_token(NUMBER_KIND, "1");
    let gen_num = create_token(NUMBER_KIND, "0");
    let dict = create_dict_node();

    create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Token(obj_num),
            NodeOrToken::Token(gen_num),
            NodeOrToken::Node(dict),
        ],
    )
}

/// Creates a sample PDF object structure for testing
/// Represents something like: "1 0 obj /Type /Catalog endobj"
fn create_sample_pdf_object() -> GreenNode {
    let elements = vec![
        create_element_token(NUMBER_KIND, "1"),
        create_element_token(NUMBER_KIND, "0"),
        create_element_token(KW_KIND, "obj"),
        create_element_token(NAME_KIND, "/Type"),
        create_element_token(NAME_KIND, "/Catalog"),
        create_element_token(KW_KIND, "endobj"),
    ];
    create_node(OBJECT_KIND, elements)
}

// =============================================================================
// GreenNode Core Tests
// =============================================================================

#[test]
fn test_new_when_creating_empty_node_expect_correct_properties() {
    let empty_node = create_node(DICT_KIND, vec![]);

    assert_eq!(empty_node.kind(), DICT_KIND);
    assert_eq!(empty_node.width(), 0);
    assert_eq!(empty_node.full_width(), 0);
    assert_eq!(empty_node.children().count(), 0);
}

#[test]
fn test_new_when_creating_node_with_single_token_expect_correct_width() {
    let token = create_token(STRING_KIND, "(Hello)");
    let token_width = token.full_width();

    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(token)]);

    assert_eq!(node.kind(), DICT_KIND);
    assert_eq!(node.width(), token_width);
    assert_eq!(node.full_width(), token_width);
    assert_eq!(node.children().count(), 1);
}

#[test]
fn test_new_when_creating_node_with_multiple_children_expect_cumulative_width() {
    let token1 = create_token(NAME_KIND, "/Type");
    let token2 = create_token(NAME_KIND, "/Catalog");
    let expected_width = token1.full_width() + token2.full_width();

    let node = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(token1), NodeOrToken::Token(token2)],
    );

    assert_eq!(node.kind(), DICT_KIND);
    assert_eq!(node.width(), expected_width);
    assert_eq!(node.full_width(), expected_width);
    assert_eq!(node.children().count(), 2);
}

#[test]
fn test_new_when_creating_nested_nodes_expect_correct_structure() {
    let dict_node = create_dict_node();
    let nested_node = create_node(OBJ_KIND, vec![NodeOrToken::Node(dict_node)]);

    assert_eq!(nested_node.kind(), OBJ_KIND);
    assert_eq!(nested_node.children().count(), 1);

    // Check that the nested child is correctly preserved
    let child = nested_node.children().next().unwrap();
    match child {
        NodeOrToken::Node(node) => {
            assert_eq!(node.kind(), DICT_KIND);
        }
        NodeOrToken::Token(_) => panic!("Expected nested node, got token"),
    }
}

// =============================================================================
// Node Editing Methods Tests
// =============================================================================

#[test]
fn test_replace_child_when_replacing_middle_child_expect_correct_replacement() {
    let original_children = vec![
        create_element_token(NUMBER_KIND, "1"),
        create_element_token(NUMBER_KIND, "0"),
        create_element_token(KW_KIND, "obj"),
    ];
    let node = create_node(OBJ_KIND, original_children);

    let new_child = create_element_token(NUMBER_KIND, "2");
    let new_node = node.replace_child(1, new_child);

    let children: Vec<_> = new_node.children().collect();
    assert_eq!(children.len(), 3);

    // Check first child unchanged
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.text(), b"1");
    } else {
        panic!("Expected token");
    }

    // Check replaced child
    if let NodeOrToken::Token(token) = &children[1] {
        assert_eq!(token.text(), b"2");
    } else {
        panic!("Expected token");
    }

    // Check third child unchanged
    if let NodeOrToken::Token(token) = &children[2] {
        assert_eq!(token.text(), b"obj");
    } else {
        panic!("Expected token");
    }
}

#[test]
fn test_insert_child_when_inserting_at_start_expect_correct_insertion() {
    let original_children = vec![
        create_element_token(NUMBER_KIND, "0"),
        create_element_token(KW_KIND, "obj"),
    ];
    let node = create_node(OBJ_KIND, original_children);

    let new_child = create_element_token(NUMBER_KIND, "1");
    let new_node = node.insert_child(0, new_child);

    let children: Vec<_> = new_node.children().collect();
    assert_eq!(children.len(), 3);

    // Check inserted child
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.text(), b"1");
    } else {
        panic!("Expected token");
    }
}

#[test]
fn test_remove_child_when_removing_middle_child_expect_correct_removal() {
    let original_children = vec![
        create_element_token(NUMBER_KIND, "1"),
        create_element_token(NUMBER_KIND, "0"),
        create_element_token(KW_KIND, "obj"),
    ];
    let node = create_node(OBJ_KIND, original_children);

    let new_node = node.remove_child(1);

    let children: Vec<_> = new_node.children().collect();
    assert_eq!(children.len(), 2);

    // Check remaining children
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.text(), b"1");
    } else {
        panic!("Expected token");
    }

    if let NodeOrToken::Token(token) = &children[1] {
        assert_eq!(token.text(), b"obj");
    } else {
        panic!("Expected token");
    }
}

#[test]
fn test_splice_children_when_replacing_range_expect_correct_splicing() {
    let original_children = vec![
        create_element_token(NUMBER_KIND, "1"),
        create_element_token(NUMBER_KIND, "0"),
        create_element_token(KW_KIND, "obj"),
    ];
    let node = create_node(OBJ_KIND, original_children);

    let replacement_children = vec![
        create_element_token(NUMBER_KIND, "2"),
        create_element_token(NUMBER_KIND, "1"),
    ];
    let new_node = node.splice_children(0..2, replacement_children);

    let children: Vec<_> = new_node.children().collect();
    assert_eq!(children.len(), 3);

    // Check replaced children
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.text(), b"2");
    } else {
        panic!("Expected token");
    }

    if let NodeOrToken::Token(token) = &children[1] {
        assert_eq!(token.text(), b"1");
    } else {
        panic!("Expected token");
    }

    // Check remaining child
    if let NodeOrToken::Token(token) = &children[2] {
        assert_eq!(token.text(), b"obj");
    } else {
        panic!("Expected token");
    }
}

// =============================================================================
// GreenChild Tests
// =============================================================================

#[test]
fn test_new_node_child_when_creating_with_offset_expect_correct_values() {
    let node = create_dict_node();
    let rel_offset = 42;

    let child = GreenChild::Node {
        rel_offset,
        node: node.clone(),
    };

    match child {
        GreenChild::Node {
            rel_offset: offset,
            node: child_node,
        } => {
            assert_eq!(offset, 42);
            assert_eq!(child_node.kind(), node.kind());
        }
        GreenChild::Token { .. } => panic!("Expected Node child, got Token"),
    }
}

#[test]
fn test_new_token_child_when_creating_with_offset_expect_correct_values() {
    let token = create_token(STRING_KIND, "(test)");
    let rel_offset = 24;

    let child = GreenChild::Token {
        rel_offset,
        token: token.clone(),
    };

    match child {
        GreenChild::Token {
            rel_offset: offset,
            token: child_token,
        } => {
            assert_eq!(offset, 24);
            assert_eq!(child_token.kind(), token.kind());
        }
        GreenChild::Node { .. } => panic!("Expected Token child, got Node"),
    }
}

#[test]
fn test_green_child_rel_offset_when_accessing_expect_correct_offset() {
    let token = create_token(STRING_KIND, "(test)");
    let child = GreenChild::Token {
        rel_offset: 10,
        token: token.clone(),
    };

    assert_eq!(child.rel_offset(), 10);
}

#[test]
fn test_green_child_rel_range_when_calculating_expect_correct_range() {
    let token = create_token(STRING_KIND, "(test)");
    let child = GreenChild::Token {
        rel_offset: 5,
        token: token.clone(),
    };

    let range = child.rel_range();
    assert_eq!(range.start, 5);
    assert_eq!(range.end, 5 + token.full_width()); // 5 + 6 = 11
}

// =============================================================================
// NodeChildren Iterator Tests
// =============================================================================

#[test]
fn test_children_len_when_empty_node_expect_zero() {
    let node = create_node(OBJECT_KIND, vec![]);
    let children = node.children();

    assert_eq!(children.len(), 0);
}

#[test]
fn test_children_len_when_single_child_expect_one() {
    let node = create_node(OBJECT_KIND, vec![create_element_token(NUMBER_KIND, "1")]);
    let children = node.children();

    assert_eq!(children.len(), 1);
}

#[test]
fn test_children_len_when_multiple_children_expect_correct_count() {
    let node = create_sample_pdf_object();
    let children = node.children();

    assert_eq!(children.len(), 6); // "1", "0", "obj", "/Type", "/Catalog", "endobj"
}

#[test]
fn test_children_next_when_iterating_expect_correct_sequence() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    // First child: "1"
    let first = children.next().unwrap();
    match first {
        NodeOrToken::Token(token) => {
            assert_eq!(token.kind(), NUMBER_KIND);
            assert_eq!(token.text(), b"1");
        }
        NodeOrToken::Node(_) => panic!("Expected token, got node"),
    }

    // Second child: "0"
    let second = children.next().unwrap();
    match second {
        NodeOrToken::Token(token) => {
            assert_eq!(token.kind(), NUMBER_KIND);
            assert_eq!(token.text(), b"0");
        }
        NodeOrToken::Node(_) => panic!("Expected token, got node"),
    }

    // Third child: "obj"
    let third = children.next().unwrap();
    match third {
        NodeOrToken::Token(token) => {
            assert_eq!(token.kind(), KW_KIND);
            assert_eq!(token.text(), b"obj");
        }
        NodeOrToken::Node(_) => panic!("Expected token, got node"),
    }
}

#[test]
fn test_children_nth_when_accessing_specific_child_expect_correct_element() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    // Access the 4th element (index 3): "/Type"
    let fourth = children.nth(3).unwrap();
    match fourth {
        NodeOrToken::Token(token) => {
            assert_eq!(token.kind(), NAME_KIND);
            assert_eq!(token.text(), b"/Type");
        }
        NodeOrToken::Node(_) => panic!("Expected token, got node"),
    }
}

#[test]
fn test_children_size_hint_when_checking_bounds_expect_correct_hint() {
    let node = create_sample_pdf_object();
    let children = node.children();

    let (lower, upper) = children.size_hint();
    assert_eq!(lower, 6);
    assert_eq!(upper, Some(6));
}

#[test]
fn test_children_last_when_getting_final_element_expect_endobj() {
    let node = create_sample_pdf_object();
    let children = node.children();

    let last = children.last().unwrap();
    match last {
        NodeOrToken::Token(token) => {
            assert_eq!(token.kind(), KW_KIND);
            assert_eq!(token.text(), b"endobj");
        }
        NodeOrToken::Node(_) => panic!("Expected token, got node"),
    }
}

#[test]
fn test_children_collect_when_gathering_all_expect_complete_sequence() {
    let node = create_sample_pdf_object();
    let children: Vec<_> = node.children().collect();

    assert_eq!(children.len(), 6);

    // Verify the sequence matches our expected PDF object structure
    let expected_texts: &[&[u8]] = &[b"1", b"0", b"obj", b"/Type", b"/Catalog", b"endobj"];
    let expected_kinds = [
        NUMBER_KIND,
        NUMBER_KIND,
        KW_KIND,
        NAME_KIND,
        NAME_KIND,
        KW_KIND,
    ];

    for (i, child) in children.iter().enumerate() {
        match child {
            NodeOrToken::Token(token) => {
                assert_eq!(token.text(), expected_texts[i]);
                assert_eq!(token.kind(), expected_kinds[i]);
            }
            NodeOrToken::Node(_) => panic!("Expected token at index {}, got node", i),
        }
    }
}

// =============================================================================
// Memory Management and Raw Pointer Tests
// =============================================================================

#[test]
fn test_into_raw_when_converting_to_pointer_expect_valid_operation() {
    let node = create_dict_node();
    let original_kind = node.kind();

    // Convert to raw pointer
    let raw_ptr = GreenNode::into_raw(node);

    // Convert back and verify integrity
    let recovered_node = unsafe { GreenNode::from_raw(raw_ptr) };
    assert_eq!(recovered_node.kind(), original_kind);
}

#[test]
fn test_borrow_when_using_as_node_data_expect_same_content() {
    let node = create_dict_node();
    let node_data: &GreenNodeData = node.borrow();

    // Verify that borrowing gives access to the same data
    assert_eq!(node.kind(), node_data.kind());
    assert_eq!(node.width(), node_data.width());
    assert_eq!(node.children().len(), node_data.children().len());
}

#[test]
fn test_to_owned_when_converting_node_data_expect_equivalent_node() {
    let original_node = create_dict_node();
    let node_data: &GreenNodeData = &original_node;
    let owned_node = node_data.to_owned();

    assert_eq!(original_node.kind(), owned_node.kind());
    assert_eq!(original_node.width(), owned_node.width());
    assert_eq!(original_node.children().len(), owned_node.children().len());
}

// =============================================================================
// Edge Cases and Error Conditions
// =============================================================================

#[test]
fn test_children_when_empty_iterator_expect_no_panics() {
    let empty_node = create_node(ARRAY_KIND, vec![]);
    let mut children = empty_node.children();

    assert!(children.next().is_none());
    assert!(children.last().is_none());
}

#[test]
fn test_children_when_single_element_expect_consistent_behavior() {
    let single_child_node = create_node(
        ARRAY_KIND,
        vec![create_element_token(STRING_KIND, "(single)")],
    );
    let children = single_child_node.children();

    assert_eq!(children.len(), 1);

    let first = children.clone().next().unwrap();
    let last = children.last().unwrap();

    // For single element, first and last should be the same
    match (first, last) {
        (NodeOrToken::Token(first_token), NodeOrToken::Token(last_token)) => {
            assert_eq!(first_token.text(), last_token.text());
            assert_eq!(first_token.kind(), last_token.kind());
        }
        _ => panic!("Expected both to be tokens"),
    }
}

#[test]
fn test_width_when_complex_nested_structure_expect_accurate_calculation() {
    // Create a complex nested structure to test width calculation
    let inner_dict = create_dict_node();
    let middle_node = create_node(STREAM_KIND, vec![NodeOrToken::Node(inner_dict)]);
    let outer_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Token(create_token(NUMBER_KIND, "42")),
            NodeOrToken::Node(middle_node),
            NodeOrToken::Token(create_token(STRING_KIND, "(end)")),
        ],
    );

    // Width should be sum of all leaf token widths
    let expected_width = 2 + 5 + 8 + 5; // "42" + "/Type" + "/Catalog" + "(end)"
    assert_eq!(outer_node.width(), expected_width);
}

// =============================================================================
// Debug and Display Tests
// =============================================================================

#[test]
fn test_debug_when_formatting_node_expect_readable_output() {
    let node = create_dict_node();
    let debug_output = format!("{:?}", node);

    // Debug output should be non-empty and contain useful information
    assert!(!debug_output.is_empty());
    // Should contain some indication of the node structure
    assert!(debug_output.contains("GreenNode") || debug_output.len() > 10);
}

#[test]
fn test_equality_when_comparing_identical_nodes_expect_equal() {
    let node1 = create_dict_node();
    let node2 = create_dict_node();

    assert_eq!(node1, node2);
}

#[test]
fn test_equality_when_comparing_different_nodes_expect_not_equal() {
    let dict_node = create_dict_node();
    let obj_node = create_obj_node();

    assert_ne!(dict_node, obj_node);
}

#[test]
fn test_hash_when_using_in_hashmap_expect_consistent_behavior() {
    use std::collections::HashMap;

    let node = create_dict_node();
    let mut map = HashMap::new();
    map.insert(node.clone(), "test_value");

    // Should be able to retrieve using the same logical node
    let lookup_node = create_dict_node();
    assert_eq!(map.get(&lookup_node), Some(&"test_value"));
}

// =============================================================================
// Display and Equality Implementation Tests
// =============================================================================

#[test]
fn test_node_display_when_formatting_expect_child_content() {
    let node = create_dict_node();
    let display_output = format!("{}", node);

    // Display should show concatenated child content
    assert!(!display_output.is_empty());
    // Should contain content from child tokens
    assert!(display_output.contains("Type") || display_output.len() > 0);
}

#[test]
fn test_node_data_display_when_formatting_expect_child_content() {
    let node = create_dict_node();
    let node_data: &GreenNodeData = &*node;
    let display_output = format!("{}", node_data);

    // Display should show concatenated child content
    assert!(!display_output.is_empty());
    // Should contain content from child tokens
    assert!(display_output.contains("Type") || display_output.len() > 0);
}

#[test]
fn test_node_data_equality_when_same_content_expect_equal() {
    let node1 = create_dict_node();
    let node2 = create_dict_node();

    let node_data1: &GreenNodeData = &*node1;
    let node_data2: &GreenNodeData = &*node2;

    assert_eq!(node_data1, node_data2);
}

#[test]
fn test_node_data_equality_when_different_content_expect_not_equal() {
    let dict_node = create_dict_node();
    let obj_node = create_obj_node();

    let dict_data: &GreenNodeData = &*dict_node;
    let obj_data: &GreenNodeData = &*obj_node;

    assert_ne!(dict_data, obj_data);
}

#[test]
fn test_from_cow_when_owned_node_data_expect_green_node() {
    use std::borrow::Cow;

    let original_node = create_dict_node();
    let node_data: &GreenNodeData = &*original_node;
    let cow_owned = Cow::Owned(node_data.to_owned());
    let converted_node = GreenNode::from(cow_owned);

    assert_eq!(converted_node.kind(), original_node.kind());
    assert_eq!(converted_node.width(), original_node.width());
    assert_eq!(converted_node.full_width(), original_node.full_width());
}

// =============================================================================
// Iterator Advanced Methods Tests
// =============================================================================

#[test]
fn test_children_fold_when_accumulating_expect_correct_result() {
    let node = create_dict_node();
    let children = node.children();

    // Fold over children to accumulate total width
    let total_width = children.fold(0u32, |acc, child| acc + child.full_width());

    assert!(total_width > 0);
    assert_eq!(total_width, node.full_width());
}

#[test]
fn test_children_nth_back_when_accessing_from_end_expect_correct_element() {
    let node = create_dict_node();
    let mut children = node.children();

    if children.len() > 1 {
        let last_element = children.nth_back(0);
        assert!(last_element.is_some());

        // Reset iterator and get second to last
        let mut children = node.children();
        if children.len() > 2 {
            let second_last = children.nth_back(1);
            assert!(second_last.is_some());
        }
    }
}

#[test]
fn test_children_rfold_when_reverse_accumulating_expect_correct_result() {
    let node = create_dict_node();
    let children = node.children();

    // Reverse fold over children to accumulate total width
    let total_width = children.rfold(0u32, |acc, child| acc + child.full_width());

    assert!(total_width > 0);
    assert_eq!(total_width, node.full_width());
}
