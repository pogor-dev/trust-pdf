//! Common test fixtures for cursor tests
//!
//! This module provides reusable constants, helper functions, and tree creation
//! utilities that are commonly used across multiple cursor test files.

use crate::{
    SyntaxKind,
    cursor::node::SyntaxNode,
    green::{element::GreenElement, node::GreenNode, token::GreenToken, trivia::GreenTrivia},
};

// =============================================================================
// Common SyntaxKind Constants
// =============================================================================

/// Test constants for different PDF syntax kinds used across all cursor tests
pub const STRING_KIND: SyntaxKind = SyntaxKind(1);
pub const NUMBER_KIND: SyntaxKind = SyntaxKind(2);
pub const NAME_KIND: SyntaxKind = SyntaxKind(3);
pub const DICT_KIND: SyntaxKind = SyntaxKind(4);
pub const ARRAY_KIND: SyntaxKind = SyntaxKind(5);
pub const OBJ_KIND: SyntaxKind = SyntaxKind(6);
pub const COMMENT_KIND: SyntaxKind = SyntaxKind(7);
pub const OBJ_KW: SyntaxKind = SyntaxKind(8);
pub const ENDOBJ_KW: SyntaxKind = SyntaxKind(9);
pub const STREAM_KIND: SyntaxKind = SyntaxKind(10);
pub const WHITESPACE_KIND: SyntaxKind = SyntaxKind(11);

// =============================================================================
// Basic Helper Functions
// =============================================================================

/// Creates a simple GreenToken for testing purposes
pub fn create_green_token(kind: SyntaxKind, text: &str) -> GreenToken {
    let empty_trivia = GreenTrivia::new([]);
    GreenToken::new(kind, text.as_bytes(), empty_trivia.clone(), empty_trivia)
}

/// Creates a GreenToken with trivia for more complex testing scenarios
pub fn create_green_token_with_trivia(
    kind: SyntaxKind,
    text: &[u8],
    leading: &[u8],
    trailing: &[u8],
) -> GreenToken {
    let leading_trivia = if leading.is_empty() {
        GreenTrivia::new([])
    } else {
        GreenTrivia::new([crate::green::trivia::GreenTriviaChild::new(
            WHITESPACE_KIND,
            leading,
        )])
    };
    let trailing_trivia = if trailing.is_empty() {
        GreenTrivia::new([])
    } else {
        GreenTrivia::new([crate::green::trivia::GreenTriviaChild::new(
            WHITESPACE_KIND,
            trailing,
        )])
    };
    GreenToken::new(kind, text, leading_trivia, trailing_trivia)
}

/// Creates a GreenNode with the given kind and children
pub fn create_green_node(kind: SyntaxKind, children: Vec<GreenElement>) -> GreenNode {
    GreenNode::new(kind, children)
}

// =============================================================================
// Common Tree Creation Functions
// =============================================================================

/// Creates a basic tree with a single child: OBJ -> [DICT -> [STRING]]
/// This is the most basic tree structure used across many tests.
pub fn create_simple_tree() -> SyntaxNode {
    let string_token = create_green_token(STRING_KIND, "(Hello)");
    let dict_node = create_green_node(DICT_KIND, vec![string_token.into()]);
    let obj_node = create_green_node(OBJ_KIND, vec![dict_node.into()]);
    SyntaxNode::new_root(obj_node)
}

/// Creates a single node tree with no children: DICT
/// Used by preorder tests that expect only Enter/Leave events for the root.
pub fn create_single_node_tree() -> SyntaxNode {
    let dict_node = create_green_node(DICT_KIND, vec![]);
    SyntaxNode::new_root(dict_node)
}

/// Creates a simple two-level tree: DICT -> [STRING]
/// Used by token tests that expect tokens to have only one ancestor.
pub fn create_simple_syntax_tree() -> SyntaxNode {
    let string_token = create_green_token(STRING_KIND, "(Hello)");
    let dict_node = create_green_node(DICT_KIND, vec![string_token.into()]);
    SyntaxNode::new_root(dict_node)
}

/// Creates a tree with multiple token children: ARRAY -> [NAME, NAME, NUMBER]
/// Useful for testing sibling navigation and flat traversal.
pub fn create_flat_tree() -> SyntaxNode {
    let token1 = create_green_token(NAME_KIND, "/Type");
    let token2 = create_green_token(NAME_KIND, "/Catalog");
    let token3 = create_green_token(NUMBER_KIND, "42");

    let node = create_green_node(
        ARRAY_KIND,
        vec![token1.into(), token2.into(), token3.into()],
    );
    SyntaxNode::new_root(node)
}

/// Creates a nested tree structure: OBJ -> DICT -> [NAME, NAME]
/// Good for testing depth-first traversal and parent-child relationships.
pub fn create_nested_tree() -> SyntaxNode {
    // Inner dictionary: /Type /Page
    let inner_token1 = create_green_token(NAME_KIND, "/Type");
    let inner_token2 = create_green_token(NAME_KIND, "/Page");
    let inner_dict = create_green_node(DICT_KIND, vec![inner_token1.into(), inner_token2.into()]);

    // Outer object containing the dictionary and a number
    let number_token = create_green_token(NUMBER_KIND, "123");
    let outer_obj = create_green_node(OBJ_KIND, vec![inner_dict.into(), number_token.into()]);

    SyntaxNode::new_root(outer_obj)
}

