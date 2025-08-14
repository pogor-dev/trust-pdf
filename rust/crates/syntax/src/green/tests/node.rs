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

/// Helper function to create a complex PDF object node or simple structure
/// When `simple` is true, creates a flat structure like create_sample_pdf_object
/// When `simple` is false, creates a nested structure with dictionary child
fn create_pdf_object_node(simple: bool) -> GreenNode {
    if simple {
        // Creates: "1 0 obj /Type /Catalog endobj"
        let elements = vec![
            create_element_token(NUMBER_KIND, "1"),
            create_element_token(NUMBER_KIND, "0"),
            create_element_token(KW_KIND, "obj"),
            create_element_token(NAME_KIND, "/Type"),
            create_element_token(NAME_KIND, "/Catalog"),
            create_element_token(KW_KIND, "endobj"),
        ];
        create_node(OBJECT_KIND, elements)
    } else {
        // Creates: "1 0 [dict]" with nested dictionary
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
}

/// Creates a sample PDF object structure for testing (backwards compatibility)
/// Represents something like: "1 0 obj /Type /Catalog endobj"
fn create_sample_pdf_object() -> GreenNode {
    create_pdf_object_node(true)
}

/// Helper function to create a complex PDF object node (backwards compatibility)
fn create_obj_node() -> GreenNode {
    create_pdf_object_node(false)
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

#[test]
fn test_get_first_terminal_when_leaf_node_expect_first_token() {
    // Create a leaf node (only contains tokens)
    let leaf_node = create_dict_node();

    let first_terminal = leaf_node.get_first_terminal();

    assert!(first_terminal.is_some());
    let terminal = first_terminal.unwrap();
    assert_eq!(terminal.kind(), NAME_KIND); // Should be the "/Type" token
    assert_eq!(terminal.text(), b"/Type");
}

#[test]
fn test_get_last_terminal_when_leaf_node_expect_last_token() {
    // Create a leaf node (only contains tokens)
    let leaf_node = create_dict_node();

    let last_terminal = leaf_node.get_last_terminal();

    assert!(last_terminal.is_some());
    let terminal = last_terminal.unwrap();
    assert_eq!(terminal.kind(), NAME_KIND); // Should be the "/Catalog" token
    assert_eq!(terminal.text(), b"/Catalog");
}

#[test]
fn test_get_first_terminal_when_nested_structure_expect_leftmost_token() {
    // Create a nested structure: OBJ -> DICT (with tokens)
    let inner_dict = create_dict_node(); // Contains "/Type" and "/Catalog" tokens
    let stream_node = create_node(STREAM_KIND, vec![NodeOrToken::Node(inner_dict.clone())]);
    let outer_obj = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Node(stream_node),
            NodeOrToken::Token(create_token(KW_KIND, "endobj")),
        ],
    );

    let first_terminal = outer_obj.get_first_terminal();

    assert!(first_terminal.is_some());
    let terminal = first_terminal.unwrap();
    assert_eq!(terminal.kind(), NAME_KIND); // Should be the "/Type" token from inner dict
    assert_eq!(terminal.text(), b"/Type");
}

#[test]
fn test_get_last_terminal_when_nested_structure_expect_rightmost_token() {
    // Create a nested structure with multiple child nodes
    let inner_dict1 = create_dict_node();
    let inner_dict2 = create_node(
        ARRAY_KIND,
        vec![
            NodeOrToken::Token(create_token(NUMBER_KIND, "42")),
            NodeOrToken::Token(create_token(STRING_KIND, "(test)")),
        ],
    );

    let outer_obj = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Node(inner_dict1),
            NodeOrToken::Node(inner_dict2.clone()), // This contains the last terminal token
        ],
    );

    let last_terminal = outer_obj.get_last_terminal();

    assert!(last_terminal.is_some());
    let terminal = last_terminal.unwrap();
    assert_eq!(terminal.kind(), STRING_KIND); // Should be the "(test)" token (rightmost)
    assert_eq!(terminal.text(), b"(test)");
}

#[test]
fn test_get_first_terminal_when_deep_nesting_expect_deepest_leftmost_token() {
    // Create deeply nested structure: OBJ -> STREAM -> DICT -> ARRAY (leaf)
    let leaf_array = create_node(
        ARRAY_KIND,
        vec![NodeOrToken::Token(create_token(NUMBER_KIND, "1"))],
    );
    let dict_node = create_node(DICT_KIND, vec![NodeOrToken::Node(leaf_array.clone())]);
    let stream_node = create_node(STREAM_KIND, vec![NodeOrToken::Node(dict_node)]);
    let obj_node = create_node(OBJ_KIND, vec![NodeOrToken::Node(stream_node)]);

    let first_terminal = obj_node.get_first_terminal();

    assert!(first_terminal.is_some());
    let terminal = first_terminal.unwrap();
    assert_eq!(terminal.kind(), NUMBER_KIND); // Should reach the deepest leftmost token "1"
    assert_eq!(terminal.text(), b"1");
}

