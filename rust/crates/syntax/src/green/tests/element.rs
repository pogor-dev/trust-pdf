use std::borrow::Cow;

use crate::{
    NodeOrToken, SyntaxKind,
    green::{
        element::{GreenElement, GreenElementRef},
        node::{GreenNode, GreenNodeData},
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

/// Helper function to create test tokens with different content types
fn create_token(kind: SyntaxKind, text: &str) -> GreenToken {
    let empty_trivia = GreenTrivia::new([]);
    GreenToken::new(kind, text.as_bytes(), empty_trivia.clone(), empty_trivia)
}

/// Helper function to create test nodes with given children
fn create_node(kind: SyntaxKind, children: Vec<GreenElement>) -> GreenNode {
    GreenNode::new(kind, children)
}

// =============================================================================
// GreenElement Core Tests
// =============================================================================

#[test]
fn test_kind_when_node_element_expect_node_kind() {
    // Test that GreenElement correctly returns the kind for a node
    let children = vec![
        GreenElement::from(create_token(STRING_KIND, "(Hello)")),
        GreenElement::from(create_token(NUMBER_KIND, "123")),
    ];
    let node = create_node(DICT_KIND, children);
    let element = GreenElement::from(node);

    assert_eq!(element.kind(), DICT_KIND);
}

#[test]
fn test_kind_when_token_element_expect_token_kind() {
    // Test that GreenElement correctly returns the kind for a token
    let token = create_token(STRING_KIND, "(Hello, World!)");
    let element = GreenElement::from(token);

    assert_eq!(element.kind(), STRING_KIND);
}

#[test]
fn test_width_when_various_structures_expect_correct_calculations() {
    // Test width calculation for different element types in one test
    let test_cases = [
        // Token element
        (
            GreenElement::from(create_token(STRING_KIND, "(Hello)")),
            7usize,
        ),
        // Empty node
        (GreenElement::from(create_node(ARRAY_KIND, vec![])), 0usize),
        // Node with children
        (
            GreenElement::from(create_node(
                DICT_KIND,
                vec![
                    GreenElement::from(create_token(STRING_KIND, "(Hello)")), // 7 bytes
                    GreenElement::from(create_token(NUMBER_KIND, "123")),     // 3 bytes
                ],
            )),
            10usize,
        ),
    ];

    for (element, expected_width) in test_cases {
        assert_eq!(element.width(), expected_width);
        assert_eq!(element.full_width(), expected_width); // For these tests, full_width == width
    }
}

// =============================================================================
// Element Conversion Tests
// =============================================================================

#[test]
fn test_from_green_node_when_converting_expect_node_element() {
    // Test From<GreenNode> implementation
    let children = vec![GreenElement::from(create_token(STRING_KIND, "(Test)"))];
    let node = create_node(OBJ_KIND, children);
    let element = GreenElement::from(node);

    match element {
        NodeOrToken::Node(_) => {
            // Verify it's a node
            assert_eq!(element.kind(), OBJ_KIND);
        }
        NodeOrToken::Token(_) => panic!("Expected Node, got Token"),
    }
}

#[test]
fn test_from_green_token_when_converting_expect_token_element() {
    // Test From<GreenToken> implementation
    let token = create_token(NUMBER_KIND, "42");
    let element = GreenElement::from(token);

    match element {
        NodeOrToken::Token(_) => {
            // Verify it's a token
            assert_eq!(element.kind(), NUMBER_KIND);
            assert_eq!(element.width(), 2); // "42" is 2 bytes
        }
        NodeOrToken::Node(_) => panic!("Expected Token, got Node"),
    }
}

#[test]
fn test_from_cow_green_node_data_when_owned_expect_node_element() {
    // Test From<Cow<'_, GreenNodeData>> implementation with owned data
    let children = vec![GreenElement::from(create_token(STRING_KIND, "(Owned)"))];
    let node = create_node(DICT_KIND, children);
    let node_data: &GreenNodeData = &node;
    let cow_owned = Cow::Owned(node_data.to_owned());
    let element = GreenElement::from(cow_owned);

    match element {
        NodeOrToken::Node(_) => {
            assert_eq!(element.kind(), DICT_KIND);
        }
        NodeOrToken::Token(_) => panic!("Expected Node, got Token"),
    }
}

#[test]
fn test_from_cow_green_node_data_when_borrowed_expect_node_element() {
    // Test From<Cow<'_, GreenNodeData>> implementation with borrowed data
    let children = vec![GreenElement::from(create_token(STRING_KIND, "(Borrowed)"))];
    let node = create_node(ARRAY_KIND, children);
    let node_data: &GreenNodeData = &node;
    let cow_borrowed = Cow::Borrowed(node_data);
    let element = GreenElement::from(cow_borrowed);

    match element {
        NodeOrToken::Node(_) => {
            assert_eq!(element.kind(), ARRAY_KIND);
        }
        NodeOrToken::Token(_) => panic!("Expected Node, got Token"),
    }
}

// =============================================================================
// Width Calculation Tests
// =============================================================================

#[test]
fn test_width_when_nested_nodes_expect_correct_sum() {
    // Test width calculation with nested node structures
    let inner_children = vec![
        GreenElement::from(create_token(NAME_KIND, "/Length")), // 7 bytes
        GreenElement::from(create_token(NUMBER_KIND, "100")),   // 3 bytes
    ];
    let inner_node = create_node(DICT_KIND, inner_children);

    let outer_children = vec![
        GreenElement::from(inner_node),
        GreenElement::from(create_token(STRING_KIND, "(data)")), // 6 bytes
    ];
    let outer_node = create_node(OBJ_KIND, outer_children);
    let element = GreenElement::from(outer_node);

    assert_eq!(element.width(), 16); // (7 + 3) + 6 = 16 bytes
}

#[test]
fn test_width_when_deeply_nested_expect_correct_calculation() {
    // Test with deeply nested structures
    let deepest = create_token(NAME_KIND, "/Deep");
    let level2 = create_node(DICT_KIND, vec![GreenElement::from(deepest)]);
    let level1 = create_node(ARRAY_KIND, vec![GreenElement::from(level2)]);
    let root = create_node(OBJ_KIND, vec![GreenElement::from(level1)]);
    let element = GreenElement::from(root);

    assert_eq!(element.width(), 5); // "/Deep" is 5 bytes
}

#[test]
fn test_full_width_when_complex_structure_expect_accurate_calculation() {
    // Test full_width with a complex PDF-like structure
    let dict_children = vec![
        GreenElement::from(create_token(NAME_KIND, "/Type")), // 5 bytes
        GreenElement::from(create_token(NAME_KIND, "/Catalog")), // 8 bytes
        GreenElement::from(create_token(NAME_KIND, "/Pages")), // 6 bytes
        GreenElement::from(create_token(NUMBER_KIND, "1")),   // 1 byte
        GreenElement::from(create_token(NUMBER_KIND, "0")),   // 1 byte
        GreenElement::from(create_token(NAME_KIND, "R")),     // 1 byte
    ];
    let dict_node = create_node(DICT_KIND, dict_children);
    let element = GreenElement::from(dict_node);

    assert_eq!(element.full_width(), 22); // 5 + 8 + 6 + 1 + 1 + 1 = 22 bytes
}

// =============================================================================
// PDF-Specific Element Tests
// =============================================================================

#[test]
fn test_kind_when_pdf_specific_syntax_expect_correct_kinds() {
    // Test with PDF-specific syntax elements to ensure kind preservation
    let pdf_elements = vec![
        (OBJ_KIND, create_token(OBJ_KIND, "obj")),
        (STRING_KIND, create_token(STRING_KIND, "(PDF string)")),
        (NUMBER_KIND, create_token(NUMBER_KIND, "3.14")),
        (NAME_KIND, create_token(NAME_KIND, "/Type")),
        (STREAM_KIND, create_token(STREAM_KIND, "stream")),
    ];

    for (expected_kind, token) in pdf_elements {
        let element = GreenElement::from(token);
        assert_eq!(element.kind(), expected_kind);
    }
}

#[test]
fn test_pdf_object_structure_when_creating_expect_correct_hierarchy() {
    // Test a typical PDF object structure: "1 0 obj <</Type /Catalog>> endobj"
    let dict_children = vec![
        GreenElement::from(create_token(NAME_KIND, "/Type")),
        GreenElement::from(create_token(NAME_KIND, "/Catalog")),
    ];
    let dict_node = create_node(DICT_KIND, dict_children);

    let obj_children = vec![
        GreenElement::from(create_token(NUMBER_KIND, "1")),
        GreenElement::from(create_token(NUMBER_KIND, "0")),
        GreenElement::from(dict_node),
    ];
    let obj_node = create_node(OBJ_KIND, obj_children);
    let element = GreenElement::from(obj_node);

    assert_eq!(element.kind(), OBJ_KIND);
    // Total width: "1" (1) + "0" (1) + "/Type" (5) + "/Catalog" (8) = 15 bytes
    assert_eq!(element.width(), 15);
}

#[test]
fn test_pdf_array_structure_when_creating_expect_correct_handling() {
    // Test PDF array structure: "[1 2 3 /Name (String)]"
    let array_children = vec![
        GreenElement::from(create_token(NUMBER_KIND, "1")),
        GreenElement::from(create_token(NUMBER_KIND, "2")),
        GreenElement::from(create_token(NUMBER_KIND, "3")),
        GreenElement::from(create_token(NAME_KIND, "/Name")),
        GreenElement::from(create_token(STRING_KIND, "(String)")),
    ];
    let array_node = create_node(ARRAY_KIND, array_children);
    let element = GreenElement::from(array_node);

    assert_eq!(element.kind(), ARRAY_KIND);
    // Total width: "1" (1) + "2" (1) + "3" (1) + "/Name" (5) + "(String)" (8) = 16 bytes
    assert_eq!(element.width(), 16);
}

// =============================================================================
// Element Reference Tests
// =============================================================================

#[test]
fn test_as_deref_when_node_element_expect_node_ref() {
    let children = vec![GreenElement::from(create_token(STRING_KIND, "(Test)"))];
    let node = create_node(DICT_KIND, children);
    let element = GreenElement::from(node);

    let element_ref = element.as_deref();
    match element_ref {
        NodeOrToken::Node(node_data) => {
            assert_eq!(node_data.kind(), DICT_KIND);
        }
        NodeOrToken::Token(_) => panic!("Expected Node reference, got Token"),
    }
}

#[test]
fn test_as_deref_when_token_element_expect_token_ref() {
    let token = create_token(NAME_KIND, "/Test");
    let element = GreenElement::from(token);

    let element_ref = element.as_deref();
    match element_ref {
        NodeOrToken::Token(token_data) => {
            assert_eq!(token_data.kind(), NAME_KIND);
            assert_eq!(token_data.text(), b"/Test");
        }
        NodeOrToken::Node(_) => panic!("Expected Token reference, got Node"),
    }
}

// =============================================================================
// Equality and Hash Tests
// =============================================================================

#[test]
fn test_equality_when_identical_elements_expect_equal() {
    let element1 = GreenElement::from(create_token(STRING_KIND, "(Hello)"));
    let element2 = GreenElement::from(create_token(STRING_KIND, "(Hello)"));

    assert_eq!(element1, element2);
}

#[test]
fn test_equality_when_different_elements_expect_not_equal() {
    let element1 = GreenElement::from(create_token(STRING_KIND, "(Hello)"));
    let element2 = GreenElement::from(create_token(STRING_KIND, "(World)"));

    assert_ne!(element1, element2);
}

#[test]
fn test_equality_when_node_vs_token_expect_not_equal() {
    let token_element = GreenElement::from(create_token(STRING_KIND, "(Test)"));
    let node_element = GreenElement::from(create_node(DICT_KIND, vec![]));

    assert_ne!(token_element, node_element);
}

#[test]
fn test_hash_when_using_in_collections_expect_consistent_behavior() {
    use std::collections::HashMap;

    let element = GreenElement::from(create_token(NAME_KIND, "/Type"));
    let mut map = HashMap::new();
    map.insert(element.clone(), "test_value");

    // Should be able to retrieve using equivalent element
    let lookup_element = GreenElement::from(create_token(NAME_KIND, "/Type"));
    assert_eq!(map.get(&lookup_element), Some(&"test_value"));
}

// =============================================================================
// Edge Cases and Error Conditions
// =============================================================================

#[test]
fn test_empty_structures_when_creating_expect_valid_handling() {
    // Test with empty nodes
    let empty_dict = create_node(DICT_KIND, vec![]);
    let empty_array = create_node(ARRAY_KIND, vec![]);

    let dict_element = GreenElement::from(empty_dict);
    let array_element = GreenElement::from(empty_array);

    assert_eq!(dict_element.width(), 0);
    assert_eq!(array_element.width(), 0);
    assert_eq!(dict_element.full_width(), 0);
    assert_eq!(array_element.full_width(), 0);
}

#[test]
fn test_single_child_nodes_when_creating_expect_correct_behavior() {
    // Test nodes with single children
    let single_child_dict = create_node(
        DICT_KIND,
        vec![GreenElement::from(create_token(NAME_KIND, "/Single"))],
    );
    let element = GreenElement::from(single_child_dict);

    assert_eq!(element.kind(), DICT_KIND);
    assert_eq!(element.width(), 7); // "/Single" is 7 bytes
}

#[test]
fn test_maximum_nesting_when_creating_expect_no_stack_overflow() {
    // Test with reasonable nesting depth
    let mut current_element = GreenElement::from(create_token(NAME_KIND, "/Deep"));

    for i in 0..100 {
        let node = create_node(
            SyntaxKind(i + 100), // Use different kinds to avoid confusion
            vec![current_element],
        );
        current_element = GreenElement::from(node);
    }

    // Should still calculate width correctly
    assert_eq!(current_element.width(), 5); // "/Deep" is 5 bytes
}

// =============================================================================
// Debug and Display Tests
// =============================================================================

#[test]
fn test_debug_when_formatting_element_expect_readable_output() {
    let element = GreenElement::from(create_token(STRING_KIND, "(test)"));
    let debug_output = format!("{:?}", element);

    // Debug output should be non-empty and contain useful information
    assert!(!debug_output.is_empty());
}

#[test]
fn test_clone_when_copying_element_expect_shared_memory() {
    let original = GreenElement::from(create_token(NAME_KIND, "/SharedElement"));
    let cloned = original.clone();

    // Both should have same content
    assert_eq!(original.kind(), cloned.kind());
    assert_eq!(original.width(), cloned.width());
    assert_eq!(original.full_width(), cloned.full_width());

    // Memory should be shared (reference counted)
    assert_eq!(original, cloned);
}

// =============================================================================
// GreenElementRef From Tests
// =============================================================================

#[test]
fn test_from_green_node_ref_when_converting_expect_element_ref() {
    let node = create_node(
        DICT_KIND,
        vec![GreenElement::from(create_token(NAME_KIND, "/Type"))],
    );
    let element_ref: GreenElementRef = (&node).into();

    match element_ref {
        NodeOrToken::Node(node_data) => {
            assert_eq!(node_data.kind(), DICT_KIND);
        }
        NodeOrToken::Token(_) => panic!("Expected Node reference, got Token"),
    }
}

#[test]
fn test_from_green_token_ref_when_converting_expect_element_ref() {
    let token = create_token(STRING_KIND, "(test)");
    let element_ref: GreenElementRef = (&token).into();

    match element_ref {
        NodeOrToken::Token(token_data) => {
            assert_eq!(token_data.kind(), STRING_KIND);
            assert_eq!(token_data.text(), b"(test)");
        }
        NodeOrToken::Node(_) => panic!("Expected Token reference, got Node"),
    }
}

#[test]
fn test_element_ref_to_owned_when_node_ref_expect_owned_element() {
    let node = create_node(
        DICT_KIND,
        vec![GreenElement::from(create_token(NAME_KIND, "/Type"))],
    );
    let element_ref: GreenElementRef = (&node).into();
    let owned_element = element_ref.to_owned();

    assert_eq!(owned_element.kind(), DICT_KIND);
    match owned_element {
        NodeOrToken::Node(_) => {
            // Verify it's an owned node element
        }
        NodeOrToken::Token(_) => panic!("Expected Node element, got Token"),
    }
}
