use crate::{
    NodeOrToken,
    cursor::{node::SyntaxNode, element::SyntaxElement},
    utility_types::TokenAtOffset,
};

use super::fixtures::{
    // Common constants
    STRING_KIND, NUMBER_KIND, NAME_KIND, DICT_KIND, ARRAY_KIND, OBJ_KIND, COMMENT_KIND,
    // Common helper functions
    create_green_token, create_green_node,
};

// Test constants for local use
const BOOLEAN_KIND: crate::SyntaxKind = crate::SyntaxKind(8);

// Local specialized fixtures for this test file

/// Creates a tree with mixed tokens and nodes for comprehensive testing
fn create_mixed_tree() -> SyntaxNode {
    // Create tokens
    let name_token = create_green_token(NAME_KIND, "/Type");
    let number_token = create_green_token(NUMBER_KIND, "42");
    let string_token = create_green_token(STRING_KIND, "(text)");
    let boolean_token = create_green_token(BOOLEAN_KIND, "true");

    // Create nested structure: OBJ -> DICT -> [NAME, NUMBER, ARRAY -> [STRING, BOOLEAN]]
    let array_children = vec![string_token.into(), boolean_token.into()];
    let array_node = create_green_node(ARRAY_KIND, array_children);

    let dict_children = vec![name_token.into(), number_token.into(), array_node.into()];
    let dict_node = create_green_node(DICT_KIND, dict_children);

    let obj_node = create_green_node(OBJ_KIND, vec![dict_node.into()]);
    SyntaxNode::new_root(obj_node)
}

/// Creates a tree with only tokens for testing token-specific behavior
fn create_token_only_tree() -> SyntaxNode {
    let name_token = create_green_token(NAME_KIND, "/Key");
    let string_token = create_green_token(STRING_KIND, "(value)");
    let number_token = create_green_token(NUMBER_KIND, "123");

    let dict_children = vec![name_token.into(), string_token.into(), number_token.into()];
    let dict_node = create_green_node(DICT_KIND, dict_children);
    let obj_node = create_green_node(OBJ_KIND, vec![dict_node.into()]);
    SyntaxNode::new_root(obj_node)
}

/// Creates a simple single element tree for basic testing
fn create_simple_element_tree() -> SyntaxNode {
    let token = create_green_token(STRING_KIND, "(hello)");
    let dict = create_green_node(DICT_KIND, vec![token.into()]);
    let obj = create_green_node(OBJ_KIND, vec![dict.into()]);
    SyntaxNode::new_root(obj)
}

// =============================================================================
// SyntaxElement Construction Tests
// =============================================================================

#[test]
fn test_syntax_element_from_node_when_converting_expect_node_variant() {
    let tree = create_mixed_tree();
    let node = tree.first_child().unwrap();

    let element: SyntaxElement = node.clone().into();

    match element {
        NodeOrToken::Node(syntax_node) => {
            assert_eq!(syntax_node.kind(), node.kind());
            assert_eq!(syntax_node.index(), node.index());
        }
        NodeOrToken::Token(_) => panic!("Expected Node variant"),
    }
}

#[test]
fn test_syntax_element_from_token_when_converting_expect_token_variant() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let token_element = dict.first_child_or_token().unwrap();

    match token_element {
        NodeOrToken::Token(syntax_token) => {
            let element: SyntaxElement = syntax_token.into();
            match element {
                NodeOrToken::Token(converted_token) => {
                    assert_eq!(converted_token.kind(), NAME_KIND);
                }
                NodeOrToken::Node(_) => panic!("Expected Token variant"),
            }
        }
        NodeOrToken::Node(_) => panic!("Expected to get a token from first child"),
    }
}

// =============================================================================
// Basic Property Tests
// =============================================================================

#[test]
fn test_span_when_node_element_expect_correct_range() {
    let tree = create_mixed_tree();
    let node = tree.first_child().unwrap();
    let element: SyntaxElement = node.into();

    let span = element.span();
    assert!(span.end > span.start);
    assert_eq!(span.start, 0); // First child should start at offset 0
}

