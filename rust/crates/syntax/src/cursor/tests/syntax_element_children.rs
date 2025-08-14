use crate::{
    SyntaxKind,
    cursor::{
        node::SyntaxNode, syntax_element::SyntaxElement,
        syntax_element_children::SyntaxElementChildren,
    },
    green::{element::GreenElement, node::GreenNode, token::GreenToken, trivia::GreenTrivia},
};

// Test constants for different PDF syntax kinds
const STRING_KIND: SyntaxKind = SyntaxKind(1);
const NUMBER_KIND: SyntaxKind = SyntaxKind(2);
const NAME_KIND: SyntaxKind = SyntaxKind(3);
const DICT_KIND: SyntaxKind = SyntaxKind(4);
const ARRAY_KIND: SyntaxKind = SyntaxKind(5);
const OBJ_KIND: SyntaxKind = SyntaxKind(6);
const COMMENT_KIND: SyntaxKind = SyntaxKind(7);
const BOOLEAN_KIND: SyntaxKind = SyntaxKind(8);

// =============================================================================
// Helper Functions
// =============================================================================

fn create_green_token(kind: SyntaxKind, text: &str) -> GreenToken {
    let empty_trivia = GreenTrivia::new([]);
    GreenToken::new(kind, text.as_bytes(), empty_trivia.clone(), empty_trivia)
}

fn create_green_node(kind: SyntaxKind, children: Vec<GreenElement>) -> GreenNode {
    GreenNode::new(kind, children)
}

/// Creates a tree with only tokens as children
fn create_token_only_tree() -> SyntaxNode {
    let name_token = create_green_token(NAME_KIND, "/Type");
    let number_token = create_green_token(NUMBER_KIND, "42");
    let string_token = create_green_token(STRING_KIND, "(text)");

    let dict_children = vec![name_token.into(), number_token.into(), string_token.into()];
    let dict_node = create_green_node(DICT_KIND, dict_children);
    let obj_node = create_green_node(OBJ_KIND, vec![dict_node.into()]);
    SyntaxNode::new_root(obj_node)
}

/// Creates a tree with mixed tokens and nodes
fn create_mixed_children_tree() -> SyntaxNode {
    // Create tokens
    let name_token = create_green_token(NAME_KIND, "/Key1");
    let number_token = create_green_token(NUMBER_KIND, "123");

    // Create a nested array node
    let array_token1 = create_green_token(STRING_KIND, "(item1)");
    let array_token2 = create_green_token(BOOLEAN_KIND, "true");
    let array_node = create_green_node(ARRAY_KIND, vec![array_token1.into(), array_token2.into()]);

    // Create another token after the node
    let final_token = create_green_token(COMMENT_KIND, "% comment");

    // Structure: DICT -> [NAME, NUMBER, ARRAY, COMMENT]
    let dict_children = vec![
        name_token.into(),
        number_token.into(),
        array_node.into(),
        final_token.into(),
    ];
    let dict_node = create_green_node(DICT_KIND, dict_children);
    let obj_node = create_green_node(OBJ_KIND, vec![dict_node.into()]);
    SyntaxNode::new_root(obj_node)
}

/// Creates a tree with nested structure for testing deep iteration
fn create_deeply_nested_tree() -> SyntaxNode {
    let innermost_token = create_green_token(STRING_KIND, "(deep)");
    let level2_node = create_green_node(DICT_KIND, vec![innermost_token.into()]);
    let level1_node = create_green_node(ARRAY_KIND, vec![level2_node.into()]);
    let root_node = create_green_node(OBJ_KIND, vec![level1_node.into()]);
    SyntaxNode::new_root(root_node)
}

/// Creates an empty tree for edge case testing
fn create_empty_tree() -> SyntaxNode {
    let empty_dict = create_green_node(DICT_KIND, vec![]);
    let obj_node = create_green_node(OBJ_KIND, vec![empty_dict.into()]);
    SyntaxNode::new_root(obj_node)
}

/// Creates a tree with many children for performance testing
fn create_large_children_tree() -> SyntaxNode {
    let mut children = Vec::new();
    for i in 0..20 {
        let token = create_green_token(NUMBER_KIND, &format!("{}", i));
        children.push(token.into());
    }

    let dict_node = create_green_node(DICT_KIND, children);
    let obj_node = create_green_node(OBJ_KIND, vec![dict_node.into()]);
    SyntaxNode::new_root(obj_node)
}

