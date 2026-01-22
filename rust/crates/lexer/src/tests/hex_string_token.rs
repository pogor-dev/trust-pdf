use crate::Lexer;
use super::utils::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{DiagnosticKind, DiagnosticSeverity::Error, SyntaxKind, tree};

#[test]
fn test_scan_hex_string_when_simple_hex_string_expect_hex_string_literal_token() {
    // Example 1 from §7.3.4.3
    let mut lexer = Lexer::new(b"<4E6F762073686D6F7A206B6120706F702E>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, b"<4E6F762073686D6F7A206B6120706F702E>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_empty_string_expect_hex_string_literal_token() {
    // Empty hex string: <>
    let mut lexer = Lexer::new(b"<>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, b"<>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_even_number_of_digits_expect_hex_string_literal_token() {
    // Example 2 from §7.3.4.3: <901FA3> is a 3-byte string (90, 1F, A3)
    let mut lexer = Lexer::new(b"<901FA3>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, b"<901FA3>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_odd_number_of_digits_expect_hex_string_literal_token() {
    // Example 2 from §7.3.4.3: <901FA> is a 3-byte string (90, 1F, A0)
    // Final digit assumed to be 0
    let mut lexer = Lexer::new(b"<901FA>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, b"<901FA>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_contains_whitespace_expect_hex_string_literal_token() {
    // Whitespace should be ignored per §7.3.4.3
    let mut lexer = Lexer::new(b"<48 65 6C 6C 6F>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, b"<48 65 6C 6C 6F>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_contains_newlines_expect_hex_string_literal_token() {
    // Newlines (EOL) are whitespace and should be ignored
    let input = b"<48656C6C6F\n576F726C64>";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_lowercase_digits_expect_hex_string_literal_token() {
    // Both uppercase and lowercase hex digits are valid
    let mut lexer = Lexer::new(b"<abcdef>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, b"<abcdef>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_mixed_case_expect_hex_string_literal_token() {
    let mut lexer = Lexer::new(b"<AbCdEf123456>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, b"<AbCdEf123456>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_two_strings_delimited_expect_two_hex_string_literal_tokens() {
    let mut lexer = Lexer::new(b"<48656C6C6F> <576F726C64>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken) => {
                text(b"<48656C6C6F>"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            },
            (SyntaxKind::HexStringLiteralToken, b"<576F726C64>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_single_digit_expect_hex_string_literal_token() {
    // Single digit should be treated as odd number (trailing 0 assumed)
    let mut lexer = Lexer::new(b"<A>");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, b"<A>")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_contains_tabs_expect_hex_string_literal_token() {
    // Tabs are whitespace and should be ignored
    let input = b"<48\t65\t6C\t6C\t6F>";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::HexStringLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_invalid_character_expect_invalid_character_diagnostic() {
    // Non-hex characters (other than whitespace) should emit a single error diagnostic
    let input = b"<48ZZ6C>";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            @diagnostic(Error, DiagnosticKind::InvalidCharacterInHexString.into(), "Invalid character in hex string"),
            (SyntaxKind::HexStringLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_hex_string_when_unclosed_expect_invalid_character_diagnostic() {
    // Missing closing '>' should emit a single error diagnostic
    let input = b"<48A9";
    let mut lexer = Lexer::new(input);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            @diagnostic(Error, DiagnosticKind::UnbalancedHexString.into(), "Unbalanced hex string"),
            (SyntaxKind::HexStringLiteralToken, input)
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
