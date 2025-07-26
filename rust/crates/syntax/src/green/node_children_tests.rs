use crate::{
    NodeOrToken, SyntaxKind,
    green::{element::GreenElement, node::GreenNode, token::GreenToken, trivia::GreenTrivia},
};

// Test constants for different PDF syntax kinds
const OBJECT_KIND: SyntaxKind = SyntaxKind(1);
const STRING_KIND: SyntaxKind = SyntaxKind(2);
const NUMBER_KIND: SyntaxKind = SyntaxKind(3);
const NAME_KIND: SyntaxKind = SyntaxKind(4);
const KW_KIND: SyntaxKind = SyntaxKind(5);

/// Helper function to create test tokens for PDF elements
fn create_token(kind: SyntaxKind, text: &str) -> GreenToken {
    let trivia = GreenTrivia::new([]);
    GreenToken::new(kind, text.as_bytes(), trivia.clone(), trivia)
}

/// Helper function to create test elements (either nodes or tokens)
fn create_element_token(kind: SyntaxKind, text: &str) -> GreenElement {
    NodeOrToken::Token(create_token(kind, text))
}

/// Helper function to create a simple test node with given children
fn create_test_node(kind: SyntaxKind, children: Vec<GreenElement>) -> GreenNode {
    GreenNode::new(kind, children)
}

/// Creates a sample PDF object structure for testing
/// Represents something like: "1 0 obj /Type /Catalog endobj"
fn create_sample_pdf_object() -> GreenNode {
    let elements = vec![
        create_element_token(NUMBER_KIND, "1"),
        create_element_token(NUMBER_KIND, "0"),
        create_element_token(KW_KIND, "obj"),
        create_element_token(NAME_KIND, "/Type"),
        create_element_token(NAME_KIND, "/Catalog"),
        create_element_token(KW_KIND, "endobj"),
    ];
    create_test_node(OBJECT_KIND, elements)
}

#[test]
fn test_len_when_empty_node_expect_zero() {
    let node = create_test_node(OBJECT_KIND, vec![]);
    let children = node.children();

    assert_eq!(children.len(), 0);
}

#[test]
fn test_len_when_node_has_children_expect_correct_count() {
    let node = create_sample_pdf_object();
    let children = node.children();

    assert_eq!(children.len(), 6);
}

#[test]
fn test_iter_when_empty_node_expect_no_elements() {
    let node = create_test_node(OBJECT_KIND, vec![]);
    let children = node.children();

    let collected: Vec<_> = children.collect();
    assert!(collected.is_empty());
}

#[test]
fn test_iter_when_node_has_children_expect_all_elements() {
    let node = create_sample_pdf_object();
    let children = node.children();

    let collected: Vec<_> = children.collect();
    assert_eq!(collected.len(), 6);

    // Verify the first few elements match expected kinds
    assert_eq!(collected[0].kind(), NUMBER_KIND);
    assert_eq!(collected[1].kind(), NUMBER_KIND);
    assert_eq!(collected[2].kind(), KW_KIND);
}

#[test]
fn test_next_when_iterating_expect_correct_sequence() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    // Test forward iteration
    let first = children.next().unwrap();
    assert_eq!(first.kind(), NUMBER_KIND);

    let second = children.next().unwrap();
    assert_eq!(second.kind(), NUMBER_KIND);

    let third = children.next().unwrap();
    assert_eq!(third.kind(), KW_KIND);

    // Continue until exhausted
    assert!(children.next().is_some()); // /Type
    assert!(children.next().is_some()); // /Catalog
    assert!(children.next().is_some()); // endobj
    assert!(children.next().is_none());
}

#[test]
fn test_size_hint_when_iterating_expect_accurate_bounds() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    let (lower, upper) = children.size_hint();
    assert_eq!(lower, 6);
    assert_eq!(upper, Some(6));

    // After consuming one element
    children.next();
    let (lower, upper) = children.size_hint();
    assert_eq!(lower, 5);
    assert_eq!(upper, Some(5));
}

#[test]
fn test_count_when_consuming_iterator_expect_correct_total() {
    let node = create_sample_pdf_object();
    let children = node.children();

    assert_eq!(children.count(), 6);
}

#[test]
fn test_nth_when_accessing_by_index_expect_correct_element() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    // Access third element (0-indexed)
    let third = children.nth(2).unwrap();
    assert_eq!(third.kind(), KW_KIND);

    // Iterator should now be at position 3
    let fourth = children.next().unwrap();
    assert_eq!(fourth.kind(), NAME_KIND); // Should be /Type
}

#[test]
fn test_nth_when_index_out_of_bounds_expect_none() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    assert!(children.nth(10).is_none());
}

#[test]
fn test_last_when_iterating_expect_final_element() {
    let node = create_sample_pdf_object();
    let children = node.children();

    let last = children.last().unwrap();
    assert_eq!(last.kind(), KW_KIND); // Should be "endobj"
}

