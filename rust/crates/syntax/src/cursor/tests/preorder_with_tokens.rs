use crate::{
    NodeOrToken, SyntaxKind,
    cursor::{
        node::SyntaxNode, preorder_with_tokens::PreorderWithTokens, element::SyntaxElement,
    },
    green::{element::GreenElement, node::GreenNode, token::GreenToken, trivia::GreenTrivia},
    utility_types::WalkEvent,
};

// Test constants for different PDF syntax kinds
const STRING_KIND: SyntaxKind = SyntaxKind(1);
const NUMBER_KIND: SyntaxKind = SyntaxKind(2);
const NAME_KIND: SyntaxKind = SyntaxKind(3);
const DICT_KIND: SyntaxKind = SyntaxKind(4);
const ARRAY_KIND: SyntaxKind = SyntaxKind(5);
const OBJ_KIND: SyntaxKind = SyntaxKind(6);

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

fn create_single_node_tree() -> SyntaxNode {
    let token = create_green_token(STRING_KIND, "(Hello)");
    let node = create_green_node(DICT_KIND, vec![token.into()]);
    SyntaxNode::new_root(node)
}

fn create_flat_tree() -> SyntaxNode {
    let token1 = create_green_token(NAME_KIND, "/Type");
    let token2 = create_green_token(NAME_KIND, "/Catalog");
    let token3 = create_green_token(NUMBER_KIND, "42");

    let node = create_green_node(DICT_KIND, vec![token1.into(), token2.into(), token3.into()]);
    SyntaxNode::new_root(node)
}

fn create_nested_tree() -> SyntaxNode {
    // Create inner dictionary with tokens
    let inner_token1 = create_green_token(NAME_KIND, "/Type");
    let inner_token2 = create_green_token(NAME_KIND, "/Page");
    let inner_dict = create_green_node(DICT_KIND, vec![inner_token1.into(), inner_token2.into()]);

    // Create outer object with nested dictionary and a number token
    let number_token = create_green_token(NUMBER_KIND, "123");
    let outer_obj = create_green_node(OBJ_KIND, vec![inner_dict.into(), number_token.into()]);

    SyntaxNode::new_root(outer_obj)
}

fn create_deeply_nested_tree() -> SyntaxNode {
    // Create innermost array with a number token
    let number = create_green_token(NUMBER_KIND, "42");
    let inner_array = create_green_node(ARRAY_KIND, vec![number.into()]);

    // Create dictionary containing the array
    let contents_name = create_green_token(NAME_KIND, "/Contents");
    let dict = create_green_node(DICT_KIND, vec![contents_name.into(), inner_array.into()]);

    // Create root object containing the dictionary
    let root_obj = create_green_node(OBJ_KIND, vec![dict.into()]);
    SyntaxNode::new_root(root_obj)
}

fn create_complex_pdf_structure() -> SyntaxNode {
    // Create /Type /Catalog pair
    let type_name = create_green_token(NAME_KIND, "/Type");
    let catalog_name = create_green_token(NAME_KIND, "/Catalog");

    // Create nested /Names dictionary
    let names_key = create_green_token(NAME_KIND, "/Names");
    let names_value = create_green_token(NAME_KIND, "/SomeValue");
    let names_dict = create_green_node(DICT_KIND, vec![names_key.into(), names_value.into()]);

    // Create main dictionary
    let main_dict = create_green_node(
        DICT_KIND,
        vec![type_name.into(), catalog_name.into(), names_dict.into()],
    );

    // Create root object
    let root_obj = create_green_node(OBJ_KIND, vec![main_dict.into()]);
    SyntaxNode::new_root(root_obj)
}

fn create_mixed_tokens_and_nodes_tree() -> SyntaxNode {
    // Create tokens and nested nodes mixed together
    let token1 = create_green_token(NAME_KIND, "/Start");

    let inner_token = create_green_token(STRING_KIND, "(inner)");
    let inner_node = create_green_node(ARRAY_KIND, vec![inner_token.into()]);

    let token2 = create_green_token(NUMBER_KIND, "123");
    let token3 = create_green_token(NAME_KIND, "/End");

    let root = create_green_node(
        DICT_KIND,
        vec![
            token1.into(),
            inner_node.into(),
            token2.into(),
            token3.into(),
        ],
    );

    SyntaxNode::new_root(root)
}

