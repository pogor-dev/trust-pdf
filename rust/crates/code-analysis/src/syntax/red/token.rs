use std::{fmt, hash, ops};

use crate::{GreenDiagnostic, GreenNodeElement, GreenTokenElement, SyntaxKind, SyntaxNode};

/// Typed token value borrowed from the underlying green token variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyntaxTokenValueRef<'a> {
    Int(i32),
    Float(f32),
    String(&'a str),
}

#[derive(Clone)]
#[repr(C)]
pub struct SyntaxToken<'a> {
    underlying_node: GreenNodeElement, // 16 bytes
    parent: &'a SyntaxNode<'a>,        // 8 bytes
    position: u32,                     // 4 bytes
    index: u16,                        // 2 bytes
}

impl<'a> SyntaxToken<'a> {
    /// Creates a new root token (rarely used).
    #[inline]
    pub(crate) fn new(parent: &'a SyntaxNode<'a>, token: GreenNodeElement, position: u32, index: u16) -> Self {
        debug_assert!(!parent.underlying_node().is_list(), "List cannot be a parent of a token");
        debug_assert!(token.is_token(), "GreenNodeElement must be a token");

        Self {
            parent,
            underlying_node: token,
            position,
            index,
        }
    }

    /// Returns the kind of this token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.underlying_node.kind()
    }

    #[inline]
    pub fn parent(&self) -> &'a SyntaxNode<'a> {
        self.parent
    }

    /// Returns a reference to the underlying green token.
    #[inline]
    pub(crate) fn underlying_node(&self) -> GreenNodeElement {
        self.underlying_node.clone()
    }

    /// Returns the absolute byte position of this token in the source.
    #[inline]
    pub(crate) fn position(&self) -> u32 {
        self.position
    }

    /// Returns the index of this token within its parent's children.
    #[inline]
    pub(crate) fn index(&self) -> u16 {
        self.index
    }

    /// Returns the token text.
    #[inline]
    pub fn text(&self) -> Vec<u8> {
        self.underlying_node.text()
    }

    #[inline]
    pub(crate) fn width(&self) -> u32 {
        self.underlying_node.width()
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        self.underlying_node.full_text()
    }

    #[inline]
    pub(crate) fn full_width(&self) -> u32 {
        self.underlying_node.full_width()
    }

    #[inline]
    pub fn span(&self) -> ops::Range<u32> {
        let start = self.position + self.underlying_node.leading_trivia_width();
        let end = start + self.width();
        start..end
    }

    /// Returns the byte range span of this token.
    #[inline]
    pub fn full_span(&self) -> ops::Range<u32> {
        let start = self.position;
        let end = start + self.full_width();
        start..end
    }

    #[inline]
    pub fn contains_diagnostics(&self) -> bool {
        self.underlying_node.contains_diagnostics()
    }

    #[inline]
    pub(crate) fn diagnostics(&self) -> Option<Vec<GreenDiagnostic>> {
        self.underlying_node.diagnostics()
    }

    #[inline]
    pub fn is_missing(&self) -> bool {
        self.underlying_node.is_missing()
    }

    #[inline]
    pub fn has_leading_trivia(&self) -> bool {
        self.underlying_node.leading_trivia().is_some()
    }

    #[inline]
    pub fn has_trailing_trivia(&self) -> bool {
        self.underlying_node.trailing_trivia().is_some()
    }

    /// Returns the token's typed semantic value when present.
    #[inline]
    pub fn value(&self) -> Option<SyntaxTokenValueRef<'_>> {
        if let Some(value) = self.int_value() {
            return Some(SyntaxTokenValueRef::Int(value));
        }

        if let Some(value) = self.float_value() {
            return Some(SyntaxTokenValueRef::Float(value));
        }

        self.string_value().map(SyntaxTokenValueRef::String)
    }

    /// Returns integer value for numeric tokens parsed as an integer.
    #[inline]
    pub fn int_value(&self) -> Option<i32> {
        let token = self.token_element();

        token
            .as_token_with_int_value()
            .map(|t| *t.value())
            .or_else(|| token.as_token_with_int_value_and_trivia().map(|t| *t.value()))
            .or_else(|| token.as_token_with_int_value_and_trailing_trivia().map(|t| *t.value()))
    }

    /// Returns floating-point value for numeric tokens parsed as a float.
    #[inline]
    pub fn float_value(&self) -> Option<f32> {
        let token = self.token_element();

        token
            .as_token_with_float_value()
            .map(|t| *t.value())
            .or_else(|| token.as_token_with_float_value_and_trivia().map(|t| *t.value()))
            .or_else(|| token.as_token_with_float_value_and_trailing_trivia().map(|t| *t.value()))
    }

    /// Returns string value for string-like tokens.
    #[inline]
    pub fn string_value(&self) -> Option<&str> {
        let token = self.token_element();

        token
            .as_token_with_string_value()
            .map(|t| t.value().as_str())
            .or_else(|| token.as_token_with_string_value_and_trivia().map(|t| t.value().as_str()))
            .or_else(|| token.as_token_with_string_value_and_trailing_trivia().map(|t| t.value().as_str()))
    }

    #[inline]
    fn token_element(&self) -> &GreenTokenElement {
        match &self.underlying_node {
            GreenNodeElement::Token(token) => token,
            _ => unreachable!("SyntaxToken must wrap a green token variant"),
        }
    }
}

