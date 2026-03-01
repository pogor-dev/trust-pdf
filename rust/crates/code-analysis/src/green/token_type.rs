use std::{fmt, ops::Deref};

/// Generic token discriminated union for plain and valued token variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenType<T1, T2, T3, T4, T5, T6, T7, T8> {
    Token(T1),
    TokenWithTrivia(T2),
    TokenWithIntValue(T3),
    TokenWithFloatValue(T4),
    TokenWithStringValue(T5),
    TokenWithIntValueAndTrivia(T6),
    TokenWithFloatValueAndTrivia(T7),
    TokenWithStringValueAndTrivia(T8),
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> TokenType<T1, T2, T3, T4, T5, T6, T7, T8> {
    pub fn into_token(self) -> Option<T1> {
        match self {
            TokenType::Token(token) => Some(token),
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn as_token(&self) -> Option<&T1> {
        match self {
            TokenType::Token(token) => Some(token),
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn into_token_with_trivia(self) -> Option<T2> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(token) => Some(token),
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn as_token_with_trivia(&self) -> Option<&T2> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(token) => Some(token),
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn into_token_with_int_value(self) -> Option<T3> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(token) => Some(token),
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn as_token_with_int_value(&self) -> Option<&T3> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(token) => Some(token),
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn into_token_with_float_value(self) -> Option<T4> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(token) => Some(token),
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn as_token_with_float_value(&self) -> Option<&T4> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(token) => Some(token),
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn into_token_with_string_value(self) -> Option<T5> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(token) => Some(token),
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn as_token_with_string_value(&self) -> Option<&T5> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(token) => Some(token),
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn into_token_with_int_value_and_trivia(self) -> Option<T6> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(token) => Some(token),
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn as_token_with_int_value_and_trivia(&self) -> Option<&T6> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(token) => Some(token),
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn into_token_with_float_value_and_trivia(self) -> Option<T7> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(token) => Some(token),
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn as_token_with_float_value_and_trivia(&self) -> Option<&T7> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(token) => Some(token),
            TokenType::TokenWithStringValueAndTrivia(_) => None,
        }
    }

    pub fn into_token_with_string_value_and_trivia(self) -> Option<T8> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(token) => Some(token),
        }
    }

    pub fn as_token_with_string_value_and_trivia(&self) -> Option<&T8> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithTrivia(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
            TokenType::TokenWithIntValueAndTrivia(_) => None,
            TokenType::TokenWithFloatValueAndTrivia(_) => None,
            TokenType::TokenWithStringValueAndTrivia(token) => Some(token),
        }
    }
}

impl<T1: Deref, T2: Deref, T3: Deref, T4: Deref, T5: Deref, T6: Deref, T7: Deref, T8: Deref> TokenType<T1, T2, T3, T4, T5, T6, T7, T8> {
    pub(crate) fn as_deref(&self) -> TokenType<&T1::Target, &T2::Target, &T3::Target, &T4::Target, &T5::Target, &T6::Target, &T7::Target, &T8::Target> {
        match self {
            TokenType::Token(token) => TokenType::Token(token),
            TokenType::TokenWithTrivia(token) => TokenType::TokenWithTrivia(token),
            TokenType::TokenWithIntValue(token) => TokenType::TokenWithIntValue(token),
            TokenType::TokenWithFloatValue(token) => TokenType::TokenWithFloatValue(token),
            TokenType::TokenWithStringValue(token) => TokenType::TokenWithStringValue(token),
            TokenType::TokenWithIntValueAndTrivia(token) => TokenType::TokenWithIntValueAndTrivia(token),
            TokenType::TokenWithFloatValueAndTrivia(token) => TokenType::TokenWithFloatValueAndTrivia(token),
            TokenType::TokenWithStringValueAndTrivia(token) => TokenType::TokenWithStringValueAndTrivia(token),
        }
    }
}

