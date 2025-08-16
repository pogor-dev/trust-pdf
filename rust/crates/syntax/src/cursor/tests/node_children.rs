use crate::{
    SyntaxKind,
    cursor::{node::SyntaxNode, node_children::SyntaxNodeChildren},
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

fn create_single_child_tree() -> SyntaxNode {
    let child_token = create_green_token(STRING_KIND, "(Hello)");
    let child_node = create_green_node(DICT_KIND, vec![child_token.into()]);
    let parent_node = create_green_node(OBJ_KIND, vec![child_node.into()]);
    SyntaxNode::new_root(parent_node)
}

fn create_multiple_children_tree() -> SyntaxNode {
    // Create multiple child nodes with different kinds
    let child1_token = create_green_token(NAME_KIND, "/Type");
    let child1 = create_green_node(DICT_KIND, vec![child1_token.into()]);

    let child2_token = create_green_token(NUMBER_KIND, "42");
    let child2 = create_green_node(ARRAY_KIND, vec![child2_token.into()]);

    let child3_token = create_green_token(STRING_KIND, "(text)");
    let child3 = create_green_node(DICT_KIND, vec![child3_token.into()]);

    let parent_node =
        create_green_node(OBJ_KIND, vec![child1.into(), child2.into(), child3.into()]);
    SyntaxNode::new_root(parent_node)
}

fn create_mixed_kind_children_tree() -> SyntaxNode {
    // Create children with mixed kinds for testing filtering
    let dict1_token = create_green_token(NAME_KIND, "/Key1");
    let dict1 = create_green_node(DICT_KIND, vec![dict1_token.into()]);

    let array1_token = create_green_token(NUMBER_KIND, "123");
    let array1 = create_green_node(ARRAY_KIND, vec![array1_token.into()]);

    let dict2_token = create_green_token(NAME_KIND, "/Key2");
    let dict2 = create_green_node(DICT_KIND, vec![dict2_token.into()]);

    let array2_token = create_green_token(BOOLEAN_KIND, "true");
    let array2 = create_green_node(ARRAY_KIND, vec![array2_token.into()]);

    let comment_token = create_green_token(COMMENT_KIND, "% comment");
    let comment = create_green_node(COMMENT_KIND, vec![comment_token.into()]);

    let parent_node = create_green_node(
        OBJ_KIND,
        vec![
            dict1.into(),
            array1.into(),
            dict2.into(),
            array2.into(),
            comment.into(),
        ],
    );
    SyntaxNode::new_root(parent_node)
}

fn create_empty_parent_tree() -> SyntaxNode {
    let empty_parent = create_green_node(OBJ_KIND, vec![]);
    SyntaxNode::new_root(empty_parent)
}

fn create_deeply_nested_tree() -> SyntaxNode {
    // Create nested structure for complex traversal testing
    let innermost_token = create_green_token(STRING_KIND, "(deep)");
    let innermost = create_green_node(DICT_KIND, vec![innermost_token.into()]);

    let middle_token = create_green_token(NAME_KIND, "/Middle");
    let middle = create_green_node(ARRAY_KIND, vec![middle_token.into(), innermost.into()]);

    let outer_token = create_green_token(NUMBER_KIND, "999");
    let outer = create_green_node(DICT_KIND, vec![outer_token.into(), middle.into()]);

    let root = create_green_node(OBJ_KIND, vec![outer.into()]);
    SyntaxNode::new_root(root)
}

fn create_large_children_tree() -> SyntaxNode {
    let mut children = Vec::new();
    for i in 0..10 {
        let token = create_green_token(NUMBER_KIND, &format!("{}", i));
        let child = create_green_node(DICT_KIND, vec![token.into()]);
        children.push(child.into());
    }

    let parent = create_green_node(OBJ_KIND, children);
    SyntaxNode::new_root(parent)
}

// =============================================================================
// SyntaxNodeChildren Basic Tests
// =============================================================================

#[test]
fn test_new_when_creating_syntax_node_children_expect_correct_initial_state() {
    let tree = create_single_child_tree();
    let children = SyntaxNodeChildren::new(tree.clone());

    // Debug format should be readable
    let debug_str = format!("{:?}", children);
    assert!(debug_str.contains("SyntaxNodeChildren"));

    // Clone should work
    let _cloned = children.clone();
}

#[test]
fn test_iterator_when_single_child_expect_one_item() {
    let tree = create_single_child_tree();
    let children = SyntaxNodeChildren::new(tree);
    let collected: Vec<SyntaxNode> = children.collect();

    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].kind(), DICT_KIND);
}

