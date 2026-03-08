use std::{fmt, hash, ops};

use crate::{GreenDiagnostic, GreenNodeElement, SyntaxKind, SyntaxToken};

#[derive(Clone)]
#[repr(C)]
pub struct SyntaxTrivia<'a> {
    underlying_node: GreenNodeElement, // 16 bytes
    token: &'a SyntaxToken<'a>,        // 8 bytes
    position: u32,                     // 4 bytes
    index: u16,                        // 2 bytes
}

impl<'a> SyntaxTrivia<'a> {
    /// Creates a new root trivia (rarely used).
    #[inline]
    pub(crate) fn new(token: &'a SyntaxToken<'a>, trivia_node: GreenNodeElement, position: u32, index: u16) -> Self {
        Self {
            token,
            underlying_node: trivia_node,
            position,
            index,
        }
    }

    /// Returns the kind of this trivia.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.underlying_node.kind()
    }

    /// Returns a reference to the underlying green trivia.
    #[inline]
    pub(crate) fn underlying_node(&self) -> GreenNodeElement {
        self.underlying_node.clone()
    }

    #[inline]
    pub fn token(&self) -> &'a SyntaxToken<'a> {
        self.token
    }

    /// Returns the absolute byte position of this trivia in the source.
    #[inline]
    pub(crate) fn position(&self) -> u32 {
        self.position
    }

    /// Returns the index of this trivia within its parent's children.
    #[inline]
    pub(crate) fn index(&self) -> u16 {
        self.index
    }

    /// Returns the trivia text.
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

    /// Returns the byte range span of this trivia.
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
}

impl<'a> PartialEq for SyntaxTrivia<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token && self.underlying_node == other.underlying_node && self.position == other.position && self.index == other.index
    }
}

impl<'a> Eq for SyntaxTrivia<'a> {}

impl<'a> hash::Hash for SyntaxTrivia<'a> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.token.hash(state);
        self.underlying_node.hash(state);
        self.position.hash(state);
        self.index.hash(state);
    }
}

impl<'a> fmt::Debug for SyntaxTrivia<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxTrivia")
            .field("kind", &self.kind())
            .field("text", &String::from_utf8_lossy(&self.text()))
            .field("full_text", &String::from_utf8_lossy(&self.full_text()))
            .field("position", &self.position)
            .field("index", &self.index)
            .finish()
    }
}
