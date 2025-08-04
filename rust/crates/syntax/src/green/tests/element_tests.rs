use std::borrow::Cow;

use crate::{
    NodeOrToken, SyntaxKind,
    green::{
        element::GreenElement, node::GreenNode, node::GreenNodeData, token::GreenToken,
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
