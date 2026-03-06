use std::{fmt, ops::Deref};

/// Generic token discriminated union for plain and valued token variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenType<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> {
    Token(T1),
    TokenWithTrivia(T2),
    TokenWithIntValue(T3),
    TokenWithFloatValue(T4),
    TokenWithStringValue(T5),
    TokenWithTrailingTrivia(T6),
    TokenWithIntValueAndTrivia(T7),
    TokenWithFloatValueAndTrivia(T8),
    TokenWithStringValueAndTrivia(T9),
    TokenWithIntValueAndTrailingTrivia(T10),
    TokenWithFloatValueAndTrailingTrivia(T11),
    TokenWithStringValueAndTrailingTrivia(T12),
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> TokenType<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> {
    impl_token_type_accessors!(
        (into_token, as_token, Token, T1),
        (into_token_with_trivia, as_token_with_trivia, TokenWithTrivia, T2),
        (into_token_with_int_value, as_token_with_int_value, TokenWithIntValue, T3),
        (into_token_with_float_value, as_token_with_float_value, TokenWithFloatValue, T4),
        (into_token_with_string_value, as_token_with_string_value, TokenWithStringValue, T5),
        (into_token_with_trailing_trivia, as_token_with_trailing_trivia, TokenWithTrailingTrivia, T6),
        (
            into_token_with_int_value_and_trivia,
            as_token_with_int_value_and_trivia,
            TokenWithIntValueAndTrivia,
            T7
        ),
        (
            into_token_with_float_value_and_trivia,
            as_token_with_float_value_and_trivia,
            TokenWithFloatValueAndTrivia,
            T8
        ),
        (
            into_token_with_string_value_and_trivia,
            as_token_with_string_value_and_trivia,
            TokenWithStringValueAndTrivia,
            T9
        ),
        (
            into_token_with_int_value_and_trailing_trivia,
            as_token_with_int_value_and_trailing_trivia,
            TokenWithIntValueAndTrailingTrivia,
            T10
        ),
        (
            into_token_with_float_value_and_trailing_trivia,
            as_token_with_float_value_and_trailing_trivia,
            TokenWithFloatValueAndTrailingTrivia,
            T11
        ),
        (
            into_token_with_string_value_and_trailing_trivia,
            as_token_with_string_value_and_trailing_trivia,
            TokenWithStringValueAndTrailingTrivia,
            T12
        ),
    );
}

impl<T1: Deref, T2: Deref, T3: Deref, T4: Deref, T5: Deref, T6: Deref, T7: Deref, T8: Deref, T9: Deref, T10: Deref, T11: Deref, T12: Deref>
    TokenType<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>
{
    pub(crate) fn as_deref(
        &self,
    ) -> TokenType<
        &T1::Target,
        &T2::Target,
        &T3::Target,
        &T4::Target,
        &T5::Target,
        &T6::Target,
        &T7::Target,
        &T8::Target,
        &T9::Target,
        &T10::Target,
        &T11::Target,
        &T12::Target,
    > {
        match self {
            TokenType::Token(token) => TokenType::Token(token),
            TokenType::TokenWithTrivia(token) => TokenType::TokenWithTrivia(token),
            TokenType::TokenWithIntValue(token) => TokenType::TokenWithIntValue(token),
            TokenType::TokenWithFloatValue(token) => TokenType::TokenWithFloatValue(token),
            TokenType::TokenWithStringValue(token) => TokenType::TokenWithStringValue(token),
            TokenType::TokenWithTrailingTrivia(token) => TokenType::TokenWithTrailingTrivia(token),
            TokenType::TokenWithIntValueAndTrivia(token) => TokenType::TokenWithIntValueAndTrivia(token),
            TokenType::TokenWithFloatValueAndTrivia(token) => TokenType::TokenWithFloatValueAndTrivia(token),
            TokenType::TokenWithStringValueAndTrivia(token) => TokenType::TokenWithStringValueAndTrivia(token),
            TokenType::TokenWithIntValueAndTrailingTrivia(token) => TokenType::TokenWithIntValueAndTrailingTrivia(token),
            TokenType::TokenWithFloatValueAndTrailingTrivia(token) => TokenType::TokenWithFloatValueAndTrailingTrivia(token),
            TokenType::TokenWithStringValueAndTrailingTrivia(token) => TokenType::TokenWithStringValueAndTrailingTrivia(token),
        }
    }
}

