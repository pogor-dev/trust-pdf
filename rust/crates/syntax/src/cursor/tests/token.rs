use std::{collections::HashMap, ops::Range};

use crate::{
    cursor::{node::SyntaxNode, element::SyntaxElement, token::SyntaxToken},
    utility_types::Direction,
};

use super::fixtures::{
    DICT_KIND,
    ENDOBJ_KW,
    NAME_KIND,
    NUMBER_KIND,
    OBJ_KIND,
    OBJ_KW,
    // Common constants
    STRING_KIND,
    create_flat_tree as create_multi_token_syntax_tree,
    create_green_node,
    // Common helper functions
    create_green_token,
    // Common tree creation functions
    create_simple_syntax_tree,
    create_trivia_tree as create_trivia_syntax_tree,
    get_first_token,
};

// Local specialized fixtures for this test file

/// Creates a nested syntax tree specifically for token tests: OBJ -> DICT -> ["/Type", "/Page"] + "endobj"
fn create_nested_syntax_tree() -> SyntaxNode {
    let inner_token1 = create_green_token(NAME_KIND, "/Type");
    let inner_token2 = create_green_token(NAME_KIND, "/Page");
    let inner_dict = create_green_node(DICT_KIND, vec![inner_token1.into(), inner_token2.into()]);

    let outer_token = create_green_token(ENDOBJ_KW, "endobj");
    let outer_node = create_green_node(OBJ_KIND, vec![inner_dict.into(), outer_token.into()]);

    SyntaxNode::new_root(outer_node)
}

// =============================================================================
// Core SyntaxToken Tests
// =============================================================================

#[test]
fn test_kind_when_accessing_token_expect_correct_kind() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    assert_eq!(token.kind(), STRING_KIND);
}

#[test]
fn test_span_when_accessing_token_expect_correct_range() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    let span = token.span();
    assert_eq!(span, Range { start: 0, end: 7 }); // "(Hello)" has 7 bytes
}

#[test]
fn test_full_span_when_accessing_token_expect_correct_range() {
    let tree = create_trivia_syntax_tree();
    let token = get_first_token(&tree);

    let full_span = token.full_span();
    // "  /Type " has leading "  " (2 bytes) + "/Type" (5 bytes) + trailing " " (1 byte) = 8 bytes
    assert_eq!(full_span, Range { start: 0, end: 8 });
}

#[test]
fn test_index_when_accessing_token_expect_correct_position() {
    let tree = create_multi_token_syntax_tree();
    let mut tokens: Vec<SyntaxToken> = Vec::new();

    // Collect all tokens in order
    for element in tree.children_with_tokens() {
        if let Some(token) = element.into_token() {
            tokens.push(token);
        }
    }

    // Verify indices
    assert_eq!(tokens[0].index(), 0);
    assert_eq!(tokens[1].index(), 1);
    assert_eq!(tokens[2].index(), 2);
}

#[test]
fn test_text_when_accessing_content_expect_raw_bytes() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    assert_eq!(token.text(), b"(Hello)");
}

#[test]
fn test_full_text_when_accessing_content_expect_bytes_with_trivia() {
    let tree = create_trivia_syntax_tree();
    let token = get_first_token(&tree);

    assert_eq!(token.full_text(), b"  /Type ");
}

#[test]
fn test_width_when_calculating_expect_content_length() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    assert_eq!(token.width(), 7); // "(Hello)" is 7 bytes
}

#[test]
fn test_full_width_when_calculating_expect_content_plus_trivia_length() {
    let tree = create_trivia_syntax_tree();
    let token = get_first_token(&tree);

    assert_eq!(token.full_width(), 8); // "  /Type " is 8 bytes total
}

#[test]
fn test_green_when_accessing_green_data_expect_correct_reference() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    let green_data = token.green();
    assert_eq!(green_data.kind(), STRING_KIND);
    assert_eq!(green_data.text(), b"(Hello)");
}

#[test]
fn test_parent_when_accessing_parent_expect_correct_node() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    let parent = token.parent().expect("Token should have a parent");
    assert_eq!(parent.kind(), DICT_KIND);
}

// =============================================================================
// Navigation Tests
// =============================================================================

#[test]
fn test_next_sibling_or_token_when_has_sibling_expect_next_element() {
    let tree = create_multi_token_syntax_tree();
    let first_token = get_first_token(&tree);

    let next_sibling = first_token
        .next_sibling_or_token()
        .expect("Should have next sibling");

    if let Some(next_token) = next_sibling.as_token() {
        assert_eq!(next_token.kind(), NAME_KIND);
        assert_eq!(next_token.text(), b"/Catalog");
    } else {
        panic!("Expected token sibling");
    }
}

