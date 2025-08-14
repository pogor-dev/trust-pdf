use crate::{
    NodeOrToken, SyntaxKind,
    green::{builder::GreenNodeBuilder, node_cache::NodeCache, trivia::GreenTrivia},
};

// Test constants following the same pattern as other test files
const TOKEN_KIND: SyntaxKind = SyntaxKind(1);
const NODE_KIND: SyntaxKind = SyntaxKind(2);
const STRING_KIND: SyntaxKind = SyntaxKind(3);
const NUMBER_KIND: SyntaxKind = SyntaxKind(4);
const ARRAY_KIND: SyntaxKind = SyntaxKind(5);
const DICT_KIND: SyntaxKind = SyntaxKind(6);
const COMMENT_KIND: SyntaxKind = SyntaxKind(10);
const WHITESPACE_KIND: SyntaxKind = SyntaxKind(11);

#[test]
fn test_new_when_creating_builder_expect_empty_state() {
    let builder = GreenNodeBuilder::new();

    // Builder should be created successfully
    // We can't directly inspect internal state, but we can verify through behavior
    assert!(format!("{:?}", builder).contains("GreenNodeBuilder"));
}

#[test]
fn test_with_cache_when_using_external_cache_expect_cache_sharing() {
    let mut cache = NodeCache::default();
    let builder = GreenNodeBuilder::with_cache(&mut cache);

    // Builder should be created successfully with external cache
    assert!(format!("{:?}", builder).contains("GreenNodeBuilder"));
}

#[test]
fn test_token_when_adding_simple_token_expect_correct_storage() {
    let mut builder = GreenNodeBuilder::new();

    builder.start_node(NODE_KIND);
    builder.token(STRING_KIND, b"hello");
    builder.finish_node();

    let node = builder.finish();

    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 1);

    let child = node.children().next().unwrap();
    if let NodeOrToken::Token(token) = child {
        assert_eq!(token.kind(), STRING_KIND);
        assert_eq!(token.text(), b"hello");
        assert_eq!(token.leading_trivia().children().len(), 0);
        assert_eq!(token.trailing_trivia().children().len(), 0);
    } else {
        panic!("Expected token, got node");
    }
}

#[test]
fn test_token_when_adding_multiple_tokens_expect_correct_sequence() {
    let mut builder = GreenNodeBuilder::new();

    builder.start_node(ARRAY_KIND);
    builder.token(STRING_KIND, b"first");
    builder.token(NUMBER_KIND, b"42");
    builder.token(STRING_KIND, b"last");
    builder.finish_node();

    let node = builder.finish();

    assert_eq!(node.kind(), ARRAY_KIND);
    assert_eq!(node.children().count(), 3);

    let children: Vec<_> = node.children().collect();

    // Check first token
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.kind(), STRING_KIND);
        assert_eq!(token.text(), b"first");
    } else {
        panic!("Expected token at index 0");
    }

    // Check second token
    if let NodeOrToken::Token(token) = &children[1] {
        assert_eq!(token.kind(), NUMBER_KIND);
        assert_eq!(token.text(), b"42");
    } else {
        panic!("Expected token at index 1");
    }

    // Check third token
    if let NodeOrToken::Token(token) = &children[2] {
        assert_eq!(token.kind(), STRING_KIND);
        assert_eq!(token.text(), b"last");
    } else {
        panic!("Expected token at index 2");
    }
}

#[test]
fn test_token_with_trivia_when_adding_token_with_leading_trivia_expect_correct_trivia() {
    use crate::green::trivia::GreenTriviaChild;

    let mut builder = GreenNodeBuilder::new();
    let leading_trivia = GreenTrivia::new(vec![
        GreenTriviaChild::new(WHITESPACE_KIND, b"  "),
        GreenTriviaChild::new(COMMENT_KIND, b"%comment\n"),
    ]);
    let trailing_trivia = GreenTrivia::new([]);

    builder.start_node(NODE_KIND);
    builder.token_with_trivia(STRING_KIND, b"value", leading_trivia, trailing_trivia);
    builder.finish_node();

    let node = builder.finish();
    let child = node.children().next().unwrap();

    if let NodeOrToken::Token(token) = child {
        assert_eq!(token.kind(), STRING_KIND);
        assert_eq!(token.text(), b"value");
        assert_eq!(token.leading_trivia().children().len(), 2);
        assert_eq!(token.trailing_trivia().children().len(), 0);

        // Check leading trivia content
        let leading_children: Vec<_> = token.leading_trivia().children().iter().collect();
        assert_eq!(leading_children[0].kind(), WHITESPACE_KIND);
        assert_eq!(leading_children[0].text(), b"  ");
        assert_eq!(leading_children[1].kind(), COMMENT_KIND);
        assert_eq!(leading_children[1].text(), b"%comment\n");
    } else {
        panic!("Expected token, got node");
    }
}