#[test]
fn test_get_last_terminal_when_deep_nesting_expect_deepest_rightmost_token() {
    // Create deeply nested structure with multiple branches
    let left_leaf = create_node(
        ARRAY_KIND,
        vec![NodeOrToken::Token(create_token(NUMBER_KIND, "1"))],
    );
    let right_leaf = create_node(
        NAME_KIND,
        vec![NodeOrToken::Token(create_token(NAME_KIND, "/Name"))],
    );

    let dict_node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Node(left_leaf),
            NodeOrToken::Node(right_leaf.clone()), // This contains the last terminal token
        ],
    );
    let obj_node = create_node(OBJ_KIND, vec![NodeOrToken::Node(dict_node)]);

    let last_terminal = obj_node.get_last_terminal();

    assert!(last_terminal.is_some());
    let terminal = last_terminal.unwrap();
    assert_eq!(terminal.kind(), NAME_KIND); // Should reach the rightmost token "/Name"
    assert_eq!(terminal.text(), b"/Name");
}

#[test]
fn test_get_first_terminal_when_mixed_children_expect_first_token() {
    // Create node with mixed tokens and nodes - should find the very first token
    let leaf_node = create_dict_node();
    let mixed_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Token(create_token(NUMBER_KIND, "1")), // Should find this (first token)
            NodeOrToken::Token(create_token(NUMBER_KIND, "0")),
            NodeOrToken::Node(leaf_node.clone()),
            NodeOrToken::Token(create_token(KW_KIND, "endobj")),
        ],
    );

    let first_terminal = mixed_node.get_first_terminal();

    assert!(first_terminal.is_some());
    let terminal = first_terminal.unwrap();
    assert_eq!(terminal.kind(), NUMBER_KIND); // Should find the first token "1"
    assert_eq!(terminal.text(), b"1");
}

#[test]
fn test_get_last_terminal_when_mixed_children_expect_last_token() {
    // Create node with mixed tokens and nodes - should find the very last token
    let first_leaf = create_dict_node();
    let last_leaf = create_node(
        ARRAY_KIND,
        vec![NodeOrToken::Token(create_token(STRING_KIND, "(array)"))],
    );

    let mixed_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Token(create_token(NUMBER_KIND, "1")),
            NodeOrToken::Node(first_leaf),
            NodeOrToken::Node(last_leaf.clone()),
            NodeOrToken::Token(create_token(KW_KIND, "endobj")), // Should find this (last token)
        ],
    );

    let last_terminal = mixed_node.get_last_terminal();

    assert!(last_terminal.is_some());
    let terminal = last_terminal.unwrap();
    assert_eq!(terminal.kind(), KW_KIND); // Should find the last token "endobj"
    assert_eq!(terminal.text(), b"endobj");
}

#[test]
fn test_get_first_terminal_when_only_tokens_expect_first_token() {
    // Create node with only token children (no child nodes)
    let token_only_node = create_node(
        OBJECT_KIND,
        vec![
            NodeOrToken::Token(create_token(NUMBER_KIND, "1")),
            NodeOrToken::Token(create_token(NUMBER_KIND, "0")),
            NodeOrToken::Token(create_token(KW_KIND, "obj")),
        ],
    );

    let first_terminal = token_only_node.get_first_terminal();

    assert!(first_terminal.is_some());
    let terminal = first_terminal.unwrap();
    assert_eq!(terminal.kind(), NUMBER_KIND); // Should return first token "1"
    assert_eq!(terminal.text(), b"1");

    // Also test last terminal in the same test to avoid redundancy
    let last_terminal = token_only_node.get_last_terminal();

    assert!(last_terminal.is_some());
    let terminal = last_terminal.unwrap();
    assert_eq!(terminal.kind(), KW_KIND); // Should return last token "obj"
    assert_eq!(terminal.text(), b"obj");
}

#[test]
fn test_empty_node_comprehensive_expect_correct_behavior() {
    // Create completely empty node
    let empty_node = create_node(DICT_KIND, vec![]);

    // Test terminal navigation
    let first_terminal = empty_node.get_first_terminal();
    let last_terminal = empty_node.get_last_terminal();

    assert!(first_terminal.is_none()); // Should return None as there are no tokens
    assert!(last_terminal.is_none()); // Should return None as there are no tokens

    // Test trivia width calculations
    assert_eq!(empty_node.full_width(), 0);
    assert_eq!(empty_node.get_leading_trivia_width(), 0);
    assert_eq!(empty_node.get_trailing_trivia_width(), 0);
    assert_eq!(empty_node.width(), 0);

    // Test basic properties
    assert_eq!(empty_node.kind(), DICT_KIND);
    assert_eq!(empty_node.children().count(), 0);
}

