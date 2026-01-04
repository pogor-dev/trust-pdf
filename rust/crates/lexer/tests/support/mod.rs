use lexer::Lexer;
use pretty_assertions::assert_eq;
use syntax::{GreenNode, GreenNodeBuilder, GreenToken, NodeOrToken, SyntaxKind};

/// Asserts that two green nodes have identical token streams and diagnostics.
pub fn assert_nodes_equal(actual: &GreenNode, expected: &GreenNode) {
    let actual_children: Vec<GreenToken> = actual
        .children()
        .filter_map(|child| match child {
            NodeOrToken::Token(token) => Some(token),
            _ => None,
        })
        .collect();

    let expected_children: Vec<GreenToken> = expected
        .children()
        .filter_map(|child| match child {
            NodeOrToken::Token(token) => Some(token),
            _ => None,
        })
        .collect();

    assert_eq!(actual_children, expected_children);

    // Also verify diagnostics equality at node and token levels.
    assert_eq!(actual.diagnostics(), expected.diagnostics());

    for (actual_tok, expected_tok) in actual_children.iter().zip(expected_children.iter()) {
        assert_eq!(actual_tok.diagnostics(), expected_tok.diagnostics());
    }
}

/// Rebuilds a lexer node from emitted tokens while preserving token-level diagnostics.
pub fn generate_node_from_lexer(lexer: &mut Lexer) -> GreenNode {
    const MAX_TOKENS: usize = 999;

    let tokens: Vec<_> = std::iter::from_fn(|| Some(lexer.next_token()))
        .take_while(|t| t.kind() != SyntaxKind::EndOfFileToken.into())
        .take(MAX_TOKENS + 1)
        .collect();

    if tokens.len() > MAX_TOKENS {
        println!("Lexer appears stuck: collected {} tokens (limit: {})", tokens.len(), MAX_TOKENS);
    }

    let mut builder = GreenNodeBuilder::new();
    builder.start_node(SyntaxKind::LexerNode.into());
    tokens.iter().for_each(|token| {
        builder.token(token.kind(), &token.bytes(), token.leading_trivia().pieces(), token.trailing_trivia().pieces());
        // Propagate token diagnostics into the rebuilt node so comparisons include them
        for diag in token.diagnostics() {
            builder.add_diagnostic(diag.severity, diag.code, diag.message).expect("Token already added");
        }
    });
    builder.finish_node();
    builder.finish()
}
