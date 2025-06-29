use std::ops::Range;

use crate::syntax::{language::Language, syntax_trivia_piece::SyntaxTriviaPiece};

#[derive(Debug, Clone)]
pub struct SyntaxTriviaPieceWhitespace<L: Language>(pub SyntaxTriviaPiece<L>);

impl<L: Language> SyntaxTriviaPieceWhitespace<L> {
    pub fn text(&self) -> &[u8] {
        self.0.text()
    }

    pub fn text_len(&self) -> u32 {
        self.0.text_len()
    }

    pub fn text_range(&self) -> Range<u32> {
        self.0.text_range()
    }

    /// Returns a reference to its [SyntaxTriviaPiece]
    pub fn as_piece(&self) -> &SyntaxTriviaPiece<L> {
        &self.0
    }

    /// Returns its [SyntaxTriviaPiece]
    pub fn into_piece(self) -> SyntaxTriviaPiece<L> {
        self.0
    }
}
