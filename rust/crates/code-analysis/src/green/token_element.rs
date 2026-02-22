use std::{fmt, ops::Deref};

use crate::{GreenToken, GreenTokenData, GreenTokenWithValue, GreenTokenWithValueData, SyntaxKind, green::GreenFlags};

pub type GreenTokenElement = TokenType<GreenToken, GreenTokenWithValue>;
pub(crate) type GreenTokenElementRef<'a> = TokenType<&'a GreenTokenData, &'a GreenTokenWithValueData>;

impl GreenTokenElement {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenTokenElement::Token(t) => t.kind(),
            GreenTokenElement::TokenWithValue(t) => t.kind(),
        }
    }

    #[inline]
    pub fn text(&self) -> Vec<u8> {
        match self {
            GreenTokenElement::Token(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithValue(t) => t.text().to_vec(),
        }
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        match self {
            GreenTokenElement::Token(t) => t.text().to_vec(),
            GreenTokenElement::TokenWithValue(t) => t.text().to_vec(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenTokenElement::Token(t) => t.width().into(),
            GreenTokenElement::TokenWithValue(t) => t.width().into(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            GreenTokenElement::Token(t) => t.width().into(),
            GreenTokenElement::TokenWithValue(t) => t.width().into(),
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
            GreenTokenElement::TokenWithValue(t) => t.flags(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenType<T, V> {
    Token(T),
    TokenWithValue(V),
}

impl<T, V> TokenType<T, V> {
    pub fn into_token(self) -> Option<T> {
        match self {
            TokenType::Token(token) => Some(token),
            TokenType::TokenWithValue(_) => None,
        }
    }

    pub fn as_token(&self) -> Option<&T> {
        match self {
            TokenType::Token(token) => Some(token),
            TokenType::TokenWithValue(_) => None,
        }
    }

    pub fn into_token_with_value(self) -> Option<V> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithValue(token) => Some(token),
        }
    }

    pub fn as_token_with_value(&self) -> Option<&V> {
        match self {
            TokenType::Token(_) => None,
            TokenType::TokenWithValue(token) => Some(token),
        }
    }
}

impl<T: Deref, V: Deref> TokenType<T, V> {
    pub(crate) fn as_deref(&self) -> TokenType<&T::Target, &V::Target> {
        match self {
            TokenType::Token(token) => TokenType::Token(token),
            TokenType::TokenWithValue(token) => TokenType::TokenWithValue(token),
        }
    }
}

impl<T: fmt::Display, V: fmt::Display> fmt::Display for TokenType<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Token(token) => fmt::Display::fmt(token, f),
            TokenType::TokenWithValue(token) => fmt::Display::fmt(token, f),
        }
    }
}
