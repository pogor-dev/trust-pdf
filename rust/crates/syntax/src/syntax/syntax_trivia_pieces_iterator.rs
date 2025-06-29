use std::marker::PhantomData;

use crate::{
    cursor,
    syntax::{Language, SyntaxTriviaPiece},
};

#[derive(Clone)]
pub struct SyntaxTriviaPiecesIterator<L: Language> {
    pub(crate) iter: cursor::SyntaxTriviaPiecesIterator,
    pub(crate) _p: PhantomData<L>,
}

impl<L: Language> Iterator for SyntaxTriviaPiecesIterator<L> {
    type Item = SyntaxTriviaPiece<L>;

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, trivia) = self.iter.next()?;
        Some(SyntaxTriviaPiece {
            raw: self.iter.raw.clone(),
            offset,
            trivia,
            _p: PhantomData,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<L: Language> DoubleEndedIterator for SyntaxTriviaPiecesIterator<L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (offset, trivia) = self.iter.next_back()?;
        Some(SyntaxTriviaPiece {
            raw: self.iter.raw.clone(),
            offset,
            trivia,
            _p: PhantomData,
        })
    }
}

impl<L: Language> ExactSizeIterator for SyntaxTriviaPiecesIterator<L> {}