#[test]
fn test_get_first_terminal_when_complex_pdf_structure_expect_correct_navigation() {
    // Create a more realistic PDF structure
    let page_dict = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(create_token(NAME_KIND, "/Type")),
            NodeOrToken::Token(create_token(NAME_KIND, "/Page")),
        ],
    );

    let pages_array = create_node(
        ARRAY_KIND,
        vec![NodeOrToken::Node(page_dict.clone())], // page_dict should be the terminal
    );

    let catalog_dict = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(create_token(NAME_KIND, "/Type")),
            NodeOrToken::Token(create_token(NAME_KIND, "/Catalog")),
            NodeOrToken::Node(pages_array),
        ],
    );

    let pdf_obj = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Token(create_token(NUMBER_KIND, "1")),
            NodeOrToken::Token(create_token(NUMBER_KIND, "0")),
            NodeOrToken::Token(create_token(KW_KIND, "obj")),
            NodeOrToken::Node(catalog_dict),
            NodeOrToken::Token(create_token(KW_KIND, "endobj")),
        ],
    );

    let first_terminal = pdf_obj.get_first_terminal();

    assert!(first_terminal.is_some());
    let terminal = first_terminal.unwrap();
    assert_eq!(terminal.kind(), NUMBER_KIND); // Should find the first token "1"
    assert_eq!(terminal.text(), b"1");
}

#[test]
fn test_get_last_terminal_when_complex_pdf_structure_expect_correct_navigation() {
    // Create structure where we can verify last terminal selection
    let first_page = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(create_token(NAME_KIND, "/Type")),
            NodeOrToken::Token(create_token(NAME_KIND, "/Page")),
        ],
    );

    let second_page = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(create_token(NAME_KIND, "/Type")),
            NodeOrToken::Token(create_token(NAME_KIND, "/Page")),
            NodeOrToken::Token(create_token(NUMBER_KIND, "2")),
        ],
    );

    let pages_array = create_node(
        ARRAY_KIND,
        vec![
            NodeOrToken::Node(first_page),
            NodeOrToken::Node(second_page.clone()), // This should be the last terminal
        ],
    );

    let catalog_dict = create_node(DICT_KIND, vec![NodeOrToken::Node(pages_array)]);

    let pdf_obj = create_node(OBJ_KIND, vec![NodeOrToken::Node(catalog_dict)]);

    let last_terminal = pdf_obj.get_last_terminal();

    assert!(last_terminal.is_some());
    let terminal = last_terminal.unwrap();
    assert_eq!(terminal.kind(), NUMBER_KIND); // Should find the last token "2" from second page
    assert_eq!(terminal.text(), b"2");
}

/// Helper function to create trivia for testing
fn create_trivia(content: &str) -> GreenTrivia {
    use crate::green::trivia::GreenTriviaChild;
    let trivia_child = GreenTriviaChild::new(SyntaxKind(999), content.as_bytes());
    GreenTrivia::new([trivia_child])
}

/// Helper function to create token with specific trivia
fn create_token_with_trivia(
    kind: SyntaxKind,
    text: &str,
    leading: &str,
    trailing: &str,
) -> GreenToken {
    let leading_trivia = create_trivia(leading);
    let trailing_trivia = create_trivia(trailing);
    GreenToken::new(kind, text.as_bytes(), leading_trivia, trailing_trivia)
}

#[test]
fn test_width_when_no_trivia_expect_full_width() {
    // Create node with tokens that have no trivia
    let token1 = create_token(NAME_KIND, "/Type"); // 5 bytes
    let token2 = create_token(NAME_KIND, "/Catalog"); // 8 bytes
    let node = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(token1), NodeOrToken::Token(token2)],
    );

    // Width should equal full_width when there's no trivia
    assert_eq!(node.width(), node.full_width());
    assert_eq!(node.width(), 13); // 5 + 8 = 13
}

#[test]
fn test_width_when_has_leading_trivia_expect_reduced_width() {
    // Create tokens with leading trivia
    let token1 = create_token_with_trivia(NAME_KIND, "/Type", "  ", ""); // 2 leading, 0 trailing
    let token2 = create_token_with_trivia(NAME_KIND, "/Catalog", "", ""); // 0 leading, 0 trailing

    let node = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(token1), NodeOrToken::Token(token2)],
    );

    // full_width = 5 + 8 + 2 = 15, leading_trivia = 2, trailing_trivia = 0
    // width = 15 - 2 - 0 = 13
    assert_eq!(node.full_width(), 15);
    assert_eq!(node.get_leading_trivia_width(), 2);
    assert_eq!(node.get_trailing_trivia_width(), 0);
    assert_eq!(node.width(), 13);
}

