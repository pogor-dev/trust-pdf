mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax_2::{SyntaxKind, tree};

#[test]
fn test_scan_trivia_when_single_space_expect_whitespace_trivia() {
    let mut lexer = Lexer::new(b"009 345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                text(b"009"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::NumericLiteralToken, b"345")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_trivia_when_multiple_spaces_expect_whitespace_trivia() {
    let mut lexer = Lexer::new(b"009       345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                text(b"009"),
                trivia(SyntaxKind::WhitespaceTrivia, b"       ")
            },
            (SyntaxKind::NumericLiteralToken, b"345")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_trivia_when_mixed_whitespace_types_expect_appropriate_trivia() {
    let mut lexer = Lexer::new(b"\r\0009 \t \x0C\r\n345\0\t\x0C \n");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                trivia(SyntaxKind::EndOfLineTrivia, b"\r"),
                trivia(SyntaxKind::WhitespaceTrivia, b"\0"),
                text(b"009"),
                trivia(SyntaxKind::WhitespaceTrivia, b" \t \x0C"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r\n")
            },
            (SyntaxKind::NumericLiteralToken) => {
                text(b"345"),
                trivia(SyntaxKind::WhitespaceTrivia, b"\0\t\x0C "),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_trivia_when_comments_present_expect_comment_trivia() {
    let mut lexer = Lexer::new(b"% This is a comment\n009 % Another comment\r\n345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                trivia(SyntaxKind::CommentTrivia, b"% This is a comment"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"009"),
                trivia(SyntaxKind::WhitespaceTrivia, b" "),
                trivia(SyntaxKind::CommentTrivia, b"% Another comment"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r\n")
            },
            (SyntaxKind::NumericLiteralToken, b"345")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_trivia_when_consecutive_lf_expect_separate_eol_trivia() {
    let mut lexer = Lexer::new(b"009\n\n345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                text(b"009"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::NumericLiteralToken, b"345")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_trivia_when_consecutive_cr_expect_separate_eol_trivia() {
    let mut lexer = Lexer::new(b"009\r\r345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                text(b"009"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r")
            },
            (SyntaxKind::NumericLiteralToken, b"345")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_trivia_when_consecutive_crlf_expect_separate_eol_trivia() {
    let mut lexer = Lexer::new(b"009\r\n\r\n345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                text(b"009"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r\n"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r\n")
            },
            (SyntaxKind::NumericLiteralToken, b"345")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_trivia_when_mixed_eol_sequences_expect_separate_eol_trivia() {
    let mut lexer = Lexer::new(b"009\n\r\r\n345");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                text(b"009"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r\n")
            },
            (SyntaxKind::NumericLiteralToken, b"345")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
