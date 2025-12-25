mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{DiagnosticKind, DiagnosticSeverity::Error, SyntaxKind, tree};

/// Tests for SafeDocs PDF Compacted Syntax Matrix whitespace rules.
///
/// The SafeDocs program produced a specification of valid PDF token sequences.
/// Many transitions between non-delimited token types require whitespace separators
/// to be valid according to ISO 32000-2:2020 §7.2.3.
///
/// See: [SafeDocs test matrix](https://github.com/pdf-association/safedocs/blob/a6fd37308c91a0d2c17ebcace970367181bc0da7/CompactedSyntax/CompactedPDFSyntaxMatrix.pdf) and PDF Association GitHub repository.

// ============================================================================
// Boolean → Integer/Real transitions (all require whitespace per SafeDocs)
// ============================================================================

#[test]
fn test_scan_keyword_when_true_immediately_followed_by_zero_expect_error_diagnostic() {
    // Boolean → Integer requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"true0");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::TrueKeyword.into(), b"true"),
            (SyntaxKind::NumericLiteralToken.into(), b"0")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_false_immediately_followed_by_digit_expect_error_diagnostic() {
    // Boolean → Integer requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"false9");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::FalseKeyword.into(), b"false"),
            (SyntaxKind::NumericLiteralToken.into(), b"9")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_null_immediately_followed_by_digit_expect_error_diagnostic() {
    // Boolean → Integer requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"null1");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::NullKeyword.into(), b"null"),
            (SyntaxKind::NumericLiteralToken.into(), b"1")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_false_immediately_followed_by_decimal_expect_error_diagnostic() {
    // Boolean → Real requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"false3.14");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::FalseKeyword.into(), b"false"),
            (SyntaxKind::NumericLiteralToken.into(), b"3.14")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_null_immediately_followed_by_negative_number_expect_error_diagnostic() {
    // Boolean → Real requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"null-5");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::NullKeyword.into(), b"null"),
            (SyntaxKind::NumericLiteralToken.into(), b"-5")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_true_followed_by_decimal_point_expect_error_diagnostic() {
    // Boolean → Real requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"true.25");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::TrueKeyword.into(), b"true"),
            (SyntaxKind::NumericLiteralToken.into(), b".25")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Integer → Boolean/Null transitions (all require whitespace per SafeDocs)
// ============================================================================

#[test]
fn test_scan_numeric_when_123_immediately_followed_by_true_expect_error_diagnostic() {
    // Integer → Boolean requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"123true");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::NumericLiteralToken.into(), b"123"),
            (SyntaxKind::TrueKeyword.into(), b"true")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_numeric_when_456_immediately_followed_by_false_expect_error_diagnostic() {
    // Integer → Boolean requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"456false");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::NumericLiteralToken.into(), b"456"),
            (SyntaxKind::FalseKeyword.into(), b"false")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_numeric_when_789_immediately_followed_by_null_expect_error_diagnostic() {
    // Integer → Null requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"789null");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::NumericLiteralToken.into(), b"789"),
            (SyntaxKind::NullKeyword.into(), b"null")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_numeric_when_decimal_immediately_followed_by_true_expect_error_diagnostic() {
    // Real → Boolean requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"3.14true");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::NumericLiteralToken.into(), b"3.14"),
            (SyntaxKind::TrueKeyword.into(), b"true")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_numeric_when_negative_followed_by_null_expect_error_diagnostic() {
    // Real → Null requires whitespace per SafeDocs Matrix
    let mut lexer = Lexer::new(b"-5null");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::NumericLiteralToken.into(), b"-5"),
            (SyntaxKind::NullKeyword.into(), b"null")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Valid cases: transitions with whitespace or delimiters
// ============================================================================

#[test]
fn test_scan_keyword_when_true_followed_by_space_and_digit_expect_no_diagnostic() {
    // Boolean → Integer with whitespace is valid
    let mut lexer = Lexer::new(b"true 0");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::TrueKeyword.into()) => {
                text(b"true"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::NumericLiteralToken.into(), b"0")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_numeric_when_123_followed_by_space_and_true_expect_no_diagnostic() {
    // Integer → Boolean with whitespace is valid
    let mut lexer = Lexer::new(b"123 true");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into()) => {
                text(b"123"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::TrueKeyword.into(), b"true")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_null_followed_by_name_delimiter_expect_no_diagnostic() {
    // Null → Name (delimited by `/`) requires no whitespace
    let mut lexer = Lexer::new(b"null/Type");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NullKeyword.into(), b"null"),
            (SyntaxKind::NameLiteralToken.into(), b"/Type")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_numeric_when_123_followed_by_array_bracket_expect_no_diagnostic() {
    // Integer → Array (delimited by `[`) requires no whitespace
    let mut lexer = Lexer::new(b"123[");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"123"),
            (SyntaxKind::OpenBracketToken.into(), b"[")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
