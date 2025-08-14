use crate::{NodeOrToken, utility_types::*};

#[test]
fn test_into_node_when_node_variant_expect_some_node() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Node("test_node");
    assert_eq!(node_or_token.into_node(), Some("test_node"));
}

#[test]
fn test_into_node_when_token_variant_expect_none() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Token("test_token");
    assert_eq!(node_or_token.into_node(), None);
}

#[test]
fn test_into_token_when_token_variant_expect_some_token() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Token("test_token");
    assert_eq!(node_or_token.into_token(), Some("test_token"));
}

#[test]
fn test_into_token_when_node_variant_expect_none() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Node("test_node");
    assert_eq!(node_or_token.into_token(), None);
}

#[test]
fn test_as_node_when_node_variant_expect_some_node_ref() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Node("test_node");
    assert_eq!(node_or_token.as_node(), Some(&"test_node"));
}

#[test]
fn test_as_node_when_token_variant_expect_none() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Token("test_token");
    assert_eq!(node_or_token.as_node(), None);
}

#[test]
fn test_as_token_when_token_variant_expect_some_token_ref() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Token("test_token");
    assert_eq!(node_or_token.as_token(), Some(&"test_token"));
}

#[test]
fn test_as_token_when_node_variant_expect_none() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Node("test_node");
    assert_eq!(node_or_token.as_token(), None);
}

#[test]
fn test_as_deref_when_node_variant_expect_dereferenced_node() {
    let node_data = String::from("test_node");
    let node_or_token: NodeOrToken<&String, &String> = NodeOrToken::Node(&node_data);
    let deref_result = node_or_token.as_deref();

    match deref_result {
        NodeOrToken::Node(node_str) => assert_eq!(node_str, "test_node"),
        NodeOrToken::Token(_) => panic!("Expected Node variant"),
    }
}

#[test]
fn test_as_deref_when_token_variant_expect_dereferenced_token() {
    let token_data = String::from("test_token");
    let node_or_token: NodeOrToken<&String, &String> = NodeOrToken::Token(&token_data);
    let deref_result = node_or_token.as_deref();

    match deref_result {
        NodeOrToken::Node(_) => panic!("Expected Token variant"),
        NodeOrToken::Token(token_str) => assert_eq!(token_str, "test_token"),
    }
}

#[test]
fn test_fmt_when_node_variant_expect_node_display() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Node("test_node");
    let formatted = format!("{}", node_or_token);
    assert_eq!(formatted, "test_node");
}

#[test]
fn test_fmt_when_token_variant_expect_token_display() {
    let node_or_token: NodeOrToken<&str, &str> = NodeOrToken::Token("test_token");
    let formatted = format!("{}", node_or_token);
    assert_eq!(formatted, "test_token");
}

// Delta tests
#[test]
fn test_delta_add_when_adding_value_expect_correct_result() {
    let mut value = 10u32;
    let delta = Delta::Add(5u32);
    value += delta;
    assert_eq!(value, 15);
}

#[test]
fn test_delta_sub_when_subtracting_value_expect_correct_result() {
    let mut value = 10u32;
    let delta = Delta::Sub(3u32);
    value += delta;
    assert_eq!(value, 7);
}

#[test]
fn test_delta_debug_when_formatting_expect_readable_output() {
    let delta_add = Delta::Add(5u32);
    let delta_sub = Delta::Sub(3u32);
    assert_eq!(format!("{:?}", delta_add), "Add(5)");
    assert_eq!(format!("{:?}", delta_sub), "Sub(3)");
}

#[test]
fn test_delta_clone_when_copying_expect_same_values() {
    let delta = Delta::Add(42u32);
    let cloned = delta.clone();
    match (delta, cloned) {
        (Delta::Add(a), Delta::Add(b)) => assert_eq!(a, b),
        _ => panic!("Clone should produce identical delta"),
    }
}