#[test]
fn test_iterator_when_multiple_children_expect_all_items_in_order() {
    let tree = create_multiple_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let collected: Vec<SyntaxNode> = children.collect();

    assert_eq!(collected.len(), 3);
    assert_eq!(collected[0].kind(), DICT_KIND);
    assert_eq!(collected[1].kind(), ARRAY_KIND);
    assert_eq!(collected[2].kind(), DICT_KIND);
}

#[test]
fn test_iterator_when_empty_parent_expect_no_items() {
    let tree = create_empty_parent_tree();
    let children = SyntaxNodeChildren::new(tree);
    let collected: Vec<SyntaxNode> = children.collect();

    assert_eq!(collected.len(), 0);
}

#[test]
fn test_iterator_when_consumed_partially_expect_remaining_items() {
    let tree = create_multiple_children_tree();
    let mut children = SyntaxNodeChildren::new(tree);

    // Consume first item
    let first = children.next().unwrap();
    assert_eq!(first.kind(), DICT_KIND);

    // Collect remaining
    let remaining: Vec<SyntaxNode> = children.collect();
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining[0].kind(), ARRAY_KIND);
    assert_eq!(remaining[1].kind(), DICT_KIND);
}

#[test]
fn test_iterator_when_multiple_calls_to_next_expect_correct_sequence() {
    let tree = create_multiple_children_tree();
    let mut children = SyntaxNodeChildren::new(tree);

    let first = children.next();
    assert!(first.is_some());
    assert_eq!(first.unwrap().kind(), DICT_KIND);

    let second = children.next();
    assert!(second.is_some());
    assert_eq!(second.unwrap().kind(), ARRAY_KIND);

    let third = children.next();
    assert!(third.is_some());
    assert_eq!(third.unwrap().kind(), DICT_KIND);

    let fourth = children.next();
    assert!(fourth.is_none());

    // Further calls should continue to return None
    assert!(children.next().is_none());
}

#[test]
fn test_iterator_when_large_number_of_children_expect_all_visited() {
    let tree = create_large_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let collected: Vec<SyntaxNode> = children.collect();

    assert_eq!(collected.len(), 10);
    for child in collected {
        assert_eq!(child.kind(), DICT_KIND);
    }
}

// =============================================================================
// SyntaxNodeChildren Clone and State Tests
// =============================================================================

#[test]
fn test_clone_when_iterator_not_started_expect_independent_state() {
    let tree = create_multiple_children_tree();
    let children1 = SyntaxNodeChildren::new(tree);
    let children2 = children1.clone();

    let collected1: Vec<SyntaxNode> = children1.collect();
    let collected2: Vec<SyntaxNode> = children2.collect();

    assert_eq!(collected1.len(), collected2.len());
    for (c1, c2) in collected1.iter().zip(collected2.iter()) {
        assert_eq!(c1.kind(), c2.kind());
    }
}

#[test]
fn test_clone_when_iterator_partially_consumed_expect_independent_state() {
    let tree = create_multiple_children_tree();
    let mut children1 = SyntaxNodeChildren::new(tree);

    // Consume one item
    let _first = children1.next().unwrap();

    // Clone after partial consumption
    let children2 = children1.clone();

    let remaining1: Vec<SyntaxNode> = children1.collect();
    let remaining2: Vec<SyntaxNode> = children2.collect();

    assert_eq!(remaining1.len(), remaining2.len());
    assert_eq!(remaining1.len(), 2); // Should have 2 remaining

    for (c1, c2) in remaining1.iter().zip(remaining2.iter()) {
        assert_eq!(c1.kind(), c2.kind());
    }
}

