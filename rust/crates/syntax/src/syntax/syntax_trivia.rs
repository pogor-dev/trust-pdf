use std::marker::PhantomData;

use crate::{
    cursor,
    syntax::{language::Language, syntax_trivia_pieces_iterator::SyntaxTriviaPiecesIterator},
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

    /// Returns all [SyntaxTriviaPiece] of this trivia.
    ///
    /// ```
    /// use crate::*;
    /// use rome_rowan::raw_language::{RawLanguage, RawLanguageKind, RawSyntaxTreeBuilder};
    /// use rome_rowan::*;
    /// use std::iter::Iterator;
    /// let mut node = RawSyntaxTreeBuilder::wrap_with_node(RawLanguageKind::ROOT, |builder| {
    ///     builder.token_with_trivia(
    ///         RawLanguageKind::LET_TOKEN,
    ///         "\n\t /**/let \t\t",
    ///         &[
    ///             TriviaPiece::whitespace(3),
    ///             TriviaPiece::single_line_comment(4),
    ///         ],
    ///         &[TriviaPiece::whitespace(3)],
    ///     );
    /// });
    /// let pieces: Vec<_> = node.first_leading_trivia().unwrap().pieces().collect();
    /// assert_eq!(2, pieces.len());
    /// let pieces: Vec<_> = node.last_trailing_trivia().unwrap().pieces().collect();
    /// assert_eq!(1, pieces.len());
    /// ```
    pub fn pieces(&self) -> SyntaxTriviaPiecesIterator<L> {
        SyntaxTriviaPiecesIterator {
            iter: self.raw.pieces(),
            _p: PhantomData,
        }
    }
}
