use std::sync::LazyLock;

use crate::{
    GreenDiagnostic, GreenFlags, GreenNode, GreenSyntaxFactory, GreenToken, GreenTokenData, GreenTokenWithFloatValue,
    GreenTokenWithFloatValueAndTrailingTrivia, GreenTokenWithFloatValueAndTrailingTriviaData, GreenTokenWithFloatValueAndTrivia,
    GreenTokenWithFloatValueAndTriviaData, GreenTokenWithFloatValueData, GreenTokenWithIntValue, GreenTokenWithIntValueAndTrailingTrivia,
    GreenTokenWithIntValueAndTrailingTriviaData, GreenTokenWithIntValueAndTrivia, GreenTokenWithIntValueAndTriviaData, GreenTokenWithIntValueData,
    GreenTokenWithStringValue, GreenTokenWithStringValueAndTrailingTrivia, GreenTokenWithStringValueAndTrailingTriviaData, GreenTokenWithStringValueAndTrivia,
    GreenTokenWithStringValueAndTriviaData, GreenTokenWithStringValueData, GreenTokenWithTrailingTrivia, GreenTokenWithTrailingTriviaData,
    GreenTokenWithTrivia, GreenTokenWithTriviaData, SyntaxKind, syntax::green::TokenType,
};

/// Concrete token element used in node slots.
pub(crate) type GreenTokenElement = TokenType<
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

pub(crate) type GreenTokenElementRef<'a> = TokenType<
    &'a GreenTokenData,
    &'a GreenTokenWithTriviaData,
    &'a GreenTokenWithIntValueData,
    &'a GreenTokenWithFloatValueData,
    &'a GreenTokenWithStringValueData,
    &'a GreenTokenWithTrailingTriviaData,
    &'a GreenTokenWithIntValueAndTriviaData,
    &'a GreenTokenWithFloatValueAndTriviaData,
    &'a GreenTokenWithStringValueAndTriviaData,
    &'a GreenTokenWithIntValueAndTrailingTriviaData,
    &'a GreenTokenWithFloatValueAndTrailingTriviaData,
    &'a GreenTokenWithStringValueAndTrailingTriviaData,
>;

impl GreenTokenElement {
    #[inline]
    pub(crate) fn create_with_trivia(kind: SyntaxKind, leading_trivia: Option<GreenNode>, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        let raw_kind = kind as usize;
        let last_token_kind = SyntaxKind::LAST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;

        if raw_kind > last_token_kind {
            assert!(kind.is_any_token(), "this method can only be used to create tokens");
            return Self::create_missing(kind);
        }

        match (leading_trivia.clone(), trailing_trivia.clone()) {
            (None, None) => return Self::tokens_with_no_trivia()[raw_kind].clone().into(),
            (None, Some(trailing)) if trailing == GreenSyntaxFactory::space().into() => Self::tokens_with_single_space()[raw_kind].clone().into(),
            (None, Some(trailing)) if trailing == GreenSyntaxFactory::line_feed().into() => Self::tokens_with_line_feed()[raw_kind].clone().into(),
            (None, Some(trailing)) if trailing == GreenSyntaxFactory::carriage_return_line_feed().into() => {
                Self::tokens_with_carriage_return_line_feed()[raw_kind].clone().into()
            }
            _ => GreenTokenWithTrivia::new(kind, leading_trivia, trailing_trivia).into(),
        }
    }

    #[inline]
    pub(crate) fn create_with_int_value_and_trivia(
        kind: SyntaxKind,
        text: &[u8],
        value: i32,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
    ) -> GreenTokenElement {
        match (leading_trivia, trailing_trivia) {
            (None, None) => GreenTokenWithIntValue::new(kind, text, value).into(),
            (Some(leading), None) => GreenTokenWithIntValueAndTrivia::new(kind, text, value, Some(leading), None).into(),
            (None, Some(trailing)) => GreenTokenWithIntValueAndTrailingTrivia::new(kind, text, value, Some(trailing)).into(),
            (Some(leading), Some(trailing)) => GreenTokenWithIntValueAndTrivia::new(kind, text, value, Some(leading), Some(trailing)).into(),
        }
    }

