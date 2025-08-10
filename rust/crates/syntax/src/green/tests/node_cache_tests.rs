use crate::{
    SyntaxKind,
    green::{element::GreenElement, node_cache::NodeCache, token::GreenToken, trivia::GreenTrivia},
};

// Test constants for different syntax kinds
const TOKEN_KIND: SyntaxKind = SyntaxKind(1);
const NODE_KIND: SyntaxKind = SyntaxKind(2);
const STRING_KIND: SyntaxKind = SyntaxKind(3);
const NUMBER_KIND: SyntaxKind = SyntaxKind(4);
const LIST_KIND: SyntaxKind = SyntaxKind(5);

/// Helper function to create test tokens
fn create_test_token(kind: SyntaxKind, text: &str) -> GreenToken {
    let leading = GreenTrivia::new([]);
    let trailing = GreenTrivia::new([]);
    GreenToken::new(kind, text.as_bytes(), leading, trailing)
}

// =============================================================================
// NodeCache Core Functionality Tests
// =============================================================================

#[test]
fn test_new_cache_when_creating_expect_empty_state() {
    let cache = NodeCache::default();

    // Cache should be created successfully
    // We can't directly inspect internal state, but creation should not panic
    drop(cache);
}

#[test]
fn test_token_when_caching_single_token_expect_successful_storage() {
    let mut cache = NodeCache::default();

    let (hash1, cached_token1) = cache.token(
        TOKEN_KIND,
        b"test",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    // Hash should be non-zero for successful caching
    assert_ne!(hash1, 0);

    // Token content should match
    assert_eq!(cached_token1.kind(), TOKEN_KIND);
    assert_eq!(cached_token1.text(), b"test");
}

#[test]
fn test_token_when_caching_identical_tokens_expect_deduplication() {
    let mut cache = NodeCache::default();

    let (hash1, token1) = cache.token(
        TOKEN_KIND,
        b"duplicate",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    let (hash2, token2) = cache.token(
        TOKEN_KIND,
        b"duplicate",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    // Both should have same hash
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, 0);

    // Tokens should be identical (same memory)
    assert_eq!(token1.kind(), token2.kind());
    assert_eq!(token1.text(), token2.text());

    // Verify they represent the same cached instance
    let ptr1 = &*token1 as *const _ as *const u8;
    let ptr2 = &*token2 as *const _ as *const u8;
    assert_eq!(ptr1, ptr2);
}

#[test]
fn test_token_when_caching_different_tokens_expect_different_hashes() {
    let mut cache = NodeCache::default();

    let (hash1, token1) = cache.token(
        TOKEN_KIND,
        b"first",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    let (hash2, token2) = cache.token(
        TOKEN_KIND,
        b"second",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    // Different content should result in different hashes
    assert_ne!(hash1, hash2);
    assert_ne!(hash1, 0);
    assert_ne!(hash2, 0);

    // Tokens should have different content
    assert_ne!(token1.text(), token2.text());
}

#[test]
fn test_token_when_different_kinds_same_text_expect_different_hashes() {
    let mut cache = NodeCache::default();

    let (hash1, token1) = cache.token(
        STRING_KIND,
        b"123",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    let (hash2, token2) = cache.token(
        NUMBER_KIND,
        b"123",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    // Different kinds should result in different hashes even with same text
    assert_ne!(hash1, hash2);
    assert_ne!(hash1, 0);
    assert_ne!(hash2, 0);

    // Tokens should have different kinds
    assert_ne!(token1.kind(), token2.kind());
    assert_eq!(token1.text(), token2.text());
}

// =============================================================================
// Token Caching with Trivia Tests
// =============================================================================

#[test]
fn test_token_when_caching_with_leading_trivia_expect_trivia_affects_hash() {
    use crate::green::trivia::GreenTriviaChild;

    let mut cache = NodeCache::default();

    let trivia1 = GreenTrivia::new(vec![GreenTriviaChild::new(SyntaxKind(10), b"  ")]);
    let trivia2 = GreenTrivia::new(vec![GreenTriviaChild::new(SyntaxKind(11), b"    ")]);

    let (hash1, token1) = cache.token(TOKEN_KIND, b"test", trivia1, GreenTrivia::new([]));

    let (hash2, token2) = cache.token(TOKEN_KIND, b"test", trivia2, GreenTrivia::new([]));

    // Different leading trivia should result in different hashes
    assert_ne!(hash1, hash2);
    assert_ne!(hash1, 0);
    assert_ne!(hash2, 0);

    // Same text content but different trivia
    assert_eq!(token1.text(), token2.text());
    assert_ne!(
        token1.leading_trivia().text(),
        token2.leading_trivia().text()
    );
}

#[test]
fn test_token_when_caching_with_trailing_trivia_expect_trivia_affects_hash() {
    use crate::green::trivia::GreenTriviaChild;

    let mut cache = NodeCache::default();

    let trivia1 = GreenTrivia::new(vec![GreenTriviaChild::new(SyntaxKind(12), b"\n")]);
    let trivia2 = GreenTrivia::new(vec![GreenTriviaChild::new(SyntaxKind(13), b" ")]);

    let (hash1, token1) = cache.token(TOKEN_KIND, b"test", GreenTrivia::new([]), trivia1);

    let (hash2, token2) = cache.token(TOKEN_KIND, b"test", GreenTrivia::new([]), trivia2);

    // Different trailing trivia should result in different hashes
    assert_ne!(hash1, hash2);
    assert_ne!(hash1, 0);
    assert_ne!(hash2, 0);

    // Same text content but different trivia
    assert_eq!(token1.text(), token2.text());
    assert_ne!(
        token1.trailing_trivia().text(),
        token2.trailing_trivia().text()
    );
}

#[test]
fn test_token_when_caching_identical_with_trivia_expect_deduplication() {
    use crate::green::trivia::GreenTriviaChild;

    let mut cache = NodeCache::default();

    let leading = GreenTrivia::new(vec![GreenTriviaChild::new(SyntaxKind(14), b"  ")]);
    let trailing = GreenTrivia::new(vec![GreenTriviaChild::new(SyntaxKind(15), b"\n")]);

    let (hash1, token1) = cache.token(TOKEN_KIND, b"test", leading.clone(), trailing.clone());

    let (hash2, token2) = cache.token(TOKEN_KIND, b"test", leading, trailing);

    // Identical tokens with trivia should be deduplicated
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, 0);

    // Should be same cached instance
    let ptr1 = &*token1 as *const _ as *const u8;
    let ptr2 = &*token2 as *const _ as *const u8;
    assert_eq!(ptr1, ptr2);
}

// =============================================================================
// Node Caching Tests
// =============================================================================

#[test]
fn test_node_when_caching_single_node_expect_successful_storage() {
    let mut cache = NodeCache::default();

    // Create a simple node with token children
    let token1 = GreenElement::Token(create_test_token(TOKEN_KIND, "child1"));
    let token2 = GreenElement::Token(create_test_token(TOKEN_KIND, "child2"));

    let mut children = vec![(1u64, token1), (2u64, token2)];

    let (hash, node) = cache.node(NODE_KIND, &mut children, 0);

    // Hash should be non-zero for successful caching
    assert_ne!(hash, 0);

    // Node should have correct properties
    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 2);
}

#[test]
fn test_node_when_caching_identical_nodes_expect_deduplication() {
    let mut cache = NodeCache::default();

    // Create identical children
    let token1 = GreenElement::Token(create_test_token(TOKEN_KIND, "same"));

    let mut children1 = vec![(1u64, token1.clone())];
    let mut children2 = vec![(1u64, token1)];

    let (hash1, node1) = cache.node(NODE_KIND, &mut children1, 0);
    let (hash2, node2) = cache.node(NODE_KIND, &mut children2, 0);

    // Identical nodes should have same hash
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, 0);

    // Should be same cached instance
    let ptr1 = &*node1 as *const _ as *const u8;
    let ptr2 = &*node2 as *const _ as *const u8;
    assert_eq!(ptr1, ptr2);
}

#[test]
fn test_node_when_caching_different_nodes_expect_different_hashes() {
    let mut cache = NodeCache::default();

    let token1 = GreenElement::Token(create_test_token(TOKEN_KIND, "first"));
    let token2 = GreenElement::Token(create_test_token(TOKEN_KIND, "second"));

    let mut children1 = vec![(1u64, token1)];
    let mut children2 = vec![(2u64, token2)];

    let (hash1, node1) = cache.node(NODE_KIND, &mut children1, 0);
    let (hash2, node2) = cache.node(NODE_KIND, &mut children2, 0);

    // Different nodes should have different hashes
    assert_ne!(hash1, hash2);
    assert_ne!(hash1, 0);
    assert_ne!(hash2, 0);

    // Nodes should have same structure but different content
    assert_eq!(node1.kind(), node2.kind());
    assert_eq!(node1.children().count(), node2.children().count());
}

#[test]
fn test_node_when_different_kinds_same_children_expect_different_hashes() {
    let mut cache = NodeCache::default();

    let token = GreenElement::Token(create_test_token(TOKEN_KIND, "same"));

    let mut children1 = vec![(1u64, token.clone())];
    let mut children2 = vec![(1u64, token)];

    let (hash1, node1) = cache.node(NODE_KIND, &mut children1, 0);
    let (hash2, node2) = cache.node(LIST_KIND, &mut children2, 0);

    // Different kinds should result in different hashes
    assert_ne!(hash1, hash2);
    assert_ne!(hash1, 0);
    assert_ne!(hash2, 0);

    // Different kinds
    assert_ne!(node1.kind(), node2.kind());
    assert_eq!(node1.children().count(), node2.children().count());
}

#[test]
fn test_node_when_large_node_expect_no_caching() {
    let mut cache = NodeCache::default();

    // Create a node with more than 3 children (should not be cached)
    let mut children = Vec::new();
    for i in 0..5 {
        let token = GreenElement::Token(create_test_token(TOKEN_KIND, &format!("child{}", i)));
        children.push((i as u64 + 1, token));
    }

    let (hash, node) = cache.node(NODE_KIND, &mut children, 0);

    // Large nodes should not be cached (hash should be 0)
    assert_eq!(hash, 0);

    // Node should still be created correctly
    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 5);
}

#[test]
fn test_node_when_partial_children_expect_correct_subset() {
    let mut cache = NodeCache::default();

    // Create children but only use a subset
    let token1 = GreenElement::Token(create_test_token(TOKEN_KIND, "first"));
    let token2 = GreenElement::Token(create_test_token(TOKEN_KIND, "second"));
    let token3 = GreenElement::Token(create_test_token(TOKEN_KIND, "third"));

    let mut children = vec![(1u64, token1), (2u64, token2), (3u64, token3)];

    // Only use children starting from index 1
    let (hash, node) = cache.node(NODE_KIND, &mut children, 1);

    assert_ne!(hash, 0);
    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 2); // Should only include last 2 children
}

// =============================================================================
// Edge Cases and Error Conditions
// =============================================================================

#[test]
fn test_token_when_empty_text_expect_valid_caching() {
    let mut cache = NodeCache::default();

    let (hash, token) = cache.token(TOKEN_KIND, b"", GreenTrivia::new([]), GreenTrivia::new([]));

    assert_ne!(hash, 0);
    assert_eq!(token.text(), b"");
    assert_eq!(token.kind(), TOKEN_KIND);
}

#[test]
fn test_token_when_binary_content_expect_exact_preservation() {
    let mut cache = NodeCache::default();

    let binary_data = &[0xFF, 0xFE, 0x00, 0x01, 0x80, 0x7F];

    let (hash1, token1) = cache.token(
        TOKEN_KIND,
        binary_data,
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    let (hash2, token2) = cache.token(
        TOKEN_KIND,
        binary_data,
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    // Binary data should be deduplicated correctly
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, 0);
    assert_eq!(token1.text(), binary_data);
    assert_eq!(token2.text(), binary_data);

    // Should be same cached instance
    let ptr1 = &*token1 as *const _ as *const u8;
    let ptr2 = &*token2 as *const _ as *const u8;
    assert_eq!(ptr1, ptr2);
}

#[test]
fn test_node_when_empty_children_expect_valid_caching() {
    let mut cache = NodeCache::default();

    let mut children = Vec::new();
    let (hash, node) = cache.node(NODE_KIND, &mut children, 0);

    assert_ne!(hash, 0);
    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 0);
}

#[test]
fn test_node_when_child_has_zero_hash_expect_no_caching() {
    let mut cache = NodeCache::default();

    let token = GreenElement::Token(create_test_token(TOKEN_KIND, "test"));

    // Child with zero hash should prevent caching
    let mut children = vec![(0u64, token)];

    let (hash, node) = cache.node(NODE_KIND, &mut children, 0);

    // Should not be cached due to zero hash child
    assert_eq!(hash, 0);
    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 1);
}