#[test]
fn test_last_when_empty_expect_none() {
    let node = create_test_node(OBJECT_KIND, vec![]);
    let children = node.children();

    assert!(children.last().is_none());
}

#[test]
fn test_fold_when_accumulating_expect_correct_result() {
    let node = create_sample_pdf_object();
    let children = node.children();

    // Count elements by folding
    let count = children.fold(0, |acc, _| acc + 1);
    assert_eq!(count, 6);
}

#[test]
fn test_next_back_when_iterating_backwards_expect_reverse_order() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    // Test backward iteration
    let last = children.next_back().unwrap();
    assert_eq!(last.kind(), KW_KIND); // endobj

    let second_last = children.next_back().unwrap();
    assert_eq!(second_last.kind(), NAME_KIND); // /Catalog

    // Verify we can mix forward and backward iteration
    let first = children.next().unwrap();
    assert_eq!(first.kind(), NUMBER_KIND); // "1"
}

#[test]
fn test_nth_back_when_accessing_from_end_expect_correct_element() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    // Access third from end (0-indexed from back)
    let third_from_end = children.nth_back(2).unwrap();
    assert_eq!(third_from_end.kind(), NAME_KIND); // Should be /Type

    // Iterator should now be positioned accordingly
    let next_from_back = children.next_back().unwrap();
    assert_eq!(next_from_back.kind(), KW_KIND); // Should be "obj"
}

#[test]
fn test_nth_back_when_index_out_of_bounds_expect_none() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    assert!(children.nth_back(10).is_none());
}

#[test]
fn test_rfold_when_accumulating_backwards_expect_correct_result() {
    let node = create_sample_pdf_object();
    let children = node.children();

    // Collect kinds in reverse order using rfold
    let mut kinds = Vec::new();
    children.rfold((), |_, element| {
        kinds.push(element.kind());
    });

    assert_eq!(kinds.len(), 6);
    // First collected should be the last element (endobj)
    assert_eq!(kinds[0], KW_KIND);
}

#[test]
fn test_bidirectional_iteration_when_mixed_access_expect_correct_behavior() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    // Take one from front and one from back
    let first = children.next().unwrap();
    let last = children.next_back().unwrap();

    assert_eq!(first.kind(), NUMBER_KIND); // "1"
    assert_eq!(last.kind(), KW_KIND); // "endobj"

    // Remaining elements should be 4
    assert_eq!(children.count(), 4);
}

#[test]
fn test_clone_when_duplicating_iterator_expect_independent_copies() {
    let node = create_sample_pdf_object();
    let children = node.children();
    let mut cloned = children.clone();

    // Advance the cloned iterator
    cloned.next();
    cloned.next();

    // Original should still be at the beginning
    assert_eq!(children.count(), 6);
    assert_eq!(cloned.count(), 4);
}

#[test]
fn test_exact_size_iterator_when_checking_remaining_expect_accurate_count() {
    let node = create_sample_pdf_object();
    let mut children = node.children();

    // ExactSizeIterator should provide accurate remaining count
    assert_eq!(children.len(), 6);

    children.next();
    assert_eq!(children.len(), 5);

    children.next_back();
    assert_eq!(children.len(), 4);
}

#[test]
fn test_fused_iterator_when_exhausted_expect_none_forever() {
    let node = create_test_node(OBJECT_KIND, vec![create_element_token(NUMBER_KIND, "42")]);
    let mut children = node.children();

    // Consume the only element
    assert!(children.next().is_some());

    // Should return None consistently after exhaustion (FusedIterator property)
    assert!(children.next().is_none());
    assert!(children.next().is_none());
    assert!(children.next().is_none());
}

#[test]
fn test_mixed_element_types_when_node_contains_both_tokens_and_nodes_expect_correct_iteration() {
    // Create a nested structure with both tokens and nodes
    let inner_node = create_test_node(
        NAME_KIND,
        vec![create_element_token(STRING_KIND, "(Hello)")],
    );

    let elements = vec![
        create_element_token(NUMBER_KIND, "1"),
        NodeOrToken::Node(inner_node),
        create_element_token(NAME_KIND, "endobj"),
    ];

    let node = create_test_node(OBJECT_KIND, elements);
    let children = node.children();

    let collected: Vec<_> = children.collect();
    assert_eq!(collected.len(), 3);

    // First should be a token
    assert_eq!(collected[0].kind(), NUMBER_KIND);
    assert!(collected[0].as_token().is_some());

    // Second should be a node
    assert_eq!(collected[1].kind(), NAME_KIND);
    assert!(collected[1].as_node().is_some());

    // Third should be a token
    assert_eq!(collected[2].kind(), NAME_KIND);
    assert!(collected[2].as_token().is_some());
}
