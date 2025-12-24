mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{DiagnosticKind, DiagnosticSeverity::Error, SyntaxKind, tree};

#[test]
fn test_string_literal_simple() {
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
fn test_string_literal_with_newlines() {
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
fn test_string_literal_with_balanced_parentheses() {
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
fn test_string_literal_empty() {
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
fn test_string_literal_with_zero_content() {
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
fn test_string_literal_two_strings_with_space() {
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
fn test_string_literal_unbalanced_unclosed() {
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
fn test_string_literal_unbalanced_extra_open() {
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
fn test_string_literal_with_escape_sequences() {
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
fn test_string_literal_escaped_parens_do_not_affect_nesting() {
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
fn test_string_literal_trailing_backslash_at_eof() {
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
fn test_string_literal_unknown_escape_backslash_x() {
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