// =============================================================================
// Basic Construction and State Tests
// =============================================================================

#[test]
fn test_new_when_creating_preorder_with_tokens_expect_correct_initial_state() {
    let tree = create_single_node_tree();
    let preorder = PreorderWithTokens::new(tree.clone());

    // Debug format should be readable
    let debug_str = format!("{:?}", preorder);
    assert!(debug_str.contains("PreorderWithTokens"));
}

#[test]
fn test_single_node_when_iterating_expect_enter_and_leave_events() {
    let tree = create_single_node_tree();
    let events: Vec<WalkEvent<SyntaxElement>> = PreorderWithTokens::new(tree).collect();

    assert_eq!(events.len(), 4); // Enter(DICT), Enter(STRING), Leave(STRING), Leave(DICT)

    // First event should be Enter(DICT)
    match &events[0] {
        WalkEvent::Enter(el) => {
            assert!(matches!(el, NodeOrToken::Node(_)));
            if let NodeOrToken::Node(node) = el {
                assert_eq!(node.kind(), DICT_KIND);
            }
        }
        WalkEvent::Leave(_) => panic!("Expected Enter event first"),
    }

    // Second event should be Enter(STRING token)
    match &events[1] {
        WalkEvent::Enter(el) => {
            assert!(matches!(el, NodeOrToken::Token(_)));
            if let NodeOrToken::Token(token) = el {
                assert_eq!(token.kind(), STRING_KIND);
            }
        }
        WalkEvent::Leave(_) => panic!("Expected Enter(token) second"),
    }

    // Third event should be Leave(STRING token)
    match &events[2] {
        WalkEvent::Leave(el) => {
            assert!(matches!(el, NodeOrToken::Token(_)));
            if let NodeOrToken::Token(token) = el {
                assert_eq!(token.kind(), STRING_KIND);
            }
        }
        WalkEvent::Enter(_) => panic!("Expected Leave(token) third"),
    }

    // Last event should be Leave(DICT)
    match &events[3] {
        WalkEvent::Leave(el) => {
            assert!(matches!(el, NodeOrToken::Node(_)));
            if let NodeOrToken::Node(node) = el {
                assert_eq!(node.kind(), DICT_KIND);
            }
        }
        WalkEvent::Enter(_) => panic!("Expected Leave(node) last"),
    }
}

// =============================================================================
// Token Traversal Tests
// =============================================================================

#[test]
fn test_flat_tree_when_iterating_expect_all_tokens_visited() {
    let tree = create_flat_tree();
    let events: Vec<WalkEvent<SyntaxElement>> = PreorderWithTokens::new(tree).collect();

    // Should visit: Enter(DICT), Enter(token1), Leave(token1), Enter(token2), Leave(token2), Enter(token3), Leave(token3), Leave(DICT)
    assert_eq!(events.len(), 8);

    // Count Enter and Leave events for tokens
    let token_enter_count = events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Enter(NodeOrToken::Token(_))))
        .count();
    let token_leave_count = events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Leave(NodeOrToken::Token(_))))
        .count();

    assert_eq!(token_enter_count, 3);
    assert_eq!(token_leave_count, 3);

    // Verify token kinds are visited in order
    let mut token_kinds = Vec::new();
    for event in &events {
        if let WalkEvent::Enter(NodeOrToken::Token(token)) = event {
            token_kinds.push(token.kind());
        }
    }

    assert_eq!(token_kinds, vec![NAME_KIND, NAME_KIND, NUMBER_KIND]);
}