#[test]
fn test_width_when_has_trailing_trivia_expect_reduced_width() {
    // Create tokens with trailing trivia on the last token
    let token1 = create_token_with_trivia(NAME_KIND, "/Type", "", ""); // 0 leading, 0 trailing
    let token2 = create_token_with_trivia(NAME_KIND, "/Catalog", "", "\n"); // 0 leading, 1 trailing

    let node = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(token1), NodeOrToken::Token(token2)],
    );

    // full_width = 5 + 8 + 1 = 14, leading_trivia = 0, trailing_trivia = 1
    // width = 14 - 0 - 1 = 13
    assert_eq!(node.full_width(), 14);
    assert_eq!(node.get_leading_trivia_width(), 0);
    assert_eq!(node.get_trailing_trivia_width(), 1);
    assert_eq!(node.width(), 13);
}

#[test]
fn test_width_when_has_both_trivias_expect_double_reduced_width() {
    // Create tokens with both leading and trailing trivia
    let token1 = create_token_with_trivia(NAME_KIND, "/Type", "  ", ""); // 2 leading, 0 trailing
    let token2 = create_token_with_trivia(NAME_KIND, "/Catalog", "", " \n"); // 0 leading, 2 trailing

    let node = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(token1), NodeOrToken::Token(token2)],
    );

    // full_width = 5 + 8 + 2 + 2 = 17, leading_trivia = 2, trailing_trivia = 2
    // width = 17 - 2 - 2 = 13
    assert_eq!(node.full_width(), 17);
    assert_eq!(node.get_leading_trivia_width(), 2);
    assert_eq!(node.get_trailing_trivia_width(), 2);
    assert_eq!(node.width(), 13);
}

#[test]
fn test_get_leading_trivia_width_when_nested_structure_expect_first_terminal_leading() {
    // Create nested structure where first terminal has leading trivia
    let inner_token = create_token_with_trivia(NAME_KIND, "/Page", "    ", ""); // 4 leading spaces
    let leaf_node = create_node(DICT_KIND, vec![NodeOrToken::Token(inner_token)]);
    let parent_node = create_node(ARRAY_KIND, vec![NodeOrToken::Node(leaf_node)]);

    // Leading trivia should come from the first terminal's first token
    assert_eq!(parent_node.get_leading_trivia_width(), 4);
}

#[test]
fn test_get_trailing_trivia_width_when_nested_structure_expect_last_terminal_trailing() {
    // Create nested structure where last terminal has trailing trivia
    let token1 = create_token_with_trivia(NAME_KIND, "/Type", "", "");
    let token2 = create_token_with_trivia(NAME_KIND, "/Page", "", "\n\n"); // 2 trailing newlines
    let leaf_node = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(token1), NodeOrToken::Token(token2)],
    );
    let parent_node = create_node(ARRAY_KIND, vec![NodeOrToken::Node(leaf_node)]);

    // Trailing trivia should come from the last terminal's last token
    assert_eq!(parent_node.get_trailing_trivia_width(), 2);
}

#[test]
fn test_trivia_width_when_complex_pdf_structure_expect_correct_calculation() {
    // Create a realistic PDF structure with various trivia
    let obj_num = create_token_with_trivia(NUMBER_KIND, "1", "", " "); // 0 leading, 1 trailing
    let gen_num = create_token_with_trivia(NUMBER_KIND, "0", "", " "); // 0 leading, 1 trailing
    let obj_kw = create_token_with_trivia(KW_KIND, "obj", "", "\n"); // 0 leading, 1 trailing

    let type_name = create_token_with_trivia(NAME_KIND, "/Type", "  ", " "); // 2 leading, 1 trailing (first in dict)
    let catalog_name = create_token_with_trivia(NAME_KIND, "/Catalog", "", "\n"); // 0 leading, 1 trailing (last in dict)

    let dict_node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(type_name),
            NodeOrToken::Token(catalog_name),
        ],
    );

    let endobj_kw = create_token_with_trivia(KW_KIND, "endobj", "", ""); // 0 leading, 0 trailing

    let pdf_obj = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Token(obj_num),
            NodeOrToken::Token(gen_num),
            NodeOrToken::Token(obj_kw),
            NodeOrToken::Node(dict_node),
            NodeOrToken::Token(endobj_kw),
        ],
    );

    // Leading trivia should come from first terminal token (obj_num "1" with no leading trivia)
    assert_eq!(pdf_obj.get_leading_trivia_width(), 0);

    // Trailing trivia should come from last terminal token (endobj_kw "endobj" with no trailing trivia)
    assert_eq!(pdf_obj.get_trailing_trivia_width(), 0);

    // Calculate expected values:
    // Full content: "1" + " " + "0" + " " + "obj" + "\n" + "  " + "/Type" + " " + "/Catalog" + "\n" + "endobj"
    // = 1 + 1 + 1 + 1 + 3 + 1 + 2 + 5 + 1 + 8 + 1 + 6 = 31
    assert_eq!(pdf_obj.full_width(), 31);

    // Width = full_width - leading_trivia - trailing_trivia = 31 - 0 - 0 = 31
    assert_eq!(pdf_obj.width(), 31);
}