/// Creates a tree with all different kinds for comprehensive filtering tests
fn create_all_kinds_tree() -> SyntaxNode {
    let string_token = create_green_token(STRING_KIND, "(string)");
    let number_token = create_green_token(NUMBER_KIND, "42");
    let name_token = create_green_token(NAME_KIND, "/name");
    let boolean_token = create_green_token(BOOLEAN_KIND, "true");
    let comment_token = create_green_token(COMMENT_KIND, "% comment");

    let array_node = create_green_node(ARRAY_KIND, vec![boolean_token.into()]);
    let nested_dict = create_green_node(DICT_KIND, vec![name_token.into()]);

    let dict_children = vec![
        string_token.into(),
        number_token.into(),
        array_node.into(),
        nested_dict.into(),
        comment_token.into(),
    ];
    let dict_node = create_green_node(DICT_KIND, dict_children);
    let obj_node = create_green_node(OBJ_KIND, vec![dict_node.into()]);
    SyntaxNode::new_root(obj_node)
}

// =============================================================================
// SyntaxElementChildren Basic Tests
// =============================================================================

#[test]
fn test_new_when_creating_syntax_element_children_expect_correct_initial_state() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict.clone());

    // Debug format should be readable
    let debug_str = format!("{:?}", children);
    assert!(debug_str.contains("SyntaxElementChildren"));

    // Clone should work
    let _cloned = children.clone();
}

#[test]
fn test_iterator_when_token_only_children_expect_all_tokens() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let collected: Vec<SyntaxElement> = children.collect();

    assert_eq!(collected.len(), 3);
    assert_eq!(collected[0].kind(), NAME_KIND);
    assert_eq!(collected[1].kind(), NUMBER_KIND);
    assert_eq!(collected[2].kind(), STRING_KIND);

    // All should be tokens
    for element in collected {
        assert!(element.as_token().is_some());
    }
}

#[test]
fn test_iterator_when_mixed_children_expect_all_elements_in_order() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let collected: Vec<SyntaxElement> = children.collect();

    assert_eq!(collected.len(), 4);
    assert_eq!(collected[0].kind(), NAME_KIND);
    assert!(collected[0].as_token().is_some());
    assert_eq!(collected[1].kind(), NUMBER_KIND);
    assert!(collected[1].as_token().is_some());
    assert_eq!(collected[2].kind(), ARRAY_KIND);
    assert!(collected[2].as_node().is_some());
    assert_eq!(collected[3].kind(), COMMENT_KIND);
    assert!(collected[3].as_token().is_some());
}

#[test]
fn test_iterator_when_empty_parent_expect_no_items() {
    let tree = create_empty_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let collected: Vec<SyntaxElement> = children.collect();

    assert_eq!(collected.len(), 0);
}

#[test]
fn test_iterator_when_consumed_partially_expect_remaining_items() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let mut children = SyntaxElementChildren::new(dict);

    // Consume first item
    let first = children.next().unwrap();
    assert_eq!(first.kind(), NAME_KIND);

    // Collect remaining
    let remaining: Vec<SyntaxElement> = children.collect();
    assert_eq!(remaining.len(), 3);
    assert_eq!(remaining[0].kind(), NUMBER_KIND);
    assert_eq!(remaining[1].kind(), ARRAY_KIND);
    assert_eq!(remaining[2].kind(), COMMENT_KIND);
}

#[test]
fn test_iterator_when_multiple_calls_to_next_expect_correct_sequence() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let mut children = SyntaxElementChildren::new(dict);

    let first = children.next();
    assert!(first.is_some());
    assert_eq!(first.unwrap().kind(), NAME_KIND);

    let second = children.next();
    assert!(second.is_some());
    assert_eq!(second.unwrap().kind(), NUMBER_KIND);

    let third = children.next();
    assert!(third.is_some());
    assert_eq!(third.unwrap().kind(), ARRAY_KIND);

    let fourth = children.next();
    assert!(fourth.is_some());
    assert_eq!(fourth.unwrap().kind(), COMMENT_KIND);

    let fifth = children.next();
    assert!(fifth.is_none());

    // Further calls should continue to return None
    assert!(children.next().is_none());
}