impl<'a> PartialEq for SyntaxToken<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.underlying_node == other.underlying_node && self.position == other.position && self.index == other.index
    }
}

impl<'a> Eq for SyntaxToken<'a> {}

impl<'a> hash::Hash for SyntaxToken<'a> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.parent.hash(state);
        self.underlying_node.hash(state);
        self.position.hash(state);
        self.index.hash(state);
    }
}

impl<'a> fmt::Debug for SyntaxToken<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxToken")
            .field("kind", &self.kind())
            .field("text", &String::from_utf8_lossy(&self.text()))
            .field("full_text", &String::from_utf8_lossy(&self.full_text()))
            .field("position", &self.position)
            .field("index", &self.index)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GreenNode, GreenToken, GreenTokenElement, GreenTokenWithFloatValueAndTrivia, GreenTokenWithIntValue, GreenTokenWithStringValueAndTrailingTrivia,
    };

    #[test]
    fn test_value_when_int_token_expect_int_variant_and_int_value() {
        let parent_green = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![GreenToken::new(SyntaxKind::NullKeyword).into()]);
        let parent_red = SyntaxNode::new(None, parent_green.into(), 0);

        let token = GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        let token_element: GreenTokenElement = token.into();
        let red_token = SyntaxToken::new(&parent_red, token_element.into(), 0, 0);

        assert_eq!(red_token.int_value(), Some(42));
        assert_eq!(red_token.value(), Some(SyntaxTokenValueRef::Int(42)));
    }

    #[test]
    fn test_value_when_float_token_with_trivia_expect_float_variant_and_float_value() {
        let parent_green = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![GreenToken::new(SyntaxKind::NullKeyword).into()]);
        let parent_red = SyntaxNode::new(None, parent_green.into(), 0);

        let token = GreenTokenWithFloatValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14, None, None);
        let red_token = SyntaxToken::new(&parent_red, token.into(), 0, 0);

        assert_eq!(red_token.float_value(), Some(3.14));
        assert_eq!(red_token.value(), Some(SyntaxTokenValueRef::Float(3.14)));
    }

    #[test]
    fn test_value_when_string_token_with_trailing_trivia_expect_string_variant_and_string_value() {
        let parent_green = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![GreenToken::new(SyntaxKind::NullKeyword).into()]);
        let parent_red = SyntaxNode::new(None, parent_green.into(), 0);

        let token = GreenTokenWithStringValueAndTrailingTrivia::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string(), None);
        let token_element: GreenTokenElement = token.into();
        let red_token = SyntaxToken::new(&parent_red, token_element.into(), 0, 0);

        assert_eq!(red_token.string_value(), Some("Type"));
        assert_eq!(red_token.value(), Some(SyntaxTokenValueRef::String("Type")));
    }

    #[test]
    fn test_value_when_plain_token_expect_none() {
        let parent_green = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![GreenToken::new(SyntaxKind::NullKeyword).into()]);
        let parent_red = SyntaxNode::new(None, parent_green.into(), 0);

        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        let red_token = SyntaxToken::new(&parent_red, token.into(), 0, 0);

        assert_eq!(red_token.int_value(), None);
        assert_eq!(red_token.float_value(), None);
        assert_eq!(red_token.string_value(), None);
        assert_eq!(red_token.value(), None);
    }
}