// =============================================================================
// Nested Node Tests (for coverage of node_hash recursive case and element_id)
// =============================================================================

#[test]
fn test_node_when_caching_nested_nodes_expect_successful_storage() {
    let mut cache = NodeCache::default();

    // Create inner nodes first
    let token1 = GreenElement::Token(create_test_token(TOKEN_KIND, "inner1"));
    let token2 = GreenElement::Token(create_test_token(TOKEN_KIND, "inner2"));

    let mut inner_children1 = vec![(1u64, token1)];
    let mut inner_children2 = vec![(2u64, token2)];

    let (inner_hash1, inner_node1) = cache.node(LIST_KIND, &mut inner_children1, 0);
    let (inner_hash2, inner_node2) = cache.node(LIST_KIND, &mut inner_children2, 0);

    // Create outer node with inner nodes as children
    let mut outer_children = vec![
        (inner_hash1, GreenElement::Node(inner_node1)),
        (inner_hash2, GreenElement::Node(inner_node2)),
    ];

    let (outer_hash, outer_node) = cache.node(NODE_KIND, &mut outer_children, 0);

    // All hashes should be non-zero
    assert_ne!(inner_hash1, 0);
    assert_ne!(inner_hash2, 0);
    assert_ne!(outer_hash, 0);

    // Outer node should have correct structure
    assert_eq!(outer_node.kind(), NODE_KIND);
    assert_eq!(outer_node.children().count(), 2);

    // Verify nested structure
    let children: Vec<_> = outer_node.children().collect();
    assert!(matches!(children[0], crate::NodeOrToken::Node(_)));
    assert!(matches!(children[1], crate::NodeOrToken::Node(_)));
}

