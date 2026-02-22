use std::{fmt, ops::Deref};

use crate::{GreenToken, GreenTokenData, SyntaxKind, green::GreenFlags};

pub type GreenTokenElement = TokenType<GreenToken>;
pub(crate) type GreenTokenElementRef<'a> = TokenType<&'a GreenTokenData>;

impl GreenTokenElement {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenTokenElement::Token(t) => t.kind(),
        }
    }

    #[inline]
    pub fn text(&self) -> Vec<u8> {
        match self {
            GreenTokenElement::Token(t) => t.text().to_vec(),
        }
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        match self {
            GreenTokenElement::Token(t) => t.text().to_vec(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenTokenElement::Token(t) => t.width().into(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            GreenTokenElement::Token(t) => t.width().into(),
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenType<T> {
    Token(T),
}

impl<T> TokenType<T> {
    pub fn into_token(self) -> Option<T> {
        match self {
            TokenType::Token(token) => Some(token),
        }
    }

    pub fn as_token(&self) -> Option<&T> {
        match self {
            TokenType::Token(token) => Some(token),
        }
    }
}

impl<T: Deref> TokenType<T> {
    pub(crate) fn as_deref(&self) -> TokenType<&T::Target> {
        match self {
            TokenType::Token(token) => TokenType::Token(token),
        }
    }
}

impl<T: fmt::Display> fmt::Display for TokenType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Token(token) => fmt::Display::fmt(token, f),
        }
    }
}
