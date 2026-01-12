use std::{fmt, ops};

use crate::{GreenToken, SyntaxKind, SyntaxNode};

/// A semantic token in the red tree, wrapping a green token with position information.
///
/// Provides access to the underlying green token and its position in the source file.
#[repr(C)]
#[derive(Clone, Default)]
pub struct SyntaxToken<'a> {
    underlying_node: Option<GreenToken>, // 16 bytes
    parent: Option<&'a SyntaxNode<'a>>,  // 8 bytes
    position: u64,                       // 8 bytes
    index: u16,                          // 2 bytes
}

impl<'a> SyntaxToken<'a> {
    /// Creates a new `SyntaxToken` with the given properties.
    #[inline]
    pub fn new(parent: Option<&'a SyntaxNode>, underlying_node: Option<GreenToken>, position: u64, index: u16) -> Self {
        Self {
            parent,
            underlying_node,
            position,
            index,
        }
    }

    /// Returns the kind of this token.
    #[inline]
    pub fn kind(&self) -> Option<SyntaxKind> {
        self.underlying_node.as_ref().map(|t| t.kind())
    }

    /// Returns a reference to the parent node.
    #[inline]
    pub fn parent(&self) -> Option<&SyntaxNode<'a>> {
        self.parent
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
        self.underlying_node.as_ref().map_or(0, |t| t.full_width())
    }

    /// Returns the width of this token (excluding trivia).
    #[inline]
    fn width(&self) -> u32 {
        self.underlying_node.as_ref().map_or(0, |t| t.width())
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
        let leading_width = self.underlying_node.as_ref().map_or(0, |t| t.leading_trivia().full_width() as u64);
        let start = self.position + leading_width;
        let end = start + self.width() as u64;
        start..end
    }
}

impl<'a> PartialEq for SyntaxToken<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.underlying_node == other.underlying_node && self.position == other.position && self.index == other.index
    }
}

impl<'a> Eq for SyntaxToken<'a> {}

impl<'a> fmt::Debug for SyntaxToken<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxToken")
            .field("kind", &self.kind())
            .field("position", &self.position)
            .field("index", &self.index)
            .finish()
    }
}

impl<'a> fmt::Display for SyntaxToken<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.underlying_node.as_ref() {
            Some(token) => write!(f, "{}", String::from_utf8_lossy(&token.full_bytes())),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_memory_layout() {
        // 16 (GreenToken) + 8 (parent) + 8 (position) + 2 (index) = 34 bytes
        // + 6 bytes padding for 8-byte alignment = 40 bytes total
        assert_eq!(std::mem::size_of::<SyntaxToken>(), 40);
        assert_eq!(std::mem::align_of::<SyntaxToken>(), 8);
    }
}