// =============================================================================
// SyntaxNodeChildrenByKind Basic Tests
// =============================================================================

#[test]
fn test_by_kind_when_filtering_dict_kind_expect_only_dict_children() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let dict_children: Vec<SyntaxNode> = children.by_kind(|kind| kind == DICT_KIND).collect();

    assert_eq!(dict_children.len(), 2);
    for child in dict_children {
        assert_eq!(child.kind(), DICT_KIND);
    }
}

#[test]
fn test_by_kind_when_filtering_array_kind_expect_only_array_children() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let array_children: Vec<SyntaxNode> = children.by_kind(|kind| kind == ARRAY_KIND).collect();

    assert_eq!(array_children.len(), 2);
    for child in array_children {
        assert_eq!(child.kind(), ARRAY_KIND);
    }
}

#[test]
fn test_by_kind_when_filtering_non_existent_kind_expect_empty() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let obj_children: Vec<SyntaxNode> = children.by_kind(|kind| kind == OBJ_KIND).collect();

    assert_eq!(obj_children.len(), 0);
}

#[test]
fn test_by_kind_when_filtering_all_kinds_expect_all_children() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let all_children: Vec<SyntaxNode> = children.by_kind(|_| true).collect();

    assert_eq!(all_children.len(), 5); // All children should be included
}

#[test]
fn test_by_kind_when_filtering_no_kinds_expect_empty() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let no_children: Vec<SyntaxNode> = children.by_kind(|_| false).collect();

    assert_eq!(no_children.len(), 0);
}

#[test]
fn test_by_kind_when_complex_filter_expect_correct_subset() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);

    // Filter for DICT_KIND or ARRAY_KIND
    let filtered: Vec<SyntaxNode> = children
        .by_kind(|kind| kind == DICT_KIND || kind == ARRAY_KIND)
        .collect();

    assert_eq!(filtered.len(), 4); // 2 DICT + 2 ARRAY
    for child in filtered {
        assert!(child.kind() == DICT_KIND || child.kind() == ARRAY_KIND);
    }
}

// =============================================================================
// SyntaxNodeChildrenByKind State Tests
// =============================================================================

#[test]
fn test_by_kind_clone_when_creating_expect_independent_state() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let by_kind1 = children.by_kind(|kind| kind == DICT_KIND);
    let by_kind2 = by_kind1.clone();

    let collected1: Vec<SyntaxNode> = by_kind1.collect();
    let collected2: Vec<SyntaxNode> = by_kind2.collect();

    assert_eq!(collected1.len(), collected2.len());
    assert_eq!(collected1.len(), 2);
}

#[test]
fn test_by_kind_when_iterator_partially_consumed_expect_remaining_items() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let mut by_kind = children.by_kind(|kind| kind == DICT_KIND);

    // Consume first item
    let first = by_kind.next().unwrap();
    assert_eq!(first.kind(), DICT_KIND);

    // Collect remaining
    let remaining: Vec<SyntaxNode> = by_kind.collect();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].kind(), DICT_KIND);
}

#[test]
fn test_by_kind_when_multiple_next_calls_expect_correct_sequence() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);
    let mut by_kind = children.by_kind(|kind| kind == ARRAY_KIND);

    let first = by_kind.next();
    assert!(first.is_some());
    assert_eq!(first.unwrap().kind(), ARRAY_KIND);

    let second = by_kind.next();
    assert!(second.is_some());
    assert_eq!(second.unwrap().kind(), ARRAY_KIND);

    let third = by_kind.next();
    assert!(third.is_none());

    // Further calls should continue to return None
    assert!(by_kind.next().is_none());
}

// =============================================================================
// Advanced Filtering Tests
// =============================================================================

#[test]
fn test_by_kind_when_filter_by_kind_value_expect_correct_filtering() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);

    // Test filtering by specific kind values
    let specific_kinds = [DICT_KIND, COMMENT_KIND];
    let filtered: Vec<SyntaxNode> = children
        .by_kind(|kind| specific_kinds.contains(&kind))
        .collect();

    assert_eq!(filtered.len(), 3); // 2 DICT + 1 COMMENT
    for child in filtered {
        assert!(specific_kinds.contains(&child.kind()));
    }
}

