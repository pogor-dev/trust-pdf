use std::borrow::{Borrow, Cow};

use crate::{
    NodeOrToken, SyntaxKind,
    green::{
        element::GreenElement, node::GreenNode, node_data::GreenNodeData, node_head::GreenNodeHead,
        token::GreenToken, trivia::GreenTrivia,
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

// Tests for GreenNode construction and basic properties

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
    let obj_node = create_obj_node();

    assert_eq!(obj_node.kind(), OBJ_KIND);
    assert_eq!(obj_node.children().count(), 3);

    // Verify child structure
    let children: Vec<_> = obj_node.children().collect();
    assert!(matches!(children[0], NodeOrToken::Token(_)));
    assert!(matches!(children[1], NodeOrToken::Token(_)));
    assert!(matches!(children[2], NodeOrToken::Node(_)));
}

#[test]
fn test_new_when_creating_array_node_expect_correct_children_order() {
    let num1 = create_token(NUMBER_KIND, "1");
    let num2 = create_token(NUMBER_KIND, "2");
    let num3 = create_token(NUMBER_KIND, "3");

    let array_node = create_node(
        ARRAY_KIND,
        vec![
            NodeOrToken::Token(num1.clone()),
            NodeOrToken::Token(num2.clone()),
            NodeOrToken::Token(num3.clone()),
        ],
    );

    let children: Vec<_> = array_node.children().collect();
    assert_eq!(children.len(), 3);

    // Verify order is preserved
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.text(), b"1");
    } else {
        panic!("Expected token");
    }

    if let NodeOrToken::Token(token) = &children[2] {
        assert_eq!(token.text(), b"3");
    } else {
        panic!("Expected token");
    }
}

// Tests for GreenNode memory management and sharing

#[test]
fn test_clone_when_cloning_node_expect_shared_properties() {
    let original = create_dict_node();
    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.kind(), cloned.kind());
    assert_eq!(original.full_width(), cloned.full_width());
    assert_eq!(original.children().count(), cloned.children().count());
}

#[test]
fn test_eq_when_comparing_identical_nodes_expect_equality() {
    let node1 = create_dict_node();
    let node2 = create_dict_node();

    assert_eq!(node1, node2);
}

#[test]
fn test_eq_when_comparing_different_kinds_expect_inequality() {
    let dict_node = create_dict_node();
    let array_node = create_node(ARRAY_KIND, vec![]);

    assert_ne!(dict_node, array_node);
}

#[test]
fn test_eq_when_comparing_different_children_expect_inequality() {
    let node1 = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(create_token(NAME_KIND, "/Type"))],
    );
    let node2 = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(create_token(NAME_KIND, "/Pages"))],
    );

    assert_ne!(node1, node2);
}

// Tests for GreenNode raw pointer operations (unsafe operations)

#[test]
fn test_into_raw_and_from_raw_when_roundtrip_expect_identical_node() {
    let original = create_dict_node();
    let original_kind = original.kind();
    let original_width = original.full_width();

    let raw_ptr = GreenNode::into_raw(original);
    let reconstructed = unsafe { GreenNode::from_raw(raw_ptr) };

    assert_eq!(reconstructed.kind(), original_kind);
    assert_eq!(reconstructed.full_width(), original_width);
}

#[test]
fn test_into_raw_when_converting_to_pointer_expect_valid_roundtrip() {
    let node = create_dict_node();
    let original_kind = node.kind();
    let original_width = node.full_width();

    let raw_ptr = GreenNode::into_raw(node);

    // Clean up by reconstructing and verifying
    let reconstructed = unsafe { GreenNode::from_raw(raw_ptr) };
    assert_eq!(reconstructed.kind(), original_kind);
    assert_eq!(reconstructed.full_width(), original_width);
}

// Tests for GreenNode formatting and display

#[test]
fn test_debug_when_formatting_simple_node_expect_readable_output() {
    let node = create_dict_node();
    let debug_str = format!("{:?}", node);

    // Should contain kind information and be non-empty
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("4")); // DICT_KIND value
}