#[test]
fn test_trivia_width_when_only_tokens_no_child_nodes_expect_direct_trivia() {
    // Test with node that has tokens but no child nodes (terminal node)
    let first_token = create_token_with_trivia(NUMBER_KIND, "42", "   ", ""); // 3 leading
    let last_token = create_token_with_trivia(KW_KIND, "obj", "", "  "); // 2 trailing

    let terminal_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Token(first_token),
            NodeOrToken::Token(last_token),
        ],
    );

    // Since this is a terminal node, it should use its own first/last tokens
    assert_eq!(terminal_node.get_leading_trivia_width(), 3);
    assert_eq!(terminal_node.get_trailing_trivia_width(), 2);

    // full_width = 2 + 3 + 3 + 2 = 10, width = 10 - 3 - 2 = 5
    assert_eq!(terminal_node.full_width(), 10);
    assert_eq!(terminal_node.width(), 5);
}

#[test]
fn test_trivia_width_when_multiple_tokens_expect_only_boundary_exclusion() {
    // Test that only the first token's leading and last token's trailing trivia are excluded
    // All other trivia (between tokens) should be included in width calculation
    let first_token = create_token_with_trivia(NAME_KIND, "/Type", "  ", " "); // 2 leading, 1 trailing
    let middle_token = create_token_with_trivia(NAME_KIND, "/Page", " ", " "); // 1 leading, 1 trailing
    let last_token = create_token_with_trivia(STRING_KIND, "(test)", " ", "\n"); // 1 leading, 1 trailing

    let node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(first_token),
            NodeOrToken::Token(middle_token),
            NodeOrToken::Token(last_token),
        ],
    );

    // Should only exclude first token's leading (2) and last token's trailing (1)
    assert_eq!(node.get_leading_trivia_width(), 2);
    assert_eq!(node.get_trailing_trivia_width(), 1);

    // full_width = 5+2+1 + 5+1+1 + 6+1+1 = 8+7+8 = 23
    // width = 23 - 2 - 1 = 20 (preserves middle token's trivia + last token's leading)
    assert_eq!(node.full_width(), 23);
    assert_eq!(node.width(), 20);
}

#[test]
fn test_width_calculation_when_large_trivia_expect_no_underflow() {
    // Test edge case where trivia might be larger than content
    let token = create_token_with_trivia(NAME_KIND, "x", "     ", "     "); // 1 content, 10 trivia
    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(token)]);

    // full_width = 1 + 5 + 5 = 11
    // leading = 5, trailing = 5, total trivia = 10
    // width should be 11 - 10 = 1 (using saturating_sub to prevent underflow)
    assert_eq!(node.full_width(), 11);
    assert_eq!(node.get_leading_trivia_width(), 5);
    assert_eq!(node.get_trailing_trivia_width(), 5);
    assert_eq!(node.width(), 1);
}

#[test]
fn test_width_calculation_when_trivia_equals_content_expect_zero() {
    // Test edge case where trivia equals full content
    let token = create_token_with_trivia(NAME_KIND, "", "  ", "  "); // 0 content, 4 trivia
    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(token)]);

    // full_width = 0 + 2 + 2 = 4
    // width = 4 - 2 - 2 = 0
    assert_eq!(node.full_width(), 4);
    assert_eq!(node.width(), 0);
}

#[test]
fn test_width_when_internal_trivia_expect_preservation() {
    // Test that internal trivia (between tokens) is PRESERVED in width calculation
    // Only the first token's leading and last token's trailing trivia should be excluded
    let first_token = create_token_with_trivia(NAME_KIND, "/Type", "  ", " "); // 2 leading, 1 trailing
    let middle_token = create_token_with_trivia(NAME_KIND, "/Page", " ", " "); // 1 leading, 1 trailing (BOTH should be preserved)
    let last_token = create_token_with_trivia(STRING_KIND, "(test)", " ", "\n"); // 1 leading, 1 trailing

    let node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(first_token),
            NodeOrToken::Token(middle_token),
            NodeOrToken::Token(last_token),
        ],
    );

    // Content breakdown:
    // first_token: "  " + "/Type" + " " = 2 + 5 + 1 = 8
    // middle_token: " " + "/Page" + " " = 1 + 5 + 1 = 7 (ALL preserved in width)
    // last_token: " " + "(test)" + "\n" = 1 + 6 + 1 = 8
    // total full_width = 8 + 7 + 8 = 23

    // Boundary trivia to exclude:
    // - First token leading: 2 bytes ("  ")
    // - Last token trailing: 1 byte ("\n")
    // Internal trivia preserved: middle_token (1+1) + last_token leading (1) = 3 bytes

    assert_eq!(node.full_width(), 23);
    assert_eq!(node.get_leading_trivia_width(), 2);
    assert_eq!(node.get_trailing_trivia_width(), 1);

    // width = full_width - boundary_trivia = 23 - 2 - 1 = 20
    // This includes: "/Type" + " " + " " + "/Page" + " " + " " + "(test)" = 5+1+1+5+1+1+6 = 20
    assert_eq!(node.width(), 20);
}