#[test]
fn test_by_kind_when_filter_by_kind_range_expect_correct_filtering() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);

    // Filter kinds in a specific range
    let filtered: Vec<SyntaxNode> = children
        .by_kind(|kind| kind.0 >= 4 && kind.0 <= 5) // DICT_KIND(4) and ARRAY_KIND(5)
        .collect();

    assert_eq!(filtered.len(), 4); // 2 DICT + 2 ARRAY
    for child in filtered {
        assert!(child.kind() == DICT_KIND || child.kind() == ARRAY_KIND);
    }
}

#[test]
fn test_by_kind_when_chaining_operations_expect_correct_behavior() {
    let tree = create_mixed_kind_children_tree();
    let children = SyntaxNodeChildren::new(tree);

    // Chain operations on filtered iterator
    let mut by_kind = children.by_kind(|kind| kind == DICT_KIND);
    let first_dict = by_kind.next().unwrap();
    assert_eq!(first_dict.kind(), DICT_KIND);

    // Count remaining
    let remaining_count = by_kind.count();
    assert_eq!(remaining_count, 1);
}

// =============================================================================
// Integration and Edge Case Tests
// =============================================================================

#[test]
fn test_integration_when_children_from_nested_structure_expect_only_direct_children() {
    let tree = create_deeply_nested_tree();
    let children = SyntaxNodeChildren::new(tree);
    let collected: Vec<SyntaxNode> = children.collect();

    // Should only get direct children, not nested ones
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].kind(), DICT_KIND); // The outer DICT
}

#[test]
fn test_integration_when_by_kind_from_deeply_nested_expect_correct_filtering() {
    let tree = create_deeply_nested_tree();
    let children = SyntaxNodeChildren::new(tree);
    let dict_children: Vec<SyntaxNode> = children.by_kind(|kind| kind == DICT_KIND).collect();

    assert_eq!(dict_children.len(), 1);
    assert_eq!(dict_children[0].kind(), DICT_KIND);
}

#[test]
fn test_edge_case_when_single_child_by_kind_expect_correct_behavior() {
    let tree = create_single_child_tree();
    let children = SyntaxNodeChildren::new(tree);

    // Filter for existing kind
    let dict_children: Vec<SyntaxNode> =
        children.clone().by_kind(|kind| kind == DICT_KIND).collect();
    assert_eq!(dict_children.len(), 1);

    // Filter for non-existing kind
    let array_children: Vec<SyntaxNode> = children.by_kind(|kind| kind == ARRAY_KIND).collect();
    assert_eq!(array_children.len(), 0);
}

