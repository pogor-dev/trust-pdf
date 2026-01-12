use std::{fmt, ops};

use crate::{GreenTrivia, SyntaxKind, SyntaxToken};

/// A semantic trivia in the red tree, wrapping a green trivia with position information.
///
/// Provides access to the underlying green trivia and its position in the source file.
#[repr(C)]
#[derive(Clone)]
pub struct SyntaxTrivia<'a> {
    underlying_node: GreenTrivia, // 16 bytes
    token: &'a SyntaxToken<'a>,   // 8 bytes
    position: u64,                // 8 bytes
    index: u16,                   // 2 bytes
}

impl<'a> SyntaxTrivia<'a> {
    /// Creates a new `SyntaxTrivia` with the given properties.
    #[inline]
    pub fn new(token: &'a SyntaxToken, underlying_node: GreenTrivia, position: u64, index: u16) -> Self {
        Self {
            token,
            underlying_node,
            position,
            index,
        }
    }

    /// Returns the kind of this trivia.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.underlying_node.kind()
    }

    /// Returns a reference to the associated token.
    #[inline]
    pub fn token(&self) -> &SyntaxToken<'a> {
        &self.token
    }

    /// Returns the position of this trivia in the source.
    #[inline]
    fn position(&self) -> u64 {
        self.position
    }

    /// Returns the index of this trivia within its token.
    #[inline]
    fn index(&self) -> u16 {
        self.index
    }

    /// Returns the full width of this trivia.
    #[inline]
    fn full_width(&self) -> u16 {
        self.underlying_node.full_width()
    }

    /// Returns the span of this trivia in the source.
    #[inline]
    pub fn full_span(&self) -> ops::Range<u64> {
        let start = self.position;
        let end = start + self.full_width() as u64;
        start..end
    }
}

impl<'a> PartialEq for SyntaxTrivia<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token && self.underlying_node == other.underlying_node && self.position == other.position && self.index == other.index
    }
}

impl<'a> Eq for SyntaxTrivia<'a> {}

impl<'a> fmt::Debug for SyntaxTrivia<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxTrivia")
            .field("kind", &self.kind())
            .field("bytes", &self.underlying_node.full_bytes())
            .field("position", &self.position)
            .field("index", &self.index)
            .finish()
    }
}

impl<'a> fmt::Display for SyntaxTrivia<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.underlying_node)
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_memory_layout() {
        // 16 (GreenTrivia) + 8 (token) + 8 (position) + 2 (index) = 34 bytes
        // + 6 bytes padding for 8-byte alignment = 40 bytes total
        assert_eq!(std::mem::size_of::<SyntaxTrivia>(), 40);
        assert_eq!(std::mem::align_of::<SyntaxTrivia>(), 8);
    }
}
