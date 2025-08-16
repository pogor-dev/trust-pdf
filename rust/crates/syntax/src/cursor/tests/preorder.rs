use crate::{
    cursor::{node::SyntaxNode, preorder::Preorder},
    utility_types::WalkEvent,
};

use super::fixtures::{
    ARRAY_KIND,
    DICT_KIND,
    NAME_KIND,
    NUMBER_KIND,
    OBJ_KIND,
    STREAM_KIND,
    // Common constants
    STRING_KIND,
    create_complex_pdf_structure,
    create_deeply_nested_tree,
    create_flat_tree,
    create_green_node,
    // Common helper functions
    create_green_token,
    create_nested_tree,
    // Common tree creation functions
    create_single_node_tree,
};

// Local specialized fixtures for this test file
fn create_sibling_tree_for_preorder_test() -> SyntaxNode {
    // Create a tree with siblings to test the next_sibling() branch
    // Structure: root -> [child1, child2, child3]
    let child1_token = create_green_token(STRING_KIND, "(string1)");
    let child2_token = create_green_token(NAME_KIND, "/name2");
    let child3_token = create_green_token(NUMBER_KIND, "123");

    let child1 = create_green_node(DICT_KIND, vec![child1_token.into()]);
    let child2 = create_green_node(DICT_KIND, vec![child2_token.into()]);
    let child3 = create_green_node(DICT_KIND, vec![child3_token.into()]);

    let children = vec![child1.into(), child2.into(), child3.into()];
    SyntaxNode::new_root(create_green_node(ARRAY_KIND, children))
}

// =============================================================================
// Basic Preorder Iterator Tests
// =============================================================================

#[test]
fn test_new_when_creating_preorder_expect_correct_initial_state() {
    let tree = create_single_node_tree();
    let preorder = Preorder::new(tree.clone());

    // Should be able to create preorder iterator
    assert_eq!(format!("{:?}", preorder).contains("Preorder"), true);
}

#[test]
fn test_single_node_when_iterating_expect_enter_and_leave_events() {
    let tree = create_single_node_tree();
    let events: Vec<WalkEvent<SyntaxNode>> = tree.preorder().collect();

    assert_eq!(events.len(), 2);

    match &events[0] {
        WalkEvent::Enter(node) => {
            assert_eq!(node.kind(), DICT_KIND);
        }
        WalkEvent::Leave(_) => panic!("Expected Enter event first"),
    }

    match &events[1] {
        WalkEvent::Leave(node) => {
            assert_eq!(node.kind(), DICT_KIND);
        }
        WalkEvent::Enter(_) => panic!("Expected Leave event second"),
    }
}

#[test]
fn test_flat_tree_when_iterating_expect_all_children_visited() {
    let tree = create_flat_tree();
    let events: Vec<WalkEvent<SyntaxNode>> = tree.preorder().collect();

    // Should have: Enter(ARRAY), Enter(child1), Leave(child1), Enter(child2), Leave(child2),
    // Enter(child3), Leave(child3), Leave(ARRAY)
    // But since tokens are leaf nodes, we get: Enter(ARRAY), Leave(ARRAY)
    assert_eq!(events.len(), 2);

    match &events[0] {
        WalkEvent::Enter(node) => {
            assert_eq!(node.kind(), ARRAY_KIND);
        }
        WalkEvent::Leave(_) => panic!("Expected Enter event first"),
    }

    match &events[1] {
        WalkEvent::Leave(node) => {
            assert_eq!(node.kind(), ARRAY_KIND);
        }
        WalkEvent::Enter(_) => panic!("Expected Leave event second"),
    }
}

#[test]
fn test_nested_tree_when_iterating_expect_depth_first_traversal() {
    let tree = create_nested_tree();
    let events: Vec<WalkEvent<SyntaxNode>> = tree.preorder().collect();

    // Expected order: Enter(OBJ), Enter(DICT), Leave(DICT), Leave(OBJ)
    assert_eq!(events.len(), 4);

    match &events[0] {
        WalkEvent::Enter(node) => assert_eq!(node.kind(), OBJ_KIND),
        WalkEvent::Leave(_) => panic!("Expected Enter(OBJ) first"),
    }

    match &events[1] {
        WalkEvent::Enter(node) => assert_eq!(node.kind(), DICT_KIND),
        WalkEvent::Leave(_) => panic!("Expected Enter(DICT) second"),
    }

    match &events[2] {
        WalkEvent::Leave(node) => assert_eq!(node.kind(), DICT_KIND),
        WalkEvent::Enter(_) => panic!("Expected Leave(DICT) third"),
    }

    match &events[3] {
        WalkEvent::Leave(node) => assert_eq!(node.kind(), OBJ_KIND),
        WalkEvent::Enter(_) => panic!("Expected Leave(OBJ) fourth"),
    }
}