    #[inline]
    pub(crate) fn create_with_float_value_and_trivia(
        kind: SyntaxKind,
        text: &[u8],
        value: f32,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
    ) -> GreenTokenElement {
        match (leading_trivia, trailing_trivia) {
            (None, None) => GreenTokenWithFloatValue::new(kind, text, value).into(),
            (Some(leading), None) => GreenTokenWithFloatValueAndTrivia::new(kind, text, value, Some(leading), None).into(),
            (None, Some(trailing)) => GreenTokenWithFloatValueAndTrailingTrivia::new(kind, text, value, Some(trailing)).into(),
            (Some(leading), Some(trailing)) => GreenTokenWithFloatValueAndTrivia::new(kind, text, value, Some(leading), Some(trailing)).into(),
        }
    }

    #[inline]
    pub(crate) fn create_with_string_value_and_trivia(
        kind: SyntaxKind,
        text: &[u8],
        value: String,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
    ) -> GreenTokenElement {
        match (leading_trivia, trailing_trivia) {
            (None, None) => GreenTokenWithStringValue::new(kind, text, value).into(),
            (Some(leading), None) => GreenTokenWithStringValueAndTrivia::new(kind, text, value, Some(leading), None).into(),
            (None, Some(trailing)) => GreenTokenWithStringValueAndTrailingTrivia::new(kind, text, value, Some(trailing)).into(),
            (Some(leading), Some(trailing)) => GreenTokenWithStringValueAndTrivia::new(kind, text, value, Some(leading), Some(trailing)).into(),
        }
    }

    #[inline]
    pub(crate) fn create_missing(kind: SyntaxKind) -> GreenTokenElement {
        let raw_kind = kind as usize;
        let first_token_kind = SyntaxKind::FIRST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
        let last_token_kind = SyntaxKind::LAST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;

        match raw_kind {
            k if (first_token_kind..=last_token_kind).contains(&k) => Self::tokens_missing_with_no_trivia()[k].clone().into(),
            _ => GreenToken::new_missing(kind).into(),
        }
    }

    #[inline]
    pub(crate) fn create_missing_with_trivia(kind: SyntaxKind, leading_trivia: Option<GreenNode>, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenWithTrivia::new_missing(kind, leading_trivia, trailing_trivia).into()
    }

    #[inline]
    pub(crate) fn kind(&self) -> SyntaxKind {
        match_token_type!(self, t => t.kind())
    }

    #[inline]
    pub(crate) fn text(&self) -> Vec<u8> {
        match_token_type!(self, t => t.text().to_vec())
    }

    #[inline]
    pub(crate) fn full_text(&self) -> Vec<u8> {
        match_token_type!(self, t => t.full_text())
    }

    #[inline]
    pub(crate) fn width(&self) -> u32 {
        match_token_type!(self, t => t.width().into())
    }

    #[inline]
    pub(crate) fn full_width(&self) -> u32 {
        match_token_type!(self, t => t.full_width().into())
    }

    #[inline]
    pub(crate) fn leading_trivia(&self) -> Option<GreenNode> {
        match_token_type!(self, t => t.leading_trivia())
    }

    #[inline]
    pub(crate) fn trailing_trivia(&self) -> Option<GreenNode> {
        match_token_type!(self, t => t.trailing_trivia())
    }