#[test]
fn test_node_when_caching_identical_nested_nodes_expect_deduplication() {
    let mut cache = NodeCache::default();

    // Create identical inner structure twice
    let token = GreenElement::Token(create_test_token(TOKEN_KIND, "nested"));

    let mut inner_children1 = vec![(1u64, token.clone())];
    let mut inner_children2 = vec![(1u64, token)];

    let (inner_hash1, inner_node1) = cache.node(LIST_KIND, &mut inner_children1, 0);
    let (inner_hash2, _inner_node2) = cache.node(LIST_KIND, &mut inner_children2, 0);

    // Inner nodes should be deduplicated
    assert_eq!(inner_hash1, inner_hash2);

    // Create identical outer structures (both use inner_node1 since they're identical)
    let mut outer_children1 = vec![(inner_hash1, GreenElement::Node(inner_node1.clone()))];
    let mut outer_children2 = vec![(inner_hash2, GreenElement::Node(inner_node1))];

    let (outer_hash1, outer_node1) = cache.node(NODE_KIND, &mut outer_children1, 0);
    let (outer_hash2, outer_node2) = cache.node(NODE_KIND, &mut outer_children2, 0);

    // Outer nodes should also be deduplicated
    assert_eq!(outer_hash1, outer_hash2);
    assert_ne!(outer_hash1, 0);

    // Should be same cached instance
    let ptr1 = &*outer_node1 as *const _ as *const u8;
    let ptr2 = &*outer_node2 as *const _ as *const u8;
    assert_eq!(ptr1, ptr2);
}