#[test]
fn test_display_when_formatting_node_expect_text_content() {
    let token = create_token(STRING_KIND, "(Hello)");
    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(token)]);
    let display_str = format!("{}", node);

    // Should contain the text content
    assert!(display_str.contains("(Hello)"));
}

// Tests for GreenNode deref behavior

#[test]
fn test_deref_when_accessing_node_data_expect_correct_methods() {
    let node = create_dict_node();

    // These methods should be available through Deref to GreenNodeData
    assert_eq!(node.kind(), DICT_KIND);
    assert!(node.children().count() > 0);
}

// Tests for PDF-specific node structures

#[test]
fn test_new_when_creating_stream_object_expect_correct_structure() {
    let dict = create_dict_node();
    let stream_content = create_token(STRING_KIND, "BT /F1 12 Tf ET");

    let stream_node = create_node(
        STREAM_KIND,
        vec![NodeOrToken::Node(dict), NodeOrToken::Token(stream_content)],
    );

    assert_eq!(stream_node.kind(), STREAM_KIND);
    assert_eq!(stream_node.children().count(), 2);

    let children: Vec<_> = stream_node.children().collect();
    assert!(matches!(children[0], NodeOrToken::Node(_))); // Dictionary
    assert!(matches!(children[1], NodeOrToken::Token(_))); // Stream content
}

#[test]
fn test_new_when_creating_deep_nested_structure_expect_correct_traversal() {
    // Create a deeply nested PDF structure: obj -> dict -> array -> tokens
    let num1 = create_token(NUMBER_KIND, "1");
    let num2 = create_token(NUMBER_KIND, "2");
    let array = create_node(
        ARRAY_KIND,
        vec![NodeOrToken::Token(num1), NodeOrToken::Token(num2)],
    );

    let type_name = create_token(NAME_KIND, "/Type");
    let dict = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(type_name), NodeOrToken::Node(array)],
    );

    let obj_num = create_token(NUMBER_KIND, "1");
    let obj = create_node(
        OBJ_KIND,
        vec![NodeOrToken::Token(obj_num), NodeOrToken::Node(dict)],
    );

    assert_eq!(obj.kind(), OBJ_KIND);
    assert_eq!(obj.children().count(), 2);

    // Verify we can traverse the structure
    let children: Vec<_> = obj.children().collect();
    if let NodeOrToken::Node(dict_node) = &children[1] {
        assert_eq!(dict_node.kind(), DICT_KIND);
        let dict_children: Vec<_> = dict_node.children().collect();
        if let NodeOrToken::Node(array_node) = &dict_children[1] {
            assert_eq!(array_node.kind(), ARRAY_KIND);
            assert_eq!(array_node.children().count(), 2);
        } else {
            panic!("Expected array node");
        }
    } else {
        panic!("Expected dict node");
    }
}

// Tests for GreenNodeHead

#[test]
fn test_node_head_new_when_creating_with_values_expect_correct_fields() {
    let head = GreenNodeHead::new(DICT_KIND, 10, 15);

    assert_eq!(head.kind, DICT_KIND);
    assert_eq!(head.width, 10);
    assert_eq!(head.full_width, 15);
}

#[test]
fn test_node_head_clone_when_cloning_head_expect_identical_values() {
    let original = GreenNodeHead::new(OBJ_KIND, 42, 50);
    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(cloned.kind, OBJ_KIND);
    assert_eq!(cloned.width, 42);
    assert_eq!(cloned.full_width, 50);
}

#[test]
fn test_node_head_eq_when_comparing_identical_heads_expect_equality() {
    let head1 = GreenNodeHead::new(STRING_KIND, 5, 8);
    let head2 = GreenNodeHead::new(STRING_KIND, 5, 8);

    assert_eq!(head1, head2);
}

#[test]
fn test_node_head_eq_when_comparing_different_kinds_expect_inequality() {
    let head1 = GreenNodeHead::new(STRING_KIND, 5, 8);
    let head2 = GreenNodeHead::new(NUMBER_KIND, 5, 8);

    assert_ne!(head1, head2);
}