#[test]
fn test_iterator_when_large_number_of_children_expect_all_visited() {
    let tree = create_large_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let collected: Vec<SyntaxElement> = children.collect();

    assert_eq!(collected.len(), 20);
    for (i, element) in collected.iter().enumerate() {
        assert_eq!(element.kind(), NUMBER_KIND);
        assert_eq!(element.index(), i);
        assert!(element.as_token().is_some());
    }
}

// =============================================================================
// SyntaxElementChildren Clone and State Tests
// =============================================================================

#[test]
fn test_clone_when_iterator_not_started_expect_independent_state() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let children1 = SyntaxElementChildren::new(dict);
    let children2 = children1.clone();

    let collected1: Vec<SyntaxElement> = children1.collect();
    let collected2: Vec<SyntaxElement> = children2.collect();

    assert_eq!(collected1.len(), collected2.len());
    for (c1, c2) in collected1.iter().zip(collected2.iter()) {
        assert_eq!(c1.kind(), c2.kind());
        assert_eq!(c1.index(), c2.index());
    }
}

#[test]
fn test_clone_when_iterator_partially_consumed_expect_independent_state() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let mut children1 = SyntaxElementChildren::new(dict);

    // Consume one item
    let _first = children1.next().unwrap();

    // Clone after partial consumption
    let children2 = children1.clone();

    let remaining1: Vec<SyntaxElement> = children1.collect();
    let remaining2: Vec<SyntaxElement> = children2.collect();

    assert_eq!(remaining1.len(), remaining2.len());
    assert_eq!(remaining1.len(), 3); // Should have 3 remaining

    for (c1, c2) in remaining1.iter().zip(remaining2.iter()) {
        assert_eq!(c1.kind(), c2.kind());
        assert_eq!(c1.index(), c2.index());
    }
}

// =============================================================================
// SyntaxElementChildrenByKind Basic Tests
// =============================================================================

#[test]
fn test_by_kind_when_filtering_token_kind_expect_only_matching_tokens() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let string_elements: Vec<SyntaxElement> =
        children.by_kind(|kind| kind == STRING_KIND).collect();

    assert_eq!(string_elements.len(), 1);
    assert_eq!(string_elements[0].kind(), STRING_KIND);
    assert!(string_elements[0].as_token().is_some());
}

#[test]
fn test_by_kind_when_filtering_node_kind_expect_only_matching_nodes() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let array_elements: Vec<SyntaxElement> = children.by_kind(|kind| kind == ARRAY_KIND).collect();

    assert_eq!(array_elements.len(), 1);
    assert_eq!(array_elements[0].kind(), ARRAY_KIND);
    assert!(array_elements[0].as_node().is_some());
}

#[test]
fn test_by_kind_when_filtering_multiple_kinds_expect_all_matches() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let dict_elements: Vec<SyntaxElement> = children.by_kind(|kind| kind == DICT_KIND).collect();

    assert_eq!(dict_elements.len(), 1);
    assert_eq!(dict_elements[0].kind(), DICT_KIND);
    assert!(dict_elements[0].as_node().is_some());
}

#[test]
fn test_by_kind_when_filtering_non_existent_kind_expect_empty() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let obj_elements: Vec<SyntaxElement> = children.by_kind(|kind| kind == OBJ_KIND).collect();

    assert_eq!(obj_elements.len(), 0);
}

#[test]
fn test_by_kind_when_filtering_all_kinds_expect_all_elements() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let all_elements: Vec<SyntaxElement> = children.by_kind(|_| true).collect();

    assert_eq!(all_elements.len(), 4); // All children should be included
}

#[test]
fn test_by_kind_when_filtering_no_kinds_expect_empty() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let no_elements: Vec<SyntaxElement> = children.by_kind(|_| false).collect();

    assert_eq!(no_elements.len(), 0);
}

#[test]
fn test_by_kind_when_complex_filter_expect_correct_subset() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);

    // Filter for STRING_KIND or NUMBER_KIND
    let filtered: Vec<SyntaxElement> = children
        .by_kind(|kind| kind == STRING_KIND || kind == NUMBER_KIND)
        .collect();

    assert_eq!(filtered.len(), 2); // 1 STRING + 1 NUMBER
    for element in filtered {
        assert!(element.kind() == STRING_KIND || element.kind() == NUMBER_KIND);
        assert!(element.as_token().is_some());
    }
}

// =============================================================================
// SyntaxElementChildrenByKind State Tests
// =============================================================================