// TokenAtOffset tests
#[test]
fn test_token_at_offset_map_when_none_expect_none() {
    let token_offset: TokenAtOffset<i32> = TokenAtOffset::None;
    let result = token_offset.map(|x| x * 2);
    match result {
        TokenAtOffset::None => (),
        _ => panic!("Expected None variant"),
    }
}

#[test]
fn test_token_at_offset_map_when_single_expect_mapped_single() {
    let token_offset = TokenAtOffset::Single(5);
    let result = token_offset.map(|x| x * 2);
    match result {
        TokenAtOffset::Single(value) => assert_eq!(value, 10),
        _ => panic!("Expected Single variant"),
    }
}

#[test]
fn test_token_at_offset_map_when_between_expect_mapped_between() {
    let token_offset = TokenAtOffset::Between(3, 7);
    let result = token_offset.map(|x| x * 2);
    match result {
        TokenAtOffset::Between(left, right) => {
            assert_eq!(left, 6);
            assert_eq!(right, 14);
        }
        _ => panic!("Expected Between variant"),
    }
}

#[test]
fn test_token_at_offset_right_biased_when_none_expect_none() {
    let token_offset: TokenAtOffset<&str> = TokenAtOffset::None;
    assert_eq!(token_offset.right_biased(), None);
}

#[test]
fn test_token_at_offset_right_biased_when_single_expect_value() {
    let token_offset = TokenAtOffset::Single("token");
    assert_eq!(token_offset.right_biased(), Some("token"));
}

#[test]
fn test_token_at_offset_right_biased_when_between_expect_right_value() {
    let token_offset = TokenAtOffset::Between("left", "right");
    assert_eq!(token_offset.right_biased(), Some("right"));
}

#[test]
fn test_token_at_offset_left_biased_when_none_expect_none() {
    let token_offset: TokenAtOffset<&str> = TokenAtOffset::None;
    assert_eq!(token_offset.left_biased(), None);
}

#[test]
fn test_token_at_offset_left_biased_when_single_expect_value() {
    let token_offset = TokenAtOffset::Single("token");
    assert_eq!(token_offset.left_biased(), Some("token"));
}

#[test]
fn test_token_at_offset_left_biased_when_between_expect_left_value() {
    let token_offset = TokenAtOffset::Between("left", "right");
    assert_eq!(token_offset.left_biased(), Some("left"));
}

#[test]
fn test_token_at_offset_iterator_when_none_expect_no_items() {
    let mut token_offset: TokenAtOffset<i32> = TokenAtOffset::None;
    assert_eq!(token_offset.next(), None);
    assert_eq!(token_offset.next(), None); // Should remain None
}

#[test]
fn test_token_at_offset_iterator_when_single_expect_one_item() {
    let mut token_offset = TokenAtOffset::Single(42);
    assert_eq!(token_offset.next(), Some(42));
    assert_eq!(token_offset.next(), None);
    assert_eq!(token_offset.next(), None); // Should remain None
}

#[test]
fn test_token_at_offset_iterator_when_between_expect_two_items() {
    let mut token_offset = TokenAtOffset::Between(10, 20);
    assert_eq!(token_offset.next(), Some(10)); // Left first
    assert_eq!(token_offset.next(), Some(20)); // Then right
    assert_eq!(token_offset.next(), None);
    assert_eq!(token_offset.next(), None); // Should remain None
}

#[test]
fn test_token_at_offset_size_hint_when_none_expect_zero() {
    let token_offset: TokenAtOffset<i32> = TokenAtOffset::None;
    assert_eq!(token_offset.size_hint(), (0, Some(0)));
}

#[test]
fn test_token_at_offset_size_hint_when_single_expect_one() {
    let token_offset = TokenAtOffset::Single(42);
    assert_eq!(token_offset.size_hint(), (1, Some(1)));
}

#[test]
fn test_token_at_offset_size_hint_when_between_expect_two() {
    let token_offset = TokenAtOffset::Between(10, 20);
    assert_eq!(token_offset.size_hint(), (2, Some(2)));
}