#[test]
fn test_span_when_token_element_expect_correct_range() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let token_element = dict.first_child_or_token().unwrap();

    let span = token_element.span();
    assert!(span.end > span.start);
}

#[test]
fn test_full_span_when_element_expect_includes_trivia() {
    let tree = create_mixed_tree();
    let element = tree.first_child_or_token().unwrap();

    let span = element.span();
    let full_span = element.full_span();

    // Full span should be at least as large as span
    assert!(full_span.start <= span.start);
    assert!(full_span.end >= span.end);
}

#[test]
fn test_index_when_multiple_elements_expect_correct_positions() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let elements: Vec<_> = dict.children_with_tokens().collect();

    for (expected_index, element) in elements.iter().enumerate() {
        assert_eq!(element.index(), expected_index);
    }
}

#[test]
fn test_kind_when_different_elements_expect_correct_kinds() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let elements: Vec<_> = dict.children_with_tokens().collect();

    assert_eq!(elements[0].kind(), NAME_KIND);
    assert_eq!(elements[1].kind(), STRING_KIND);
    assert_eq!(elements[2].kind(), NUMBER_KIND);
}

// =============================================================================
// Parent and Ancestor Tests
// =============================================================================

#[test]
fn test_parent_when_node_element_expect_correct_parent() {
    let tree = create_mixed_tree();
    let child_node = tree.first_child().unwrap();
    let element: SyntaxElement = child_node.into();

    let parent = element.parent().unwrap();
    assert_eq!(parent.kind(), OBJ_KIND);
}

#[test]
fn test_parent_when_token_element_expect_correct_parent() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let token_element = dict.first_child_or_token().unwrap();

    let parent = token_element.parent().unwrap();
    assert_eq!(parent.kind(), DICT_KIND);
}

#[test]
fn test_parent_when_root_element_expect_none() {
    let tree = create_mixed_tree();
    let root_element: SyntaxElement = tree.into();

    assert!(root_element.parent().is_none());
}

#[test]
fn test_ancestors_when_node_element_expect_correct_chain() {
    let tree = create_mixed_tree();
    let child_element: SyntaxElement = tree.first_child().unwrap().into();

    let ancestors: Vec<_> = child_element.ancestors().collect();

    // ancestors() includes the element itself as the first ancestor, then actual parents
    assert_eq!(ancestors.len(), 2);
    assert_eq!(ancestors[0].kind(), DICT_KIND); // The element itself
    assert_eq!(ancestors[1].kind(), OBJ_KIND); // The actual parent
}

#[test]
fn test_ancestors_when_token_element_expect_correct_chain() {
    let tree = create_mixed_tree();
    let dict = tree.first_child().unwrap();
    let token_element = dict.first_child_or_token().unwrap();

    let ancestors: Vec<_> = token_element.ancestors().collect();

    // Should include dict parent and obj grandparent
    assert!(ancestors.len() >= 2);
    assert_eq!(ancestors[0].kind(), DICT_KIND); // Immediate parent
    assert_eq!(ancestors[1].kind(), OBJ_KIND); // Grandparent
}

// =============================================================================
// Token Navigation Tests
// =============================================================================

#[test]
fn test_first_token_when_node_element_expect_leftmost_token() {
    let tree = create_mixed_tree();
    let dict_element: SyntaxElement = tree.first_child().unwrap().into();

    let first_token = dict_element.first_token().unwrap();
    assert_eq!(first_token.kind(), NAME_KIND);
}

#[test]
fn test_first_token_when_token_element_expect_self() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let token_element = dict.first_child_or_token().unwrap();

    let first_token = token_element.first_token().unwrap();
    assert_eq!(first_token.kind(), token_element.kind());
}

