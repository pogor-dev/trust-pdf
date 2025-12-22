use lexer::Lexer;
use syntax::{GreenNode, GreenNodeBuilder, GreenToken, NodeOrToken, SyntaxKind, tree};

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
}

pub fn generate_node_from_lexer(lexer: &mut Lexer) -> GreenNode {
    let tokens: Vec<_> = std::iter::from_fn(|| Some(lexer.next_token()))
        .take_while(|t| t.kind() != SyntaxKind::EndOfFileToken.into())
        .collect();

    let mut builder = GreenNodeBuilder::new();
    builder.start_node(SyntaxKind::LexerNode.into());
    tokens.iter().for_each(|token| {
        builder.token(token.kind(), &token.bytes(), token.leading_trivia().pieces(), token.trailing_trivia().pieces());
    });
    builder.finish_node();
    builder.finish()
}
