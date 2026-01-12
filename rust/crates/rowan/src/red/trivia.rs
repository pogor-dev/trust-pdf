use std::{fmt, ops};

use crate::{GreenTrivia, SyntaxKind, SyntaxToken};

/// A semantic trivia in the red tree, wrapping a green trivia with position information.
///
/// Provides access to the underlying green trivia and its position in the source file.
pub struct SyntaxTrivia {
    token: SyntaxToken,
    underlying_node: GreenTrivia,
    position: u64,
    index: u16,
}

impl SyntaxTrivia {
    /// Creates a new `SyntaxTrivia` with the given properties.
    #[inline]
    pub fn new(token: SyntaxToken, underlying_node: GreenTrivia, position: u64, index: u16) -> Self {
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
    pub fn token(&self) -> &SyntaxToken {
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

impl PartialEq for SyntaxTrivia {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token && self.underlying_node == other.underlying_node && self.position == other.position && self.index == other.index
    }
}

impl Eq for SyntaxTrivia {}

impl fmt::Debug for SyntaxTrivia {
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

impl fmt::Display for SyntaxTrivia {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.underlying_node)
    }
}