#[test]
fn test_by_kind_clone_when_creating_expect_independent_state() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let by_kind1 = children.by_kind(|kind| kind == DICT_KIND);
    let by_kind2 = by_kind1.clone();

    let collected1: Vec<SyntaxElement> = by_kind1.collect();
    let collected2: Vec<SyntaxElement> = by_kind2.collect();

    assert_eq!(collected1.len(), collected2.len());
    assert_eq!(collected1.len(), 1);

    for (c1, c2) in collected1.iter().zip(collected2.iter()) {
        assert_eq!(c1.kind(), c2.kind());
    }
}

#[test]
fn test_by_kind_when_iterator_partially_consumed_expect_remaining_items() {
    let tree = create_large_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let mut by_kind = children.by_kind(|kind| kind == NUMBER_KIND);

    // Consume first few items
    for _ in 0..5 {
        assert!(by_kind.next().is_some());
    }

    // Collect remaining
    let remaining: Vec<SyntaxElement> = by_kind.collect();
    assert_eq!(remaining.len(), 15); // Should have 15 remaining out of 20

    for element in remaining {
        assert_eq!(element.kind(), NUMBER_KIND);
    }
}

#[test]
fn test_by_kind_when_multiple_next_calls_expect_correct_sequence() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let mut by_kind = children.by_kind(|kind| matches!(kind, STRING_KIND | NUMBER_KIND));

    let first = by_kind.next();
    assert!(first.is_some());
    assert_eq!(first.unwrap().kind(), STRING_KIND);

    let second = by_kind.next();
    assert!(second.is_some());
    assert_eq!(second.unwrap().kind(), NUMBER_KIND);

    let third = by_kind.next();
    assert!(third.is_none());

    // Further calls should continue to return None
    assert!(by_kind.next().is_none());
}

// =============================================================================
// Advanced Filtering Tests
// =============================================================================

#[test]
fn test_by_kind_when_filter_by_element_type_expect_correct_filtering() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);

    // Filter for all token kinds
    let token_kinds = [
        NAME_KIND,
        NUMBER_KIND,
        STRING_KIND,
        COMMENT_KIND,
        BOOLEAN_KIND,
    ];
    let token_elements: Vec<SyntaxElement> = children
        .by_kind(|kind| token_kinds.contains(&kind))
        .collect();

    // Should get 3 tokens: NAME, NUMBER, COMMENT (STRING and BOOLEAN are inside ARRAY)
    assert_eq!(token_elements.len(), 3);
    for element in token_elements {
        assert!(element.as_token().is_some());
    }
}

#[test]
fn test_by_kind_when_filter_by_node_types_expect_correct_filtering() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);

    // Filter for node kinds
    let node_kinds = [DICT_KIND, ARRAY_KIND, OBJ_KIND];
    let node_elements: Vec<SyntaxElement> = children
        .by_kind(|kind| node_kinds.contains(&kind))
        .collect();

    // Should get 1 node: ARRAY
    assert_eq!(node_elements.len(), 1);
    assert_eq!(node_elements[0].kind(), ARRAY_KIND);
    assert!(node_elements[0].as_node().is_some());
}

#[test]
fn test_by_kind_when_filter_by_kind_range_expect_correct_filtering() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);

    // Filter kinds in a specific range
    let filtered: Vec<SyntaxElement> = children
        .by_kind(|kind| kind.0 >= 1 && kind.0 <= 3) // STRING_KIND(1), NUMBER_KIND(2), NAME_KIND(3)
        .collect();

    assert_eq!(filtered.len(), 2); // STRING and NUMBER (NAME is nested inside DICT)
    for element in filtered {
        assert!(element.kind().0 >= 1 && element.kind().0 <= 3);
    }
}

#[test]
fn test_by_kind_when_chaining_operations_expect_correct_behavior() {
    let tree = create_large_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);

    // Chain operations on filtered iterator
    let mut by_kind = children.by_kind(|kind| kind == NUMBER_KIND);

    // Take first 5 elements
    let first_five: Vec<SyntaxElement> = by_kind.by_ref().take(5).collect();
    assert_eq!(first_five.len(), 5);

    // Count remaining
    let remaining_count = by_kind.count();
    assert_eq!(remaining_count, 15);
}

// =============================================================================
// Edge Case Tests - Testing by_kind after partial consumption
// =============================================================================