    #[inline]
    pub(crate) fn diagnostics(&self) -> Option<Vec<GreenDiagnostic>> {
        match_token_type!(self, t => t.diagnostics())
    }

    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        match_token_type!(self, t => t.flags())
    }

    #[inline]
    pub(crate) fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        match_token_type!(self, t => t.write_to(leading, trailing))
    }

    pub(crate) fn tokens_with_no_trivia() -> &'static [GreenToken] {
        static CACHE: LazyLock<Box<[GreenToken]>> = LazyLock::new(|| {
            let first_token_kind = SyntaxKind::FIRST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let last_token_kind = SyntaxKind::LAST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let mut arr = Vec::with_capacity(last_token_kind + 1);

            for kind_value in first_token_kind..=last_token_kind {
                let kind = SyntaxKind::try_from(kind_value as u16).expect("token kind value must be valid");
                arr[kind_value] = GreenToken::new(kind);
            }

            arr.into_boxed_slice()
        });
        CACHE.as_ref()
    }

    pub(crate) fn tokens_with_single_space() -> &'static [GreenTokenWithTrivia] {
        static CACHE: LazyLock<Box<[GreenTokenWithTrivia]>> = LazyLock::new(|| {
            let first_token_kind = SyntaxKind::FIRST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let last_token_kind = SyntaxKind::LAST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let mut arr = Vec::with_capacity(last_token_kind + 1);

            for kind_value in first_token_kind..=last_token_kind {
                let kind = SyntaxKind::try_from(kind_value as u16).expect("token kind value must be valid");
                let space = GreenSyntaxFactory::space().into();
                let space_node = GreenNode::new(SyntaxKind::List, vec![space]);
                arr[kind_value] = GreenTokenWithTrivia::new(kind, None, Some(space_node));
            }

            arr.into_boxed_slice()
        });
        CACHE.as_ref()
    }

    pub(crate) fn tokens_with_line_feed() -> &'static [GreenTokenWithTrivia] {
        static CACHE: LazyLock<Box<[GreenTokenWithTrivia]>> = LazyLock::new(|| {
            let first_token_kind = SyntaxKind::FIRST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let last_token_kind = SyntaxKind::LAST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let mut arr = Vec::with_capacity(last_token_kind + 1);

            for kind_value in first_token_kind..=last_token_kind {
                let kind = SyntaxKind::try_from(kind_value as u16).expect("token kind value must be valid");
                let lf = GreenSyntaxFactory::line_feed().into();
                let lf_node = GreenNode::new(SyntaxKind::List, vec![lf]);
                arr[kind_value] = GreenTokenWithTrivia::new(kind, None, Some(lf_node));
            }

            arr.into_boxed_slice()
        });
        CACHE.as_ref()
    }

    pub(crate) fn tokens_with_carriage_return_line_feed() -> &'static [GreenTokenWithTrivia] {
        static CACHE: LazyLock<Box<[GreenTokenWithTrivia]>> = LazyLock::new(|| {
            let first_token_kind = SyntaxKind::FIRST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let last_token_kind = SyntaxKind::LAST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let mut arr = Vec::with_capacity(last_token_kind + 1);

            for kind_value in first_token_kind..=last_token_kind {
                let kind = SyntaxKind::try_from(kind_value as u16).expect("token kind value must be valid");
                let crlf = GreenSyntaxFactory::carriage_return_line_feed().into();
                let crlf_node = GreenNode::new(SyntaxKind::List, vec![crlf]);
                arr[kind_value] = GreenTokenWithTrivia::new(kind, None, Some(crlf_node));
            }

            arr.into_boxed_slice()
        });
        CACHE.as_ref()
    }

    pub(crate) fn tokens_missing_with_no_trivia() -> &'static [GreenToken] {
        static CACHE: LazyLock<Box<[GreenToken]>> = LazyLock::new(|| {
            let first_token_kind = SyntaxKind::FIRST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let last_token_kind = SyntaxKind::LAST_WELL_KNOWN_TEXT_TOKEN_KIND as usize;
            let mut arr = Vec::with_capacity(last_token_kind + 1);

            for kind_value in first_token_kind..=last_token_kind {
                let kind = SyntaxKind::try_from(kind_value as u16).expect("token kind value must be valid");
                arr[kind_value] = GreenToken::new_missing(kind);
            }

            arr.into_boxed_slice()
        });
        CACHE.as_ref()
    }
}

impl_from_token_variant!(
    GreenToken => Token,
    GreenTokenWithTrivia => TokenWithTrivia,
    GreenTokenWithIntValue => TokenWithIntValue,
    GreenTokenWithFloatValue => TokenWithFloatValue,
    GreenTokenWithStringValue => TokenWithStringValue,
    GreenTokenWithTrailingTrivia => TokenWithTrailingTrivia,
    GreenTokenWithIntValueAndTrivia => TokenWithIntValueAndTrivia,
    GreenTokenWithFloatValueAndTrivia => TokenWithFloatValueAndTrivia,
    GreenTokenWithStringValueAndTrivia => TokenWithStringValueAndTrivia,
    GreenTokenWithIntValueAndTrailingTrivia => TokenWithIntValueAndTrailingTrivia,
    GreenTokenWithFloatValueAndTrailingTrivia => TokenWithFloatValueAndTrailingTrivia,
    GreenTokenWithStringValueAndTrailingTrivia => TokenWithStringValueAndTrailingTrivia,
);

