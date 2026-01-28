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
fn test_scan_pdf_version_malformed_expect_comments() {
    // Test that malformed PDF version patterns are treated as comments, not tokens
    let mut lexer = Lexer::new(b"%PDF-\n%PDF-1\n%PDF-1.\n%PDF-1.2.3\n%PDF-1.2.3.4\n%PDF-12.0\n%PDF-1.23\n123");
    let actual_node = generate_node_from_lexer(&mut lexer);

    // All malformed versions become comments (leading trivia on the numeric token)
    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                trivia(SyntaxKind::CommentTrivia, b"%PDF-"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%PDF-1"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%PDF-1."),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%PDF-1.2.3"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%PDF-1.2.3.4"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%PDF-12.0"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%PDF-1.23"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"123")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_pdf_version_incomplete_expect_comments() {
    // Test that %PDF- without complete version is treated as comment
    let mut lexer = Lexer::new(b"%PDF-\n%PDF- \n%PDF-1.7abc\n%PDF-1.7123\n456");
    let actual_node = generate_node_from_lexer(&mut lexer);

    // Both %PDF- sequences become comments (leading trivia on the numeric token)
    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NumericLiteralToken) => {
                trivia(SyntaxKind::CommentTrivia, b"%PDF-"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%PDF-"),
                trivia(SyntaxKind::WhitespaceTrivia, b" "),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%PDF-1.7abc"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%PDF-1.7123"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"456")
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

#[test]
fn test_scan_end_of_file_edge_cases() {
    // Tests partial markers, case sensitivity, trailing content, and multiple markers
    let mut lexer = Lexer::new(b"%\n%%\n%%E\n%%EO\n%EOF\n%%EOf\n%%EoF\n%%eOF\n%%EOF\n%%EOF123\n%%EOFabc\n%%EOF.5\n%%EOF 123\n%%EOF");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            // All partial/malformed markers become comments (leading trivia on first real token)
            (SyntaxKind::EndOfFileMarkerToken) => {
                trivia(SyntaxKind::CommentTrivia, b"%"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%%"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%%E"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%%EO"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%EOF"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%%EOf"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%%EoF"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%%eOF"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%%EOF123"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%%EOFabc"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                trivia(SyntaxKind::CommentTrivia, b"%%EOF.5"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"%%EOF")
            },
            // EOF marker with trailing whitespace
            (SyntaxKind::EndOfFileMarkerToken) => {
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"%%EOF"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            // Numeric token after whitespace
            (SyntaxKind::NumericLiteralToken) => {
                text(b"123"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            // Another EOF marker (incremental PDF)
            (SyntaxKind::EndOfFileMarkerToken, b"%%EOF")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