#[test]
fn test_by_kind_when_called_after_partial_consumption_expect_all_matches_from_start() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let mut children = SyntaxElementChildren::new(dict);

    // Consume first element (should be STRING_KIND)
    let first = children.next().unwrap();
    assert_eq!(first.kind(), STRING_KIND);

    // Now call by_kind looking for NUMBER_KIND - should find all NUMBER elements
    let number_elements: Vec<SyntaxElement> =
        children.by_kind(|kind| kind == NUMBER_KIND).collect();

    // Should find the NUMBER element
    assert_eq!(number_elements.len(), 1);
    assert_eq!(number_elements[0].kind(), NUMBER_KIND);
}

#[test]
fn test_by_kind_when_called_after_consuming_all_expect_empty() {
    let tree = create_token_only_tree();
    let dict = tree.first_child().unwrap();
    let mut children = SyntaxElementChildren::new(dict);

    // Consume all elements
    while children.next().is_some() {}

    // Now call by_kind - should return empty since iterator is exhausted
    let filtered: Vec<SyntaxElement> = children.by_kind(|_| true).collect();
    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_by_kind_coverage_of_else_branch_with_current_match() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let mut children = SyntaxElementChildren::new(dict);

    // Advance to the second element (which is NUMBER_KIND)
    let _first = children.next().unwrap(); // STRING

    // Now by_kind should use the else branch and check if current element matches
    let number_matches: Vec<SyntaxElement> = children.by_kind(|kind| kind == NUMBER_KIND).collect();

    // This tests the case where matcher(element.kind()) returns true in the else branch
    assert_eq!(number_matches.len(), 1);
    assert_eq!(number_matches[0].kind(), NUMBER_KIND);
}

#[test]
fn test_by_kind_coverage_of_else_branch_with_current_no_match() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();
    let mut children = SyntaxElementChildren::new(dict);

    // Advance to the second element (which is NUMBER_KIND)
    let _first = children.next().unwrap(); // STRING

    // Now by_kind should use the else branch and since current doesn't match,
    // it should call element.next_sibling_or_token_by_kind
    let comment_matches: Vec<SyntaxElement> =
        children.by_kind(|kind| kind == COMMENT_KIND).collect();

    // This tests the case where element.next_sibling_or_token_by_kind is called in the else branch
    assert_eq!(comment_matches.len(), 1);
    assert_eq!(comment_matches[0].kind(), COMMENT_KIND);
}

// =============================================================================
// Integration and Edge Case Tests
// =============================================================================

#[test]
fn test_integration_when_children_from_nested_structure_expect_only_direct_children() {
    let tree = create_deeply_nested_tree();
    let children = SyntaxElementChildren::new(tree);
    let collected: Vec<SyntaxElement> = children.collect();

    // Should only get direct children, not nested ones
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].kind(), ARRAY_KIND); // The direct child
}

#[test]
fn test_integration_when_by_kind_from_deeply_nested_expect_correct_filtering() {
    let tree = create_deeply_nested_tree();
    let children = SyntaxElementChildren::new(tree);
    let array_elements: Vec<SyntaxElement> = children.by_kind(|kind| kind == ARRAY_KIND).collect();

    assert_eq!(array_elements.len(), 1);
    assert_eq!(array_elements[0].kind(), ARRAY_KIND);
    assert!(array_elements[0].as_node().is_some());
}

#[test]
fn test_edge_case_when_single_element_by_kind_expect_correct_behavior() {
    let tree = create_deeply_nested_tree();
    let children = SyntaxElementChildren::new(tree);

    // Filter for existing kind
    let array_elements: Vec<SyntaxElement> = children
        .clone()
        .by_kind(|kind| kind == ARRAY_KIND)
        .collect();
    assert_eq!(array_elements.len(), 1);

    // Filter for non-existing kind
    let string_elements: Vec<SyntaxElement> =
        children.by_kind(|kind| kind == STRING_KIND).collect();
    assert_eq!(string_elements.len(), 0);
}

#[test]
fn test_edge_case_when_empty_parent_by_kind_expect_empty() {
    let tree = create_empty_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);
    let filtered: Vec<SyntaxElement> = children.by_kind(|_| true).collect();

    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_performance_when_large_children_set_expect_efficient_iteration() {
    let tree = create_large_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);

    // Test that iteration is efficient for large sets
    let start = std::time::Instant::now();
    let count = children.count();
    let duration = start.elapsed();

    assert_eq!(count, 20);
    assert!(duration.as_millis() < 100); // Should be very fast
}

