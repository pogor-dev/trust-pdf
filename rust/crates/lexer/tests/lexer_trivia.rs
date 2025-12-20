mod lexer_test_support;

use lexer::Lexer;
use syntax::{SyntaxKind, tree};
use lexer_test_support::{assert_nodes_equal, generate_node_from_lexer};

#[test]
fn test_trivia_single_space() {
    let mut lexer = Lexer::new(b"009 345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into()) => {
                text(b"009"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" "),
            },
            (SyntaxKind::NumericLiteralToken.into(), b"345")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_trivia_multiple_spaces() {
    let mut lexer = Lexer::new(b"009       345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into()) => {
                text(b"009"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b"       "),
            },
            (SyntaxKind::NumericLiteralToken.into(), b"345")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_trivia_different_whitespaces() {
    let mut lexer = Lexer::new(b"\r\0009 \t \x0C\r\n345\0\t\x0C \n");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into()) => {
                trivia(SyntaxKind::EndOfLineTrivia.into(), b"\r"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b"\0"),
                text(b"009"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" \t \x0C"),
                trivia(SyntaxKind::EndOfLineTrivia.into(), b"\r\n"),
            },
            (SyntaxKind::NumericLiteralToken.into()) => {
                text(b"345"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b"\0\t\x0C "),
                trivia(SyntaxKind::EndOfLineTrivia.into(), b"\n"),
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_trivia_comments() {
    let mut lexer = Lexer::new(b"% This is a comment\n009 % Another comment\r\n345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into()) => {
                trivia(SyntaxKind::CommentTrivia.into(), b"% This is a comment"),
                trivia(SyntaxKind::EndOfLineTrivia.into(), b"\n"),
                text(b"009"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" "),
                trivia(SyntaxKind::CommentTrivia.into(), b"% Another comment"),
                trivia(SyntaxKind::EndOfLineTrivia.into(), b"\r\n"),
            },
            (SyntaxKind::NumericLiteralToken.into(), b"345"),
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