#[test]
fn test_last_token_when_node_element_expect_rightmost_token() {
    let tree = create_mixed_tree();
    let dict_element: SyntaxElement = tree.first_child().unwrap().into();

    let last_token = dict_element.last_token().unwrap();
    // Should be the last token in the tree structure
    assert!(last_token.kind() == BOOLEAN_KIND || last_token.kind() == STRING_KIND);
}

#[test]
fn test_last_token_when_token_element_expect_self() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let token_element = dict.first_child_or_token().unwrap();

    let last_token = token_element.last_token().unwrap();
    assert_eq!(last_token.kind(), token_element.kind());
}

// =============================================================================
// Sibling Navigation Tests
// =============================================================================

#[test]
fn test_next_sibling_or_token_when_has_sibling_expect_next_element() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let first_element = dict.first_child_or_token().unwrap();

    let next_element = first_element.next_sibling_or_token().unwrap();
    assert_eq!(next_element.kind(), STRING_KIND);
    assert_eq!(next_element.index(), 1);
}

#[test]
fn test_next_sibling_or_token_when_last_element_expect_none() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let last_element = dict.last_child_or_token().unwrap();

    assert!(last_element.next_sibling_or_token().is_none());
}

#[test]
fn test_to_next_sibling_or_token_when_can_optimize_expect_efficient_navigation() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let first_element = dict.first_child_or_token().unwrap();

    // This should consume the element and return the next one efficiently
    let next_element = first_element.to_next_sibling_or_token().unwrap();
    assert_eq!(next_element.kind(), STRING_KIND);
}

#[test]
fn test_to_next_sibling_or_token_when_last_element_expect_none() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let last_element = dict.last_child_or_token().unwrap();

    assert!(last_element.to_next_sibling_or_token().is_none());
}

#[test]
fn test_next_sibling_or_token_by_kind_when_matching_expect_next_match() {
    let tree = create_mixed_tree();
    let dict = tree.first_child().unwrap();
    let first_element = dict.first_child_or_token().unwrap(); // NAME token

    // Find next NUMBER token (should skip to it)
    let number_element = first_element
        .next_sibling_or_token_by_kind(&|kind| kind == NUMBER_KIND)
        .unwrap();
    assert_eq!(number_element.kind(), NUMBER_KIND);
}

#[test]
fn test_next_sibling_or_token_by_kind_when_no_match_expect_none() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let first_element = dict.first_child_or_token().unwrap();

    // Look for non-existent kind
    let result = first_element.next_sibling_or_token_by_kind(&|kind| kind == COMMENT_KIND);
    assert!(result.is_none());
}

#[test]
fn test_prev_sibling_or_token_when_has_sibling_expect_prev_element() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let elements: Vec<_> = dict.children_with_tokens().collect();
    let second_element = &elements[1];

    let prev_element = second_element.prev_sibling_or_token().unwrap();
    assert_eq!(prev_element.kind(), NAME_KIND);
    assert_eq!(prev_element.index(), 0);
}

#[test]
fn test_prev_sibling_or_token_when_first_element_expect_none() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let first_element = dict.first_child_or_token().unwrap();

    assert!(first_element.prev_sibling_or_token().is_none());
}

// =============================================================================
// Token At Offset Tests
// =============================================================================

#[test]
fn test_token_at_offset_when_token_element_expect_self() {
    let tree = create_simple_element_tree();
    let dict = tree.first_child().unwrap();
    let token_element = dict.first_child_or_token().unwrap();

    let span = token_element.full_span();
    let result = token_element.token_at_offset(span.start);

    match result {
        TokenAtOffset::Single(token) => {
            assert_eq!(token.kind(), token_element.kind());
        }
        _ => panic!("Expected single token result for token element"),
    }
}