#[test]
fn test_next_sibling_or_token_when_no_sibling_expect_none() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    let next_sibling = token.next_sibling_or_token();
    assert!(next_sibling.is_none());
}

#[test]
fn test_next_sibling_or_token_by_kind_when_matching_kind_expect_found() {
    let tree = create_multi_token_syntax_tree();
    let first_token = get_first_token(&tree);

    let next_number = first_token.next_sibling_or_token_by_kind(&|k| k == NUMBER_KIND);

    assert!(next_number.is_some());
    if let Some(element) = next_number {
        if let Some(token) = element.as_token() {
            assert_eq!(token.kind(), NUMBER_KIND);
            assert_eq!(token.text(), b"42");
        } else {
            panic!("Expected token");
        }
    }
}

#[test]
fn test_next_sibling_or_token_by_kind_when_no_matching_kind_expect_none() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    let next_number = token.next_sibling_or_token_by_kind(&|k| k == NUMBER_KIND);
    assert!(next_number.is_none());
}

#[test]
fn test_prev_sibling_or_token_when_has_sibling_expect_previous_element() {
    let tree = create_multi_token_syntax_tree();
    let mut tokens: Vec<SyntaxToken> = Vec::new();

    // Collect all tokens
    for element in tree.children_with_tokens() {
        if let Some(token) = element.into_token() {
            tokens.push(token);
        }
    }

    let second_token = &tokens[1];
    let prev_sibling = second_token
        .prev_sibling_or_token()
        .expect("Should have previous sibling");

    if let Some(prev_token) = prev_sibling.as_token() {
        assert_eq!(prev_token.kind(), NAME_KIND);
        assert_eq!(prev_token.text(), b"/Type");
    } else {
        panic!("Expected token sibling");
    }
}

#[test]
fn test_prev_sibling_or_token_when_no_sibling_expect_none() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    let prev_sibling = token.prev_sibling_or_token();
    assert!(prev_sibling.is_none());
}

#[test]
fn test_siblings_with_tokens_when_direction_next_expect_forward_iteration() {
    let tree = create_multi_token_syntax_tree();
    let first_token = get_first_token(&tree);

    let siblings: Vec<SyntaxElement> = first_token
        .siblings_with_tokens(Direction::Next)
        .take(3)
        .collect();

    assert_eq!(siblings.len(), 3);

    // First element should be the token itself
    if let Some(token) = siblings[0].as_token() {
        assert_eq!(token.text(), b"/Type");
    }

    // Second element should be the next sibling
    if let Some(token) = siblings[1].as_token() {
        assert_eq!(token.text(), b"/Catalog");
    }

    // Third element should be the last token
    if let Some(token) = siblings[2].as_token() {
        assert_eq!(token.text(), b"42");
    }
}

#[test]
fn test_siblings_with_tokens_when_direction_prev_expect_backward_iteration() {
    let tree = create_multi_token_syntax_tree();
    let mut tokens: Vec<SyntaxToken> = Vec::new();

    // Collect all tokens
    for element in tree.children_with_tokens() {
        if let Some(token) = element.into_token() {
            tokens.push(token);
        }
    }

    let last_token = &tokens[2]; // Get the "42" token
    let siblings: Vec<SyntaxElement> = last_token
        .siblings_with_tokens(Direction::Prev)
        .take(3)
        .collect();

    assert_eq!(siblings.len(), 3);

    // First element should be the token itself
    if let Some(token) = siblings[0].as_token() {
        assert_eq!(token.text(), b"42");
    }

    // Second element should be the previous sibling
    if let Some(token) = siblings[1].as_token() {
        assert_eq!(token.text(), b"/Catalog");
    }

    // Third element should be the first token
    if let Some(token) = siblings[2].as_token() {
        assert_eq!(token.text(), b"/Type");
    }
}

// =============================================================================
// Token Navigation Tests
// =============================================================================

#[test]
fn test_next_token_when_has_next_token_expect_found() {
    let tree = create_multi_token_syntax_tree();
    let first_token = get_first_token(&tree);

    let next_token = first_token.next_token().expect("Should have next token");
    assert_eq!(next_token.kind(), NAME_KIND);
    assert_eq!(next_token.text(), b"/Catalog");
}