impl<
    T1: fmt::Display,
    T2: fmt::Display,
    T3: fmt::Display,
    T4: fmt::Display,
    T5: fmt::Display,
    T6: fmt::Display,
    T7: fmt::Display,
    T8: fmt::Display,
    T9: fmt::Display,
    T10: fmt::Display,
    T11: fmt::Display,
    T12: fmt::Display,
> fmt::Display for TokenType<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match_token_type!(self, t => fmt::Display::fmt(t, f))
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::TokenType;

    type U8TokenType = TokenType<u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8>;
    type PointerTokenType = TokenType<usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize, usize>;

    #[test]
    fn test_token_type_u8_payload_memory_layout() {
        // Small payloads still require a discriminant for the 12 variants.
        assert_eq!(std::mem::size_of::<U8TokenType>(), 2);
        assert_eq!(std::mem::align_of::<U8TokenType>(), 1);
    }

    #[test]
    fn test_token_type_pointer_payload_memory_layout() {
        // Pointer-sized payload + discriminant rounds up to pointer alignment.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<PointerTokenType>(), 16);
            assert_eq!(std::mem::align_of::<PointerTokenType>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<PointerTokenType>(), 8);
            assert_eq!(std::mem::align_of::<PointerTokenType>(), 4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TokenType;
    use crate::{
        GreenToken, GreenTokenWithFloatValue, GreenTokenWithFloatValueAndTrailingTrivia, GreenTokenWithFloatValueAndTrivia, GreenTokenWithIntValue,
        GreenTokenWithIntValueAndTrailingTrivia, GreenTokenWithIntValueAndTrivia, GreenTokenWithStringValue, GreenTokenWithStringValueAndTrailingTrivia,
        GreenTokenWithStringValueAndTrivia, GreenTokenWithTrailingTrivia, GreenTokenWithTrivia, SyntaxKind,
    };

    type TestTokenType = TokenType<
        GreenToken,
        GreenTokenWithTrivia,
        GreenTokenWithIntValue,
        GreenTokenWithFloatValue,
        GreenTokenWithStringValue,
        GreenTokenWithTrailingTrivia,
        GreenTokenWithIntValueAndTrivia,
        GreenTokenWithFloatValueAndTrivia,
        GreenTokenWithStringValueAndTrivia,
        GreenTokenWithIntValueAndTrailingTrivia,
        GreenTokenWithFloatValueAndTrailingTrivia,
        GreenTokenWithStringValueAndTrailingTrivia,
    >;

    #[test]
    fn test_into_token_when_token_variant_expect_some() {
        let token = GreenToken::new(SyntaxKind::OpenBracketToken);
        let element: TestTokenType = TokenType::Token(token.clone());
        assert_eq!(element.into_token().map(|t| t.kind()), Some(SyntaxKind::OpenBracketToken));
    }

    #[test]
    fn test_into_token_with_trivia_when_variant_expect_some() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let element: TestTokenType = TokenType::TokenWithTrivia(token.clone());
        assert_eq!(element.into_token_with_trivia().map(|t| t.kind()), Some(SyntaxKind::TrueKeyword));
    }

    #[test]
    fn test_into_token_with_int_value_when_variant_expect_some() {
        let token = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        let element: TestTokenType = TokenType::TokenWithIntValue(token.clone());
        assert_eq!(element.into_token_with_int_value().map(|t| t.kind()), Some(SyntaxKind::NumericLiteralToken));
    }

    #[test]
    fn test_into_token_with_float_value_when_variant_expect_some() {
        let token = GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14);
        let element: TestTokenType = TokenType::TokenWithFloatValue(token.clone());
        assert_eq!(element.into_token_with_float_value().map(|t| t.kind()), Some(SyntaxKind::NumericLiteralToken));
    }

    #[test]
    fn test_into_token_with_string_value_when_variant_expect_some() {
        let token = GreenTokenWithStringValue::new(SyntaxKind::StringLiteralToken, b"hello", "world".to_string());
        let element: TestTokenType = TokenType::TokenWithStringValue(token.clone());
        assert_eq!(element.into_token_with_string_value().map(|t| t.kind()), Some(SyntaxKind::StringLiteralToken));
    }

    #[test]
    fn test_into_token_with_trailing_trivia_when_variant_expect_some() {
        let token = GreenTokenWithTrailingTrivia::new(SyntaxKind::TrueKeyword, None);
        let element: TestTokenType = TokenType::TokenWithTrailingTrivia(token);
        assert_eq!(element.into_token_with_trailing_trivia().map(|t| t.kind()), Some(SyntaxKind::TrueKeyword));
    }

    #[test]
    fn test_into_token_with_value_and_trivia_when_variant_expect_some() {
        let int_token = GreenTokenWithIntValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 42, None, None);
        let float_token = GreenTokenWithFloatValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5, None, None);
        let string_token = GreenTokenWithStringValueAndTrivia::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string(), None, None);

        let int_element: TestTokenType = TokenType::TokenWithIntValueAndTrivia(int_token);
        let float_element: TestTokenType = TokenType::TokenWithFloatValueAndTrivia(float_token);
        let string_element: TestTokenType = TokenType::TokenWithStringValueAndTrivia(string_token);

        assert!(int_element.into_token_with_int_value_and_trivia().is_some());
        assert!(float_element.into_token_with_float_value_and_trivia().is_some());
        assert!(string_element.into_token_with_string_value_and_trivia().is_some());
    }

    #[test]
    fn test_into_token_with_value_and_trailing_trivia_when_variant_expect_some() {
        let int_token = GreenTokenWithIntValueAndTrailingTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 42, None);
        let float_token = GreenTokenWithFloatValueAndTrailingTrivia::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5, None);
        let string_token = GreenTokenWithStringValueAndTrailingTrivia::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string(), None);

        let int_element: TestTokenType = TokenType::TokenWithIntValueAndTrailingTrivia(int_token);
        let float_element: TestTokenType = TokenType::TokenWithFloatValueAndTrailingTrivia(float_token);
        let string_element: TestTokenType = TokenType::TokenWithStringValueAndTrailingTrivia(string_token);

        assert!(int_element.into_token_with_int_value_and_trailing_trivia().is_some());
        assert!(float_element.into_token_with_float_value_and_trailing_trivia().is_some());
        assert!(string_element.into_token_with_string_value_and_trailing_trivia().is_some());
    }

    #[test]
    fn test_as_accessors_when_matching_variants_expect_some() {
        let token = GreenToken::new(SyntaxKind::CloseBracketToken);
        let trivia_token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let int_token = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        let float_token = GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14);
        let string_token = GreenTokenWithStringValue::new(SyntaxKind::StringLiteralToken, b"hello", "world".to_string());
        let trailing_token = GreenTokenWithTrailingTrivia::new(SyntaxKind::TrueKeyword, None);

        let element1: TestTokenType = TokenType::Token(token);
        let element2: TestTokenType = TokenType::TokenWithTrivia(trivia_token);
        let element3: TestTokenType = TokenType::TokenWithIntValue(int_token);
        let element4: TestTokenType = TokenType::TokenWithFloatValue(float_token);
        let element5: TestTokenType = TokenType::TokenWithStringValue(string_token);
        let element6: TestTokenType = TokenType::TokenWithTrailingTrivia(trailing_token);

        assert!(element1.as_token().is_some());
        assert!(element2.as_token_with_trivia().is_some());
        assert!(element3.as_token_with_int_value().is_some());
        assert!(element4.as_token_with_float_value().is_some());
        assert!(element5.as_token_with_string_value().is_some());
        assert!(element6.as_token_with_trailing_trivia().is_some());
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