#[test]
fn test_token_at_offset_when_node_element_expect_descendant_token() {
    let tree = create_simple_element_tree();
    let dict_element: SyntaxElement = tree.first_child().unwrap().into();

    let span = dict_element.full_span();
    let result = dict_element.token_at_offset(span.start);

    // Should find the token inside the dict
    match result {
        TokenAtOffset::Single(token) => {
            assert_eq!(token.kind(), STRING_KIND);
        }
        TokenAtOffset::Between(_, _) => {
            // This is also acceptable for boundary cases
        }
        TokenAtOffset::None => panic!("Expected to find a token in the node"),
    }
}

// =============================================================================
// Element Type Checking Tests
// =============================================================================

#[test]
fn test_as_node_when_node_element_expect_some() {
    let tree = create_mixed_tree();
    let node = tree.first_child().unwrap();
    let element: SyntaxElement = node.into();

    assert!(element.as_node().is_some());
    assert!(element.as_token().is_none());
}

#[test]
fn test_as_token_when_token_element_expect_some() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let token_element = dict.first_child_or_token().unwrap();

    assert!(token_element.as_token().is_some());
    assert!(token_element.as_node().is_none());
}

// =============================================================================
// Mixed Content Navigation Tests
// =============================================================================

#[test]
fn test_mixed_content_navigation_when_traversing_expect_correct_sequence() {
    let tree = create_mixed_tree();
    let dict = tree.first_child().unwrap();

    // Traverse all children with tokens
    let elements: Vec<_> = dict.children_with_tokens().collect();

    // Should have: NAME token, NUMBER token, ARRAY node
    assert_eq!(elements.len(), 3);
    assert_eq!(elements[0].kind(), NAME_KIND);
    assert!(elements[0].as_token().is_some());
    assert_eq!(elements[1].kind(), NUMBER_KIND);
    assert!(elements[1].as_token().is_some());
    assert_eq!(elements[2].kind(), ARRAY_KIND);
    assert!(elements[2].as_node().is_some());
}

#[test]
fn test_cross_node_token_navigation_when_nested_expect_correct_traversal() {
    let tree = create_mixed_tree();
    let dict = tree.first_child().unwrap();
    let first_token = dict.first_child_or_token().unwrap(); // NAME token

    // Navigate through siblings to reach the array node
    let number_token = first_token.next_sibling_or_token().unwrap();
    let array_node = number_token.next_sibling_or_token().unwrap();

    assert_eq!(number_token.kind(), NUMBER_KIND);
    assert_eq!(array_node.kind(), ARRAY_KIND);
    assert!(array_node.as_node().is_some());

    // Navigate into the array to find its first token
    if let Some(array_syntax_node) = array_node.as_node() {
        let first_array_token = array_syntax_node.first_child_or_token().unwrap();
        assert_eq!(first_array_token.kind(), STRING_KIND);
    }
}

// =============================================================================
// Performance and Memory Tests
// =============================================================================

#[test]
fn test_element_cloning_when_multiple_references_expect_consistent_behavior() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let element = dict.first_child_or_token().unwrap();

    // Clone the element
    let cloned_element = element.clone();

    // Both should have same properties
    assert_eq!(element.kind(), cloned_element.kind());
    assert_eq!(element.index(), cloned_element.index());
    assert_eq!(element.span(), cloned_element.span());

    // Both should navigate to same siblings
    let next1 = element.next_sibling_or_token();
    let next2 = cloned_element.next_sibling_or_token();

    match (next1, next2) {
        (Some(n1), Some(n2)) => {
            assert_eq!(n1.kind(), n2.kind());
            assert_eq!(n1.index(), n2.index());
        }
        (None, None) => {
            // Both are None - that's consistent
        }
        _ => panic!("Inconsistent navigation results"),
    }
}