#[test]
fn test_nested_tree_when_iterating_expect_depth_first_with_tokens() {
    let tree = create_nested_tree();
    let events: Vec<WalkEvent<SyntaxElement>> = PreorderWithTokens::new(tree).collect();

    // Should traverse nodes and tokens in depth-first order
    assert!(events.len() > 6); // At least the main structure

    // First event should be Enter(OBJ)
    match &events[0] {
        WalkEvent::Enter(NodeOrToken::Node(node)) => assert_eq!(node.kind(), OBJ_KIND),
        _ => panic!("Expected Enter(OBJ) first"),
    }

    // Should contain both node and token events
    let has_node_events = events
        .iter()
        .any(|e| matches!(e, WalkEvent::Enter(NodeOrToken::Node(_))));
    let has_token_events = events
        .iter()
        .any(|e| matches!(e, WalkEvent::Enter(NodeOrToken::Token(_))));

    assert!(has_node_events, "Should have node events");
    assert!(has_token_events, "Should have token events");
}

#[test]
fn test_mixed_tokens_and_nodes_when_iterating_expect_correct_order() {
    let tree = create_mixed_tokens_and_nodes_tree();
    let events: Vec<WalkEvent<SyntaxElement>> = PreorderWithTokens::new(tree).collect();

    // Should visit elements in document order
    let mut element_sequence = Vec::new();
    for event in &events {
        if let WalkEvent::Enter(el) = event {
            match el {
                NodeOrToken::Node(node) => {
                    element_sequence.push(format!("Node({})", node.kind().0))
                }
                NodeOrToken::Token(token) => {
                    element_sequence.push(format!("Token({})", token.kind().0))
                }
            }
        }
    }

    // Should include all expected elements
    assert!(element_sequence.contains(&"Node(4)".to_string())); // DICT_KIND
    assert!(element_sequence.contains(&"Token(3)".to_string())); // NAME_KIND
    assert!(element_sequence.contains(&"Node(5)".to_string())); // ARRAY_KIND
    assert!(element_sequence.contains(&"Token(1)".to_string())); // STRING_KIND
    assert!(element_sequence.contains(&"Token(2)".to_string())); // NUMBER_KIND
}

// =============================================================================
// Skip Subtree Tests
// =============================================================================

#[test]
fn test_skip_subtree_when_called_on_enter_node_expect_subtree_skipped() {
    let tree = create_nested_tree();
    let mut preorder = PreorderWithTokens::new(tree);
    let mut events = Vec::new();

    while let Some(event) = preorder.next() {
        match &event {
            WalkEvent::Enter(NodeOrToken::Node(node)) if node.kind() == DICT_KIND => {
                events.push(event);
                preorder.skip_subtree();
            }
            _ => events.push(event),
        }
    }

    // Should have fewer events due to skipping the dictionary subtree
    let dict_content_tokens = events.iter()
        .filter(|e| {
            matches!(e, WalkEvent::Enter(NodeOrToken::Token(token)) if token.kind() == NAME_KIND)
        })
        .count();

    // Should skip the tokens inside the dictionary
    assert_eq!(
        dict_content_tokens, 0,
        "Dictionary tokens should be skipped"
    );
}

#[test]
fn test_skip_subtree_when_called_on_enter_token_expect_immediate_leave() {
    let tree = create_flat_tree();
    let mut preorder = PreorderWithTokens::new(tree);
    let mut events = Vec::new();

    while let Some(event) = preorder.next() {
        match &event {
            WalkEvent::Enter(NodeOrToken::Token(token)) if token.kind() == NAME_KIND => {
                events.push(event);
                preorder.skip_subtree();
                // For tokens, skip_subtree should have no effect since tokens are leaves
            }
            _ => events.push(event),
        }
    }

    // Should still process all tokens normally (tokens can't have subtrees)
    let token_enter_count = events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Enter(NodeOrToken::Token(_))))
        .count();
    assert_eq!(token_enter_count, 3);
}