/// Creates a deeply nested tree structure: OBJ -> STREAM -> DICT -> ARRAY -> NUMBER
/// Perfect for testing recursive operations and deep traversal.
pub fn create_deeply_nested_tree() -> SyntaxNode {
    // Level 3: innermost array [42]
    let number = create_green_token(NUMBER_KIND, "42");
    let inner_array = create_green_node(ARRAY_KIND, vec![number.into()]);

    // Level 2: dictionary containing the array { /Contents [42] }
    let contents_name = create_green_token(NAME_KIND, "/Contents");
    let dict = create_green_node(DICT_KIND, vec![contents_name.into(), inner_array.into()]);

    // Level 1: stream containing the dictionary
    let stream_data = create_green_token(STRING_KIND, "stream_data");
    let stream = create_green_node(STREAM_KIND, vec![dict.into(), stream_data.into()]);

    // Level 0: root object containing the stream
    let obj_num = create_green_token(NUMBER_KIND, "1");
    let root = create_green_node(OBJ_KIND, vec![obj_num.into(), stream.into()]);

    SyntaxNode::new_root(root)
}

/// Creates an empty tree for testing edge cases
pub fn create_empty_tree() -> SyntaxNode {
    let empty_node = create_green_node(OBJ_KIND, vec![]);
    SyntaxNode::new_root(empty_node)
}

/// Creates a tree with siblings for testing sibling navigation
/// Structure: OBJ -> [DICT, ARRAY, DICT]
pub fn create_sibling_tree() -> SyntaxNode {
    // Create multiple child nodes at the same level
    let dict1 = create_green_node(
        DICT_KIND,
        vec![create_green_token(NAME_KIND, "/Key1").into()],
    );
    let array1 = create_green_node(
        ARRAY_KIND,
        vec![create_green_token(NUMBER_KIND, "123").into()],
    );
    let dict2 = create_green_node(
        DICT_KIND,
        vec![create_green_token(NAME_KIND, "/Key2").into()],
    );

    let obj_node = create_green_node(OBJ_KIND, vec![dict1.into(), array1.into(), dict2.into()]);
    SyntaxNode::new_root(obj_node)
}

/// Creates a tree with multiple token children for testing token iteration
/// Structure: OBJ -> DICT -> [NAME, NUMBER, STRING]
pub fn create_multi_token_tree() -> SyntaxNode {
    let name_token = create_green_token(NAME_KIND, "/Type");
    let number_token = create_green_token(NUMBER_KIND, "42");
    let string_token = create_green_token(STRING_KIND, "(text)");

    let dict_children = vec![name_token.into(), number_token.into(), string_token.into()];
    let dict_node = create_green_node(DICT_KIND, dict_children);
    let obj_node = create_green_node(OBJ_KIND, vec![dict_node.into()]);
    SyntaxNode::new_root(obj_node)
}

/// Creates a tree with trivia for testing trivia handling
pub fn create_trivia_tree() -> SyntaxNode {
    let token_with_trivia = create_green_token_with_trivia(NAME_KIND, b"/Type", b"  ", b" ");
    let green_node = create_green_node(DICT_KIND, vec![token_with_trivia.into()]);
    SyntaxNode::new_root(green_node)
}

/// Creates a complex PDF-like structure for comprehensive testing
/// Structure: 1 0 obj << /Type /Catalog /Pages 2 0 R /Names << /JavaScript 3 0 R >> >> endobj
pub fn create_complex_pdf_structure() -> SyntaxNode {
    let obj_num = create_green_token(NUMBER_KIND, "1");
    let gen_num = create_green_token(NUMBER_KIND, "0");
    let obj_kw = create_green_token(OBJ_KW, "obj");

    // Names dictionary: /JavaScript 3 0 R
    let js_name = create_green_token(NAME_KIND, "/JavaScript");
    let js_ref_num = create_green_token(NUMBER_KIND, "3");
    let js_ref_gen = create_green_token(NUMBER_KIND, "0");
    let js_ref_r = create_green_token(STRING_KIND, "R");
    let names_dict = create_green_node(
        DICT_KIND,
        vec![
            js_name.into(),
            js_ref_num.into(),
            js_ref_gen.into(),
            js_ref_r.into(),
        ],
    );

    // Main dictionary contents
    let type_name = create_green_token(NAME_KIND, "/Type");
    let catalog_name = create_green_token(NAME_KIND, "/Catalog");
    let pages_name = create_green_token(NAME_KIND, "/Pages");
    let pages_ref_num = create_green_token(NUMBER_KIND, "2");
    let pages_ref_gen = create_green_token(NUMBER_KIND, "0");
    let pages_ref_r = create_green_token(STRING_KIND, "R");
    let names_key = create_green_token(NAME_KIND, "/Names");

    let main_dict = create_green_node(
        DICT_KIND,
        vec![
            type_name.into(),
            catalog_name.into(),
            pages_name.into(),
            pages_ref_num.into(),
            pages_ref_gen.into(),
            pages_ref_r.into(),
            names_key.into(),
            names_dict.into(),
        ],
    );

    let endobj_kw = create_green_token(ENDOBJ_KW, "endobj");

    let root = create_green_node(
        OBJ_KIND,
        vec![
            obj_num.into(),
            gen_num.into(),
            obj_kw.into(),
            main_dict.into(),
            endobj_kw.into(),
        ],
    );

    SyntaxNode::new_root(root)
}

// =============================================================================
// Convenience Helper Functions
// =============================================================================

/// Helper to get the first token from a syntax tree
pub fn get_first_token(node: &SyntaxNode) -> crate::cursor::token::SyntaxToken {
    node.first_token()
        .expect("Tree should have at least one token")
}