#[test]
fn test_next_token_when_crossing_nodes_expect_found() {
    let tree = create_nested_syntax_tree();
    let first_token = get_first_token(&tree);

    let next_token = first_token.next_token().expect("Should have next token");
    assert_eq!(next_token.kind(), NAME_KIND);
    assert_eq!(next_token.text(), b"/Page");
}

#[test]
fn test_prev_token_when_has_previous_token_expect_found() {
    let tree = create_multi_token_syntax_tree();
    let mut tokens: Vec<SyntaxToken> = Vec::new();

    // Collect all tokens
    for element in tree.children_with_tokens() {
        if let Some(token) = element.into_token() {
            tokens.push(token);
        }
    }

    let second_token = &tokens[1];
    let prev_token = second_token
        .prev_token()
        .expect("Should have previous token");
    assert_eq!(prev_token.kind(), NAME_KIND);
    assert_eq!(prev_token.text(), b"/Type");
}

#[test]
fn test_prev_token_when_crossing_nodes_expect_found() {
    let tree = create_nested_syntax_tree();

    // Get all tokens using a simple traversal to understand the order
    let all_tokens: Vec<SyntaxToken> = tree
        .descendants_with_tokens()
        .filter_map(|elem| elem.into_token())
        .collect();

    // Should have exactly 3 tokens in our structure
    assert_eq!(all_tokens.len(), 3);

    // The last token should be "endobj"
    let last_token = &all_tokens[2];
    assert_eq!(last_token.text(), b"endobj");

    // Test prev_token() functionality
    if let Some(prev_token) = last_token.prev_token() {
        // The previous token should be "/Page" (the last token before "endobj")
        assert_eq!(prev_token.text(), b"/Page");
    } else {
        // If prev_token() returns None, that might be expected behavior
        // depending on how the traversal works across node boundaries
        panic!("Expected to find a previous token, but got None");
    }
}

// =============================================================================
// Ancestors Tests
// =============================================================================

#[test]
fn test_ancestors_when_nested_structure_expect_parent_chain() {
    let tree = create_nested_syntax_tree();
    let first_token = get_first_token(&tree);

    let ancestors: Vec<SyntaxNode> = first_token.ancestors().collect();

    assert_eq!(ancestors.len(), 2);
    assert_eq!(ancestors[0].kind(), DICT_KIND); // Immediate parent
    assert_eq!(ancestors[1].kind(), OBJ_KIND); // Root parent
}

#[test]
fn test_ancestors_when_single_level_expect_single_parent() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    let ancestors: Vec<SyntaxNode> = token.ancestors().collect();

    assert_eq!(ancestors.len(), 1);
    assert_eq!(ancestors[0].kind(), DICT_KIND);
}

// =============================================================================
// Mutation Tests (for mutable trees)
// =============================================================================

#[test]
fn test_detach_when_mutable_tree_expect_successful_detach() {
    let green_token = create_green_token(STRING_KIND, "(Hello)");
    let green_node = create_green_node(DICT_KIND, vec![green_token.into()]);
    let tree = SyntaxNode::new_root_mut(green_node);

    let token = get_first_token(&tree);

    // Should not panic for mutable tree
    token.detach();

    // After detachment, the token should no longer be accessible through the tree
    assert!(tree.first_token().is_none());
}

#[test]
#[should_panic(expected = "immutable tree")]
fn test_detach_when_immutable_tree_expect_panic() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    // Should panic for immutable tree
    token.detach();
}

#[test]
fn test_replace_with_when_same_kind_expect_successful_replacement() {
    let green_token = create_green_token(STRING_KIND, "(Hello)");
    let green_node = create_green_node(DICT_KIND, vec![green_token.into()]);
    let tree = SyntaxNode::new_root(green_node);

    let token = get_first_token(&tree);
    let replacement = create_green_token(STRING_KIND, "(World)");

    let new_root = token.replace_with(replacement);

    // The new root should contain the replacement token
    let new_tree = SyntaxNode::new_root(new_root);
    let new_token = get_first_token(&new_tree);
    assert_eq!(new_token.text(), b"(World)");
}

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn test_replace_with_when_different_kind_expect_panic() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);
    let replacement = create_green_token(NUMBER_KIND, "42"); // Different kind

    // Should panic because kinds don't match
    token.replace_with(replacement);
}

// =============================================================================
// Clone and Memory Management Tests
// =============================================================================

#[test]
fn test_clone_when_copying_token_expect_shared_reference() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);
    let cloned_token = token.clone();

    // Both tokens should have the same content
    assert_eq!(token.text(), cloned_token.text());
    assert_eq!(token.kind(), cloned_token.kind());
    assert_eq!(token.span(), cloned_token.span());

    // They should be equal (identity semantics)
    assert_eq!(token, cloned_token);
}