#[test]
fn test_performance_when_large_children_by_kind_expect_efficient_filtering() {
    let tree = create_large_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);

    // Test filtering performance
    let start = std::time::Instant::now();
    let filtered_count = children.by_kind(|kind| kind == NUMBER_KIND).count();
    let duration = start.elapsed();

    assert_eq!(filtered_count, 20);
    assert!(duration.as_millis() < 100); // Should be very fast
}

#[test]
fn test_memory_safety_when_multiple_iterators_expect_no_conflicts() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();

    // Create multiple iterators from the same parent
    let children1 = SyntaxElementChildren::new(dict.clone());
    let children2 = SyntaxElementChildren::new(dict.clone());
    let children3 = SyntaxElementChildren::new(dict);

    // Collect from all iterators
    let collected1: Vec<SyntaxElement> = children1.collect();
    let collected2: Vec<SyntaxElement> = children2.collect();
    let collected3: Vec<SyntaxElement> = children3.collect();

    // All should have the same results
    assert_eq!(collected1.len(), 4);
    assert_eq!(collected2.len(), 4);
    assert_eq!(collected3.len(), 4);

    for i in 0..4 {
        assert_eq!(collected1[i].kind(), collected2[i].kind());
        assert_eq!(collected2[i].kind(), collected3[i].kind());
        assert_eq!(collected1[i].index(), collected2[i].index());
        assert_eq!(collected2[i].index(), collected3[i].index());
    }
}

// =============================================================================
// Debug and Display Tests
// =============================================================================

#[test]
fn test_debug_format_when_element_children_expect_readable_output() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);

    let debug_str = format!("{:?}", children);

    // Should contain useful information
    assert!(debug_str.contains("SyntaxElementChildren"));
}

// =============================================================================
// Complex Scenario Tests
// =============================================================================

#[test]
fn test_complex_mixed_iteration_when_tokens_and_nodes_expect_correct_handling() {
    let tree = create_mixed_children_tree();
    let dict = tree.first_child().unwrap();
    let children = SyntaxElementChildren::new(dict);

    let elements: Vec<SyntaxElement> = children.collect();

    // Verify the mixed content structure
    assert_eq!(elements.len(), 4);

    // First element: NAME token
    assert_eq!(elements[0].kind(), NAME_KIND);
    assert!(elements[0].as_token().is_some());

    // Second element: NUMBER token
    assert_eq!(elements[1].kind(), NUMBER_KIND);
    assert!(elements[1].as_token().is_some());

    // Third element: ARRAY node
    assert_eq!(elements[2].kind(), ARRAY_KIND);
    assert!(elements[2].as_node().is_some());

    // Fourth element: COMMENT token
    assert_eq!(elements[3].kind(), COMMENT_KIND);
    assert!(elements[3].as_token().is_some());

    // Verify that we can navigate into the ARRAY node
    if let Some(array_node) = elements[2].as_node() {
        let array_children: Vec<SyntaxElement> = array_node.children_with_tokens().collect();
        assert_eq!(array_children.len(), 2);
        assert_eq!(array_children[0].kind(), STRING_KIND);
        assert_eq!(array_children[1].kind(), BOOLEAN_KIND);
    }
}

#[test]
fn test_filtered_iteration_with_mixed_content_expect_correct_results() {
    let tree = create_all_kinds_tree();
    let dict = tree.first_child().unwrap();

    // Test filtering for only tokens
    let children = SyntaxElementChildren::new(dict.clone());
    let tokens: Vec<SyntaxElement> = children
        .by_kind(|kind| matches!(kind, STRING_KIND | NUMBER_KIND | COMMENT_KIND))
        .collect();

    assert_eq!(tokens.len(), 3); // STRING, NUMBER, COMMENT
    for token in tokens {
        assert!(token.as_token().is_some());
    }

    // Test filtering for only nodes
    let children = SyntaxElementChildren::new(dict.clone());
    let nodes: Vec<SyntaxElement> = children
        .by_kind(|kind| matches!(kind, ARRAY_KIND | DICT_KIND))
        .collect();

    assert_eq!(nodes.len(), 2); // ARRAY, DICT
    for node in nodes {
        assert!(node.as_node().is_some());
    }
}
