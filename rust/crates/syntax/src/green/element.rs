//! # Green Element - Unified PDF Syntax Tree Element
//!
//! Polymorphic wrapper for PDF syntax tree nodes and tokens with unified operations.

use std::borrow::Cow;

use crate::{
    NodeOrToken, SyntaxKind,
    green::{
        node::{GreenNode, GreenNodeData},
        token::{GreenToken, GreenTokenData},
    },
};
/// Unified interface for PDF syntax tree nodes and tokens.
pub(crate) type GreenElement = NodeOrToken<GreenNode, GreenToken>;

/// Borrowed reference to a PDF syntax tree element.
pub(crate) type GreenElementRef<'a> = NodeOrToken<&'a GreenNodeData, &'a GreenTokenData>;

impl GreenElement {
    /// Returns the semantic kind of this syntax element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.as_deref().kind()
    }

    /// Returns the byte length of text content within this element.
    #[inline]
    pub fn width(&self) -> usize {
        self.as_deref().width()
    }

    /// Returns the total byte span including leading and trailing trivia.
    #[inline]
    pub fn full_width(&self) -> usize {
        self.as_deref().full_width()
    }
}

impl From<GreenNode> for GreenElement {
    #[inline]
    fn from(node: GreenNode) -> GreenElement {
        NodeOrToken::Node(node)
    }
}

impl From<GreenToken> for GreenElement {
    #[inline]
    fn from(token: GreenToken) -> GreenElement {
        NodeOrToken::Token(token)
    }
}

impl From<Cow<'_, GreenNodeData>> for GreenElement {
    #[inline]
    fn from(cow: Cow<'_, GreenNodeData>) -> Self {
        NodeOrToken::Node(cow.into_owned())
    }
}

impl GreenElementRef<'_> {
    /// Returns the semantic kind of this borrowed syntax element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    /// Returns the byte length of text content for borrowed elements.
    #[inline]
    pub fn width(self) -> usize {
        match self {
            NodeOrToken::Node(it) => it.width(),
            NodeOrToken::Token(it) => it.width(),
        }
    }

    /// Returns total byte span including trivia for borrowed elements.
    #[inline]
    pub fn full_width(self) -> usize {
        match self {
            NodeOrToken::Node(it) => it.full_width(),
            NodeOrToken::Token(it) => it.full_width(),
        }
    }
}

impl<'a> From<&'a GreenNode> for GreenElementRef<'a> {
    #[inline]
    fn from(node: &'a GreenNode) -> GreenElementRef<'a> {
        NodeOrToken::Node(node)
    }
}

impl<'a> From<&'a GreenToken> for GreenElementRef<'a> {
    #[inline]
    fn from(token: &'a GreenToken) -> GreenElementRef<'a> {
        NodeOrToken::Token(token)
    }
}

impl GreenElementRef<'_> {
    /// Converts borrowed element reference to owned element.
    pub fn to_owned(self) -> GreenElement {
        match self {
            NodeOrToken::Node(it) => NodeOrToken::Node(it.to_owned()),
            NodeOrToken::Token(it) => NodeOrToken::Token(it.to_owned()),
        }
    }
}