#[test]
fn test_deeply_nested_tree_when_iterating_expect_correct_order() {
    let tree = create_deeply_nested_tree();
    let events: Vec<WalkEvent<SyntaxNode>> = tree.preorder().collect();

    // Expected traversal order:
    // Enter(OBJ) -> Enter(STREAM) -> Enter(DICT) -> Enter(ARRAY) ->
    // Leave(ARRAY) -> Leave(DICT) -> Leave(STREAM) -> Leave(OBJ)
    assert_eq!(events.len(), 8);

    let expected_kinds = [
        (true, OBJ_KIND),     // Enter OBJ
        (true, STREAM_KIND),  // Enter STREAM
        (true, DICT_KIND),    // Enter DICT
        (true, ARRAY_KIND),   // Enter ARRAY
        (false, ARRAY_KIND),  // Leave ARRAY
        (false, DICT_KIND),   // Leave DICT
        (false, STREAM_KIND), // Leave STREAM
        (false, OBJ_KIND),    // Leave OBJ
    ];

    for (i, event) in events.iter().enumerate() {
        let (is_enter, expected_kind) = expected_kinds[i];
        match (event, is_enter) {
            (WalkEvent::Enter(node), true) => {
                assert_eq!(
                    node.kind(),
                    expected_kind,
                    "Event {}: Expected Enter({:?})",
                    i,
                    expected_kind
                );
            }
            (WalkEvent::Leave(node), false) => {
                assert_eq!(
                    node.kind(),
                    expected_kind,
                    "Event {}: Expected Leave({:?})",
                    i,
                    expected_kind
                );
            }
            _ => panic!("Event {}: Unexpected event type", i),
        }
    }
}

// =============================================================================
// Skip Subtree Tests
// =============================================================================

#[test]
fn test_skip_subtree_when_called_on_enter_expect_subtree_skipped() {
    let tree = create_nested_tree();
    let mut preorder = tree.preorder();

    // Get first event (should be Enter(OBJ))
    let first_event = preorder.next().unwrap();
    match first_event {
        WalkEvent::Enter(node) => {
            assert_eq!(node.kind(), OBJ_KIND);
            // Skip the subtree
            preorder.skip_subtree();
        }
        WalkEvent::Leave(_) => panic!("Expected Enter event first"),
    }

    // Next event should be Leave(OBJ), skipping the inner dictionary
    let second_event = preorder.next().unwrap();
    match second_event {
        WalkEvent::Leave(node) => {
            assert_eq!(node.kind(), OBJ_KIND);
        }
        WalkEvent::Enter(_) => panic!("Expected Leave event after skip_subtree"),
    }

    // Should be no more events
    assert!(preorder.next().is_none());
}

#[test]
fn test_skip_subtree_when_called_on_deeply_nested_expect_correct_skipping() {
    let tree = create_deeply_nested_tree();
    let mut preorder = tree.preorder();

    // Enter OBJ
    let first = preorder.next().unwrap();
    assert!(matches!(first, WalkEvent::Enter(_)));

    // Enter STREAM - skip this subtree
    let second = preorder.next().unwrap();
    match second {
        WalkEvent::Enter(node) => {
            assert_eq!(node.kind(), STREAM_KIND);
            preorder.skip_subtree();
        }
        WalkEvent::Leave(_) => panic!("Expected Enter(STREAM)"),
    }

    // Should jump to Leave(STREAM), skipping all nested content
    let third = preorder.next().unwrap();
    match third {
        WalkEvent::Leave(node) => {
            assert_eq!(node.kind(), STREAM_KIND);
        }
        WalkEvent::Enter(_) => panic!("Expected Leave(STREAM) after skipping"),
    }

    // Then Leave(OBJ)
    let fourth = preorder.next().unwrap();
    match fourth {
        WalkEvent::Leave(node) => {
            assert_eq!(node.kind(), OBJ_KIND);
        }
        WalkEvent::Enter(_) => panic!("Expected Leave(OBJ)"),
    }

    // Should be done
    assert!(preorder.next().is_none());
}