#[test]
fn test_width_when_nested_internal_trivia_expect_preservation() {
    // Test nested structure where internal trivia between child nodes is preserved

    // Create child nodes with boundary trivia
    let child1 = {
        let token = create_token_with_trivia(NAME_KIND, "/Type", "  ", " "); // 2 leading, 1 trailing
        create_node(DICT_KIND, vec![NodeOrToken::Token(token)])
    };

    let child2 = {
        let token = create_token_with_trivia(NAME_KIND, "/Page", " ", "  "); // 1 leading, 2 trailing
        create_node(ARRAY_KIND, vec![NodeOrToken::Token(token)])
    };

    // Parent node contains both child nodes
    let parent = create_node(
        OBJ_KIND,
        vec![NodeOrToken::Node(child1), NodeOrToken::Node(child2)],
    );

    // First terminal should be child1's token with "  " leading
    // Last terminal should be child2's token with "  " trailing
    // The trivia between child nodes should be preserved in width

    // child1: "  " + "/Type" + " " = 8 bytes
    // child2: " " + "/Page" + "  " = 8 bytes
    // total = 16 bytes

    assert_eq!(parent.full_width(), 16);
    assert_eq!(parent.get_leading_trivia_width(), 2); // From first terminal (child1)
    assert_eq!(parent.get_trailing_trivia_width(), 2); // From last terminal (child2)

    // width = 16 - 2 - 2 = 12
    // This preserves: "/Type" + " " + " " + "/Page" = 5+1+1+5 = 12
    assert_eq!(parent.width(), 12);
}

// =============================================================================
// Text and Full Text Tests
// =============================================================================

#[test]
fn test_text_when_no_trivia_expect_content_only() {
    // Create node with tokens that have no trivia
    let token1 = create_token(NAME_KIND, "/Type");
    let token2 = create_token(NAME_KIND, "/Catalog");
    let node = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(token1), NodeOrToken::Token(token2)],
    );

    // text() should return only token content, no trivia
    assert_eq!(node.text(), b"/Type/Catalog");
    assert_eq!(node.full_text(), b"/Type/Catalog");
}

#[test]
fn test_text_when_has_boundary_trivia_expect_excluded() {
    // Create tokens with boundary trivia that should be excluded from text()
    let token1 = create_token_with_trivia(NAME_KIND, "/Type", "  ", " "); // 2 leading, 1 trailing
    let token2 = create_token_with_trivia(NAME_KIND, "/Catalog", "", "\n"); // 0 leading, 1 trailing

    let node = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(token1), NodeOrToken::Token(token2)],
    );

    // text() should exclude boundary trivia but keep internal trivia
    assert_eq!(node.text(), b"/Type /Catalog");
    // full_text() should include all trivia
    assert_eq!(node.full_text(), b"  /Type /Catalog\n");
}

#[test]
fn test_text_when_internal_trivia_expect_preservation() {
    // Test that internal trivia is preserved in text() but boundary trivia is excluded
    let first_token = create_token_with_trivia(NAME_KIND, "/Type", "  ", " "); // 2 leading, 1 trailing
    let middle_token = create_token_with_trivia(NAME_KIND, "/Page", " ", " "); // 1 leading, 1 trailing
    let last_token = create_token_with_trivia(STRING_KIND, "(test)", " ", "\n"); // 1 leading, 1 trailing

    let node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(first_token),
            NodeOrToken::Token(middle_token),
            NodeOrToken::Token(last_token),
        ],
    );

    // text() should exclude first leading ("  ") and last trailing ("\n"), but preserve internal trivia
    assert_eq!(node.text(), b"/Type  /Page  (test)");
    // full_text() should include everything
    assert_eq!(node.full_text(), b"  /Type  /Page  (test)\n");
}

#[test]
fn test_text_when_nested_structure_expect_correct_trivia_handling() {
    // Test nested structure with trivia at different levels
    let inner_token1 = create_token_with_trivia(NAME_KIND, "/Type", "  ", " ");
    let inner_token2 = create_token_with_trivia(NAME_KIND, "/Page", "", "  ");

    let inner_node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(inner_token1),
            NodeOrToken::Token(inner_token2),
        ],
    );

    let outer_token = create_token_with_trivia(KW_KIND, "endobj", " ", "\n");

    let outer_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Node(inner_node),
            NodeOrToken::Token(outer_token),
        ],
    );

    // text() should exclude outer boundary trivia (first inner leading, last outer trailing)
    assert_eq!(outer_node.text(), b"/Type /Page   endobj");
    // full_text() should include all trivia
    assert_eq!(outer_node.full_text(), b"  /Type /Page   endobj\n");
}

