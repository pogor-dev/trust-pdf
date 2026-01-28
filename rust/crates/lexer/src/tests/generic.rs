use super::utils::{assert_nodes_equal, generate_node_from_lexer};
use crate::Lexer;
use syntax::{SyntaxKind, tree};

#[test]
fn test_scan_token_when_unknown_characters_expect_bad_token() {
    let mut lexer = Lexer::new(b" @#$%");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::BadToken) => {
                trivia(SyntaxKind::WhitespaceTrivia, b" "),
                text(b"@#$"),
                trivia(SyntaxKind::CommentTrivia, b"%")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_token_when_unmatched_closing_paren_expect_bad_token() {
    let mut lexer = Lexer::new(b" ) ");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::BadToken) => {
                trivia(SyntaxKind::WhitespaceTrivia, b" "),
                text(b")"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_pdf_version_expect_pdf_token() {
    let mut lexer = Lexer::new(b"%PDF-1.0\n%PDF-1.1\n%PDF-1.2\n%PDF-1.3\n%PDF-1.4\n%PDF-1.5\n%PDF-1.6\n%PDF-1.7\n%PDF-2.0\n");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::PdfVersionToken) => {
                text(b"%PDF-1.0"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::PdfVersionToken) => {
                text(b"%PDF-1.1"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::PdfVersionToken) => {
                text(b"%PDF-1.2"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::PdfVersionToken) => {
                text(b"%PDF-1.3"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::PdfVersionToken) => {
                text(b"%PDF-1.4"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::PdfVersionToken) => {
                text(b"%PDF-1.5"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::PdfVersionToken) => {
                text(b"%PDF-1.6"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::PdfVersionToken) => {
                text(b"%PDF-1.7"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::PdfVersionToken) => {
                text(b"%PDF-2.0"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_end_of_file_expect_end_of_file_marker_token() {
    let mut lexer = Lexer::new(b"%%EOF");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::EndOfFileMarkerToken, b"%%EOF")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