#[test]
fn test_token_with_trivia_when_adding_token_with_trailing_trivia_expect_correct_trivia() {
    use crate::green::trivia::GreenTriviaChild;

    let mut builder = GreenNodeBuilder::new();
    let leading_trivia = GreenTrivia::new([]);
    let trailing_trivia = GreenTrivia::new(vec![
        GreenTriviaChild::new(WHITESPACE_KIND, b" "),
        GreenTriviaChild::new(COMMENT_KIND, b"%trailing comment"),
    ]);

    builder.start_node(NODE_KIND);
    builder.token_with_trivia(NUMBER_KIND, b"123", leading_trivia, trailing_trivia);
    builder.finish_node();

    let node = builder.finish();
    let child = node.children().next().unwrap();

    if let NodeOrToken::Token(token) = child {
        assert_eq!(token.kind(), NUMBER_KIND);
        assert_eq!(token.text(), b"123");
        assert_eq!(token.leading_trivia().children().len(), 0);
        assert_eq!(token.trailing_trivia().children().len(), 2);

        // Check trailing trivia content
        let trailing_children: Vec<_> = token.trailing_trivia().children().iter().collect();
        assert_eq!(trailing_children[0].kind(), WHITESPACE_KIND);
        assert_eq!(trailing_children[0].text(), b" ");
        assert_eq!(trailing_children[1].kind(), COMMENT_KIND);
        assert_eq!(trailing_children[1].text(), b"%trailing comment");
    } else {
        panic!("Expected token, got node");
    }
}

#[test]
fn test_start_finish_node_when_creating_nested_structure_expect_correct_hierarchy() {
    let mut builder = GreenNodeBuilder::new();

    // Build: DICT_KIND -> ARRAY_KIND -> [STRING_KIND, NUMBER_KIND]
    builder.start_node(DICT_KIND);
    builder.start_node(ARRAY_KIND);
    builder.token(STRING_KIND, b"item1");
    builder.token(NUMBER_KIND, b"42");
    builder.finish_node(); // Finish ARRAY_KIND
    builder.finish_node(); // Finish DICT_KIND

    let node = builder.finish();

    assert_eq!(node.kind(), DICT_KIND);
    assert_eq!(node.children().count(), 1);

    let array_child = node.children().next().unwrap();
    if let NodeOrToken::Node(array_node) = array_child {
        assert_eq!(array_node.kind(), ARRAY_KIND);
        assert_eq!(array_node.children().count(), 2);

        let array_children: Vec<_> = array_node.children().collect();

        // Check first token in array
        if let NodeOrToken::Token(token) = &array_children[0] {
            assert_eq!(token.kind(), STRING_KIND);
            assert_eq!(token.text(), b"item1");
        } else {
            panic!("Expected token at array index 0");
        }

        // Check second token in array
        if let NodeOrToken::Token(token) = &array_children[1] {
            assert_eq!(token.kind(), NUMBER_KIND);
            assert_eq!(token.text(), b"42");
        } else {
            panic!("Expected token at array index 1");
        }
    } else {
        panic!("Expected node child, got token");
    }
}

#[test]
fn test_checkpoint_when_creating_checkpoint_expect_valid_checkpoint() {
    let mut builder = GreenNodeBuilder::new();

    builder.start_node(NODE_KIND);
    builder.token(STRING_KIND, b"before");

    let checkpoint = builder.checkpoint();

    builder.token(NUMBER_KIND, b"123");
    builder.finish_node();

    let node = builder.finish();

    // Checkpoint should be valid and builder should work correctly
    assert_eq!(node.children().count(), 2);

    // We can't directly test checkpoint value, but we can verify it doesn't break the builder
    assert!(format!("{:?}", checkpoint).contains("Checkpoint"));
}

