mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax_2::SyntaxKind;
use syntax_2::tree;

#[test]
fn test_scan_array_open_bracket_expect_open_bracket_token() {
    let mut lexer = Lexer::new(b"[");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::OpenBracketToken, b"[")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_array_close_bracket_expect_close_bracket_token() {
    let mut lexer = Lexer::new(b"]");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::CloseBracketToken, b"]")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_dict_open_expect_open_dict_token() {
    let mut lexer = Lexer::new(b"<<");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::OpenDictToken, b"<<")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_dict_close_expect_close_dict_token() {
    let mut lexer = Lexer::new(b">>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::CloseDictToken, b">>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_single_less_than_expect_hex_string() {
    // Single `<` starts a hex string, not a dictionary
    let mut lexer = Lexer::new(b"<48656C6C6F>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, b"<48656C6C6F>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_single_greater_than_expect_bad_token() {
    // Single `>` by itself is invalid - not a closing delimiter
    let mut lexer = Lexer::new(b">");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::BadToken, b">")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_array_with_elements_expect_tokens() {
    // Example: [549 3.14 false]
    let mut lexer = Lexer::new(b"[549 3.14 false]");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::OpenBracketToken, b"["),
            (SyntaxKind::NumericLiteralToken) => {
                text(b"549"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::NumericLiteralToken) => {
                text(b"3.14"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::FalseKeyword, b"false"),
            (SyntaxKind::CloseBracketToken, b"]")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_dictionary_example_expect_tokens() {
    // Example: << /Type /Example >>
    let mut lexer = Lexer::new(b"<</Type /Example>>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::OpenDictToken, b"<<"),
            (SyntaxKind::NameLiteralToken) => {
                text(b"/Type"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::NameLiteralToken, b"/Example"),
            (SyntaxKind::CloseDictToken, b">>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_nested_arrays_expect_tokens() {
    // Nested arrays: [[1 2] [3 4]]
    let mut lexer = Lexer::new(b"[[1 2][3 4]]");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::OpenBracketToken, b"["),
            (SyntaxKind::OpenBracketToken, b"["),
            (SyntaxKind::NumericLiteralToken) => {
                text(b"1"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::NumericLiteralToken, b"2"),
            (SyntaxKind::CloseBracketToken, b"]"),
            (SyntaxKind::OpenBracketToken, b"["),
            (SyntaxKind::NumericLiteralToken) => {
                text(b"3"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::NumericLiteralToken, b"4"),
            (SyntaxKind::CloseBracketToken, b"]"),
            (SyntaxKind::CloseBracketToken, b"]")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_array_in_dictionary_expect_tokens() {
    // Dictionary with array: <</Items [1 2 3]>>
    let mut lexer = Lexer::new(b"<</Items[1 2 3]>>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::OpenDictToken, b"<<"),
            (SyntaxKind::NameLiteralToken, b"/Items"),
            (SyntaxKind::OpenBracketToken, b"["),
            (SyntaxKind::NumericLiteralToken) => {
                text(b"1"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::NumericLiteralToken) => {
                text(b"2"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::NumericLiteralToken, b"3"),
            (SyntaxKind::CloseBracketToken, b"]"),
            (SyntaxKind::CloseDictToken, b">>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_empty_array_expect_tokens() {
    let mut lexer = Lexer::new(b"[]");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::OpenBracketToken, b"["),
            (SyntaxKind::CloseBracketToken, b"]")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_empty_dictionary_expect_tokens() {
    let mut lexer = Lexer::new(b"<<>>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::OpenDictToken, b"<<"),
            (SyntaxKind::CloseDictToken, b">>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