#[test]
fn test_node_head_eq_when_comparing_different_widths_expect_inequality() {
    let head1 = GreenNodeHead::new(STRING_KIND, 5, 8);
    let head2 = GreenNodeHead::new(STRING_KIND, 6, 8);

    assert_ne!(head1, head2);
}

#[test]
fn test_node_head_debug_when_formatting_expect_readable_output() {
    let head = GreenNodeHead::new(ARRAY_KIND, 20, 25);
    let debug_str = format!("{:?}", head);

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("20")); // width
    assert!(debug_str.contains("25")); // full_width
}

// Tests for GreenNodeData methods

#[test]
fn test_node_data_replace_child_when_replacing_first_child_expect_new_node() {
    let original_token = create_token(NAME_KIND, "/Type");
    let replacement_token = create_token(NAME_KIND, "/Pages");
    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(original_token)]);

    let new_node = node.replace_child(0, NodeOrToken::Token(replacement_token.clone()));

    assert_eq!(new_node.kind(), DICT_KIND);
    assert_eq!(new_node.children().count(), 1);

    let children: Vec<_> = new_node.children().collect();
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.text(), b"/Pages");
    } else {
        panic!("Expected token");
    }
}

#[test]
fn test_node_data_replace_child_when_replacing_middle_child_expect_others_unchanged() {
    // Create a node with multiple children to test the else branch (line 63)
    let token1 = create_token(NAME_KIND, "/Type");
    let token2 = create_token(NAME_KIND, "/Catalog");
    let token3 = create_token(NAME_KIND, "/Pages");
    let node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(token1),
            NodeOrToken::Token(token2),
            NodeOrToken::Token(token3),
        ],
    );

    // Replace the middle child (index 1)
    let replacement_token = create_token(NAME_KIND, "/Root");
    let new_node = node.replace_child(1, NodeOrToken::Token(replacement_token));

    assert_eq!(new_node.kind(), DICT_KIND);
    assert_eq!(new_node.children().count(), 3);

    let children: Vec<_> = new_node.children().collect();

    // First child should remain unchanged (covers child.to_owned() on line 63)
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.text(), b"/Type");
    } else {
        panic!("Expected token");
    }

    // Middle child should be replaced
    if let NodeOrToken::Token(token) = &children[1] {
        assert_eq!(token.text(), b"/Root");
    } else {
        panic!("Expected token");
    }

    // Last child should remain unchanged (covers child.to_owned() on line 63)
    if let NodeOrToken::Token(token) = &children[2] {
        assert_eq!(token.text(), b"/Pages");
    } else {
        panic!("Expected token");
    }
}

#[test]
fn test_node_data_insert_child_when_inserting_at_beginning_expect_new_first_child() {
    let original_token = create_token(NAME_KIND, "/Catalog");
    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(original_token)]);

    let new_token = create_token(NAME_KIND, "/Type");
    let new_node = node.insert_child(0, NodeOrToken::Token(new_token));

    assert_eq!(new_node.children().count(), 2);

    let children: Vec<_> = new_node.children().collect();
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.text(), b"/Type");
    } else {
        panic!("Expected token");
    }
    if let NodeOrToken::Token(token) = &children[1] {
        assert_eq!(token.text(), b"/Catalog");
    } else {
        panic!("Expected token");
    }
}

#[test]
fn test_node_data_remove_child_when_removing_only_child_expect_empty_node() {
    let token = create_token(NAME_KIND, "/Type");
    let node = create_node(DICT_KIND, vec![NodeOrToken::Token(token)]);

    let new_node = node.remove_child(0);

    assert_eq!(new_node.children().count(), 0);
}

#[test]
fn test_node_data_children_when_accessing_iterator_expect_correct_traversal() {
    let token1 = create_token(NAME_KIND, "/Type");
    let token2 = create_token(NAME_KIND, "/Catalog");
    let token3 = create_token(NAME_KIND, "/Pages");

    let node = create_node(
        DICT_KIND,
        vec![
            NodeOrToken::Token(token1),
            NodeOrToken::Token(token2),
            NodeOrToken::Token(token3),
        ],
    );

    let children: Vec<_> = node.children().collect();
    assert_eq!(children.len(), 3);

    // Verify iterator provides access to all children
    for (i, child) in node.children().enumerate() {
        match i {
            0 => {
                if let NodeOrToken::Token(token) = child {
                    assert_eq!(token.text(), b"/Type");
                }
            }
            1 => {
                if let NodeOrToken::Token(token) = child {
                    assert_eq!(token.text(), b"/Catalog");
                }
            }
            2 => {
                if let NodeOrToken::Token(token) = child {
                    assert_eq!(token.text(), b"/Pages");
                }
            }
            _ => panic!("Unexpected child index"),
        }
    }
}