#[test]
fn test_start_node_at_when_wrapping_existing_content_expect_correct_wrapping() {
    let mut builder = GreenNodeBuilder::new();

    builder.start_node(NODE_KIND);

    // Add some content
    builder.token(STRING_KIND, b"first");

    // Create checkpoint before adding more content
    let checkpoint = builder.checkpoint();
    builder.token(NUMBER_KIND, b"42");
    builder.token(STRING_KIND, b"last");

    // Wrap the content after checkpoint in a new node
    builder.start_node_at(checkpoint, ARRAY_KIND);
    builder.finish_node(); // Finish ARRAY_KIND wrapper

    builder.finish_node(); // Finish NODE_KIND

    let node = builder.finish();

    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 2);

    let children: Vec<_> = node.children().collect();

    // First child should be the "first" token
    if let NodeOrToken::Token(token) = &children[0] {
        assert_eq!(token.kind(), STRING_KIND);
        assert_eq!(token.text(), b"first");
    } else {
        panic!("Expected token at index 0");
    }

    // Second child should be the wrapped array node
    if let NodeOrToken::Node(array_node) = &children[1] {
        assert_eq!(array_node.kind(), ARRAY_KIND);
        assert_eq!(array_node.children().count(), 2);

        let array_children: Vec<_> = array_node.children().collect();

        // Check wrapped content
        if let NodeOrToken::Token(token) = &array_children[0] {
            assert_eq!(token.kind(), NUMBER_KIND);
            assert_eq!(token.text(), b"42");
        } else {
            panic!("Expected token in wrapped array at index 0");
        }

        if let NodeOrToken::Token(token) = &array_children[1] {
            assert_eq!(token.kind(), STRING_KIND);
            assert_eq!(token.text(), b"last");
        } else {
            panic!("Expected token in wrapped array at index 1");
        }
    } else {
        panic!("Expected wrapped array node at index 1");
    }
}

#[test]
fn test_finish_when_completing_tree_expect_root_node() {
    let mut builder = GreenNodeBuilder::new();

    builder.start_node(NODE_KIND);
    builder.token(STRING_KIND, b"content");
    builder.finish_node();

    let node = builder.finish();

    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 1);
}

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn test_finish_when_no_nodes_added_expect_panic() {
    let builder = GreenNodeBuilder::new();
    builder.finish(); // Should panic because no nodes were added
}

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn test_finish_when_multiple_root_nodes_expect_panic() {
    let mut builder = GreenNodeBuilder::new();

    builder.start_node(NODE_KIND);
    builder.token(STRING_KIND, b"first");
    builder.finish_node();

    builder.start_node(NODE_KIND);
    builder.token(STRING_KIND, b"second");
    builder.finish_node();

    builder.finish(); // Should panic because there are multiple root nodes
}

#[test]
#[should_panic]
fn test_finish_when_root_is_token_expect_panic() {
    let mut builder = GreenNodeBuilder::new();

    builder.token(STRING_KIND, b"token");

    builder.finish(); // Should panic because root is a token, not a node
}

#[test]
#[should_panic]
fn test_finish_node_when_no_started_node_expect_panic() {
    let mut builder = GreenNodeBuilder::new();

    builder.token(STRING_KIND, b"token");
    builder.finish_node(); // Should panic because no node was started
}

#[test]
fn test_cache_reuse_when_using_shared_cache_expect_memory_efficiency() {
    let mut cache = NodeCache::default();

    // Create first tree
    let mut builder1 = GreenNodeBuilder::with_cache(&mut cache);
    builder1.start_node(NODE_KIND);
    builder1.token(STRING_KIND, b"shared");
    builder1.finish_node();
    let node1 = builder1.finish();

    // Create second tree with same content
    let mut builder2 = GreenNodeBuilder::with_cache(&mut cache);
    builder2.start_node(NODE_KIND);
    builder2.token(STRING_KIND, b"shared");
    builder2.finish_node();
    let node2 = builder2.finish();

    // Both nodes should be equivalent (though we can't test pointer equality directly)
    assert_eq!(node1.kind(), node2.kind());
    assert_eq!(node1.children().count(), node2.children().count());

    let child1 = node1.children().next().unwrap();
    let child2 = node2.children().next().unwrap();

    match (child1, child2) {
        (NodeOrToken::Token(token1), NodeOrToken::Token(token2)) => {
            assert_eq!(token1.kind(), token2.kind());
            assert_eq!(token1.text(), token2.text());
        }
        _ => panic!("Expected tokens in both nodes"),
    }
}