impl<T1: fmt::Display, T2: fmt::Display, T3: fmt::Display, T4: fmt::Display, T5: fmt::Display, T6: fmt::Display, T7: fmt::Display, T8: fmt::Display>
    fmt::Display for TokenType<T1, T2, T3, T4, T5, T6, T7, T8>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Token(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithTrivia(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithIntValue(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithFloatValue(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithStringValue(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithIntValueAndTrivia(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithFloatValueAndTrivia(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithStringValueAndTrivia(token) => fmt::Display::fmt(token, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TokenType;
    use crate::{
        GreenToken, GreenTokenWithFloatValue, GreenTokenWithFloatValueAndTrivia, GreenTokenWithIntValue, GreenTokenWithIntValueAndTrivia,
        GreenTokenWithStringValue, GreenTokenWithStringValueAndTrivia, GreenTokenWithTrivia, SyntaxKind,
    };

    type TestTokenType = TokenType<
        GreenToken,
        GreenTokenWithTrivia,
        GreenTokenWithIntValue,
        GreenTokenWithFloatValue,
        GreenTokenWithStringValue,
        GreenTokenWithIntValueAndTrivia,
        GreenTokenWithFloatValueAndTrivia,
        GreenTokenWithStringValueAndTrivia,
    >;

    #[test]
    fn test_into_token_when_token_variant_expect_some() {
        let token = GreenToken::new(SyntaxKind::OpenBracketToken);
        let element: TokenType<
            GreenToken,
            GreenTokenWithTrivia,
            GreenTokenWithIntValue,
            GreenTokenWithFloatValue,
            GreenTokenWithStringValue,
            GreenTokenWithIntValueAndTrivia,
            GreenTokenWithFloatValueAndTrivia,
            GreenTokenWithStringValueAndTrivia,
        > = TokenType::Token(token.clone());
        assert_eq!(element.into_token().map(|t| t.kind()), Some(SyntaxKind::OpenBracketToken));
    }

    #[test]
    fn test_into_token_with_trivia_when_variant_expect_some() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let element: TokenType<
            GreenToken,
            GreenTokenWithTrivia,
            GreenTokenWithIntValue,
            GreenTokenWithFloatValue,
            GreenTokenWithStringValue,
            GreenTokenWithIntValueAndTrivia,
            GreenTokenWithFloatValueAndTrivia,
            GreenTokenWithStringValueAndTrivia,
        > = TokenType::TokenWithTrivia(token.clone());
        assert_eq!(element.into_token_with_trivia().map(|t| t.kind()), Some(SyntaxKind::TrueKeyword));
    }

    #[test]
    fn test_into_token_with_int_value_when_variant_expect_some() {
        let token = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        let element: TokenType<
            GreenToken,
            GreenTokenWithTrivia,
            GreenTokenWithIntValue,
            GreenTokenWithFloatValue,
            GreenTokenWithStringValue,
            GreenTokenWithIntValueAndTrivia,
            GreenTokenWithFloatValueAndTrivia,
            GreenTokenWithStringValueAndTrivia,
        > = TokenType::TokenWithIntValue(token.clone());
        assert_eq!(element.into_token_with_int_value().map(|t| t.kind()), Some(SyntaxKind::NumericLiteralToken));
    }

    #[test]
    fn test_into_token_with_float_value_when_variant_expect_some() {
        let token = GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14);
        let element: TokenType<
            GreenToken,
            GreenTokenWithTrivia,
            GreenTokenWithIntValue,
            GreenTokenWithFloatValue,
            GreenTokenWithStringValue,
            GreenTokenWithIntValueAndTrivia,
            GreenTokenWithFloatValueAndTrivia,
            GreenTokenWithStringValueAndTrivia,
        > = TokenType::TokenWithFloatValue(token.clone());
        assert_eq!(element.into_token_with_float_value().map(|t| t.kind()), Some(SyntaxKind::NumericLiteralToken));
    }

    #[test]
    fn test_into_token_with_string_value_when_variant_expect_some() {
        let token = GreenTokenWithStringValue::new(SyntaxKind::StringLiteralToken, b"hello", "world".to_string());
        let element: TokenType<
            GreenToken,
            GreenTokenWithTrivia,
            GreenTokenWithIntValue,
            GreenTokenWithFloatValue,
            GreenTokenWithStringValue,
            GreenTokenWithIntValueAndTrivia,
            GreenTokenWithFloatValueAndTrivia,
            GreenTokenWithStringValueAndTrivia,
        > = TokenType::TokenWithStringValue(token.clone());
        assert_eq!(element.into_token_with_string_value().map(|t| t.kind()), Some(SyntaxKind::StringLiteralToken));
    }

    #[test]
    fn test_into_token_with_value_and_trivia_when_variant_expect_some() {
        let int_token = GreenTokenWithIntValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 42, None, None);
        let float_token = GreenTokenWithFloatValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5, None, None);
        let string_token = GreenTokenWithStringValueAndTrivia::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string(), None, None);

        let int_element: TokenType<
            GreenToken,
            GreenTokenWithTrivia,
            GreenTokenWithIntValue,
            GreenTokenWithFloatValue,
            GreenTokenWithStringValue,
            GreenTokenWithIntValueAndTrivia,
            GreenTokenWithFloatValueAndTrivia,
            GreenTokenWithStringValueAndTrivia,
        > = TokenType::TokenWithIntValueAndTrivia(int_token);

        let float_element: TokenType<
            GreenToken,
            GreenTokenWithTrivia,
            GreenTokenWithIntValue,
            GreenTokenWithFloatValue,
            GreenTokenWithStringValue,
            GreenTokenWithIntValueAndTrivia,
            GreenTokenWithFloatValueAndTrivia,
            GreenTokenWithStringValueAndTrivia,
        > = TokenType::TokenWithFloatValueAndTrivia(float_token);

        let string_element: TokenType<
            GreenToken,
            GreenTokenWithTrivia,
            GreenTokenWithIntValue,
            GreenTokenWithFloatValue,
            GreenTokenWithStringValue,
            GreenTokenWithIntValueAndTrivia,
            GreenTokenWithFloatValueAndTrivia,
            GreenTokenWithStringValueAndTrivia,
        > = TokenType::TokenWithStringValueAndTrivia(string_token);

        assert!(int_element.into_token_with_int_value_and_trivia().is_some());
        assert!(float_element.into_token_with_float_value_and_trivia().is_some());
        assert!(string_element.into_token_with_string_value_and_trivia().is_some());
    }

    #[test]
    fn test_as_accessors_when_matching_variants_expect_some() {
        let token = GreenToken::new(SyntaxKind::CloseBracketToken);
        let trivia_token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let int_token = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        let float_token = GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14);
        let string_token = GreenTokenWithStringValue::new(SyntaxKind::StringLiteralToken, b"hello", "world".to_string());

        let element1: TestTokenType = TokenType::Token(token);
        let element2: TestTokenType = TokenType::TokenWithTrivia(trivia_token);
        let element3: TestTokenType = TokenType::TokenWithIntValue(int_token);
        let element4: TestTokenType = TokenType::TokenWithFloatValue(float_token);
        let element5: TestTokenType = TokenType::TokenWithStringValue(string_token);

        assert!(element1.as_token().is_some());
        assert!(element2.as_token_with_trivia().is_some());
        assert!(element3.as_token_with_int_value().is_some());
        assert!(element4.as_token_with_float_value().is_some());
        assert!(element5.as_token_with_string_value().is_some());
    }

    #[test]
    fn test_display_when_each_variant_expect_inner_display() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        let trivia_token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let int_token = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"int", 123);
        let float_token = GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14);
        let string_token = GreenTokenWithStringValue::new(SyntaxKind::StringLiteralToken, b"s", "string".to_string());

        let element1: TestTokenType = TokenType::Token(token);
        let element2: TestTokenType = TokenType::TokenWithTrivia(trivia_token);
        let element3: TestTokenType = TokenType::TokenWithIntValue(int_token);
        let element4: TestTokenType = TokenType::TokenWithFloatValue(float_token);
        let element5: TestTokenType = TokenType::TokenWithStringValue(string_token);

        // Verify display works for each variant
        let _1 = element1.to_string();
        let _2 = element2.to_string();
        let _3 = element3.to_string();
        let _4 = element4.to_string();
        let _5 = element5.to_string();

        assert!(_1.len() > 0);
        assert!(_2.len() > 0);
        assert!(_3.len() > 0);
        assert!(_4.len() > 0);
        assert!(_5.len() > 0);
    }
}
