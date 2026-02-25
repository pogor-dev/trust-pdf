use std::{fmt, ops::Deref};

/// Generic token discriminated union for plain and valued token variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenType<T1, T2, T3, T4> {
    Token(T1),
    TokenWithIntValue(T2),
    TokenWithFloatValue(T3),
    TokenWithStringValue(T4),
}

impl<T1, T2, T3, T4> TokenType<T1, T2, T3, T4> {
    pub fn into_token(self) -> Option<T1> {
        match self {
            TokenType::Token(token) => Some(token),
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
        }
    }

    pub fn as_token(&self) -> Option<&T1> {
        match self {
            TokenType::Token(token) => Some(token),
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
        }
    }

    pub fn into_token_with_int_value(self) -> Option<T2> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithIntValue(token) => Some(token),
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
        }
    }

    pub fn as_token_with_int_value(&self) -> Option<&T2> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithIntValue(token) => Some(token),
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(_) => None,
        }
    }

    pub fn into_token_with_float_value(self) -> Option<T3> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(token) => Some(token),
            TokenType::TokenWithStringValue(_) => None,
        }
    }

    pub fn as_token_with_float_value(&self) -> Option<&T3> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(token) => Some(token),
            TokenType::TokenWithStringValue(_) => None,
        }
    }

    pub fn into_token_with_string_value(self) -> Option<T4> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(token) => Some(token),
        }
    }

    pub fn as_token_with_string_value(&self) -> Option<&T4> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithIntValue(_) => None,
            TokenType::TokenWithFloatValue(_) => None,
            TokenType::TokenWithStringValue(token) => Some(token),
        }
    }
}

impl<T1: Deref, T2: Deref, T3: Deref, T4: Deref> TokenType<T1, T2, T3, T4> {
    pub(crate) fn as_deref(&self) -> TokenType<&T1::Target, &T2::Target, &T3::Target, &T4::Target> {
        match self {
            TokenType::Token(token) => TokenType::Token(token),
            TokenType::TokenWithIntValue(token) => TokenType::TokenWithIntValue(token),
            TokenType::TokenWithFloatValue(token) => TokenType::TokenWithFloatValue(token),
            TokenType::TokenWithStringValue(token) => TokenType::TokenWithStringValue(token),
        }
    }
}

impl<T1: fmt::Display, T2: fmt::Display, T3: fmt::Display, T4: fmt::Display> fmt::Display for TokenType<T1, T2, T3, T4> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Token(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithIntValue(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithFloatValue(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithStringValue(token) => fmt::Display::fmt(token, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TokenType;
    use crate::{GreenToken, GreenTokenWithFloatValue, GreenTokenWithIntValue, GreenTokenWithStringValue, SyntaxKind};

    #[test]
    fn test_into_token_when_token_variant_expect_some() {
        let token = GreenToken::new(SyntaxKind::OpenBracketToken);
        let element: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> = TokenType::Token(token.clone());
        assert_eq!(element.into_token().map(|t| t.kind()), Some(SyntaxKind::OpenBracketToken));
    }

    #[test]
    fn test_into_token_with_int_value_when_variant_expect_some() {
        let token = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        let element: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> =
            TokenType::TokenWithIntValue(token.clone());
        assert_eq!(element.into_token_with_int_value().map(|t| t.kind()), Some(SyntaxKind::NumericLiteralToken));
    }

    #[test]
    fn test_into_token_with_float_value_when_variant_expect_some() {
        let token = GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14);
        let element: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> =
            TokenType::TokenWithFloatValue(token.clone());
        assert_eq!(element.into_token_with_float_value().map(|t| t.kind()), Some(SyntaxKind::NumericLiteralToken));
    }

    #[test]
    fn test_into_token_with_string_value_when_variant_expect_some() {
        let token = GreenTokenWithStringValue::new(SyntaxKind::StringLiteralToken, b"hello", "world".to_string());
        let element: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> =
            TokenType::TokenWithStringValue(token.clone());
        assert_eq!(element.into_token_with_string_value().map(|t| t.kind()), Some(SyntaxKind::StringLiteralToken));
    }

    #[test]
    fn test_as_accessors_when_matching_variants_expect_some() {
        let token = GreenToken::new(SyntaxKind::CloseBracketToken);
        let int_token = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        let float_token = GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14);
        let string_token = GreenTokenWithStringValue::new(SyntaxKind::StringLiteralToken, b"hello", "world".to_string());

        let element1: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> = TokenType::Token(token);
        let element2: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> =
            TokenType::TokenWithIntValue(int_token);
        let element3: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> =
            TokenType::TokenWithFloatValue(float_token);
        let element4: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> =
            TokenType::TokenWithStringValue(string_token);

        assert!(element1.as_token().is_some());
        assert!(element2.as_token_with_int_value().is_some());
        assert!(element3.as_token_with_float_value().is_some());
        assert!(element4.as_token_with_string_value().is_some());
    }

    #[test]
    fn test_display_when_each_variant_expect_inner_display() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        let int_token = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"int", 123);
        let float_token = GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14);
        let string_token = GreenTokenWithStringValue::new(SyntaxKind::StringLiteralToken, b"s", "string".to_string());

        let element1: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> = TokenType::Token(token);
        let element2: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> =
            TokenType::TokenWithIntValue(int_token);
        let element3: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> =
            TokenType::TokenWithFloatValue(float_token);
        let element4: TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue> =
            TokenType::TokenWithStringValue(string_token);

        // Verify display works for each variant
        let _1 = element1.to_string();
        let _2 = element2.to_string();
        let _3 = element3.to_string();
        let _4 = element4.to_string();

        assert!(_1.len() > 0);
        assert!(_2.len() > 0);
        assert!(_3.len() > 0);
        assert!(_4.len() > 0);
    }
}