#[test]
fn test_large_sibling_chain_when_navigating_expect_efficient_traversal() {
    // Create a tree with many siblings to test performance
    let mut children = Vec::new();
    for i in 0..100 {
        let token = create_green_token(NUMBER_KIND, &format!("{}", i));
        children.push(token.into());
    }

    let dict = create_green_node(DICT_KIND, children);
    let obj = create_green_node(OBJ_KIND, vec![dict.into()]);
    let tree = SyntaxNode::new_root(obj);

    let dict_node = tree.first_child().unwrap();
    let first_element = dict_node.first_child_or_token().unwrap();

    // Navigate through all siblings
    let mut current = Some(first_element);
    let mut count = 0;

    while let Some(element) = current {
        count += 1;
        current = element.next_sibling_or_token();
    }

    assert_eq!(count, 100);
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_empty_parent_when_no_children_expect_no_elements() {
    let empty_dict = create_green_node(DICT_KIND, vec![]);
    let obj = create_green_node(OBJ_KIND, vec![empty_dict.into()]);
    let tree = SyntaxNode::new_root(obj);

    let dict_node = tree.first_child().unwrap();

    assert!(dict_node.first_child_or_token().is_none());
    assert!(dict_node.last_child_or_token().is_none());

    let elements: Vec<_> = dict_node.children_with_tokens().collect();
    assert!(elements.is_empty());
}

#[test]
fn test_single_element_when_only_child_expect_correct_navigation() {
    let tree = create_simple_element_tree();
    let dict = tree.first_child().unwrap();
    let only_element = dict.first_child_or_token().unwrap();

    // Should be both first and last
    let first = dict.first_child_or_token().unwrap();
    let last = dict.last_child_or_token().unwrap();

    assert_eq!(only_element.kind(), first.kind());
    assert_eq!(only_element.kind(), last.kind());
    assert_eq!(only_element.index(), first.index());
    assert_eq!(only_element.index(), last.index());

    // Should have no siblings
    assert!(only_element.next_sibling_or_token().is_none());
    assert!(only_element.prev_sibling_or_token().is_none());
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_element_integration_when_mixed_operations_expect_consistent_results() {
    let tree = create_mixed_tree();

    // Start from root and navigate down
    let root_element: SyntaxElement = tree.clone().into();
    let dict_element = root_element.first_token().unwrap().parent().unwrap().into();

    // Navigate through children
    let elements: Vec<_> = if let NodeOrToken::Node(dict_node) = dict_element {
        dict_node.children_with_tokens().collect()
    } else {
        vec![]
    };

    assert!(!elements.is_empty());

    // Test that we can navigate back to parent from each child
    for element in elements {
        let parent = element.parent().unwrap();
        assert_eq!(parent.kind(), DICT_KIND);

        // Test ancestor chain
        let ancestors: Vec<_> = element.ancestors().collect();
        assert!(ancestors.len() >= 1);
    }
}

#[test]
fn test_element_debugging_when_formatting_expect_useful_output() {
    let tree = create_mixed_tree();
    let dict = tree.first_child().unwrap();
    let element = dict.first_child_or_token().unwrap();

    let debug_str = format!("{:?}", element);

    // Should contain useful information for debugging
    // The exact format depends on the implementation, but it should be informative
    assert!(!debug_str.is_empty());
}

// =============================================================================
// Detach Operation Tests (if supported)
// =============================================================================

#[test]
fn test_detach_when_mutable_tree_expect_element_detached() {
    // Create a mutable tree
    let green = create_green_node(
        DICT_KIND,
        vec![
            create_green_token(NAME_KIND, "/Key").into(),
            create_green_token(STRING_KIND, "(value)").into(),
        ],
    );
    let obj = create_green_node(OBJ_KIND, vec![green.into()]);
    let tree = SyntaxNode::new_root_mut(obj);

    let dict = tree.first_child().unwrap();
    let first_element = dict.first_child_or_token().unwrap();

    // Detach the element
    first_element.detach();

    // After detach, the dict should have one fewer child
    let remaining_elements: Vec<_> = dict.children_with_tokens().collect();
    assert_eq!(remaining_elements.len(), 1);
    assert_eq!(remaining_elements[0].kind(), STRING_KIND);
}
