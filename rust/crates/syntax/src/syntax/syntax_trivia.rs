use std::{fmt, marker::PhantomData, ops::Range};

use crate::{
    cursor,
    syntax::{Language, SyntaxTriviaPiece, SyntaxTriviaPiecesIterator},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxTrivia<L: Language> {
    raw: cursor::SyntaxTrivia,
    _p: PhantomData<L>,
}

impl<L: Language> SyntaxTrivia<L> {
    pub(super) fn new(raw: cursor::SyntaxTrivia) -> Self {
        Self {
            raw,
            _p: PhantomData,
        }
    }

    pub fn pieces(&self) -> SyntaxTriviaPiecesIterator<L> {
        SyntaxTriviaPiecesIterator {
            iter: self.raw.pieces(),
            _p: PhantomData,
        }
    }

    pub fn last(&self) -> Option<SyntaxTriviaPiece<L>> {
        let piece = self.raw.last()?;

        Some(SyntaxTriviaPiece {
            raw: self.raw.clone(),
            offset: self.raw.text_range().end() - piece.length,
            trivia: *piece,
            _p: Default::default(),
        })
    }

    pub fn first(&self) -> Option<SyntaxTriviaPiece<L>> {
        let piece = self.raw.first()?;

        Some(SyntaxTriviaPiece {
            raw: self.raw.clone(),
            offset: self.raw.text_range().start(),
            trivia: *piece,
            _p: Default::default(),
        })
    }

    pub fn text(&self) -> &[u8] {
        self.raw.text()
    }

    pub fn text_range(&self) -> Range<u32> {
        self.raw.text_range()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.len() == 0
    }

    pub fn has_skipped(&self) -> bool {
        self.pieces().any(|piece| piece.is_skipped())
    }
}

impl<L: Language> std::fmt::Debug for SyntaxTrivia<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut first_piece = true;

        for piece in self.pieces() {
            if !first_piece {
                write!(f, ", ")?;
            }
            first_piece = false;
            write!(f, "{:?}", piece)?;
        }

        write!(f, "]")
    }
}