#[test]
fn test_borrow_when_borrowing_node_expect_node_data_reference() {
    let node = create_dict_node();

    // Test the Borrow<GreenNodeData> trait implementation (lines 79-80)
    // This allows using &GreenNode where &GreenNodeData is expected
    let node_data: &GreenNodeData = node.borrow();

    // Verify we can access GreenNodeData methods through the borrowed reference
    assert_eq!(node_data.kind(), DICT_KIND);
    assert_eq!(node_data.full_width(), node.full_width());
    assert_eq!(node_data.children().count(), node.children().count());
}

#[test]
fn test_from_cow_when_converting_owned_cow_expect_green_node() {
    let original_node = create_dict_node();
    let original_kind = original_node.kind();
    let original_width = original_node.full_width();

    // Test the From<Cow<'_, GreenNodeData>> trait implementation (lines 86-87)
    // Create a Cow::Owned from the node data (this will clone the underlying data)
    let node_data: &GreenNodeData = &original_node;
    let cow_owned: Cow<'_, GreenNodeData> = Cow::Owned(node_data.to_owned());

    // This calls cow.into_owned() internally
    let reconstructed_node: GreenNode = cow_owned.into();

    assert_eq!(reconstructed_node.kind(), original_kind);
    assert_eq!(reconstructed_node.full_width(), original_width);
}

#[test]
fn test_from_cow_when_converting_borrowed_cow_expect_green_node() {
    let original_node = create_dict_node();
    let original_kind = original_node.kind();
    let original_width = original_node.full_width();

    // Test the From<Cow<'_, GreenNodeData>> trait implementation (lines 86-87)
    // Create a Cow::Borrowed from the node data
    let node_data: &GreenNodeData = &original_node;
    let cow_borrowed = Cow::Borrowed(node_data);

    // This calls cow.into_owned() internally, which will clone the data
    let reconstructed_node: GreenNode = cow_borrowed.into();

    assert_eq!(reconstructed_node.kind(), original_kind);
    assert_eq!(reconstructed_node.full_width(), original_width);
}

#[test]
fn test_node_data_eq_when_comparing_identical_data_expect_equality() {
    let node1 = create_dict_node();
    let node2 = create_dict_node();

    // Test the PartialEq implementation for GreenNodeData (lines 92-94)
    // Get references to the underlying GreenNodeData
    let data1: &GreenNodeData = &node1;
    let data2: &GreenNodeData = &node2;

    // This should call GreenNodeData::eq() which compares header and slice
    assert_eq!(data1, data2);
}

#[test]
fn test_node_data_eq_when_comparing_different_kinds_expect_inequality() {
    let dict_node = create_dict_node();
    let array_node = create_node(ARRAY_KIND, vec![]);

    // Test the PartialEq implementation for GreenNodeData (lines 92-94)
    let dict_data: &GreenNodeData = &dict_node;
    let array_data: &GreenNodeData = &array_node;

    // This should call GreenNodeData::eq() and return false due to different headers
    assert_ne!(dict_data, array_data);
}

#[test]
fn test_node_data_eq_when_comparing_different_children_expect_inequality() {
    let node1 = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(create_token(NAME_KIND, "/Type"))],
    );
    let node2 = create_node(
        DICT_KIND,
        vec![NodeOrToken::Token(create_token(NAME_KIND, "/Pages"))],
    );

    // Test the PartialEq implementation for GreenNodeData (lines 92-94)
    let data1: &GreenNodeData = &node1;
    let data2: &GreenNodeData = &node2;

    // This should call GreenNodeData::eq() and return false due to different slice content
    assert_ne!(data1, data2);
}