#[test]
fn test_node_when_mixed_children_types_expect_correct_hashing() {
    let mut cache = NodeCache::default();

    // Create a node with mixed token and node children
    let token = GreenElement::Token(create_test_token(TOKEN_KIND, "token_child"));

    // Create inner node
    let inner_token = GreenElement::Token(create_test_token(STRING_KIND, "inner"));
    let mut inner_children = vec![(1u64, inner_token)];
    let (inner_hash, inner_node) = cache.node(LIST_KIND, &mut inner_children, 0);

    // Create outer node with both token and node children
    let mut mixed_children = vec![(2u64, token), (inner_hash, GreenElement::Node(inner_node))];

    let (mixed_hash, mixed_node) = cache.node(NODE_KIND, &mut mixed_children, 0);

    assert_ne!(inner_hash, 0);
    assert_ne!(mixed_hash, 0);
    assert_eq!(mixed_node.kind(), NODE_KIND);
    assert_eq!(mixed_node.children().count(), 2);

    // Verify mixed children types
    let children: Vec<_> = mixed_node.children().collect();
    assert!(matches!(children[0], crate::NodeOrToken::Token(_)));
    assert!(matches!(children[1], crate::NodeOrToken::Node(_)));
}

#[test]
fn test_node_when_deeply_nested_expect_recursive_hashing() {
    let mut cache = NodeCache::default();

    // Create deeply nested structure: outer -> middle -> inner
    let token = GreenElement::Token(create_test_token(TOKEN_KIND, "deep"));

    // Innermost node
    let mut inner_children = vec![(1u64, token)];
    let (inner_hash, inner_node) = cache.node(STRING_KIND, &mut inner_children, 0);

    // Middle node containing inner node
    let mut middle_children = vec![(inner_hash, GreenElement::Node(inner_node))];
    let (middle_hash, middle_node) = cache.node(LIST_KIND, &mut middle_children, 0);

    // Outer node containing middle node
    let mut outer_children = vec![(middle_hash, GreenElement::Node(middle_node))];
    let (outer_hash, outer_node) = cache.node(NODE_KIND, &mut outer_children, 0);

    // All levels should be cached successfully
    assert_ne!(inner_hash, 0);
    assert_ne!(middle_hash, 0);
    assert_ne!(outer_hash, 0);

    assert_eq!(outer_node.kind(), NODE_KIND);
    assert_eq!(outer_node.children().count(), 1);
}

