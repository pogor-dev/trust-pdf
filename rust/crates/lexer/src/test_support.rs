use crate::Lexer;
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

pub fn assert_numeric_literal_token(token: &GreenToken, expected_kind: SyntaxKind, expected_bytes: &[u8]) {
    let actual_node = generate_lexer_node_tree(token);
    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (expected_kind.into(), expected_bytes)
        }
    };

    let actual_token = actual_node.children().next().unwrap();
    let expected_token = expected_node.children().next().unwrap();
    assert_eq!(format!("{:?}", actual_token), format!("{:?}", expected_token));
    assert_eq!(actual_node, expected_node);
}

pub fn assert_eof_token(token: &GreenToken) {
    let actual_node = generate_lexer_node_tree(token);
    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::EndOfFileToken.into(), b"")
        }
    };

    let actual_token = actual_node.children().next().unwrap();
    let expected_token = expected_node.children().next().unwrap();
    assert_eq!(format!("{:?}", actual_token), format!("{:?}", expected_token));
    assert_eq!(actual_node, expected_node);
}

fn generate_lexer_node_tree(token: &GreenToken) -> GreenNode {
    tree! {
        SyntaxKind::LexerNode.into() => {
            (token.kind(), &token.bytes())
        }
    }
}
