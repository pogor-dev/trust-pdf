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
fn test_width_when_token_element_expect_token_width() {
    // Test width calculation for token elements
    let token = create_token(STRING_KIND, "(Hello)");
    let element = GreenElement::from(token);

    // Width should match the byte length of the token text
    assert_eq!(element.width(), 7); // "(Hello)" is 7 bytes
}

#[test]
fn test_width_when_node_element_expect_sum_of_children_width() {
    // Test width calculation for node elements (sum of children)
    let children = vec![
        GreenElement::from(create_token(STRING_KIND, "(Hello)")), // 7 bytes
        GreenElement::from(create_token(NUMBER_KIND, "123")),     // 3 bytes
    ];
    let node = create_node(DICT_KIND, children);
    let element = GreenElement::from(node);

    assert_eq!(element.width(), 10); // 7 + 3 = 10 bytes
}

#[test]
fn test_width_when_empty_node_expect_zero() {
    // Test width calculation for empty nodes
    let node = create_node(ARRAY_KIND, vec![]);
    let element = GreenElement::from(node);

    assert_eq!(element.width(), 0);
}

#[test]
fn test_full_width_when_token_element_expect_token_full_width() {
    // Test full_width calculation for token elements (includes trivia)
    let token = create_token(STRING_KIND, "(PDF String)");
    let element = GreenElement::from(token);

    // For tokens without trivia, full_width should equal width
    assert_eq!(element.full_width(), element.width());
}

#[test]
fn test_full_width_when_node_element_expect_sum_of_children_full_width() {
    // Test full_width calculation for node elements
    let children = vec![
        GreenElement::from(create_token(NAME_KIND, "/Type")), // 5 bytes
        GreenElement::from(create_token(NAME_KIND, "/Catalog")), // 8 bytes
    ];
    let node = create_node(DICT_KIND, children);
    let element = GreenElement::from(node);

    assert_eq!(element.full_width(), 13); // 5 + 8 = 13 bytes
}

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
fn test_kind_when_pdf_specific_syntax_expect_correct_kinds() {
    // Test with PDF-specific syntax elements to ensure kind preservation
    let pdf_elements = vec![
        (OBJ_KIND, create_token(OBJ_KIND, "obj")),
        (STRING_KIND, create_token(STRING_KIND, "(PDF string)")),
        (NUMBER_KIND, create_token(NUMBER_KIND, "3.14159")),
        (NAME_KIND, create_token(NAME_KIND, "/Type")),
    ];

    for (expected_kind, token) in pdf_elements {
        let element = GreenElement::from(token);
        assert_eq!(element.kind(), expected_kind);
    }
}

#[test]
fn test_width_when_unicode_content_expect_byte_count() {
    // Test width calculation with Unicode content (PDF can contain various encodings)
    let unicode_token = create_token(STRING_KIND, "(café)"); // é is 2 bytes in UTF-8
    let element = GreenElement::from(unicode_token);

    // "(café)" should be 7 bytes: ( + c + a + f + é(2 bytes) + )
    assert_eq!(element.width(), 7);
}

#[test]
fn test_kind_when_node_element_ref_expect_node_kind() {
    // Test that GreenElementRef correctly returns the kind for a node reference
    let children = vec![
        GreenElement::from(create_token(STRING_KIND, "(Hello)")),
        GreenElement::from(create_token(NUMBER_KIND, "123")),
    ];
    let node = create_node(DICT_KIND, children);
    let element_ref = GreenElementRef::from(&node);

    assert_eq!(element_ref.kind(), DICT_KIND);
}

#[test]
fn test_kind_when_token_element_ref_expect_token_kind() {
    // Test that GreenElementRef correctly returns the kind for a token reference
    let token = create_token(STRING_KIND, "(Hello, World!)");
    let element_ref = GreenElementRef::from(&token);

    assert_eq!(element_ref.kind(), STRING_KIND);
}

#[test]
fn test_width_when_token_element_ref_expect_token_width() {
    // Test width calculation for token element references
    let token = create_token(STRING_KIND, "(Hello)");
    let element_ref = GreenElementRef::from(&token);

    // Width should match the byte length of the token text
    assert_eq!(element_ref.width(), 7); // "(Hello)" is 7 bytes
}

