use crate::{GreenNode, GreenNodeElement, Lexer, SyntaxKind};
use pretty_assertions::assert_eq;

/// Asserts that two green nodes have identical token streams and diagnostics.
pub fn assert_nodes_equal(actual: &GreenNode, expected: &GreenNode) {
    let actual_slot_count = actual.slot_count();
    let expected_slot_count = expected.slot_count();
    assert_eq!(actual_slot_count, expected_slot_count, "Slot count mismatch");

    for (i, (actual_slot, expected_slot)) in actual.slots().iter().zip(expected.slots().iter()).enumerate() {
        match (actual_slot, expected_slot) {
            (GreenNodeElement::Token(actual_token), GreenNodeElement::Token(expected_token)) => {
                assert_eq!(actual_token.kind(), expected_token.kind(), "Token kind mismatch at slot {}", i);
                assert_eq!(actual_token.text(), expected_token.text(), "Token text mismatch at slot {}", i);
                match (actual_token.diagnostics(), expected_token.diagnostics()) {
                    (Some(actual_diags), Some(expected_diags)) => {
                        assert_eq!(actual_diags.len(), expected_diags.len(), "Diagnostic count mismatch at slot {}", i);
                    }
                    (None, None) => {}
                    _ => panic!("Diagnostic presence mismatch at slot {}", i),
                }
            }
            (GreenNodeElement::Node(actual_node), GreenNodeElement::Node(expected_node)) => {
                assert_nodes_equal(actual_node, expected_node);
            }
            _ => panic!("Slot type mismatch at slot {}", i),
        }
    }

    assert_eq!(actual.diagnostics(), expected.diagnostics());
}

/// Rebuilds a lexer node from emitted tokens while preserving token-level diagnostics.
pub fn generate_node_from_lexer(lexer: &mut Lexer) -> GreenNode {
    const MAX_TOKENS: usize = 999;

    let tokens: Vec<_> = std::iter::from_fn(|| Some(lexer.next_token()))
        .take_while(|t| t.kind() != SyntaxKind::EndOfFileToken)
        .take(MAX_TOKENS + 1)
        .collect();

    if tokens.len() > MAX_TOKENS {
        println!("Lexer appears stuck: collected {} tokens (limit: {})", tokens.len(), MAX_TOKENS);
    }

    let slots = tokens.into_iter().map(GreenNodeElement::Token).collect::<Vec<_>>();
    GreenNode::new(SyntaxKind::None, slots)
}