#[test]
fn test_complex_pdf_structure_when_building_realistic_tree_expect_correct_structure() {
    let mut builder = GreenNodeBuilder::new();

    // Build a structure like: DICT -> ["key", ARRAY -> [NUMBER, STRING]]
    builder.start_node(DICT_KIND);

    // Add key
    builder.token(STRING_KIND, b"items");

    // Add array value
    builder.start_node(ARRAY_KIND);
    builder.token(NUMBER_KIND, b"1");
    builder.token(NUMBER_KIND, b"2");
    builder.token(STRING_KIND, b"text");
    builder.finish_node(); // Finish ARRAY_KIND

    builder.finish_node(); // Finish DICT_KIND

    let root = builder.finish();

    // Verify structure
    assert_eq!(root.kind(), DICT_KIND);
    assert_eq!(root.children().count(), 2);

    let children: Vec<_> = root.children().collect();

    // Check key
    if let NodeOrToken::Token(key_token) = &children[0] {
        assert_eq!(key_token.kind(), STRING_KIND);
        assert_eq!(key_token.text(), b"items");
    } else {
        panic!("Expected key token");
    }

    // Check array value
    if let NodeOrToken::Node(array_node) = &children[1] {
        assert_eq!(array_node.kind(), ARRAY_KIND);
        assert_eq!(array_node.children().count(), 3);

        let array_children: Vec<_> = array_node.children().collect();

        // Check array elements
        if let NodeOrToken::Token(token) = &array_children[0] {
            assert_eq!(token.kind(), NUMBER_KIND);
            assert_eq!(token.text(), b"1");
        } else {
            panic!("Expected number token at array index 0");
        }

        if let NodeOrToken::Token(token) = &array_children[1] {
            assert_eq!(token.kind(), NUMBER_KIND);
            assert_eq!(token.text(), b"2");
        } else {
            panic!("Expected number token at array index 1");
        }

        if let NodeOrToken::Token(token) = &array_children[2] {
            assert_eq!(token.kind(), STRING_KIND);
            assert_eq!(token.text(), b"text");
        } else {
            panic!("Expected string token at array index 2");
        }
    } else {
        panic!("Expected array node");
    }
}

#[test]
fn test_deep_nesting_when_creating_deeply_nested_structure_expect_correct_traversal() {
    let mut builder = GreenNodeBuilder::new();

    // Create 5 levels of nesting
    for _ in 0..5 {
        builder.start_node(NODE_KIND);
    }

    builder.token(STRING_KIND, b"deep");

    for _ in 0..5 {
        builder.finish_node();
    }

    let root = builder.finish();

    // Traverse down to the deepest level
    let mut current_node = root;
    for depth in 0..5 {
        assert_eq!(current_node.kind(), NODE_KIND);
        assert_eq!(current_node.children().count(), 1);

        let child = current_node.children().next().unwrap();
        match child {
            NodeOrToken::Node(node_data) if depth < 4 => {
                // Convert GreenNodeData to GreenNode for next iteration
                current_node = node_data.to_owned();
            }
            NodeOrToken::Token(token) if depth == 4 => {
                assert_eq!(token.kind(), STRING_KIND);
                assert_eq!(token.text(), b"deep");
                break;
            }
            _ => panic!("Unexpected structure at depth {}", depth),
        }
    }
}

#[test]
fn test_empty_node_when_creating_node_without_children_expect_valid_empty_node() {
    let mut builder = GreenNodeBuilder::new();

    builder.start_node(NODE_KIND);
    builder.finish_node();

    let node = builder.finish();

    assert_eq!(node.kind(), NODE_KIND);
    assert_eq!(node.children().count(), 0);
}

#[test]
fn test_binary_content_when_adding_binary_tokens_expect_exact_preservation() {
    let mut builder = GreenNodeBuilder::new();
    let binary_data = b"\x00\xFF\x42\x7F\x80\x01";

    builder.start_node(NODE_KIND);
    builder.token(TOKEN_KIND, binary_data);
    builder.finish_node();

    let node = builder.finish();
    let child = node.children().next().unwrap();

    if let NodeOrToken::Token(token) = child {
        assert_eq!(token.text(), binary_data);
    } else {
        panic!("Expected token with binary data");
    }
}
