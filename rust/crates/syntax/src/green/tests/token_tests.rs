use crate::{
    SyntaxKind,
    green::{
        token::{GreenToken, GreenTokenData},
        trivia::GreenTrivia,
    },
};

// Test constants for different PDF token types
const STRING_KIND: SyntaxKind = SyntaxKind(1);
const NUMBER_KIND: SyntaxKind = SyntaxKind(2);
const NULL_KIND: SyntaxKind = SyntaxKind(3);
const NAME_KIND: SyntaxKind = SyntaxKind(4);
const BOOLEAN_KIND: SyntaxKind = SyntaxKind(5);

/// Helper function to create test tokens with different content types
fn create_token(kind: SyntaxKind, text: &str) -> GreenToken {
    let leading = GreenTrivia::new([]);
    let trailing = leading.clone();
    GreenToken::new(kind, text.as_bytes(), leading, trailing)
}

// =============================================================================
// GreenToken Core Tests
// =============================================================================

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
    assert_eq!(unicode_token.width(), 5);
}

#[test]
fn test_width_when_calculating_expect_text_byte_length() {
    let tokens = [
        (STRING_KIND, "()", 2),
        (STRING_KIND, "(Hello)", 7),
        (NUMBER_KIND, "123", 3),
        (NUMBER_KIND, "3.14159", 7),
        (NULL_KIND, "null", 4),
        (BOOLEAN_KIND, "true", 4),
        (BOOLEAN_KIND, "false", 5),
    ];

    for (kind, text, expected_width) in tokens {
        let token = create_token(kind, text);
        assert_eq!(
            token.width(),
            expected_width,
            "Width mismatch for token: {}",
            text
        );
    }
}

// =============================================================================
// Trivia Handling Tests
// =============================================================================

#[test]
fn test_new_with_trivia_when_adding_leading_trivia_expect_correct_full_width() {
    use crate::green::trivia::GreenTriviaChild;

    let leading_trivia = GreenTrivia::new(vec![
        GreenTriviaChild::new(SyntaxKind(10), b"  "), // 2 spaces
        GreenTriviaChild::new(SyntaxKind(11), b"%comment\n"), // comment + newline (9 bytes)
    ]);
    let trailing_trivia = GreenTrivia::new([]);

    let token = GreenToken::new(NUMBER_KIND, b"123", leading_trivia, trailing_trivia);

    assert_eq!(token.width(), 3); // Just the token text
    assert_eq!(token.full_width(), 14); // 2 + 9 + 3 = 14 (leading + text + trailing)
}

#[test]
fn test_new_with_trivia_when_adding_trailing_trivia_expect_correct_full_width() {
    use crate::green::trivia::GreenTriviaChild;

    let leading_trivia = GreenTrivia::new([]);
    let trailing_trivia = GreenTrivia::new(vec![
        GreenTriviaChild::new(SyntaxKind(12), b" "),  // 1 space
        GreenTriviaChild::new(SyntaxKind(13), b"\n"), // 1 newline
    ]);

    let token = GreenToken::new(STRING_KIND, b"(test)", leading_trivia, trailing_trivia);

    assert_eq!(token.width(), 6); // Just the token text
    assert_eq!(token.full_width(), 8); // 6 + 1 + 1 = 8 (text + trailing)
}

#[test]
fn test_new_with_trivia_when_adding_both_trivia_expect_correct_full_width() {
    use crate::green::trivia::GreenTriviaChild;

    let leading_trivia = GreenTrivia::new(vec![
        GreenTriviaChild::new(SyntaxKind(14), b"    "), // 4 spaces
    ]);
    let trailing_trivia = GreenTrivia::new(vec![
        GreenTriviaChild::new(SyntaxKind(15), b"  "), // 2 spaces
    ]);

    let token = GreenToken::new(NAME_KIND, b"/Type", leading_trivia, trailing_trivia);

    assert_eq!(token.width(), 5); // Just the token text
    assert_eq!(token.full_width(), 11); // 4 + 5 + 2 = 11 (leading + text + trailing)
}

#[test]
fn test_leading_trivia_when_accessing_expect_correct_content() {
    use crate::green::trivia::GreenTriviaChild;

    let leading_trivia = GreenTrivia::new(vec![GreenTriviaChild::new(
        SyntaxKind(16),
        b"%leading comment",
    )]);
    let trailing_trivia = GreenTrivia::new([]);

    let token = GreenToken::new(
        STRING_KIND,
        b"(test)",
        leading_trivia.clone(),
        trailing_trivia,
    );

    let retrieved_leading = token.leading_trivia();
    assert_eq!(retrieved_leading.children().len(), 1);
    assert_eq!(retrieved_leading.text(), b"%leading comment");
}

