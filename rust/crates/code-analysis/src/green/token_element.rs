use super::token_with_value::{
    GreenTokenWithFloatValue, GreenTokenWithFloatValueData, GreenTokenWithIntValue, GreenTokenWithIntValueData, GreenTokenWithStringValue,
    GreenTokenWithStringValueData,
};

use crate::{GreenFlags, GreenToken, GreenTokenData, SyntaxKind, green::TokenType};

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

    // #[inline]
    // pub fn leading_trivia(&self) -> Option<GreenNode> {
    //     match self {
    //         GreenTokenElement::Token(t) => None,
    //     }
    // }

    // #[inline]
    // pub fn trailing_trivia(&self) -> Option<GreenNode> {
    //     match self {
    //         GreenTokenElement::Token(t) => None,
    //     }
    // }

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
