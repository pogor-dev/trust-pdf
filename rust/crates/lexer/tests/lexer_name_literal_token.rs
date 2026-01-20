mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax_2::{DiagnosticKind, DiagnosticSeverity::Error, SyntaxKind, tree};

#[test]
fn test_scan_name_when_simple_name_expect_name_literal_token() {
    let mut lexer = Lexer::new(b"/Name1");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NameLiteralToken, b"/Name1")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_empty_name_expect_name_literal_token() {
    let mut lexer = Lexer::new(b"/");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NameLiteralToken, b"/")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_contains_special_characters_expect_name_literal_token() {
    // Example from ISO 32000-2:2020 Table 4
    let mut lexer = Lexer::new(b"/A;Name_With-Various***Characters?");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NameLiteralToken, b"/A;Name_With-Various***Characters?")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_contains_hex_escape_for_space_expect_name_literal_token() {
    let mut lexer = Lexer::new(b"/Lime#20Green");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NameLiteralToken, b"/Lime#20Green")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_hex_escape_encodes_delimiter_expect_name_literal_token() {
    // #2F encodes '/'; lexer should keep the name contiguous
    let mut lexer = Lexer::new(b"/Name#2FChild");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NameLiteralToken, b"/Name#2FChild")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_two_names_separated_by_whitespace_expect_two_name_literal_tokens() {
    let mut lexer = Lexer::new(b"/First /Second");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NameLiteralToken) => {
                text(b"/First"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::NameLiteralToken, b"/Second")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_two_names_adjacent_expect_two_name_literal_tokens() {
    let mut lexer = Lexer::new(b"/Name1/Name2");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NameLiteralToken, b"/Name1"),
            (SyntaxKind::NameLiteralToken, b"/Name2")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_invalid_hex_escape_expect_invalid_hex_escape_diagnostic() {
    let input = b"/Bad#G1";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            @diagnostic(Error, DiagnosticKind::InvalidHexEscapeInName.into(), "Invalid hex escape in name"),
            (SyntaxKind::NameLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_truncated_hex_escape_expect_invalid_hex_escape_diagnostic() {
    let input = b"/Bad#";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            @diagnostic(Error, DiagnosticKind::InvalidHexEscapeInName.into(), "Invalid hex escape in name"),
            (SyntaxKind::NameLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_double_hash_expect_single_invalid_hex_escape_diagnostic() {
    let input = b"/Name##";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            @diagnostic(Error, DiagnosticKind::InvalidHexEscapeInName.into(), "Invalid hex escape in name"),
            (SyntaxKind::NameLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_multiple_hex_escapes_expect_name_literal_token() {
    // #12 and #45 are valid escapes; remaining digits are regular characters
    let input = b"/Name#123#456";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NameLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_non_regular_ascii_expect_invalid_non_regular_character_diagnostic() {
    let input = b"/Name\x7F";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            @diagnostic(Error, DiagnosticKind::InvalidNonRegularCharacterInName.into(), "Invalid character in name. Non-regular characters must be hex-escaped using #xx notation"),
            (SyntaxKind::NameLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_high_byte_expect_invalid_non_regular_character_diagnostic() {
    let input = b"/Name\x80";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            @diagnostic(Error, DiagnosticKind::InvalidNonRegularCharacterInName.into(), "Invalid character in name. Non-regular characters must be hex-escaped using #xx notation"),
            (SyntaxKind::NameLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_whitespace_in_body_splits_token_expect_whitespace_then_numeric_token() {
    // Whitespace inside a name must be hex-escaped; plain space ends the name token.
    let input = b"/Name 123";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::NameLiteralToken) => {
                text(b"/Name"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::NumericLiteralToken, b"123")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_name_when_single_hex_digit_followed_by_non_hex_expect_invalid_hex_escape_diagnostic() {
    // Single hex digit followed by non-hex character: #1G should emit diagnostic
    let input = b"/Name#1G";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            @diagnostic(Error, DiagnosticKind::InvalidHexEscapeInName.into(), "Invalid hex escape in name"),
            (SyntaxKind::NameLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