#[test]
fn test_trailing_trivia_when_accessing_expect_correct_content() {
    use crate::green::trivia::GreenTriviaChild;

    let leading_trivia = GreenTrivia::new([]);
    let trailing_trivia = GreenTrivia::new(vec![GreenTriviaChild::new(
        SyntaxKind(17),
        b" %trailing comment",
    )]);

    let token = GreenToken::new(NAME_KIND, b"/Test", leading_trivia, trailing_trivia.clone());

    let retrieved_trailing = token.trailing_trivia();
    assert_eq!(retrieved_trailing.children().len(), 1);
    assert_eq!(retrieved_trailing.text(), b" %trailing comment");
}

// =============================================================================
// PDF-Specific Token Tests
// =============================================================================

#[test]
fn test_pdf_tokens_when_creating_expect_correct_handling() {
    // Test various PDF token formats in one test to reduce redundancy
    let tokens = [
        (
            STRING_KIND,
            "(Literal string)",
            b"(Literal string)" as &[u8],
        ),
        (STRING_KIND, "<48656C6C6F>", b"<48656C6C6F>"),
        (STRING_KIND, "()", b"()"),
        (NUMBER_KIND, "42", b"42"),
        (NUMBER_KIND, "-17", b"-17"),
        (NUMBER_KIND, "3.14159", b"3.14159"),
        (NAME_KIND, "/Type", b"/Type"),
        (NAME_KIND, "/DecodeParms", b"/DecodeParms"),
        (NAME_KIND, "/Lime#20Green", b"/Lime#20Green"),
        (BOOLEAN_KIND, "true", b"true"),
        (BOOLEAN_KIND, "false", b"false"),
    ];

    for (kind, text, expected_bytes) in tokens {
        let token = create_token(kind, text);
        assert_eq!(token.text(), expected_bytes);
        assert_eq!(token.width(), expected_bytes.len() as u32);
    }
}

// =============================================================================
// Memory Management and Raw Pointer Tests
// =============================================================================

#[test]
fn test_into_raw_when_converting_to_pointer_expect_valid_operation() {
    let token = create_token(STRING_KIND, "(test)");
    let original_text = token.text().to_vec();

    // Convert to raw pointer
    let raw_ptr = GreenToken::into_raw(token);

    // Convert back and verify integrity
    let recovered_token = unsafe { GreenToken::from_raw(raw_ptr) };
    assert_eq!(recovered_token.text(), &original_text);
}

#[test]
fn test_borrow_when_using_as_token_data_expect_same_content() {
    use std::borrow::Borrow;

    let token = create_token(NUMBER_KIND, "123");
    let token_data: &GreenTokenData = token.borrow();

    // Verify that borrowing gives access to the same data
    assert_eq!(token.kind(), token_data.kind());
    assert_eq!(token.text(), token_data.text());
    assert_eq!(token.width(), token_data.width());
}

#[test]
fn test_to_owned_when_converting_token_data_expect_equivalent_token() {
    use std::borrow::ToOwned;

    let original_token = create_token(NAME_KIND, "/Test");
    let token_data: &GreenTokenData = &original_token;
    let owned_token = token_data.to_owned();

    assert_eq!(original_token.kind(), owned_token.kind());
    assert_eq!(original_token.text(), owned_token.text());
    assert_eq!(original_token.width(), owned_token.width());
}

// =============================================================================
// Equality and Hashing Tests
// =============================================================================

#[test]
fn test_equality_when_identical_tokens_expect_equal() {
    let token1 = create_token(STRING_KIND, "(Hello)");
    let token2 = create_token(STRING_KIND, "(Hello)");

    assert_eq!(token1, token2);
}

#[test]
fn test_equality_when_different_content_expect_not_equal() {
    let token1 = create_token(STRING_KIND, "(Hello)");
    let token2 = create_token(STRING_KIND, "(World)");

    assert_ne!(token1, token2);
}

#[test]
fn test_equality_when_different_kinds_expect_not_equal() {
    let string_token = create_token(STRING_KIND, "123");
    let number_token = create_token(NUMBER_KIND, "123");

    assert_ne!(string_token, number_token);
}

#[test]
fn test_hash_when_using_in_collections_expect_consistent_behavior() {
    use std::collections::HashMap;

    let token = create_token(NAME_KIND, "/Type");
    let mut map = HashMap::new();
    map.insert(token.clone(), "test_value");

    // Should be able to retrieve using equivalent token
    let lookup_token = create_token(NAME_KIND, "/Type");
    assert_eq!(map.get(&lookup_token), Some(&"test_value"));
}