#[test]
fn test_edge_case_when_empty_parent_by_kind_expect_empty() {
    let tree = create_empty_parent_tree();
    let children = SyntaxNodeChildren::new(tree);
    let filtered: Vec<SyntaxNode> = children.by_kind(|_| true).collect();

    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_performance_when_large_children_set_expect_efficient_iteration() {
    let tree = create_large_children_tree();
    let children = SyntaxNodeChildren::new(tree);

    // Test that iteration is efficient for large sets
    let start = std::time::Instant::now();
    let count = children.count();
    let duration = start.elapsed();

    assert_eq!(count, 10);
    assert!(duration.as_millis() < 100); // Should be very fast
}

#[test]
fn test_performance_when_large_children_by_kind_expect_efficient_filtering() {
    let tree = create_large_children_tree();
    let children = SyntaxNodeChildren::new(tree);

    // Test filtering performance
    let start = std::time::Instant::now();
    let filtered_count = children.by_kind(|kind| kind == DICT_KIND).count();
    let duration = start.elapsed();

    assert_eq!(filtered_count, 10);
    assert!(duration.as_millis() < 100); // Should be very fast
}

#[test]
fn test_memory_safety_when_multiple_iterators_expect_no_conflicts() {
    let tree = create_multiple_children_tree();

    // Create multiple iterators from the same tree
    let children1 = SyntaxNodeChildren::new(tree.clone());
    let children2 = SyntaxNodeChildren::new(tree.clone());
    let children3 = SyntaxNodeChildren::new(tree);

    // Collect from all iterators
    let collected1: Vec<SyntaxNode> = children1.collect();
    let collected2: Vec<SyntaxNode> = children2.collect();
    let collected3: Vec<SyntaxNode> = children3.collect();

    // All should have the same results
    assert_eq!(collected1.len(), 3);
    assert_eq!(collected2.len(), 3);
    assert_eq!(collected3.len(), 3);

    for i in 0..3 {
        assert_eq!(collected1[i].kind(), collected2[i].kind());
        assert_eq!(collected2[i].kind(), collected3[i].kind());
    }
}

// =============================================================================
// Edge Case Tests - Testing by_kind after partial consumption
// =============================================================================

#[test]
fn test_by_kind_when_called_after_partial_consumption_expect_all_matches_from_start() {
    let tree = create_mixed_kind_children_tree();
    let mut children = SyntaxNodeChildren::new(tree);

    // Consume first child (should be DICT_KIND)
    let first = children.next().unwrap();
    assert_eq!(first.kind(), DICT_KIND);

    // Now call by_kind looking for DICT_KIND - should find ALL DICT children from start
    let dict_children: Vec<SyntaxNode> = children.by_kind(|kind| kind == DICT_KIND).collect();

    // Should find both DICT_KIND children in the tree
    assert_eq!(dict_children.len(), 2);
    for child in dict_children {
        assert_eq!(child.kind(), DICT_KIND);
    }
}

#[test]
fn test_by_kind_when_called_after_partial_consumption_and_current_no_match_expect_search_siblings()
{
    let tree = create_mixed_kind_children_tree();
    let mut children = SyntaxNodeChildren::new(tree);

    // Consume first child (DICT_KIND)
    let _first = children.next().unwrap();

    // Now call by_kind looking for ARRAY_KIND - should find all ARRAY children
    let array_children: Vec<SyntaxNode> = children.by_kind(|kind| kind == ARRAY_KIND).collect();

    // Should find both ARRAY_KIND children that exist in the tree
    assert_eq!(array_children.len(), 2);
    for child in array_children {
        assert_eq!(child.kind(), ARRAY_KIND);
    }
}

#[test]
fn test_by_kind_when_called_after_consuming_all_expect_empty() {
    let tree = create_multiple_children_tree();
    let mut children = SyntaxNodeChildren::new(tree);

    // Consume all children
    while children.next().is_some() {}

    // Now call by_kind - the iterator state should affect what gets returned
    let filtered: Vec<SyntaxNode> = children.by_kind(|_| true).collect();

    // This should be empty since the iterator is exhausted
    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_by_kind_coverage_of_else_branch_with_current_match() {
    let tree = create_mixed_kind_children_tree();
    let mut children = SyntaxNodeChildren::new(tree);

    // Advance to the second child (which is ARRAY_KIND)
    let _first = children.next().unwrap(); // DICT

    // Now by_kind should use the else branch and check if current node matches
    let array_matches: Vec<SyntaxNode> = children.by_kind(|kind| kind == ARRAY_KIND).collect();

    // This tests the case where matcher(node.kind()) returns true in the else branch
    assert_eq!(array_matches.len(), 2); // Should find both ARRAY children
}

#[test]
fn test_by_kind_coverage_of_else_branch_with_current_no_match() {
    let tree = create_mixed_kind_children_tree();
    let mut children = SyntaxNodeChildren::new(tree);

    // Advance to the second child (which is ARRAY_KIND)
    let _first = children.next().unwrap(); // DICT

    // Now by_kind should use the else branch and since current doesn't match,
    // it should call node.next_sibling_by_kind
    let comment_matches: Vec<SyntaxNode> = children.by_kind(|kind| kind == COMMENT_KIND).collect();

    // This tests the case where node.next_sibling_by_kind is called in the else branch
    assert_eq!(comment_matches.len(), 1); // Should find the one COMMENT child
}
