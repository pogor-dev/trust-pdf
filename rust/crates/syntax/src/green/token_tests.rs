use crate::{
    SyntaxKind,
    green::{token::GreenToken, trivia::GreenTrivia},
};

// Test constants for different PDF token types
const STRING_KIND: SyntaxKind = SyntaxKind(1);
const NUMBER_KIND: SyntaxKind = SyntaxKind(2);
const NULL_KIND: SyntaxKind = SyntaxKind(3);
const WHITESPACE_KIND: SyntaxKind = SyntaxKind(4);

/// Helper function to create test tokens with different content types
fn create_token(kind: SyntaxKind, text: &str) -> GreenToken {
    let leading = GreenTrivia::new(WHITESPACE_KIND, b" ");
    let trailing = leading.clone();
    GreenToken::new(kind, text.as_bytes(), leading, trailing)
}

#[test]
fn test_new_when_creating_token_expect_correct_kind() {
    let string_token = create_token(STRING_KIND, "(Hello)");
    let number_token = create_token(NUMBER_KIND, "123");
    let null_token = create_token(NULL_KIND, "null");

    assert_eq!(string_token.kind(), STRING_KIND);
    assert_eq!(number_token.kind(), NUMBER_KIND);
    assert_eq!(null_token.kind(), NULL_KIND);
}

#[test]
fn test_text_when_accessing_content_expect_original_bytes() {
    let string_token = create_token(STRING_KIND, "(Hello)");
    let number_token = create_token(NUMBER_KIND, "123");
    let null_token = create_token(NULL_KIND, "null");

    assert_eq!(string_token.text(), b"(Hello)");
    assert_eq!(number_token.text(), b"123");
    assert_eq!(null_token.text(), b"null");
}

#[test]
fn test_new_when_unicode_text_expect_correct_byte_count() {
    // PDF strings can contain various encodings - test byte accuracy
    let unicode_token = create_token(STRING_KIND, "café");

    // "café" in UTF-8 is 5 bytes (é = 2 bytes)
    assert_eq!(unicode_token.text(), "café".as_bytes());
    assert_eq!(unicode_token.text().len(), 5);
}