#[test]
fn test_skip_subtree_when_called_on_leaf_expect_normal_progression() {
    let tree = create_single_node_tree();
    let mut preorder = tree.preorder();

    // Get Enter event
    let first = preorder.next().unwrap();
    match first {
        WalkEvent::Enter(_) => {
            preorder.skip_subtree(); // This should have no effect on a leaf
        }
        WalkEvent::Leave(_) => panic!("Expected Enter event first"),
    }

    // Should still get Leave event normally
    let second = preorder.next().unwrap();
    assert!(matches!(second, WalkEvent::Leave(_)));

    // Should be done
    assert!(preorder.next().is_none());
}

#[test]
fn test_skip_subtree_when_called_multiple_times_expect_single_effect() {
    let tree = create_nested_tree();
    let mut preorder = tree.preorder();

    // Enter OBJ
    let first = preorder.next().unwrap();
    match first {
        WalkEvent::Enter(_) => {
            preorder.skip_subtree();
            preorder.skip_subtree(); // Multiple calls should be safe
            preorder.skip_subtree();
        }
        WalkEvent::Leave(_) => panic!("Expected Enter event"),
    }

    // Should skip to Leave(OBJ)
    let second = preorder.next().unwrap();
    assert!(matches!(second, WalkEvent::Leave(_)));
    assert!(preorder.next().is_none());
}

#[test]
fn test_skip_subtree_when_called_on_leave_event_expect_no_effect() {
    let tree = create_nested_tree();
    let mut preorder = tree.preorder();

    // Consume all Enter events until we get to a Leave
    let enter_obj = preorder.next().unwrap();
    assert!(matches!(enter_obj, WalkEvent::Enter(_)));

    let enter_dict = preorder.next().unwrap();
    assert!(matches!(enter_dict, WalkEvent::Enter(_)));

    let leave_dict = preorder.next().unwrap();
    match leave_dict {
        WalkEvent::Leave(_) => {
            preorder.skip_subtree(); // Should have no effect on Leave events
        }
        WalkEvent::Enter(_) => panic!("Expected Leave event"),
    }

    // Should continue normally to Leave(OBJ)
    let leave_obj = preorder.next().unwrap();
    assert!(matches!(leave_obj, WalkEvent::Leave(_)));
    assert!(preorder.next().is_none());
}

// =============================================================================
// Complex Structure Tests
// =============================================================================

#[test]
fn test_complex_pdf_structure_when_iterating_expect_complete_traversal() {
    let tree = create_complex_pdf_structure();
    let events: Vec<WalkEvent<SyntaxNode>> = tree.preorder().collect();

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

    // Should have at least the main structure nodes: OBJ, main DICT, names DICT
    assert!(enter_count >= 3);

    // First event should be Enter(OBJ)
    match &events[0] {
        WalkEvent::Enter(node) => assert_eq!(node.kind(), OBJ_KIND),
        WalkEvent::Leave(_) => panic!("Expected Enter(OBJ) first"),
    }

    // Last event should be Leave(OBJ)
    match events.last().unwrap() {
        WalkEvent::Leave(node) => assert_eq!(node.kind(), OBJ_KIND),
        WalkEvent::Enter(_) => panic!("Expected Leave(OBJ) last"),
    }
}

#[test]
fn test_complex_structure_with_selective_skipping_expect_correct_traversal() {
    let tree = create_complex_pdf_structure();
    let mut preorder = tree.preorder();
    let mut visited_kinds = Vec::new();

    while let Some(event) = preorder.next() {
        match event {
            WalkEvent::Enter(node) => {
                visited_kinds.push((true, node.kind()));
                // Skip dictionary subtrees to test selective traversal
                if node.kind() == DICT_KIND {
                    preorder.skip_subtree();
                }
            }
            WalkEvent::Leave(node) => {
                visited_kinds.push((false, node.kind()));
            }
        }
    }

    // Should have visited OBJ nodes but skipped internal dictionary content
    let obj_enters = visited_kinds
        .iter()
        .filter(|(is_enter, kind)| *is_enter && *kind == OBJ_KIND)
        .count();
    let obj_leaves = visited_kinds
        .iter()
        .filter(|(is_enter, kind)| !*is_enter && *kind == OBJ_KIND)
        .count();

    assert_eq!(obj_enters, 1);
    assert_eq!(obj_leaves, 1);

    // Should have encountered dictionary nodes but skipped their content
    let dict_enters = visited_kinds
        .iter()
        .filter(|(is_enter, kind)| *is_enter && *kind == DICT_KIND)
        .count();
    assert!(dict_enters > 0);
}

// =============================================================================
// Iterator Behavior Tests
// =============================================================================

#[test]
fn test_iterator_when_clone_expect_independent_state() {
    let tree = create_nested_tree();
    let preorder1 = tree.preorder();
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
    let mut preorder = tree.preorder();

    // Consume all events
    let events: Vec<_> = (&mut preorder).collect();
    assert_eq!(events.len(), 2);

    // Iterator should be exhausted
    assert!(preorder.next().is_none());
    assert!(preorder.next().is_none()); // Multiple calls should be safe
}

