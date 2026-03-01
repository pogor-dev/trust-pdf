use crate::{
    GreenFlags, GreenNode, GreenToken, GreenTokenData, GreenTokenWithFloatValue, GreenTokenWithFloatValueData, GreenTokenWithIntValue,
    GreenTokenWithIntValueData, GreenTokenWithStringValue, GreenTokenWithStringValueData, GreenTokenWithTrivia, GreenTokenWithTriviaData, SyntaxKind,
    green::TokenType,
};

/// Concrete token element used in node slots.
pub type GreenTokenElement = TokenType<GreenToken, GreenTokenWithTrivia, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue>;

pub(crate) type GreenTokenElementRef<'a> = TokenType<
    &'a GreenTokenData,
    &'a GreenTokenWithTriviaData,
    &'a GreenTokenWithIntValueData,
    &'a GreenTokenWithFloatValueData,
    &'a GreenTokenWithStringValueData,
>;

impl GreenTokenElement {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenTokenElement::Token(t) => t.kind(),
            GreenTokenElement::TokenWithTrivia(t) => t.kind(),
            GreenTokenElement::TokenWithIntValue(t) => t.kind(),
            GreenTokenElement::TokenWithFloatValue(t) => t.kind(),
            GreenTokenElement::TokenWithStringValue(t) => t.kind(),
        }
    }

    #[inline]
    pub fn text(&self) -> Vec<u8> {
        match self {
            GreenTokenElement::Token(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithTrivia(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithIntValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithFloatValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithStringValue(t) => t.text().to_vec(),
        }
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        match self {
            GreenTokenElement::Token(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithTrivia(t) => t.full_text(),
            GreenTokenElement::TokenWithIntValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithFloatValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithStringValue(t) => t.text().to_vec(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenTokenElement::Token(t) => t.width().into(),
            GreenTokenElement::TokenWithTrivia(t) => t.width().into(),
            GreenTokenElement::TokenWithIntValue(t) => t.width().into(),
            GreenTokenElement::TokenWithFloatValue(t) => t.width().into(),
            GreenTokenElement::TokenWithStringValue(t) => t.width().into(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            GreenTokenElement::Token(t) => t.width().into(),
            GreenTokenElement::TokenWithTrivia(t) => t.full_width().into(),
            GreenTokenElement::TokenWithIntValue(t) => t.width().into(),
            GreenTokenElement::TokenWithFloatValue(t) => t.width().into(),
            GreenTokenElement::TokenWithStringValue(t) => t.width().into(),
        }
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenTokenElement::Token(_t) => None,
            GreenTokenElement::TokenWithTrivia(t) => t.leading_trivia(),
            GreenTokenElement::TokenWithIntValue(_t) => None,
            GreenTokenElement::TokenWithFloatValue(_t) => None,
            GreenTokenElement::TokenWithStringValue(_t) => None,
        }
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenTokenElement::Token(_t) => None,
            GreenTokenElement::TokenWithTrivia(t) => t.trailing_trivia(),
            GreenTokenElement::TokenWithIntValue(_t) => None,
            GreenTokenElement::TokenWithFloatValue(_t) => None,
            GreenTokenElement::TokenWithStringValue(_t) => None,
        }
    }
    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        match self {
            GreenTokenElement::Token(t) => t.flags(),
            GreenTokenElement::TokenWithTrivia(t) => t.flags(),
            GreenTokenElement::TokenWithIntValue(t) => t.flags(),
            GreenTokenElement::TokenWithFloatValue(t) => t.flags(),
            GreenTokenElement::TokenWithStringValue(t) => t.flags(),
        }
    }

    #[inline]
    pub(crate) fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        match self {
            GreenTokenElement::Token(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithTrivia(t) => t.write_to(leading, trailing),
            GreenTokenElement::TokenWithIntValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithFloatValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithStringValue(t) => t.text().to_vec(),
        }
    }
}

impl From<GreenToken> for GreenTokenElement {
    fn from(token: GreenToken) -> GreenTokenElement {
        GreenTokenElement::Token(token)
    }
}

impl From<GreenTokenWithIntValue> for GreenTokenElement {
    fn from(token: GreenTokenWithIntValue) -> GreenTokenElement {
        GreenTokenElement::TokenWithIntValue(token)
    }
}

impl From<GreenTokenWithTrivia> for GreenTokenElement {
    fn from(token: GreenTokenWithTrivia) -> GreenTokenElement {
        GreenTokenElement::TokenWithTrivia(token)
    }
}

impl From<GreenTokenWithFloatValue> for GreenTokenElement {
    fn from(token: GreenTokenWithFloatValue) -> GreenTokenElement {
        GreenTokenElement::TokenWithFloatValue(token)
    }
}

impl From<GreenTokenWithStringValue> for GreenTokenElement {
    fn from(token: GreenTokenWithStringValue) -> GreenTokenElement {
        GreenTokenElement::TokenWithStringValue(token)
    }
}

impl<'a> GreenTokenElementRef<'a> {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenTokenElementRef::Token(t) => t.kind(),
            GreenTokenElementRef::TokenWithTrivia(t) => t.kind(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.kind(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.kind(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.kind(),
        }
    }

    #[inline]
    pub fn text(&self) -> &'a [u8] {
        match self {
            GreenTokenElementRef::Token(t) => t.text(),
            GreenTokenElementRef::TokenWithTrivia(t) => t.text(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.text(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.text(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.text(),
        }
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        match self {
            GreenTokenElementRef::Token(t) => t.text().to_vec(),
            GreenTokenElementRef::TokenWithTrivia(t) => t.full_text(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.text().to_vec(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.text().to_vec(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.text().to_vec(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenTokenElementRef::Token(t) => t.width().into(),
            GreenTokenElementRef::TokenWithTrivia(t) => t.width().into(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.width().into(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.width().into(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.width().into(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            GreenTokenElementRef::Token(t) => t.width().into(),
            GreenTokenElementRef::TokenWithTrivia(t) => t.full_width().into(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.width().into(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.width().into(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.width().into(),
        }
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenTokenElementRef::Token(_t) => None,
            GreenTokenElementRef::TokenWithTrivia(t) => t.leading_trivia(),
            GreenTokenElementRef::TokenWithIntValue(_t) => None,
            GreenTokenElementRef::TokenWithFloatValue(_t) => None,
            GreenTokenElementRef::TokenWithStringValue(_t) => None,
        }
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenTokenElementRef::Token(_t) => None,
            GreenTokenElementRef::TokenWithTrivia(t) => t.trailing_trivia(),
            GreenTokenElementRef::TokenWithIntValue(_t) => None,
            GreenTokenElementRef::TokenWithFloatValue(_t) => None,
            GreenTokenElementRef::TokenWithStringValue(_t) => None,
        }
    }
    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        match self {
            GreenTokenElementRef::Token(t) => t.flags(),
            GreenTokenElementRef::TokenWithTrivia(t) => t.flags(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.flags(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.flags(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.flags(),
        }
    }

    #[inline]
    pub(crate) fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        match self {
            GreenTokenElementRef::Token(t) => t.text().to_vec(),
            GreenTokenElementRef::TokenWithTrivia(t) => t.write_to(leading, trailing),
            GreenTokenElementRef::TokenWithIntValue(t) => t.text().to_vec(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.text().to_vec(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.text().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GreenTrivia;

    fn leading_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ").into()],
        ))
    }

    fn trailing_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::EndOfLineTrivia, b"\n").into()],
        ))
    }

    fn create_owned_variants() -> [GreenTokenElement; 5] {
        [
            GreenTokenElement::Token(GreenToken::new(SyntaxKind::TrueKeyword)),
            GreenTokenElement::TokenWithTrivia(GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia())),
            GreenTokenElement::TokenWithIntValue(GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42)),
            GreenTokenElement::TokenWithFloatValue(GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5)),
            GreenTokenElement::TokenWithStringValue(GreenTokenWithStringValue::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string())),
        ]
    }

    #[test]
    fn test_kind_when_owned_variants_expect_variant_kind() {
        let variants = create_owned_variants();
        assert_eq!(variants[0].kind(), SyntaxKind::TrueKeyword);
        assert_eq!(variants[1].kind(), SyntaxKind::TrueKeyword);
        assert_eq!(variants[2].kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(variants[3].kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(variants[4].kind(), SyntaxKind::NameLiteralToken);
    }

    #[test]
    fn test_text_and_width_when_owned_variants_expect_consistent_lengths() {
        for variant in create_owned_variants() {
            let text = variant.text();
            assert_eq!(variant.width(), text.len() as u32);

            if let GreenTokenElement::TokenWithTrivia(_) = variant {
                assert_eq!(variant.full_width(), b" true\n".len() as u32);
                assert_eq!(variant.full_text(), b" true\n");
            } else {
                assert_eq!(variant.full_width(), text.len() as u32);
                assert_eq!(variant.full_text(), text);
            }
        }
    }

    #[test]
    fn test_trivia_accessors_when_owned_variants_expect_none() {
        let variants = create_owned_variants();
        for variant in [variants[0].clone(), variants[2].clone(), variants[3].clone(), variants[4].clone()] {
            assert_eq!(variant.leading_trivia(), None);
            assert_eq!(variant.trailing_trivia(), None);
        }

        assert!(variants[1].leading_trivia().is_some());
        assert!(variants[1].trailing_trivia().is_some());
    }

    #[test]
    fn test_flags_when_owned_variants_expect_is_not_missing() {
        for variant in create_owned_variants() {
            assert!(variant.flags().contains(GreenFlags::IS_NOT_MISSING));
        }
    }

    #[test]
    fn test_methods_when_ref_variants_expect_same_behavior_as_owned() {
        for owned in create_owned_variants() {
            let reference: GreenTokenElementRef<'_> = owned.as_deref();

            assert_eq!(reference.kind(), owned.kind());
            assert_eq!(reference.text(), owned.text().as_slice());
            assert_eq!(reference.width(), owned.width());
            assert_eq!(reference.full_width(), owned.full_width());

            if let GreenTokenElement::TokenWithTrivia(_) = owned {
                assert_eq!(reference.full_text(), owned.full_text());
                assert!(reference.leading_trivia().is_some());
                assert!(reference.trailing_trivia().is_some());
            } else {
                assert_eq!(reference.full_text(), owned.full_text());
                assert_eq!(reference.leading_trivia(), None);
                assert_eq!(reference.trailing_trivia(), None);
            }

            assert_eq!(reference.flags(), owned.flags());
        }
    }

    #[test]
    fn test_from_when_concrete_tokens_expect_matching_variants() {
        let plain: GreenTokenElement = GreenToken::new(SyntaxKind::TrueKeyword).into();
        let with_trivia: GreenTokenElement = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia()).into();
        let int_value: GreenTokenElement = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42).into();
        let float_value: GreenTokenElement = GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5).into();
        let string_value: GreenTokenElement = GreenTokenWithStringValue::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string()).into();

        assert!(matches!(plain, GreenTokenElement::Token(_)));
        assert!(matches!(with_trivia, GreenTokenElement::TokenWithTrivia(_)));
        assert!(matches!(int_value, GreenTokenElement::TokenWithIntValue(_)));
        assert!(matches!(float_value, GreenTokenElement::TokenWithFloatValue(_)));
        assert!(matches!(string_value, GreenTokenElement::TokenWithStringValue(_)));
    }
}