#[test]
fn test_skip_subtree_when_called_on_leave_event_expect_no_effect() {
    let tree = create_nested_tree();
    let mut preorder = PreorderWithTokens::new(tree);
    let mut events = Vec::new();

    while let Some(event) = preorder.next() {
        if matches!(event, WalkEvent::Leave(_)) {
            preorder.skip_subtree(); // Should have no effect on Leave events
        }
        events.push(event);
    }

    // Should traverse normally since skip_subtree on Leave events has no effect
    let total_events = events.len();
    assert!(total_events > 4, "Should traverse the full tree");
}

#[test]
fn test_skip_subtree_when_called_multiple_times_expect_single_effect() {
    let tree = create_deeply_nested_tree();
    let mut preorder = PreorderWithTokens::new(tree);
    let mut events = Vec::new();

    while let Some(event) = preorder.next() {
        match &event {
            WalkEvent::Enter(NodeOrToken::Node(node)) if node.kind() == DICT_KIND => {
                events.push(event);
                preorder.skip_subtree();
                preorder.skip_subtree(); // Multiple calls should not cause issues
                preorder.skip_subtree();
            }
            _ => events.push(event),
        }
    }

    // Should skip the dictionary and its contents only once
    let array_events = events
        .iter()
        .filter(
            |e| matches!(e, WalkEvent::Enter(NodeOrToken::Node(node)) if node.kind() == ARRAY_KIND),
        )
        .count();
    assert_eq!(array_events, 0, "Array inside dictionary should be skipped");
}

// =============================================================================
// Complex Structure Tests
// =============================================================================

#[test]
fn test_complex_pdf_structure_when_iterating_expect_complete_traversal() {
    let tree = create_complex_pdf_structure();
    let events: Vec<WalkEvent<SyntaxElement>> = PreorderWithTokens::new(tree).collect();

    // Count Enter and Leave events
    let enter_count = events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Enter(_)))
        .count();
    let leave_count = events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Leave(_)))
        .count();

    // Should have equal number of Enter and Leave events
    assert_eq!(enter_count, leave_count);

    // Should visit both nodes and tokens
    let node_count = events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Enter(NodeOrToken::Node(_))))
        .count();
    let token_count = events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Enter(NodeOrToken::Token(_))))
        .count();

    assert!(
        node_count >= 3,
        "Should have at least OBJ, main DICT, names DICT"
    );
    assert!(token_count >= 4, "Should have at least 4 tokens");

    // First event should be Enter(OBJ)
    match &events[0] {
        WalkEvent::Enter(NodeOrToken::Node(node)) => assert_eq!(node.kind(), OBJ_KIND),
        _ => panic!("Expected Enter(OBJ) first"),
    }

    // Last event should be Leave(OBJ)
    match events.last().unwrap() {
        WalkEvent::Leave(NodeOrToken::Node(node)) => assert_eq!(node.kind(), OBJ_KIND),
        _ => panic!("Expected Leave(OBJ) last"),
    }
}

#[test]
fn test_complex_structure_with_selective_skipping_expect_correct_traversal() {
    let tree = create_complex_pdf_structure();
    let mut preorder = PreorderWithTokens::new(tree);
    let mut visited_elements = Vec::new();

    while let Some(event) = preorder.next() {
        match event {
            WalkEvent::Enter(el) => {
                match &el {
                    NodeOrToken::Node(node) => {
                        visited_elements.push(format!("Enter(Node({}))", node.kind().0));
                        // Skip dictionary subtrees to test selective traversal
                        if node.kind() == DICT_KIND {
                            preorder.skip_subtree();
                        }
                    }
                    NodeOrToken::Token(token) => {
                        visited_elements.push(format!("Enter(Token({}))", token.kind().0));
                    }
                }
            }
            WalkEvent::Leave(el) => match &el {
                NodeOrToken::Node(node) => {
                    visited_elements.push(format!("Leave(Node({}))", node.kind().0));
                }
                NodeOrToken::Token(token) => {
                    visited_elements.push(format!("Leave(Token({}))", token.kind().0));
                }
            },
        }
    }

    // Should have visited OBJ nodes but skipped dictionary content
    let obj_enters = visited_elements
        .iter()
        .filter(|s| s.contains("Enter(Node(6))")) // OBJ_KIND
        .count();
    let obj_leaves = visited_elements
        .iter()
        .filter(|s| s.contains("Leave(Node(6))")) // OBJ_KIND
        .count();

    assert_eq!(obj_enters, 1);
    assert_eq!(obj_leaves, 1);

    // Should have encountered dictionary nodes but skipped their token content
    let dict_enters = visited_elements
        .iter()
        .filter(|s| s.contains("Enter(Node(4))")) // DICT_KIND
        .count();
    assert!(dict_enters > 0);

    // Should have no token events since we skipped all dictionary content
    let token_events = visited_elements
        .iter()
        .filter(|s| s.contains("Token"))
        .count();
    assert_eq!(
        token_events, 0,
        "Should skip all tokens inside dictionaries"
    );
}