// =============================================================================
// Equality and Hash Tests
// =============================================================================

#[test]
fn test_equality_when_same_token_expect_equal() {
    let tree = create_simple_syntax_tree();
    let token1 = get_first_token(&tree);
    let token2 = token1.clone();

    assert_eq!(token1, token2);
}

#[test]
fn test_equality_when_different_tokens_expect_not_equal() {
    let tree = create_multi_token_syntax_tree();
    let mut tokens: Vec<SyntaxToken> = Vec::new();

    // Collect all tokens
    for element in tree.children_with_tokens() {
        if let Some(token) = element.into_token() {
            tokens.push(token);
        }
    }

    assert_ne!(tokens[0], tokens[1]);
}

#[test]
fn test_hash_when_using_in_hashmap_expect_consistent_behavior() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    let mut map: HashMap<SyntaxToken, &str> = HashMap::new();
    map.insert(token.clone(), "test_value");

    // Should be able to retrieve using the same token
    assert_eq!(map.get(&token), Some(&"test_value"));
}

// =============================================================================
// Display Tests
// =============================================================================

#[test]
fn test_display_when_formatting_token_expect_utf8_text() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    let display_output = format!("{}", token);
    assert_eq!(display_output, "(Hello)");
}

#[test]
fn test_display_when_with_trivia_expect_full_text_conversion() {
    let tree = create_trivia_syntax_tree();
    let token = get_first_token(&tree);

    let display_output = format!("{}", token);
    assert_eq!(display_output, "  /Type "); // Should include trivia
}

// =============================================================================
// Edge Cases and Error Handling Tests
// =============================================================================

#[test]
fn test_text_when_empty_token_expect_empty_bytes() {
    let green_token = create_green_token(STRING_KIND, "");
    let green_node = create_green_node(DICT_KIND, vec![green_token.into()]);
    let tree = SyntaxNode::new_root(green_node);

    let token = get_first_token(&tree);

    assert_eq!(token.text(), b"");
    assert_eq!(token.width(), 0);
    assert_eq!(token.full_width(), 0);
}

#[test]
fn test_navigation_when_single_token_tree_expect_no_siblings() {
    let tree = create_simple_syntax_tree();
    let token = get_first_token(&tree);

    assert!(token.next_sibling_or_token().is_none());
    assert!(token.prev_sibling_or_token().is_none());
    assert!(token.next_token().is_none());
    assert!(token.prev_token().is_none());
}

#[test]
fn test_complex_pdf_structure_when_realistic_content_expect_correct_handling() {
    // Create a more realistic PDF structure: 1 0 obj << /Type /Catalog >> endobj
    let obj_num = create_green_token(NUMBER_KIND, "1");
    let gen_num = create_green_token(NUMBER_KIND, "0");
    let obj_kw = create_green_token(OBJ_KW, "obj");

    let type_name = create_green_token(NAME_KIND, "/Type");
    let catalog_name = create_green_token(NAME_KIND, "/Catalog");
    let dict = create_green_node(DICT_KIND, vec![type_name.into(), catalog_name.into()]);

    let endobj_kw = create_green_token(ENDOBJ_KW, "endobj");

    let obj = create_green_node(
        OBJ_KIND,
        vec![
            obj_num.into(),
            gen_num.into(),
            obj_kw.into(),
            dict.into(),
            endobj_kw.into(),
        ],
    );

    let tree = SyntaxNode::new_root(obj);
    let first_token = get_first_token(&tree);

    // Should be the object number
    assert_eq!(first_token.text(), b"1");
    assert_eq!(first_token.kind(), NUMBER_KIND);

    // Navigate through the structure
    let next_token = first_token.next_token().expect("Should have next token");
    assert_eq!(next_token.text(), b"0");

    let obj_keyword = next_token.next_token().expect("Should have next token");
    assert_eq!(obj_keyword.text(), b"obj");

    // Should be able to navigate to tokens inside the dictionary
    let type_token = obj_keyword.next_token().expect("Should have next token");
    assert_eq!(type_token.text(), b"/Type");

    let catalog_token = type_token.next_token().expect("Should have next token");
    assert_eq!(catalog_token.text(), b"/Catalog");

    let endobj_token = catalog_token.next_token().expect("Should have next token");
    assert_eq!(endobj_token.text(), b"endobj");

    // Should be the last token
    assert!(endobj_token.next_token().is_none());
}