#[test]
fn test_token_at_offset_exact_size_iterator_when_collecting_expect_correct_count() {
    let token_offset = TokenAtOffset::Between("a", "b");
    let collected: Vec<_> = token_offset.collect();
    assert_eq!(collected.len(), 2);
    assert_eq!(collected, vec!["a", "b"]);
}

#[test]
fn test_token_at_offset_clone_when_copying_expect_same_content() {
    let original = TokenAtOffset::Between(1, 2);
    let cloned = original.clone();

    match (original, cloned) {
        (TokenAtOffset::Between(a1, b1), TokenAtOffset::Between(a2, b2)) => {
            assert_eq!(a1, a2);
            assert_eq!(b1, b2);
        }
        _ => panic!("Clone should produce identical TokenAtOffset"),
    }
}

#[test]
fn test_token_at_offset_debug_when_formatting_expect_readable_output() {
    let none: TokenAtOffset<i32> = TokenAtOffset::None;
    let single = TokenAtOffset::Single(42);
    let between = TokenAtOffset::Between(1, 2);

    assert_eq!(format!("{:?}", none), "None");
    assert_eq!(format!("{:?}", single), "Single(42)");
    assert_eq!(format!("{:?}", between), "Between(1, 2)");
}

// Direction tests
#[test]
fn test_direction_variants_when_creating_expect_correct_values() {
    let next = Direction::Next;
    let prev = Direction::Prev;

    // Test Debug trait
    assert_eq!(format!("{:?}", next), "Next");
    assert_eq!(format!("{:?}", prev), "Prev");
}

#[test]
fn test_direction_equality_when_comparing_expect_correct_results() {
    assert_eq!(Direction::Next, Direction::Next);
    assert_eq!(Direction::Prev, Direction::Prev);
    assert_ne!(Direction::Next, Direction::Prev);
}

#[test]
fn test_direction_ordering_when_comparing_expect_consistent_order() {
    assert!(Direction::Next < Direction::Prev);
    assert!(Direction::Prev > Direction::Next);
}

#[test]
fn test_direction_clone_when_copying_expect_same_value() {
    let original = Direction::Next;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_direction_copy_when_assigning_expect_independent_copies() {
    let original = Direction::Prev;
    let copied = original; // Copy trait
    assert_eq!(original, copied);
}

// WalkEvent tests
#[test]
fn test_walk_event_map_when_enter_expect_mapped_enter() {
    let event = WalkEvent::Enter(5);
    let result = event.map(|x| x * 2);
    match result {
        WalkEvent::Enter(value) => assert_eq!(value, 10),
        _ => panic!("Expected Enter variant"),
    }
}

#[test]
fn test_walk_event_map_when_leave_expect_mapped_leave() {
    let event = WalkEvent::Leave("test");
    let result = event.map(|s| s.len());
    match result {
        WalkEvent::Leave(length) => assert_eq!(length, 4),
        _ => panic!("Expected Leave variant"),
    }
}

#[test]
fn test_walk_event_debug_when_formatting_expect_readable_output() {
    let enter = WalkEvent::Enter(42);
    let leave = WalkEvent::Leave("node");

    assert_eq!(format!("{:?}", enter), "Enter(42)");
    assert_eq!(format!("{:?}", leave), "Leave(\"node\")");
}

#[test]
fn test_walk_event_copy_when_assigning_expect_independent_copies() {
    let original = WalkEvent::Enter(100);
    let copied = original; // Copy trait
    match (original, copied) {
        (WalkEvent::Enter(a), WalkEvent::Enter(b)) => assert_eq!(a, b),
        _ => panic!("Copy should produce identical WalkEvent"),
    }
}

#[test]
fn test_walk_event_clone_when_copying_expect_same_content() {
    let original = WalkEvent::Leave("data");
    let cloned = original.clone();
    match (original, cloned) {
        (WalkEvent::Leave(a), WalkEvent::Leave(b)) => assert_eq!(a, b),
        _ => panic!("Clone should produce identical WalkEvent"),
    }
}