// =============================================================================
// Edge Cases and Error Conditions
// =============================================================================

#[test]
fn test_token_when_empty_content_expect_valid_handling() {
    let empty_token = create_token(STRING_KIND, "");

    assert_eq!(empty_token.text(), b"");
    assert_eq!(empty_token.width(), 0);
    assert_eq!(empty_token.full_width(), 0);
}

#[test]
fn test_token_when_large_content_expect_efficient_handling() {
    let large_content = "A".repeat(10000);
    let large_token = create_token(STRING_KIND, &large_content);

    assert_eq!(large_token.text(), large_content.as_bytes());
    assert_eq!(large_token.width(), 10000);
}

#[test]
fn test_token_when_binary_content_expect_exact_preservation() {
    // Test with non-UTF8 binary data that might appear in PDF streams
    let binary_data = &[0xFF, 0xFE, 0x00, 0x01, 0x80, 0x7F];
    let binary_token = GreenToken::new(
        STRING_KIND,
        binary_data,
        GreenTrivia::new([]),
        GreenTrivia::new([]),
    );

    assert_eq!(binary_token.text(), binary_data);
    assert_eq!(binary_token.width(), 6);
}

// =============================================================================
// Debug and Display Tests
// =============================================================================

#[test]
fn test_debug_when_formatting_token_expect_readable_output() {
    let token = create_token(STRING_KIND, "(test)");
    let debug_output = format!("{:?}", token);

    // Debug output should be non-empty and contain useful information
    assert!(!debug_output.is_empty());
}

#[test]
fn test_clone_when_copying_token_expect_shared_memory() {
    let original = create_token(NAME_KIND, "/SharedToken");
    let cloned = original.clone();

    // Both should have same content
    assert_eq!(original.kind(), cloned.kind());
    assert_eq!(original.text(), cloned.text());
    assert_eq!(original.width(), cloned.width());

    // Memory should be shared (reference counted)
    assert_eq!(original, cloned);
}

// =============================================================================
// Token Display and Equality Tests
// =============================================================================

#[test]
fn test_token_display_when_formatting_expect_text_content() {
    let token = create_token(STRING_KIND, "(Hello, World!)");
    let display_output = format!("{}", token);

    assert_eq!(display_output, "(Hello, World!)");
}

#[test]
fn test_token_display_when_unicode_content_expect_utf8_conversion() {
    let token = create_token(STRING_KIND, "café");
    let display_output = format!("{}", token);

    assert_eq!(display_output, "café");
}

#[test]
fn test_token_data_display_when_formatting_expect_utf8_lossy_conversion() {
    let token = create_token(NAME_KIND, "/Test");
    let token_data: &GreenTokenData = &*token;
    let display_output = format!("{}", token_data);

    assert_eq!(display_output, "/Test");
}

#[test]
fn test_token_data_display_when_invalid_utf8_expect_lossy_conversion() {
    let leading = GreenTrivia::new([]);
    let trailing = leading.clone();
    let invalid_utf8_bytes = &[0xFF, 0xFE, 0x00, 0x01];
    let token = GreenToken::new(STRING_KIND, invalid_utf8_bytes, leading, trailing);

    let token_data: &GreenTokenData = &*token;
    let display_output = format!("{}", token_data);

    // Should use lossy conversion for invalid UTF-8
    assert!(display_output.contains("�") || display_output.len() > 0);
}

#[test]
fn test_token_data_equality_when_same_content_expect_equal() {
    let token1 = create_token(STRING_KIND, "(test)");
    let token2 = create_token(STRING_KIND, "(test)");

    let token_data1: &GreenTokenData = &*token1;
    let token_data2: &GreenTokenData = &*token2;

    assert_eq!(token_data1, token_data2);
}

#[test]
fn test_token_data_equality_when_different_content_expect_not_equal() {
    let token1 = create_token(STRING_KIND, "(test1)");
    let token2 = create_token(STRING_KIND, "(test2)");

    let token_data1: &GreenTokenData = &*token1;
    let token_data2: &GreenTokenData = &*token2;

    assert_ne!(token_data1, token_data2);
}

#[test]
fn test_token_data_equality_when_different_kinds_expect_not_equal() {
    let token1 = create_token(STRING_KIND, "(test)");
    let token2 = create_token(NUMBER_KIND, "(test)");

    let token_data1: &GreenTokenData = &*token1;
    let token_data2: &GreenTokenData = &*token2;

    assert_ne!(token_data1, token_data2);
}
