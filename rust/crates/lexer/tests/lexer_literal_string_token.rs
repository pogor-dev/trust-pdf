mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{
    DiagnosticKind,
    DiagnosticSeverity::{Error, Warning},
    SyntaxKind, tree,
};

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
            @diagnostic(Warning, DiagnosticKind::UnknownEscapeInStringLiteral.into(), "Unknown escape sequence in string literal"),
            (SyntaxKind::StringLiteralToken.into(), input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_numeric_non_octal_escape_expect_unknown_escape_diagnostic() {
    // Numeric escape starting with 8 or 9 is not a valid octal per §7.3.4.2
    let input = b"(a \\8 b)";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            @diagnostic(Warning, DiagnosticKind::UnknownEscapeInStringLiteral.into(), "Unknown escape sequence in string literal"),
            (SyntaxKind::StringLiteralToken.into(), input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_literal_string_when_octal_escape_three_digits_followed_by_digit_expect_string_literal_token() {
    // Example 5 from §7.3.4.2: (\0053) denotes two chars, then digit '3'
    // Lexer should consume backslash + up to three octal digits, leaving the following digit intact.
    let input = b"(\\0053)";
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
fn test_scan_literal_string_when_octal_escape_one_or_two_digits_expect_string_literal_token() {
    // Example 5 from §7.3.4.2: (\053) and (\53) both denote a single '+' character
    // Lexer should consume backslash + up to three octal digits.
    let input1 = b"(\\053)";
    let input2 = b"(\\53)";

    let mut lexer1 = Lexer::new(input1);
    let mut lexer2 = Lexer::new(input2);
    let actual_node1 = generate_node_from_lexer(&mut lexer1);
    let actual_node2 = generate_node_from_lexer(&mut lexer2);

    let expected_node1 = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), input1)
        }
    };
    let expected_node2 = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), input2)
        }
    };

    assert_nodes_equal(&actual_node1, &expected_node1);
    assert_nodes_equal(&actual_node2, &expected_node2);
}

#[test]
fn test_scan_literal_string_when_line_continuation_expect_string_literal_token() {
    // Example 2 from §7.3.4.2: backslash at end-of-line indicates continuation
    // Test CRLF variant
    let input_crlf = b"(These \\\r\ntwo strings \\\r\nare the same.)";
    let mut lexer_crlf = Lexer::new(input_crlf);
    let actual_node_crlf = generate_node_from_lexer(&mut lexer_crlf);
    let expected_node_crlf = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), input_crlf)
        }
    };
    assert_nodes_equal(&actual_node_crlf, &expected_node_crlf);

    // Test LF-only variant
    let input_lf = b"(These \\\ntwo strings \\\nare the same.)";
    let mut lexer_lf = Lexer::new(input_lf);
    let actual_node_lf = generate_node_from_lexer(&mut lexer_lf);
    let expected_node_lf = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), input_lf)
        }
    };
    assert_nodes_equal(&actual_node_lf, &expected_node_lf);
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