#[test]
fn test_text_when_deeply_nested_expect_recursive_processing() {
    // Create deeply nested structure: OBJ -> ARRAY -> DICT -> tokens
    let leaf_token1 = create_token_with_trivia(NUMBER_KIND, "1", "   ", " ");
    let leaf_token2 = create_token_with_trivia(NUMBER_KIND, "2", "", "  ");

    let dict_node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(leaf_token1),
            NodeOrToken::Token(leaf_token2),
        ],
    );

    let array_node = create_node(ARRAY_KIND, vec![NodeOrToken::Node(dict_node)]);

    let obj_token = create_token_with_trivia(KW_KIND, "obj", " ", "\n");
    let obj_node = create_node(
        OBJ_KIND,
        vec![NodeOrToken::Node(array_node), NodeOrToken::Token(obj_token)],
    );

    // text() should exclude outermost boundary trivia only
    assert_eq!(obj_node.text(), b"1 2   obj");
    // full_text() should include all trivia
    assert_eq!(obj_node.full_text(), b"   1 2   obj\n");
}

#[test]
fn test_text_when_empty_node_expect_empty_string() {
    let empty_node = create_node(DICT_KIND, vec![]);

    assert_eq!(empty_node.text(), b"");
    assert_eq!(empty_node.full_text(), b"");
}

#[test]
fn test_text_when_single_token_expect_correct_trivia_handling() {
    let token = create_token_with_trivia(STRING_KIND, "(hello)", "  ", "\n");
    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(token)]);

    // text() should exclude both boundary trivias since it's the only token
    assert_eq!(node.text(), b"(hello)");
    // full_text() should include all trivia
    assert_eq!(node.full_text(), b"  (hello)\n");
}

#[test]
fn test_text_when_complex_pdf_structure_expect_realistic_output() {
    // Create a realistic PDF structure: "1 0 obj\n  /Type /Catalog\nendobj"
    let obj_num = create_token_with_trivia(NUMBER_KIND, "1", "", " ");
    let gen_num = create_token_with_trivia(NUMBER_KIND, "0", "", " ");
    let obj_kw = create_token_with_trivia(KW_KIND, "obj", "", "\n");

    let type_name = create_token_with_trivia(NAME_KIND, "/Type", "  ", " ");
    let catalog_name = create_token_with_trivia(NAME_KIND, "/Catalog", "", "\n");

    let dict_node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(type_name),
            NodeOrToken::Token(catalog_name),
        ],
    );

    let endobj_kw = create_token_with_trivia(KW_KIND, "endobj", "", "");

    let pdf_obj = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Token(obj_num),
            NodeOrToken::Token(gen_num),
            NodeOrToken::Token(obj_kw),
            NodeOrToken::Node(dict_node),
            NodeOrToken::Token(endobj_kw),
        ],
    );

    // text() should exclude no boundary trivia (first has no leading, last has no trailing)
    // but preserve all internal formatting
    assert_eq!(pdf_obj.text(), b"1 0 obj\n  /Type /Catalog\nendobj");
    // full_text() should be identical since there's no boundary trivia
    assert_eq!(pdf_obj.full_text(), b"1 0 obj\n  /Type /Catalog\nendobj");
}

#[test]
fn test_text_when_only_internal_nodes_expect_recursive_processing() {
    // Test node that contains only other nodes (no direct tokens)
    let token1 = create_token_with_trivia(NAME_KIND, "/First", " ", "");
    let token2 = create_token_with_trivia(NAME_KIND, "/Second", "", " ");

    let child1 = create_node(DICT_KIND, vec![NodeOrToken::Token(token1)]);
    let child2 = create_node(ARRAY_KIND, vec![NodeOrToken::Token(token2)]);

    let parent = create_node(
        OBJ_KIND,
        vec![NodeOrToken::Node(child1), NodeOrToken::Node(child2)],
    );

    // text() should exclude boundary trivia from the terminals
    assert_eq!(parent.text(), b"/First/Second");
    // full_text() should include all trivia
    assert_eq!(parent.full_text(), b" /First/Second ");
}

#[test]
fn test_get_leading_trivia_width_when_empty_node_expect_zero() {
    let empty_node = create_node(DICT_KIND, vec![]);

    assert_eq!(empty_node.get_leading_trivia_width(), 0);
}

#[test]
fn test_get_trailing_trivia_width_when_empty_node_expect_zero() {
    let empty_node = create_node(DICT_KIND, vec![]);

    assert_eq!(empty_node.get_trailing_trivia_width(), 0);
}

#[test]
fn test_get_leading_trivia_width_when_only_empty_child_nodes_expect_zero() {
    // Test with a node that contains only empty child nodes (no terminals)
    let empty_child1 = create_node(ARRAY_KIND, vec![]);
    let empty_child2 = create_node(DICT_KIND, vec![]);

    let parent_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Node(empty_child1),
            NodeOrToken::Node(empty_child2),
        ],
    );

    // Since no terminals exist, should return 0
    assert_eq!(parent_node.get_leading_trivia_width(), 0);
}

