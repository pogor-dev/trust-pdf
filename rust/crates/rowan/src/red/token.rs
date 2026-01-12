use std::{fmt, ops};

use crate::{GreenToken, SyntaxKind, SyntaxNode};

/// A semantic token in the red tree, wrapping a green token with position information.
///
/// Provides access to the underlying green token and its position in the source file.
pub struct SyntaxToken {
    parent: SyntaxNode,
    underlying_node: GreenToken,
    position: u64,
    index: u16,
}

impl SyntaxToken {
    /// Creates a new `SyntaxToken` with the given properties.
    #[inline]
    pub fn new(parent: SyntaxNode, underlying_node: GreenToken, position: u64, index: u16) -> Self {
        Self {
            parent,
            underlying_node,
            position,
            index,
        }
    }

    /// Returns the kind of this token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.underlying_node.kind()
    }

    /// Returns a reference to the parent node.
    #[inline]
    pub fn parent_node(&self) -> &SyntaxNode {
        &self.parent
    }

    /// Returns the position of this token in the source.
    #[inline]
    fn position(&self) -> u64 {
        self.position
    }

    /// Returns the index of this token within its parent.
    #[inline]
    fn index(&self) -> u16 {
        self.index
    }

    /// Returns the full width of this token (including trivia).
    #[inline]
    fn full_width(&self) -> u32 {
        self.underlying_node.full_width()
    }

    /// Returns the width of this token (excluding trivia).
    #[inline]
    fn width(&self) -> u32 {
        self.underlying_node.width()
    }

    /// Returns the span of this token in the source (including trivia).
    #[inline]
    pub fn full_span(&self) -> ops::Range<u64> {
        let start = self.position;
        let end = start + self.full_width() as u64;
        start..end
    }

    /// Returns the span of this token without trivia.
    #[inline]
    pub fn span(&self) -> ops::Range<u64> {
        let start = self.position + self.underlying_node.leading_trivia().full_width() as u64;
        let end = start + self.width() as u64;
        start..end
    }
}

impl PartialEq for SyntaxToken {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.underlying_node == other.underlying_node && self.position == other.position && self.index == other.index
    }
}

impl Eq for SyntaxToken {}

impl fmt::Debug for SyntaxToken {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxToken")
            .field("kind", &self.kind())
            .field("position", &self.position)
            .field("index", &self.index)
            .finish()
    }
}

impl fmt::Display for SyntaxToken {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.underlying_node.full_bytes()))
    }
}