// =============================================================================
// Complex Scenarios and Integration Tests
// =============================================================================

#[test]
fn test_mixed_caching_when_tokens_and_nodes_expect_independent_operation() {
    let mut cache = NodeCache::default();

    // Cache some tokens
    let (token_hash1, token1) = cache.token(
        TOKEN_KIND,
        b"token1",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    let (token_hash2, token2) = cache.token(
        TOKEN_KIND,
        b"token2",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    // Create a node using the cached tokens
    let mut children = vec![
        (token_hash1, GreenElement::Token(token1)),
        (token_hash2, GreenElement::Token(token2)),
    ];

    let (node_hash, node) = cache.node(NODE_KIND, &mut children, 0);

    // All should be successfully cached
    assert_ne!(token_hash1, 0);
    assert_ne!(token_hash2, 0);
    assert_ne!(node_hash, 0);

    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 2);
}

#[test]
fn test_repeated_operations_when_mixed_access_patterns_expect_consistent_behavior() {
    let mut cache = NodeCache::default();

    // Interleave token and node caching
    let (hash1, _) = cache.token(
        TOKEN_KIND,
        b"test",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    let token = GreenElement::Token(create_test_token(TOKEN_KIND, "child"));
    let mut children = vec![(1u64, token)];
    let (hash2, _) = cache.node(NODE_KIND, &mut children, 0);

    // Cache the same token again
    let (hash3, _) = cache.token(
        TOKEN_KIND,
        b"test",
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    // Cache the same node again
    let token2 = GreenElement::Token(create_test_token(TOKEN_KIND, "child"));
    let mut children2 = vec![(1u64, token2)];
    let (hash4, _) = cache.node(NODE_KIND, &mut children2, 0);

    // Should get same hashes for identical items
    assert_eq!(hash1, hash3);
    assert_eq!(hash2, hash4);
    assert_ne!(hash1, 0);
    assert_ne!(hash2, 0);
}

#[test]
fn test_cache_efficiency_when_realistic_pdf_scenario_expect_deduplication() {
    let mut cache = NodeCache::default();

    // Simulate common PDF tokens that appear frequently
    let mut space_hashes = Vec::new();
    for _ in 0..10 {
        let (hash, _) = cache.token(TOKEN_KIND, b" ", GreenTrivia::new([]), GreenTrivia::new([]));
        space_hashes.push(hash);
    }

    // All space tokens should have the same hash (deduplication)
    for hash in &space_hashes {
        assert_eq!(*hash, space_hashes[0]);
        assert_ne!(*hash, 0);
    }

    // Create nodes with these tokens
    let mut node_hashes = Vec::new();
    for _ in 0..5 {
        let token = GreenElement::Token(create_test_token(TOKEN_KIND, " "));
        let mut children = vec![(space_hashes[0], token)];
        let (hash, _) = cache.node(LIST_KIND, &mut children, 0);
        node_hashes.push(hash);
    }

    // All nodes with same structure should have same hash
    for hash in &node_hashes {
        assert_eq!(*hash, node_hashes[0]);
        assert_ne!(*hash, 0);
    }
}

#[test]
fn test_node_hash_recursive_when_node_contains_nodes_expect_recursive_calculation() {
    let mut cache = NodeCache::default();

    // Create a nested structure where we force hash calculation
    // This ensures the recursive node_hash path is exercised

    // Create innermost node with token
    let inner_token = GreenElement::Token(create_test_token(TOKEN_KIND, "innermost"));
    let mut inner_children = vec![(1u64, inner_token)];
    let (inner_hash, inner_node) = cache.node(STRING_KIND, &mut inner_children, 0);

    // Create middle node containing the inner node - this will trigger recursive hashing
    let mut middle_children = vec![(inner_hash, GreenElement::Node(inner_node))];
    let (middle_hash, middle_node) = cache.node(LIST_KIND, &mut middle_children, 0);

    // Create outer node containing the middle node - this will also trigger recursive hashing
    let mut outer_children = vec![(middle_hash, GreenElement::Node(middle_node))];
    let (outer_hash, outer_node) = cache.node(NODE_KIND, &mut outer_children, 0);

    // Verify all levels cached successfully
    assert_ne!(inner_hash, 0);
    assert_ne!(middle_hash, 0);
    assert_ne!(outer_hash, 0);

    // Now create a second identical structure to ensure hash comparison works
    // This forces the cache to compute hashes for comparison, exercising the recursive path
    let inner_token2 = GreenElement::Token(create_test_token(TOKEN_KIND, "innermost"));
    let mut inner_children2 = vec![(1u64, inner_token2)];
    let (inner_hash2, inner_node2) = cache.node(STRING_KIND, &mut inner_children2, 0);

    let mut middle_children2 = vec![(inner_hash2, GreenElement::Node(inner_node2))];
    let (middle_hash2, middle_node2) = cache.node(LIST_KIND, &mut middle_children2, 0);

    let mut outer_children2 = vec![(middle_hash2, GreenElement::Node(middle_node2))];
    let (outer_hash2, outer_node2) = cache.node(NODE_KIND, &mut outer_children2, 0);

    // Should be deduplicated due to identical structure
    assert_eq!(inner_hash, inner_hash2);
    assert_eq!(middle_hash, middle_hash2);
    assert_eq!(outer_hash, outer_hash2);

    // Verify structure
    assert_eq!(outer_node.kind(), outer_node2.kind());
    assert_eq!(outer_node.children().count(), 1);
}

#[test]
fn test_node_hash_recursive_when_large_nested_nodes_expect_hash_computation() {
    let mut cache = NodeCache::default();

    // Create a scenario that forces hash computation with nested nodes
    // by creating large nodes (>3 children) that won't be cached,
    // then creating similar smaller structures that will trigger hash comparison

    // Create inner nodes with multiple children to make them large
    let mut large_inner_children = Vec::new();
    for i in 0..4 {
        let token = GreenElement::Token(create_test_token(TOKEN_KIND, &format!("token{}", i)));
        large_inner_children.push((i as u64 + 1, token));
    }

    // This will create a node but not cache it (too many children)
    let (large_hash, large_inner) = cache.node(LIST_KIND, &mut large_inner_children, 0);
    assert_eq!(large_hash, 0); // Should not be cached

    // Now create a node that contains this large node as a child
    // This should trigger the recursive node_hash calculation when building hash
    let mut outer_children = vec![(0u64, GreenElement::Node(large_inner.clone()))];
    let (outer_hash, _outer_node) = cache.node(NODE_KIND, &mut outer_children, 0);
    assert_eq!(outer_hash, 0); // Should not be cached due to zero hash child

    // Now create a smaller nested structure that WILL be cached
    let small_token = GreenElement::Token(create_test_token(TOKEN_KIND, "small"));
    let mut small_inner_children = vec![(1u64, small_token)];
    let (small_inner_hash, small_inner) = cache.node(LIST_KIND, &mut small_inner_children, 0);
    assert_ne!(small_inner_hash, 0); // Should be cached

    // Create another node containing this small node - this should trigger recursive hashing
    let mut small_outer_children = vec![(small_inner_hash, GreenElement::Node(small_inner))];
    let (small_outer_hash, small_outer) = cache.node(NODE_KIND, &mut small_outer_children, 0);
    assert_ne!(small_outer_hash, 0); // Should be cached

    // Create an identical structure to force hash comparison and deduplication
    let small_token2 = GreenElement::Token(create_test_token(TOKEN_KIND, "small"));
    let mut small_inner_children2 = vec![(1u64, small_token2)];
    let (small_inner_hash2, small_inner2) = cache.node(LIST_KIND, &mut small_inner_children2, 0);

    let mut small_outer_children2 = vec![(small_inner_hash2, GreenElement::Node(small_inner2))];
    let (small_outer_hash2, small_outer2) = cache.node(NODE_KIND, &mut small_outer_children2, 0);

    // These should be deduplicated (same hashes)
    assert_eq!(small_inner_hash, small_inner_hash2);
    assert_eq!(small_outer_hash, small_outer_hash2);

    // Verify the nodes are structurally identical
    assert_eq!(small_outer.kind(), small_outer2.kind());
    assert_eq!(
        small_outer.children().count(),
        small_outer2.children().count()
    );
}

#[test]
fn test_node_hash_recursive_when_inserting_nested_nodes_expect_hasher_callback() {
    let mut cache = NodeCache::default();

    // Create nested structure that will trigger the hasher callback during insertion
    // The key is to create nodes that contain other nodes and ensure they're being inserted fresh

    // First, create inner nodes with different content to ensure they're unique
    let token1 = GreenElement::Token(create_test_token(TOKEN_KIND, "unique1"));
    let token2 = GreenElement::Token(create_test_token(STRING_KIND, "unique2"));

    let mut inner_children1 = vec![(1u64, token1)];
    let mut inner_children2 = vec![(2u64, token2)];

    let (inner_hash1, inner_node1) = cache.node(LIST_KIND, &mut inner_children1, 0);
    let (inner_hash2, inner_node2) = cache.node(NUMBER_KIND, &mut inner_children2, 0);

    // Now create outer nodes that contain these inner nodes
    // Each outer node will be unique and trigger the insert_with_hasher callback
    let mut outer_children1 = vec![(inner_hash1, GreenElement::Node(inner_node1))];
    let mut outer_children2 = vec![(inner_hash2, GreenElement::Node(inner_node2))];

    // These calls should trigger the hasher callback with node_hash,
    // which will recursively call node_hash for the inner nodes
    let (outer_hash1, outer_node1) = cache.node(NODE_KIND, &mut outer_children1, 0);
    let (outer_hash2, outer_node2) = cache.node(NODE_KIND, &mut outer_children2, 0);

    // Verify both outer nodes were cached (different hashes due to different children)
    assert_ne!(outer_hash1, 0);
    assert_ne!(outer_hash2, 0);
    assert_ne!(outer_hash1, outer_hash2);

    // Verify structure
    assert_eq!(outer_node1.kind(), NODE_KIND);
    assert_eq!(outer_node2.kind(), NODE_KIND);
    assert_eq!(outer_node1.children().count(), 1);
    assert_eq!(outer_node2.children().count(), 1);

    // Create a third nested structure with even more nesting to ensure deep recursion
    let deep_token = GreenElement::Token(create_test_token(TOKEN_KIND, "deep_token"));
    let mut deep_inner_children = vec![(3u64, deep_token)];
    let (deep_inner_hash, deep_inner_node) = cache.node(STRING_KIND, &mut deep_inner_children, 0);

    let mut deep_middle_children = vec![(deep_inner_hash, GreenElement::Node(deep_inner_node))];
    let (deep_middle_hash, deep_middle_node) = cache.node(LIST_KIND, &mut deep_middle_children, 0);

    let mut deep_outer_children = vec![(deep_middle_hash, GreenElement::Node(deep_middle_node))];
    let (deep_outer_hash, deep_outer_node) = cache.node(NODE_KIND, &mut deep_outer_children, 0);

    // This should have triggered multiple levels of recursive node_hash calls
    assert_ne!(deep_outer_hash, 0);
    assert_eq!(deep_outer_node.children().count(), 1);
}

#[test]
fn test_token_hash_callback_when_inserting_unique_tokens_expect_hasher_execution() {
    let mut cache = NodeCache::default();

    // Create a series of unique tokens to force the insert_with_hasher callback
    // Each token must be unique to avoid hitting the cache and force insertion

    let unique_tokens = [
        ("unique1", SyntaxKind(100)),
        ("unique2", SyntaxKind(101)),
        ("unique3", SyntaxKind(102)),
        ("same_text_diff_kind1", SyntaxKind(103)),
        ("same_text_diff_kind2", SyntaxKind(104)),
    ];

    let mut token_hashes = Vec::new();

    for (text, kind) in unique_tokens {
        // Create unique trivia for each token to ensure they're truly unique
        use crate::green::trivia::GreenTriviaChild;
        let unique_trivia = GreenTrivia::new(vec![GreenTriviaChild::new(
            SyntaxKind(200 + kind.0),
            format!("comment_{}", text).as_bytes(),
        )]);

        // This should trigger the Vacant branch and call the hasher callback
        let (hash, token) = cache.token(kind, text.as_bytes(), unique_trivia, GreenTrivia::new([]));

        assert_ne!(hash, 0);
        assert_eq!(token.text(), text.as_bytes());
        assert_eq!(token.kind(), kind);
        token_hashes.push(hash);
    }

    // All tokens should have different hashes since they're unique
    for i in 0..token_hashes.len() {
        for j in i + 1..token_hashes.len() {
            assert_ne!(
                token_hashes[i], token_hashes[j],
                "Tokens {} and {} should have different hashes",
                i, j
            );
        }
    }

    // Now test with binary content to ensure hasher callback works with non-UTF8 data
    let binary_contents = [
        &[0xFF, 0x00, 0x01][..],
        &[0x00, 0xFF, 0x02][..],
        &[0x80, 0x7F, 0x03][..],
    ];

    for (i, binary_data) in binary_contents.iter().enumerate() {
        let (hash, token) = cache.token(
            SyntaxKind(300 + i as u16),
            binary_data,
            GreenTrivia::new([]),
            GreenTrivia::new([]),
        );

        assert_ne!(hash, 0);
        assert_eq!(token.text(), *binary_data);
    }
}
