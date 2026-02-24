use crate::{
    GreenFlags, GreenNode, GreenToken, GreenTokenData, GreenTokenWithFloatValue, GreenTokenWithFloatValueData, GreenTokenWithIntValue,
    GreenTokenWithIntValueData, GreenTokenWithStringValue, GreenTokenWithStringValueData, SyntaxKind, green::TokenType,
};

pub type GreenTokenElement = TokenType<GreenToken, GreenTokenWithIntValue, GreenTokenWithFloatValue, GreenTokenWithStringValue>;

pub(crate) type GreenTokenElementRef<'a> =
    TokenType<&'a GreenTokenData, &'a GreenTokenWithIntValueData, &'a GreenTokenWithFloatValueData, &'a GreenTokenWithStringValueData>;

impl GreenTokenElement {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenTokenElement::Token(t) => t.kind(),
            GreenTokenElement::TokenWithIntValue(t) => t.kind(),
            GreenTokenElement::TokenWithFloatValue(t) => t.kind(),
            GreenTokenElement::TokenWithStringValue(t) => t.kind(),
        }
    }

    #[inline]
    pub fn text(&self) -> Vec<u8> {
        match self {
            GreenTokenElement::Token(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithIntValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithFloatValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithStringValue(t) => t.text().to_vec(),
        }
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        match self {
            GreenTokenElement::Token(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithIntValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithFloatValue(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithStringValue(t) => t.text().to_vec(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenTokenElement::Token(t) => t.width().into(),
            GreenTokenElement::TokenWithIntValue(t) => t.width().into(),
            GreenTokenElement::TokenWithFloatValue(t) => t.width().into(),
            GreenTokenElement::TokenWithStringValue(t) => t.width().into(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            GreenTokenElement::Token(t) => t.width().into(),
            GreenTokenElement::TokenWithIntValue(t) => t.width().into(),
            GreenTokenElement::TokenWithFloatValue(t) => t.width().into(),
            GreenTokenElement::TokenWithStringValue(t) => t.width().into(),
        }
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenTokenElement::Token(_t) => None,
            GreenTokenElement::TokenWithIntValue(_t) => None,
            GreenTokenElement::TokenWithFloatValue(_t) => None,
            GreenTokenElement::TokenWithStringValue(_t) => None,
        }
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenTokenElement::Token(_t) => None,
            GreenTokenElement::TokenWithIntValue(_t) => None,
            GreenTokenElement::TokenWithFloatValue(_t) => None,
            GreenTokenElement::TokenWithStringValue(_t) => None,
        }
    }

    #[inline]
    pub fn flags(&self) -> GreenFlags {
        match self {
            GreenTokenElement::Token(t) => t.flags(),
            GreenTokenElement::TokenWithIntValue(t) => t.flags(),
            GreenTokenElement::TokenWithFloatValue(t) => t.flags(),
            GreenTokenElement::TokenWithStringValue(t) => t.flags(),
        }
    }
}

impl<'a> GreenTokenElementRef<'a> {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenTokenElementRef::Token(t) => t.kind(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.kind(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.kind(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.kind(),
        }
    }

    #[inline]
    pub fn text(&self) -> &'a [u8] {
        match self {
            GreenTokenElementRef::Token(t) => t.text(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.text(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.text(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.text(),
        }
    }

    #[inline]
    pub fn full_text(&self) -> &'a [u8] {
        self.text()
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenTokenElementRef::Token(t) => t.width().into(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.width().into(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.width().into(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.width().into(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        self.width()
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenTokenElementRef::Token(_t) => None,
            GreenTokenElementRef::TokenWithIntValue(_t) => None,
            GreenTokenElementRef::TokenWithFloatValue(_t) => None,
            GreenTokenElementRef::TokenWithStringValue(_t) => None,
        }
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenTokenElementRef::Token(_t) => None,
            GreenTokenElementRef::TokenWithIntValue(_t) => None,
            GreenTokenElementRef::TokenWithFloatValue(_t) => None,
            GreenTokenElementRef::TokenWithStringValue(_t) => None,
        }
    }

    #[inline]
    pub fn flags(&self) -> GreenFlags {
        match self {
            GreenTokenElementRef::Token(t) => t.flags(),
            GreenTokenElementRef::TokenWithIntValue(t) => t.flags(),
            GreenTokenElementRef::TokenWithFloatValue(t) => t.flags(),
            GreenTokenElementRef::TokenWithStringValue(t) => t.flags(),
        }
    }
}