#[test]
fn test_empty_subtree_when_iterating_expect_enter_leave_only() {
    // Create a node with no children (effectively empty)
    let empty_node = create_green_node(DICT_KIND, vec![]);
    let tree = SyntaxNode::new_root(empty_node);
    let events: Vec<WalkEvent<SyntaxNode>> = tree.preorder().collect();

    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], WalkEvent::Enter(_)));
    assert!(matches!(events[1], WalkEvent::Leave(_)));
}

// =============================================================================
// Edge Cases and Error Conditions
// =============================================================================

#[test]
fn test_preorder_when_single_token_node_expect_correct_traversal() {
    // Test with the simplest possible structure
    let token = create_green_token(STRING_KIND, "test");
    let node = create_green_node(DICT_KIND, vec![token.into()]);
    let tree = SyntaxNode::new_root(node);

    let events: Vec<WalkEvent<SyntaxNode>> = tree.preorder().collect();

    // Should visit the node but not the token (tokens are not nodes)
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], WalkEvent::Enter(_)));
    assert!(matches!(events[1], WalkEvent::Leave(_)));
}

#[test]
fn test_preorder_traversal_order_when_complex_tree_expect_depth_first() {
    let tree = create_deeply_nested_tree();
    let mut preorder = tree.preorder();
    let mut path = Vec::new();

    // Track the depth by counting Enter/Leave events
    while let Some(event) = preorder.next() {
        match event {
            WalkEvent::Enter(node) => {
                path.push(node.kind());
            }
            WalkEvent::Leave(_) => {
                path.pop();
            }
        }
    }

    // Path should be empty at the end (all nodes properly left)
    assert!(
        path.is_empty(),
        "Tree traversal should end with empty path, got: {:?}",
        path
    );
}

#[test]
fn test_skip_subtree_state_when_interleaved_with_iteration_expect_correct_behavior() {
    let tree = create_deeply_nested_tree();
    let mut preorder = tree.preorder();
    let mut events = Vec::new();

    // Complex pattern: skip some, iterate some
    while let Some(event) = preorder.next() {
        match &event {
            WalkEvent::Enter(node) => {
                events.push(format!("Enter({})", node.kind().0));
                // Skip DICT subtrees specifically
                if node.kind() == DICT_KIND {
                    preorder.skip_subtree();
                    events.push("SKIP".to_string());
                }
            }
            WalkEvent::Leave(node) => {
                events.push(format!("Leave({})", node.kind().0));
            }
        }
    }

    // Should contain skip markers and proper event sequence
    assert!(events.contains(&"SKIP".to_string()));

    // Should have more Enter events than Leave events for skipped nodes
    let enter_count = events.iter().filter(|e| e.starts_with("Enter")).count();
    let leave_count = events.iter().filter(|e| e.starts_with("Leave")).count();

    // Due to skipping, we should have fewer leaves than enters
    assert!(leave_count <= enter_count);
}

#[test]
fn test_debug_format_when_creating_preorder_expect_readable_output() {
    let tree = create_single_node_tree();
    let preorder = tree.preorder();

    let debug_str = format!("{:?}", preorder);
    assert!(debug_str.contains("Preorder"));
}

#[test]
fn test_preorder_sibling_traversal() {
    // Create a tree with siblings to test the next_sibling() branch
    // Structure: root -> [child1, child2, child3]
    let root = create_sibling_tree_for_preorder_test();

    let preorder = root.preorder();
    let events: Vec<_> = preorder.collect();

    // Expected traversal includes all nodes and their children
    // Find sibling transitions where we Leave one child and Enter its sibling
    let mut found_sibling_transition = false;

    for i in 0..events.len() - 1 {
        if let (WalkEvent::Leave(left), WalkEvent::Enter(right)) = (&events[i], &events[i + 1]) {
            // Check if this is a sibling transition (same parent, different children)
            if let (Some(left_parent), Some(right_parent)) = (left.parent(), right.parent()) {
                if left_parent == right_parent && left != right {
                    found_sibling_transition = true;
                    break;
                }
            }
        }
    }

    assert!(
        found_sibling_transition,
        "Should find at least one sibling transition"
    );

    // Verify we traverse all three child nodes
    let enter_dict_count = events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Enter(node) if node.kind() == DICT_KIND))
        .count();
    assert_eq!(enter_dict_count, 3, "Should enter all three DICT children");
}