impl<'a> GreenTokenElementRef<'a> {
    #[inline]
    pub(crate) fn kind(&self) -> SyntaxKind {
        match_token_type!(self, t => t.kind())
    }

    #[inline]
    pub(crate) fn text(&self) -> &'a [u8] {
        match_token_type!(self, t => t.text())
    }

    #[inline]
    pub(crate) fn full_text(&self) -> Vec<u8> {
        match_token_type!(self, t => t.full_text())
    }

    #[inline]
    pub(crate) fn width(&self) -> u32 {
        match_token_type!(self, t => t.width().into())
    }

    #[inline]
    pub(crate) fn full_width(&self) -> u32 {
        match_token_type!(self, t => t.full_width().into())
    }

    #[inline]
    pub(crate) fn leading_trivia(&self) -> Option<GreenNode> {
        match_token_type!(self, t => t.leading_trivia())
    }

    #[inline]
    pub(crate) fn trailing_trivia(&self) -> Option<GreenNode> {
        match_token_type!(self, t => t.trailing_trivia())
    }

    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        match_token_type!(self, t => t.flags())
    }

    #[inline]
    pub(crate) fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        match_token_type!(self, t => t.write_to(leading, trailing))
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_token_element_memory_layout() {
        // GreenTokenElement is an enum over Arc-backed token payload variants.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenElement>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenElement>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenElement>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenElement>(), 4);
        }
    }

    #[test]
    fn test_green_token_element_ref_memory_layout() {
        // GreenTokenElementRef is an enum over borrowed token payloads.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenElementRef<'_>>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenElementRef<'_>>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenElementRef<'_>>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenElementRef<'_>>(), 4);
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

    fn create_owned_variants() -> [GreenTokenElement; 12] {
        [
            GreenTokenElement::Token(GreenToken::new(SyntaxKind::TrueKeyword)),
            GreenTokenElement::TokenWithTrivia(GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia())),
            GreenTokenElement::TokenWithIntValue(GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42)),
            GreenTokenElement::TokenWithFloatValue(GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5)),
            GreenTokenElement::TokenWithStringValue(GreenTokenWithStringValue::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string())),
            GreenTokenElement::TokenWithTrailingTrivia(GreenTokenWithTrailingTrivia::new(SyntaxKind::TrueKeyword, trailing_trivia())),
            GreenTokenElement::TokenWithIntValueAndTrivia(GreenTokenWithIntValueAndTrivia::new(
                SyntaxKind::NumericLiteralToken,
                b"42",
                42,
                leading_trivia(),
                trailing_trivia(),
            )),
            GreenTokenElement::TokenWithFloatValueAndTrivia(GreenTokenWithFloatValueAndTrivia::new(
                SyntaxKind::NumericLiteralToken,
                b"3.5",
                3.5,
                leading_trivia(),
                trailing_trivia(),
            )),
            GreenTokenElement::TokenWithStringValueAndTrivia(GreenTokenWithStringValueAndTrivia::new(
                SyntaxKind::NameLiteralToken,
                b"Type",
                "Type".to_string(),
                leading_trivia(),
                trailing_trivia(),
            )),
            GreenTokenElement::TokenWithIntValueAndTrailingTrivia(GreenTokenWithIntValueAndTrailingTrivia::new(
                SyntaxKind::NumericLiteralToken,
                b"42",
                42,
                trailing_trivia(),
            )),
            GreenTokenElement::TokenWithFloatValueAndTrailingTrivia(GreenTokenWithFloatValueAndTrailingTrivia::new(
                SyntaxKind::NumericLiteralToken,
                b"3.5",
                3.5,
                trailing_trivia(),
            )),
            GreenTokenElement::TokenWithStringValueAndTrailingTrivia(GreenTokenWithStringValueAndTrailingTrivia::new(
                SyntaxKind::NameLiteralToken,
                b"Type",
                "Type".to_string(),
                trailing_trivia(),
            )),
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
        assert_eq!(variants[5].kind(), SyntaxKind::TrueKeyword);
        assert_eq!(variants[6].kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(variants[7].kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(variants[8].kind(), SyntaxKind::NameLiteralToken);
        assert_eq!(variants[9].kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(variants[10].kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(variants[11].kind(), SyntaxKind::NameLiteralToken);
    }

    #[test]
    fn test_text_and_width_when_owned_variants_expect_consistent_lengths() {
        for variant in create_owned_variants() {
            let text = variant.text();
            assert_eq!(variant.width(), text.len() as u32);

            if matches!(
                variant,
                GreenTokenElement::TokenWithTrivia(_)
                    | GreenTokenElement::TokenWithIntValueAndTrivia(_)
                    | GreenTokenElement::TokenWithFloatValueAndTrivia(_)
                    | GreenTokenElement::TokenWithStringValueAndTrivia(_)
                    | GreenTokenElement::TokenWithTrailingTrivia(_)
                    | GreenTokenElement::TokenWithIntValueAndTrailingTrivia(_)
                    | GreenTokenElement::TokenWithFloatValueAndTrailingTrivia(_)
                    | GreenTokenElement::TokenWithStringValueAndTrailingTrivia(_)
            ) {
                assert_eq!(variant.full_width(), variant.full_text().len() as u32);
                assert_eq!(variant.full_text(), variant.write_to(true, true));
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
        assert_eq!(variants[5].leading_trivia(), None);
        assert!(variants[5].trailing_trivia().is_some());
        assert!(variants[6].leading_trivia().is_some());
        assert!(variants[6].trailing_trivia().is_some());
        assert!(variants[7].leading_trivia().is_some());
        assert!(variants[7].trailing_trivia().is_some());
        assert!(variants[8].leading_trivia().is_some());
        assert!(variants[8].trailing_trivia().is_some());
        assert_eq!(variants[9].leading_trivia(), None);
        assert!(variants[9].trailing_trivia().is_some());
        assert_eq!(variants[10].leading_trivia(), None);
        assert!(variants[10].trailing_trivia().is_some());
        assert_eq!(variants[11].leading_trivia(), None);
        assert!(variants[11].trailing_trivia().is_some());
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

            assert_eq!(reference.full_text(), owned.full_text());
            assert_eq!(reference.leading_trivia(), owned.leading_trivia());
            assert_eq!(reference.trailing_trivia(), owned.trailing_trivia());

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
        let int_value_trivia: GreenTokenElement =
            GreenTokenWithIntValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 42, leading_trivia(), trailing_trivia()).into();
        let float_value_trivia: GreenTokenElement =
            GreenTokenWithFloatValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5, leading_trivia(), trailing_trivia()).into();
        let string_value_trivia: GreenTokenElement =
            GreenTokenWithStringValueAndTrivia::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string(), leading_trivia(), trailing_trivia()).into();
        let trailing_only: GreenTokenElement = GreenTokenWithTrailingTrivia::new(SyntaxKind::TrueKeyword, trailing_trivia()).into();
        let int_value_trailing: GreenTokenElement =
            GreenTokenWithIntValueAndTrailingTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 42, trailing_trivia()).into();
        let float_value_trailing: GreenTokenElement =
            GreenTokenWithFloatValueAndTrailingTrivia::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5, trailing_trivia()).into();
        let string_value_trailing: GreenTokenElement =
            GreenTokenWithStringValueAndTrailingTrivia::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string(), trailing_trivia()).into();

        assert!(matches!(plain, GreenTokenElement::Token(_)));
        assert!(matches!(with_trivia, GreenTokenElement::TokenWithTrivia(_)));
        assert!(matches!(int_value, GreenTokenElement::TokenWithIntValue(_)));
        assert!(matches!(float_value, GreenTokenElement::TokenWithFloatValue(_)));
        assert!(matches!(string_value, GreenTokenElement::TokenWithStringValue(_)));
        assert!(matches!(trailing_only, GreenTokenElement::TokenWithTrailingTrivia(_)));
        assert!(matches!(int_value_trivia, GreenTokenElement::TokenWithIntValueAndTrivia(_)));
        assert!(matches!(float_value_trivia, GreenTokenElement::TokenWithFloatValueAndTrivia(_)));
        assert!(matches!(string_value_trivia, GreenTokenElement::TokenWithStringValueAndTrivia(_)));
        assert!(matches!(int_value_trailing, GreenTokenElement::TokenWithIntValueAndTrailingTrivia(_)));
        assert!(matches!(float_value_trailing, GreenTokenElement::TokenWithFloatValueAndTrailingTrivia(_)));
        assert!(matches!(string_value_trailing, GreenTokenElement::TokenWithStringValueAndTrailingTrivia(_)));
    }
}

