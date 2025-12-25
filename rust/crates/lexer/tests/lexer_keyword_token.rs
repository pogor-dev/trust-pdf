mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::SyntaxKind;
use syntax::tree;

#[test]
fn test_scan_keyword_when_true_expect_true_keyword() {
    let mut lexer = Lexer::new(b"true");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::TrueKeyword.into(), b"true")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_false_expect_false_keyword() {
    let mut lexer = Lexer::new(b"false");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::FalseKeyword.into(), b"false")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_null_expect_null_keyword() {
    let mut lexer = Lexer::new(b"null");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NullKeyword.into(), b"null")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_uppercase_true_expect_bad_token() {
    // Keywords are case-sensitive per PDF spec
    let mut lexer = Lexer::new(b"TRUE");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"TRUE")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_mixed_case_expect_bad_token() {
    let mut lexer = Lexer::new(b"True");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"True")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_unrecognized_expect_bad_token() {
    let mut lexer = Lexer::new(b"maybe");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"maybe")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_true_followed_by_space_expect_true_keyword_and_whitespace_trivia() {
    let mut lexer = Lexer::new(b"true ");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::TrueKeyword.into()) => {
                text(b"true"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_false_followed_by_delimiter_expect_false_keyword() {
    let mut lexer = Lexer::new(b"false]");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::FalseKeyword.into(), b"false"),
            (SyntaxKind::CloseBracketToken.into(), b"]")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_null_followed_by_numeric_expect_null_keyword_and_numeric_token() {
    let mut lexer = Lexer::new(b"null 0");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NullKeyword.into()) => {
                text(b"null"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::NumericLiteralToken.into(), b"0")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_multiple_keywords_expect_separate_tokens() {
    let mut lexer = Lexer::new(b"true false null");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::TrueKeyword.into()) => {
                text(b"true"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::FalseKeyword.into()) => {
                text(b"false"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::NullKeyword.into(), b"null")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_true_followed_by_digit_expect_keyword_and_numeric_token() {
    // ISO 32000-2:2020 §7.2.3: Tokens must be separated by delimiters or whitespace.
    // SafeDocs PDF Compacted Syntax Matrix verifies that letter-to-digit transitions
    // require whitespace. "true0" scans as two tokens with a diagnostic.
    let mut lexer = Lexer::new(b"true0");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(syntax::DiagnosticSeverity::Error, syntax::DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::TrueKeyword.into(), b"true"),
            (SyntaxKind::NumericLiteralToken.into(), b"0")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_false_followed_by_digit_expect_keyword_and_numeric_token() {
    let mut lexer = Lexer::new(b"false9");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(syntax::DiagnosticSeverity::Error, syntax::DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::FalseKeyword.into(), b"false"),
            (SyntaxKind::NumericLiteralToken.into(), b"9")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_null_followed_by_digit_expect_keyword_and_numeric_token() {
    let mut lexer = Lexer::new(b"null1");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(syntax::DiagnosticSeverity::Error, syntax::DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::NullKeyword.into(), b"null"),
            (SyntaxKind::NumericLiteralToken.into(), b"1")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_true_followed_by_multidigit_number_expect_keyword_and_numeric_token() {
    let mut lexer = Lexer::new(b"true123");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(syntax::DiagnosticSeverity::Error, syntax::DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::TrueKeyword.into(), b"true"),
            (SyntaxKind::NumericLiteralToken.into(), b"123")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_false_followed_by_decimal_expect_keyword_and_numeric_token() {
    let mut lexer = Lexer::new(b"false3.14");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(syntax::DiagnosticSeverity::Error, syntax::DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::FalseKeyword.into(), b"false"),
            (SyntaxKind::NumericLiteralToken.into(), b"3.14")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_null_followed_by_negative_number_expect_keyword_and_numeric_token() {
    let mut lexer = Lexer::new(b"null-5");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(syntax::DiagnosticSeverity::Error, syntax::DiagnosticKind::MissingWhitespaceBeforeToken.into(), "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)"),
            (SyntaxKind::NullKeyword.into(), b"null"),
            (SyntaxKind::NumericLiteralToken.into(), b"-5")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