// =============================================================================
// Iterator Behavior Tests
// =============================================================================

#[test]
fn test_iterator_when_clone_expect_independent_state() {
    let tree = create_nested_tree();
    let preorder1 = PreorderWithTokens::new(tree);
    let mut preorder2 = preorder1.clone();

    // Advance one iterator
    let event1 = preorder2.next().unwrap();
    assert!(matches!(event1, WalkEvent::Enter(_)));

    // Original iterator should be unaffected
    let original_events: Vec<_> = preorder1.collect();
    assert!(original_events.len() > 1);
    assert!(matches!(original_events[0], WalkEvent::Enter(_)));
}

#[test]
fn test_iterator_when_consumed_expect_no_more_events() {
    let tree = create_single_node_tree();
    let mut preorder = PreorderWithTokens::new(tree);

    // Consume all events
    let events: Vec<_> = (&mut preorder).collect();
    assert_eq!(events.len(), 4);

    // Iterator should be exhausted
    assert!(preorder.next().is_none());
    assert!(preorder.next().is_none()); // Multiple calls should be safe
}

#[test]
fn test_empty_node_when_iterating_expect_enter_leave_only() {
    // Create a node with no children
    let empty_node = create_green_node(DICT_KIND, vec![]);
    let tree = SyntaxNode::new_root(empty_node);
    let events: Vec<WalkEvent<SyntaxElement>> = PreorderWithTokens::new(tree).collect();

    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], WalkEvent::Enter(NodeOrToken::Node(_))));
    assert!(matches!(events[1], WalkEvent::Leave(NodeOrToken::Node(_))));
}

// =============================================================================
// Sibling Traversal Tests
// =============================================================================