#[test]
fn test_width_when_node_element_ref_expect_sum_of_children_width() {
    // Test width calculation for node element references (sum of children)
    let children = vec![
        GreenElement::from(create_token(STRING_KIND, "(Hello)")), // 7 bytes
        GreenElement::from(create_token(NUMBER_KIND, "123")),     // 3 bytes
    ];
    let node = create_node(DICT_KIND, children);
    let element_ref = GreenElementRef::from(&node);

    assert_eq!(element_ref.width(), 10); // 7 + 3 = 10 bytes
}

#[test]
fn test_width_when_empty_node_ref_expect_zero() {
    // Test width calculation for empty node references
    let node = create_node(ARRAY_KIND, vec![]);
    let element_ref = GreenElementRef::from(&node);

    assert_eq!(element_ref.width(), 0);
}

#[test]
fn test_full_width_when_token_element_ref_expect_token_full_width() {
    // Test full_width calculation for token element references (includes trivia)
    let token = create_token(STRING_KIND, "(PDF String)");
    let element_ref = GreenElementRef::from(&token);

    // For tokens without trivia, full_width should equal width
    assert_eq!(element_ref.full_width(), element_ref.width());
}

#[test]
fn test_full_width_when_node_element_ref_expect_sum_of_children_full_width() {
    // Test full_width calculation for node element references
    let children = vec![
        GreenElement::from(create_token(NAME_KIND, "/Type")), // 5 bytes
        GreenElement::from(create_token(NAME_KIND, "/Catalog")), // 8 bytes
    ];
    let node = create_node(DICT_KIND, children);
    let element_ref = GreenElementRef::from(&node);

    assert_eq!(element_ref.full_width(), 13); // 5 + 8 = 13 bytes
}

#[test]
fn test_from_green_node_ref_when_converting_expect_node_element_ref() {
    // Test From<&GreenNode> implementation
    let children = vec![GreenElement::from(create_token(STRING_KIND, "(Test)"))];
    let node = create_node(OBJ_KIND, children);
    let element_ref = GreenElementRef::from(&node);

    match element_ref {
        NodeOrToken::Node(_) => {
            // Verify it's a node reference
            assert_eq!(element_ref.kind(), OBJ_KIND);
        }
        NodeOrToken::Token(_) => panic!("Expected Node, got Token"),
    }
}

#[test]
fn test_from_green_token_ref_when_converting_expect_token_element_ref() {
    // Test From<&GreenToken> implementation
    let token = create_token(NUMBER_KIND, "42");
    let element_ref = GreenElementRef::from(&token);

    match element_ref {
        NodeOrToken::Token(_) => {
            // Verify it's a token reference
            assert_eq!(element_ref.kind(), NUMBER_KIND);
            assert_eq!(element_ref.width(), 2); // "42" is 2 bytes
        }
        NodeOrToken::Node(_) => panic!("Expected Token, got Node"),
    }
}

#[test]
fn test_to_owned_when_node_element_ref_expect_owned_element() {
    // Test to_owned() method with node element reference
    let children = vec![GreenElement::from(create_token(STRING_KIND, "(Owned)"))];
    let node = create_node(DICT_KIND, children);
    let element_ref = GreenElementRef::from(&node);
    let owned_element = element_ref.to_owned();

    match owned_element {
        NodeOrToken::Node(_) => {
            assert_eq!(owned_element.kind(), DICT_KIND);
        }
        NodeOrToken::Token(_) => panic!("Expected Node, got Token"),
    }
}

#[test]
fn test_to_owned_when_token_element_ref_expect_owned_element() {
    // Test to_owned() method with token element reference
    let token = create_token(NUMBER_KIND, "3.14159");
    let element_ref = GreenElementRef::from(&token);
    let owned_element = element_ref.to_owned();

    match owned_element {
        NodeOrToken::Token(_) => {
            assert_eq!(owned_element.kind(), NUMBER_KIND);
            assert_eq!(owned_element.width(), 7); // "3.14159" is 7 bytes
        }
        NodeOrToken::Node(_) => panic!("Expected Token, got Node"),
    }
}