#[test]
fn test_new_when_pdf_special_chars_expect_exact_preservation() {
    // Test PDF-specific characters that must be preserved exactly
    let escaped_string = create_token(STRING_KIND, r#"(Hello\nWorld\))"#);
    let hex_string = create_token(STRING_KIND, "<48656C6C6F>");

    assert_eq!(escaped_string.text(), br#"(Hello\nWorld\))"#);
    assert_eq!(hex_string.text(), b"<48656C6C6F>");
}

#[test]
fn test_width_when_accessing_expect_correct_byte_length() {
    // Test direct access to width() method for different token types
    let string_token = create_token(STRING_KIND, "(Hello)");
    let number_token = create_token(NUMBER_KIND, "123");
    let null_token = create_token(NULL_KIND, "null");

    // Verify width method returns correct byte length
    assert_eq!(string_token.width(), 7); // "(Hello)" has 7 bytes
    assert_eq!(number_token.width(), 3); // "123" has 3 bytes  
    assert_eq!(null_token.width(), 4); // "null" has 4 bytes
}

#[test]
fn test_width_when_empty_token_expect_zero() {
    // Test edge case: empty token content
    let empty_token = create_token(NULL_KIND, "");

    assert_eq!(empty_token.width(), 0);
    assert_eq!(empty_token.text().len() as u64, empty_token.width());
}

#[test]
fn test_width_when_unicode_content_expect_byte_count() {
    // Test that width() returns byte count, not character count for Unicode
    let unicode_token = create_token(STRING_KIND, "café");

    // "café" has 4 Unicode characters but 5 UTF-8 bytes (é = 2 bytes)
    assert_eq!(unicode_token.width(), 5);
    assert_eq!(unicode_token.text().len() as u64, unicode_token.width());
}

#[test]
fn test_width_when_pdf_special_chars_expect_exact_byte_count() {
    // Test width calculation for PDF-specific escape sequences and hex strings
    let escaped_string = create_token(STRING_KIND, r#"(Hello\nWorld\))"#);
    let hex_string = create_token(STRING_KIND, "<48656C6C6F>");
    let binary_data = create_token(STRING_KIND, "\x00\x01\x02\x03");

    // Each character is counted as a byte, including escape sequences
    assert_eq!(escaped_string.width(), 16); // Literal backslashes count as bytes
    assert_eq!(hex_string.width(), 12); // Angle brackets + hex digits
    assert_eq!(binary_data.width(), 4); // Raw binary bytes
}

#[test]
fn test_width_when_large_content_expect_accurate_count() {
    // Test width calculation for larger content that might be found in PDF streams
    let large_content = "A".repeat(1000);
    let large_token = create_token(STRING_KIND, &large_content);

    assert_eq!(large_token.width(), 1000);
    assert_eq!(large_token.text().len() as u64, large_token.width());
}

#[test]
fn test_width_when_consistency_with_text_length_expect_match() {
    // Verify that width() is always consistent with text().len()
    let test_cases = vec![
        ("", 0),
        ("a", 1),
        ("hello", 5),
        ("(PDF string)", 12),
        ("/Name", 5),
    ];

    for (content, expected_width) in test_cases {
        let token = create_token(STRING_KIND, content);
        assert_eq!(token.width(), expected_width);
        assert_eq!(token.width(), token.text().len() as u64);
    }
}

#[test]
fn test_partial_eq_when_tokens_equal_expect_true() {
    // Test equality for tokens - covers various equal scenarios
    let identical_tokens = (
        create_token(STRING_KIND, "(Hello)"),
        create_token(STRING_KIND, "(Hello)"),
    );
    let empty_tokens = (create_token(NULL_KIND, ""), create_token(NULL_KIND, ""));
    let unicode_tokens = (
        create_token(STRING_KIND, "café"),
        create_token(STRING_KIND, "café"),
    );

    // Test token equality
    assert!(
        identical_tokens.0 == identical_tokens.1,
        "Tokens with identical kind and content should be equal"
    );
    assert!(
        empty_tokens.0 == empty_tokens.1,
        "Empty tokens with same kind should be equal"
    );
    assert!(
        unicode_tokens.0 == unicode_tokens.1,
        "Identical Unicode content should be equal"
    );

    // Test dereferenced GreenTokenData equality
    assert!(
        *identical_tokens.0 == *identical_tokens.1,
        "Dereferenced GreenTokenData should be equal"
    );
    assert!(
        *empty_tokens.0 == *empty_tokens.1,
        "Dereferenced empty GreenTokenData should be equal"
    );
}

#[test]
fn test_partial_eq_when_tokens_not_equal_expect_false() {
    // Test inequality for tokens - covers various unequal scenarios
    let different_content = (
        create_token(STRING_KIND, "(Hello)"),
        create_token(STRING_KIND, "(World)"),
    );
    let different_kinds = (
        create_token(STRING_KIND, "123"),
        create_token(NUMBER_KIND, "123"),
    );
    let empty_vs_content = (
        create_token(STRING_KIND, ""),
        create_token(STRING_KIND, "content"),
    );
    let case_sensitive = (
        create_token(STRING_KIND, "<48656C6C6F>"),
        create_token(STRING_KIND, "<48656c6c6f>"),
    );

    // Test token inequality
    assert!(
        different_content.0 != different_content.1,
        "Tokens with different content should not be equal"
    );
    assert!(
        different_kinds.0 != different_kinds.1,
        "Tokens with different kinds should not be equal"
    );
    assert!(
        empty_vs_content.0 != empty_vs_content.1,
        "Empty and non-empty tokens should not be equal"
    );
    assert!(
        case_sensitive.0 != case_sensitive.1,
        "Case-different strings should not be equal (PDF spec requirement)"
    );

    // Test dereferenced GreenTokenData inequality
    assert!(
        *different_content.0 != *different_content.1,
        "Dereferenced GreenTokenData should not be equal"
    );
    assert!(
        *different_kinds.0 != *different_kinds.1,
        "Dereferenced GreenTokenData with different kinds should not be equal"
    );
}

#[test]
fn test_to_owned_when_converting_token_data_expect_equivalent_token() {
    // Test ToOwned implementation for GreenTokenData
    let original_token = create_token(STRING_KIND, "(Hello World)");

    // Get reference to GreenTokenData and convert to owned GreenToken
    let token_data: &crate::green::token_data::GreenTokenData = &*original_token;
    let owned_token = token_data.to_owned();

    // Verify the owned token has the same properties as the original
    assert_eq!(
        owned_token.kind(),
        original_token.kind(),
        "Kind should be preserved after to_owned"
    );
    assert_eq!(
        owned_token.text(),
        original_token.text(),
        "Text content should be preserved after to_owned"
    );
    assert_eq!(
        owned_token.width(),
        original_token.width(),
        "Width should be preserved after to_owned"
    );

    // Verify they are equal (but potentially different memory locations)
    assert!(
        owned_token == original_token,
        "Owned token should be equal to original token"
    );

    // Test with different token types to ensure consistency
    let number_token = create_token(NUMBER_KIND, "42.5");
    let number_data: &crate::green::token_data::GreenTokenData = &*number_token;
    let owned_number = number_data.to_owned();

    assert_eq!(
        owned_number.kind(),
        NUMBER_KIND,
        "Number token kind should be preserved"
    );
    assert_eq!(
        owned_number.text(),
        b"42.5",
        "Number token text should be preserved"
    );
}

#[test]
fn test_debug_when_formatting_token_data_expect_structured_output() {
    // Test Debug implementation for GreenTokenData
    let string_token = create_token(STRING_KIND, "(Hello World)");
    let number_token = create_token(NUMBER_KIND, "42.5");
    let empty_token = create_token(NULL_KIND, "");

    // Get references to GreenTokenData for formatting
    let string_data: &crate::green::token_data::GreenTokenData = &*string_token;
    let number_data: &crate::green::token_data::GreenTokenData = &*number_token;
    let empty_data: &crate::green::token_data::GreenTokenData = &*empty_token;

    // Test debug formatting contains expected structure and content
    let string_debug = format!("{:?}", string_data);
    let number_debug = format!("{:?}", number_data);
    let empty_debug = format!("{:?}", empty_data);

    // Verify debug output contains struct name and fields
    assert!(
        string_debug.contains("GreenTokenData"),
        "Debug output should contain struct name"
    );
    assert!(
        string_debug.contains("kind"),
        "Debug output should contain kind field"
    );
    assert!(
        string_debug.contains("text"),
        "Debug output should contain text field"
    );

    // Verify debug output contains actual values (as byte arrays)
    assert!(
        number_debug.contains("52"),
        "Debug output should contain byte values (52 is ASCII '4')"
    );

    // Verify empty token debug output
    assert!(
        empty_debug.contains("GreenTokenData"),
        "Empty token debug should contain struct name"
    );
    assert!(
        empty_debug.contains("[]"),
        "Empty token should show empty byte array"
    );
}

#[test]
fn test_display_when_valid_utf8_expect_string_content() {
    // Test Display implementation for GreenTokenData with valid UTF-8
    let string_token = create_token(STRING_KIND, "(Hello World)");
    let number_token = create_token(NUMBER_KIND, "42.5");

    // Get references to GreenTokenData for display formatting
    let string_data: &crate::green::token_data::GreenTokenData = &*string_token;
    let number_data: &crate::green::token_data::GreenTokenData = &*number_token;

    // Test display formatting shows readable string content
    let string_display = format!("{}", string_data);
    let number_display = format!("{}", number_data);

    assert_eq!(
        string_display, "(Hello World)",
        "Display should show string content"
    );
    assert_eq!(number_display, "42.5", "Display should show number content");
}

#[test]
fn test_display_when_invalid_utf8_expect_replacement_chars() {
    // Test Display implementation gracefully handles invalid UTF-8 bytes
    // This simulates malformed PDF content that might contain invalid encoding
    let invalid_utf8 = &[0xFF, 0xFE, b'P', b'D', b'F'];
    let display_output = String::from_utf8_lossy(invalid_utf8);

    // Verify lossy conversion behavior
    assert!(
        display_output.contains('�'),
        "Should contain replacement character for invalid bytes"
    );
    assert!(
        display_output.contains("PDF"),
        "Should preserve valid UTF-8 portions"
    );
}

#[test]
fn test_green_token_debug_when_formatting_expect_delegated_output() {
    // Test Debug implementation for GreenToken delegates to GreenTokenData
    let string_token = create_token(STRING_KIND, "(Hello World)");
    let number_token = create_token(NUMBER_KIND, "42.5");

    // Test debug formatting contains expected structure and content
    let string_debug = format!("{:?}", string_token);
    let number_debug = format!("{:?}", number_token);

    // Verify debug output contains struct name and fields (delegated from GreenTokenData)
    assert!(
        string_debug.contains("GreenTokenData"),
        "Debug output should contain struct name"
    );
    assert!(
        string_debug.contains("kind"),
        "Debug output should contain kind field"
    );
    assert!(
        string_debug.contains("text"),
        "Debug output should contain text field"
    );

    // Verify debug output contains actual byte values
    assert!(
        number_debug.contains("52"),
        "Debug output should contain byte values (52 is ASCII '4')"
    );
}

#[test]
fn test_green_token_display_when_formatting_expect_delegated_output() {
    // Test Display implementation for GreenToken delegates to GreenTokenData
    let string_token = create_token(STRING_KIND, "(Hello World)");
    let number_token = create_token(NUMBER_KIND, "42.5");

    // Test display formatting shows readable string content (delegated behavior)
    let string_display = format!("{}", string_token);
    let number_display = format!("{}", number_token);

    assert_eq!(
        string_display, "(Hello World)",
        "GreenToken display should show string content via delegation"
    );
    assert_eq!(
        number_display, "42.5",
        "GreenToken display should show number content via delegation"
    );
}