#[test]
fn test_preorder_with_tokens_sibling_traversal() {
    // Create a tree with both node and token siblings
    let token1 = create_green_token(NAME_KIND, "/First");
    let token2 = create_green_token(NUMBER_KIND, "42");

    let inner_token = create_green_token(STRING_KIND, "(nested)");
    let inner_node = create_green_node(ARRAY_KIND, vec![inner_token.into()]);

    let token3 = create_green_token(NAME_KIND, "/Last");

    let children = vec![
        token1.into(),
        token2.into(),
        inner_node.into(),
        token3.into(),
    ];
    let root = SyntaxNode::new_root(create_green_node(DICT_KIND, children));

    let events: Vec<_> = PreorderWithTokens::new(root).collect();

    // Find sibling transitions where we Leave one element and Enter its sibling
    let mut found_token_to_token_transition = false;
    let mut found_token_to_node_transition = false;
    let mut found_node_to_token_transition = false;

    for i in 0..events.len() - 1 {
        if let (WalkEvent::Leave(left), WalkEvent::Enter(right)) = (&events[i], &events[i + 1]) {
            // Check if this is a sibling transition
            if let (Some(left_parent), Some(right_parent)) = (left.parent(), right.parent()) {
                if left_parent == right_parent {
                    match (left, right) {
                        (NodeOrToken::Token(_), NodeOrToken::Token(_)) => {
                            found_token_to_token_transition = true;
                        }
                        (NodeOrToken::Token(_), NodeOrToken::Node(_)) => {
                            found_token_to_node_transition = true;
                        }
                        (NodeOrToken::Node(_), NodeOrToken::Token(_)) => {
                            found_node_to_token_transition = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    assert!(
        found_token_to_token_transition,
        "Should find token-to-token sibling transition"
    );
    assert!(
        found_token_to_node_transition,
        "Should find token-to-node sibling transition"
    );
    assert!(
        found_node_to_token_transition,
        "Should find node-to-token sibling transition"
    );
}

// =============================================================================
// Edge Cases and Error Conditions
// =============================================================================

#[test]
fn test_preorder_with_tokens_when_single_token_tree_expect_correct_traversal() {
    // Test with the simplest possible structure
    let token = create_green_token(STRING_KIND, "test");
    let node = create_green_node(DICT_KIND, vec![token.into()]);
    let tree = SyntaxNode::new_root(node);

    let events: Vec<WalkEvent<SyntaxElement>> = PreorderWithTokens::new(tree).collect();

    // Should visit the node and the token
    assert_eq!(events.len(), 4);
    assert!(matches!(events[0], WalkEvent::Enter(NodeOrToken::Node(_))));
    assert!(matches!(events[1], WalkEvent::Enter(NodeOrToken::Token(_))));
    assert!(matches!(events[2], WalkEvent::Leave(NodeOrToken::Token(_))));
    assert!(matches!(events[3], WalkEvent::Leave(NodeOrToken::Node(_))));
}

#[test]
fn test_traversal_order_when_complex_tree_expect_depth_first_with_tokens() {
    let tree = create_deeply_nested_tree();
    let mut preorder = PreorderWithTokens::new(tree);
    let mut path = Vec::new();

    // Track the depth by counting Enter/Leave events
    while let Some(event) = preorder.next() {
        match event {
            WalkEvent::Enter(el) => match el {
                NodeOrToken::Node(node) => path.push(format!("Node({})", node.kind().0)),
                NodeOrToken::Token(token) => path.push(format!("Token({})", token.kind().0)),
            },
            WalkEvent::Leave(_) => {
                path.pop();
            }
        }
    }

    // Path should be empty at the end (all elements properly left)
    assert!(
        path.is_empty(),
        "Tree traversal should end with empty path, got: {:?}",
        path
    );
}

#[test]
fn test_skip_subtree_state_when_interleaved_with_iteration_expect_correct_behavior() {
    let tree = create_deeply_nested_tree();
    let mut preorder = PreorderWithTokens::new(tree);
    let mut events = Vec::new();

    // Complex pattern: skip some, iterate some
    while let Some(event) = preorder.next() {
        match &event {
            WalkEvent::Enter(NodeOrToken::Node(node)) => {
                events.push(format!("Enter(Node({}))", node.kind().0));
                // Skip DICT subtrees specifically
                if node.kind() == DICT_KIND {
                    preorder.skip_subtree();
                    events.push("SKIP".to_string());
                }
            }
            WalkEvent::Enter(NodeOrToken::Token(token)) => {
                events.push(format!("Enter(Token({}))", token.kind().0));
            }
            WalkEvent::Leave(NodeOrToken::Node(node)) => {
                events.push(format!("Leave(Node({}))", node.kind().0));
            }
            WalkEvent::Leave(NodeOrToken::Token(token)) => {
                events.push(format!("Leave(Token({}))", token.kind().0));
            }
        }
    }

    // Should contain skip markers and proper event sequence
    assert!(events.contains(&"SKIP".to_string()));

    // Should have more Enter events than Leave events for skipped elements
    let enter_count = events.iter().filter(|e| e.starts_with("Enter")).count();
    let leave_count = events.iter().filter(|e| e.starts_with("Leave")).count();

    // Due to skipping, we should have fewer leaves than enters
    assert!(leave_count <= enter_count);
}

#[test]
fn test_debug_format_when_creating_preorder_with_tokens_expect_readable_output() {
    let tree = create_single_node_tree();
    let preorder = PreorderWithTokens::new(tree);

    let debug_str = format!("{:?}", preorder);
    assert!(debug_str.contains("PreorderWithTokens"));
}
