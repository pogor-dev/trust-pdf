use std::string;

use crate::{SyntaxKind, green::token::GreenToken};

// Test constants for different PDF trivia types
const STRING_KIND: SyntaxKind = SyntaxKind(1);
const NUMBER_KIND: SyntaxKind = SyntaxKind(2);
const NULL_KIND: SyntaxKind = SyntaxKind(3);

/// Helper function to create test trivia with different content types
fn create_token(kind: SyntaxKind, text: &str) -> GreenToken {
    GreenToken::new(kind, text.as_bytes())
}

#[test]
fn test_token_kind() {
    let string_token = create_token(STRING_KIND, "(Hello)");
    let number_token = create_token(NUMBER_KIND, "123");
    let null_token = create_token(NULL_KIND, "null");

    assert_eq!(string_token.kind(), STRING_KIND);
    assert_eq!(number_token.kind(), NUMBER_KIND);
    assert_eq!(null_token.kind(), NULL_KIND);
}
