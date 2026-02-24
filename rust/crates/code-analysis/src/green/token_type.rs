use std::{fmt, ops::Deref};

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
