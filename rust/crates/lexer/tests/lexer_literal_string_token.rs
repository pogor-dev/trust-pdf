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