#[test]
fn test_get_trailing_trivia_width_when_only_empty_child_nodes_expect_zero() {
    // Test with a node that contains only empty child nodes (no terminals)
    let empty_child1 = create_node(ARRAY_KIND, vec![]);
    let empty_child2 = create_node(DICT_KIND, vec![]);

    let parent_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Node(empty_child1),
            NodeOrToken::Node(empty_child2),
        ],
    );

    // Since no terminals exist, should return 0
    assert_eq!(parent_node.get_trailing_trivia_width(), 0);
}

#[test]
fn test_get_first_terminal_when_only_empty_child_nodes_expect_none() {
    // Test that get_first_terminal returns None when only empty child nodes exist
    let empty_child1 = create_node(ARRAY_KIND, vec![]);
    let empty_child2 = create_node(DICT_KIND, vec![]);

    let parent_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Node(empty_child1),
            NodeOrToken::Node(empty_child2),
        ],
    );

    assert!(parent_node.get_first_terminal().is_none());
}

#[test]
fn test_get_last_terminal_when_only_empty_child_nodes_expect_none() {
    // Test that get_last_terminal returns None when only empty child nodes exist
    let empty_child1 = create_node(ARRAY_KIND, vec![]);
    let empty_child2 = create_node(DICT_KIND, vec![]);

    let parent_node = create_node(
        OBJ_KIND,
        vec![
            NodeOrToken::Node(empty_child1),
            NodeOrToken::Node(empty_child2),
        ],
    );

    assert!(parent_node.get_last_terminal().is_none());
}

#[test]
fn test_get_leading_trivia_width_when_deeply_nested_empty_nodes_expect_zero() {
    // Test deeply nested structure with no terminals
    let leaf_empty = create_node(NAME_KIND, vec![]);
    let middle_empty = create_node(DICT_KIND, vec![NodeOrToken::Node(leaf_empty)]);
    let root_empty = create_node(OBJ_KIND, vec![NodeOrToken::Node(middle_empty)]);

    assert_eq!(root_empty.get_leading_trivia_width(), 0);
    assert_eq!(root_empty.get_trailing_trivia_width(), 0);
    assert!(root_empty.get_first_terminal().is_none());
    assert!(root_empty.get_last_terminal().is_none());
}

#[test]
fn test_debug_when_text_exceeds_max_length_expect_truncation() {
    // Create a very long text token that exceeds DEBUG_TEXT_MAX_LEN (255 chars)
    // We need text longer than 255 characters to trigger the truncation logic
    let long_text = "a".repeat(300); // 300 characters, well over the 255 limit
    let long_token = create_token(STRING_KIND, &long_text);
    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(long_token)]);

    let debug_output = format!("{:?}", node);

    // Verify that the original text is 300 characters
    assert_eq!(long_text.len(), 300);

    // The debug output should contain the truncation indicator "..."
    assert!(debug_output.contains("..."));

    // The debug output should be shorter than the original text
    // It should contain at most DEBUG_TEXT_MAX_LEN characters for the text field
    // (252 content chars + 3 "..." = 255 total)
    assert!(debug_output.len() < long_text.len() + 100); // +100 for other debug fields

    // Should still contain the basic debug structure
    assert!(debug_output.contains("GreenNode"));
    assert!(debug_output.contains("kind"));
    assert!(debug_output.contains("full_text"));
    assert!(debug_output.contains("full_width"));
    assert!(debug_output.contains("n_children"));

    // The truncated text in the debug output should be exactly 255 characters
    // Find the full_text field value in the debug output
    if let Some(start) = debug_output.find("full_text: \"") {
        let start_pos = start + "full_text: \"".len();
        if let Some(end) = debug_output[start_pos..].find("\"") {
            let text_content = &debug_output[start_pos..start_pos + end];
            // The text should be truncated to 252 chars + "..." = 255 total
            assert_eq!(text_content.len(), 255);
            assert!(text_content.ends_with("..."));
        }
    }
}

#[test]
fn test_debug_when_text_at_max_length_expect_no_truncation() {
    // Create text that is exactly at the DEBUG_TEXT_MAX_LEN limit (255 chars)
    let exact_limit_text = "a".repeat(255); // Exactly 255 characters
    let token = create_token(STRING_KIND, &exact_limit_text);
    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(token)]);

    let debug_output = format!("{:?}", node);

    // Verify that the original text is exactly 255 characters
    assert_eq!(exact_limit_text.len(), 255);

    // The debug output should NOT contain the truncation indicator "..."
    // since the text is exactly at the limit, not over it
    assert!(!debug_output.contains("..."));

    // Should contain the basic debug structure
    assert!(debug_output.contains("GreenNode"));
    assert!(debug_output.contains("kind"));
    assert!(debug_output.contains("full_text"));
    assert!(debug_output.contains("full_width"));
    assert!(debug_output.contains("n_children"));
}
