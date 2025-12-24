mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{DiagnosticKind, DiagnosticSeverity::Error, SyntaxKind, tree};

#[test]
fn test_scan_literal_string_when_simple_string_expect_string_literal_token() {
    let mut lexer = Lexer::new(b"(This is a string)");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), b"(This is a string)")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_contains_newlines_expect_string_literal_token() {
    let mut lexer = Lexer::new(b"(Strings can contain newlines\nand such.)");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), b"(Strings can contain newlines\nand such.)")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_balanced_parentheses_expect_string_literal_token() {
    let mut lexer = Lexer::new(b"(Strings can contain balanced parentheses () and special characters ( * ! & } ^ %and so on) .)");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), b"(Strings can contain balanced parentheses () and special characters ( * ! & } ^ %and so on) .)")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_empty_string_expect_string_literal_token() {
    let mut lexer = Lexer::new(b"()");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), b"()")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_nested_parentheses_with_digit_expect_string_literal_token() {
    let mut lexer = Lexer::new(b"(It has zero (0) length.)");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), b"(It has zero (0) length.)")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_two_strings_delimited_expect_two_string_literal_tokens() {
    let mut lexer = Lexer::new(b"(first) (second)");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into()) => {
                text(b"(first)"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::StringLiteralToken.into(), b"(second)")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_unclosed_string_expect_unbalanced_diagnostic() {
    let mut lexer = Lexer::new(b"(This is unclosed");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::UnbalancedStringLiteral.into(), "Unbalanced string literal"),
            (SyntaxKind::StringLiteralToken.into(), b"(This is unclosed")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_extra_open_paren_expect_unbalanced_diagnostic() {
    let mut lexer = Lexer::new(b"(()");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::UnbalancedStringLiteral.into(), "Unbalanced string literal"),
            (SyntaxKind::StringLiteralToken.into(), b"(()")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_escape_sequences_expect_string_literal_token() {
    // Verify recognized escape sequences are consumed and do not break lexing
    let input = b"(line\\n feed \\r cr \\t tab \\b bs \\f ff \\( left \\) right \\\\ backslash)";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_escaped_parens_expect_no_nesting_change() {
    // Escaped parentheses should not change nesting; string remains balanced
    let input = b"(a \\( b \\) c)";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_trailing_backslash_at_eof_expect_unbalanced_diagnostic() {
    // Reverse solidus at EOF should be consumed and treated as unbalanced string
    let input = b"(trailing backslash\\";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Error, DiagnosticKind::UnbalancedStringLiteral.into(), "Unbalanced string literal"),
            (SyntaxKind::StringLiteralToken.into(), input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_unknown_escape_sequence_expect_string_literal_token() {
    // Unknown escape: backslash should be ignored, next char handled normally
    let input = b"(hello \\x world)";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
#[test]
fn test_scan_literal_string_when_escaped_parentheses_expect_string_literal_token() {
    // Escaped parentheses should not affect nesting count
    // Input: (a \( b) - the \( should be skipped, leaving only one closing paren
    let input = b"(a \\( b)";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_escaped_closing_paren_expect_string_literal_token() {
    // Escaped closing paren should not terminate the string
    // Input: (text \) more) - the \) is escaped, so it doesn't close the string
    let input = b"(text \\) more)";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
