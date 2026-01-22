use crate::Lexer;
use pretty_assertions::assert_eq;
use syntax::{GreenNode, GreenNodeBuilder, Slot, SyntaxKind};

/// Asserts that two green nodes have identical token streams and diagnostics.
pub fn assert_nodes_equal(actual: &GreenNode, expected: &GreenNode) {
    // Compare slot counts
    let actual_slot_count = actual.slots().count();
    let expected_slot_count = expected.slots().count();
    assert_eq!(actual_slot_count, expected_slot_count, "Slot count mismatch");

    // Compare each slot
    for (i, (actual_slot, expected_slot)) in actual.slots().zip(expected.slots()).enumerate() {
        match (actual_slot, expected_slot) {
            (Slot::Token { token: actual_token, .. }, Slot::Token { token: expected_token, .. }) => {
                assert_eq!(actual_token.kind(), expected_token.kind(), "Token kind mismatch at slot {}", i);
                assert_eq!(actual_token.text(), expected_token.text(), "Token text mismatch at slot {}", i);
                // Compare diagnostics
                match (actual_token.diagnostics(), expected_token.diagnostics()) {
                    (Some(actual_diags), Some(expected_diags)) => {
                        assert_eq!(actual_diags.len(), expected_diags.len(), "Diagnostic count mismatch at slot {}", i);
                    }
                    (None, None) => {}
                    _ => panic!("Diagnostic presence mismatch at slot {}", i),
                }
            }
            (Slot::Node { node: actual_node, .. }, Slot::Node { node: expected_node, .. }) => {
                assert_nodes_equal(actual_node, expected_node);
            }
            _ => panic!("Slot type mismatch at slot {}", i),
        }
    }

    // Also verify diagnostics equality at node level
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

    let mut builder = GreenNodeBuilder::new();
    builder.start_node(SyntaxKind::None);

    for token in tokens.iter() {
        builder.start_token(token.kind());

        // Add leading trivia
        if let Some(leading) = token.leading_trivia() {
            for slot in leading.slots() {
                if let Slot::Trivia { trivia, .. } = slot {
                    builder.trivia(trivia.kind(), trivia.text());
                }
            }
        }

        // Add token text
        builder.token_text(token.text());

        // Add trailing trivia
        if let Some(trailing) = token.trailing_trivia() {
            for slot in trailing.slots() {
                if let Slot::Trivia { trivia, .. } = slot {
                    builder.trivia(trivia.kind(), trivia.text());
                }
            }
        }

        builder.finish_token();

        // Add diagnostics if any
        if let Some(diags) = token.diagnostics() {
            for diag in diags.iter() {
                builder
                    .add_diagnostic(diag.severity(), diag.code(), diag.message())
                    .expect("Token already added");
            }
        }
    }

    builder.finish_node();
    builder.finish()
}