#[test]
fn test_width_when_nested_node_refs_expect_correct_sum() {
    // Test width calculation with nested node reference structures
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
    let element_ref = GreenElementRef::from(&outer_node);

    assert_eq!(element_ref.width(), 16); // (7 + 3) + 6 = 16 bytes
}

#[test]
fn test_kind_when_pdf_specific_syntax_refs_expect_correct_kinds() {
    // Test with PDF-specific syntax element references to ensure kind preservation
    let pdf_tokens = vec![
        (OBJ_KIND, create_token(OBJ_KIND, "obj")),
        (STRING_KIND, create_token(STRING_KIND, "(PDF string)")),
        (NUMBER_KIND, create_token(NUMBER_KIND, "3.14159")),
        (NAME_KIND, create_token(NAME_KIND, "/Type")),
    ];

    for (expected_kind, token) in pdf_tokens {
        let element_ref = GreenElementRef::from(&token);
        assert_eq!(element_ref.kind(), expected_kind);
    }
}

#[test]
fn test_width_when_unicode_content_ref_expect_byte_count() {
    // Test width calculation with Unicode content references (PDF can contain various encodings)
    let unicode_token = create_token(STRING_KIND, "(café)"); // é is 2 bytes in UTF-8
    let element_ref = GreenElementRef::from(&unicode_token);

    // "(café)" should be 7 bytes: ( + c + a + f + é(2 bytes) + )
    assert_eq!(element_ref.width(), 7);
}

#[test]
fn test_reference_semantics_when_multiple_refs_expect_same_data() {
    // Test that multiple references to the same data behave consistently
    let token = create_token(STREAM_KIND, "stream");
    let element_ref1 = GreenElementRef::from(&token);
    let element_ref2 = GreenElementRef::from(&token);

    // Both references should report the same properties
    assert_eq!(element_ref1.kind(), element_ref2.kind());
    assert_eq!(element_ref1.width(), element_ref2.width());
    assert_eq!(element_ref1.full_width(), element_ref2.full_width());
}

#[test]
fn test_lifetime_independence_when_converting_to_owned_expect_independent_copy() {
    // Test that to_owned() creates an independent copy that outlives the reference
    let owned_element = {
        let token = create_token(NUMBER_KIND, "999");
        let element_ref = GreenElementRef::from(&token);
        element_ref.to_owned() // This should work even after token goes out of scope
    };

    // The owned element should still be valid and functional
    assert_eq!(owned_element.kind(), NUMBER_KIND);
    assert_eq!(owned_element.width(), 3);
}

#[test]
fn test_pattern_matching_when_discriminating_variants_expect_correct_behavior() {
    // Test pattern matching behavior with element references
    let token = create_token(STRING_KIND, "(pattern)");
    let node = create_node(
        DICT_KIND,
        vec![GreenElement::from(create_token(NAME_KIND, "/Key"))],
    );

    let token_ref = GreenElementRef::from(&token);
    let node_ref = GreenElementRef::from(&node);

    // Test discriminant behavior
    let token_is_token = matches!(token_ref, NodeOrToken::Token(_));
    let node_is_node = matches!(node_ref, NodeOrToken::Node(_));
    let token_is_node = matches!(token_ref, NodeOrToken::Node(_));
    let node_is_token = matches!(node_ref, NodeOrToken::Token(_));

    assert!(token_is_token);
    assert!(node_is_node);
    assert!(!token_is_node);
    assert!(!node_is_token);
}

#[test]
fn test_method_chaining_when_using_references_expect_fluent_interface() {
    // Test that methods can be chained effectively with references
    let children = vec![
        GreenElement::from(create_token(NAME_KIND, "/Type")),
        GreenElement::from(create_token(NAME_KIND, "/Font")),
    ];
    let node = create_node(DICT_KIND, children);
    let element_ref = GreenElementRef::from(&node);

    // Test that we can chain operations and get consistent results
    let kind = element_ref.kind();
    let width = element_ref.width();
    let full_width = element_ref.full_width();
    let owned = element_ref.to_owned();

    assert_eq!(kind, DICT_KIND);
    assert_eq!(width, 10); // "/Type" (5) + "/Font" (5) = 10
    assert_eq!(full_width, width); // No trivia in test tokens
    assert_eq!(owned.kind(), kind);
    assert_eq!(owned.width(), width);
}
