use std::{fmt, ops};

use crate::{GreenDiagnostic, GreenNodeElement, SyntaxKind};

#[derive(Clone, Hash)]
#[repr(C)]
pub struct SyntaxNode<'a> {
    underlying_node: GreenNodeElement,  // 16 bytes
    parent: Option<&'a SyntaxNode<'a>>, // 8 bytes
    position: u32,                      // 4 bytes
}

impl<'a> SyntaxNode<'a> {
    /// Creates a new root token (rarely used).
    #[inline]
    pub(crate) fn new(parent: Option<&'a SyntaxNode<'a>>, node: GreenNodeElement, position: u32) -> Self {
        debug_assert!(position >= 0, "Position must be non-negative");
        debug_assert!(!node.is_list(), "List cannot be a parent of a node");

        Self {
            parent,
            underlying_node: node,
            position,
        }
    }

    /// Returns the kind of this token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.underlying_node.kind()
    }

    #[inline]
    pub fn parent(&self) -> Option<&'a SyntaxNode<'a>> {
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
    pub fn diagnostics(&self) -> Option<Vec<GreenDiagnostic>> {
        self.underlying_node.diagnostics()
    }

    #[inline]
    pub fn is_missing(&self) -> bool {
        self.underlying_node.is_missing()
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        self.underlying_node.is_list()
    }

    #[inline]
    pub fn has_leading_trivia(&self) -> bool {
        self.underlying_node.leading_trivia().is_some()
    }

    #[inline]
    pub fn has_trailing_trivia(&self) -> bool {
        self.underlying_node.trailing_trivia().is_some()
    }
}

impl<'a> PartialEq for SyntaxNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.underlying_node == other.underlying_node && self.position == other.position
    }
}

impl<'a> Eq for SyntaxNode<'a> {}

impl<'a> fmt::Debug for SyntaxNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxNode")
            .field("kind", &self.kind())
            .field("text", &String::from_utf8_lossy(&self.text()))
            .field("full_text", &String::from_utf8_lossy(&self.full_text()))
            .field("position", &self.position)
            .finish()
    }
}
