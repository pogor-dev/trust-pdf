use std::marker::PhantomData;

use crate::{cursor, syntax::language::Language};

#[derive(Clone)]
pub struct SyntaxTriviaPiecesIterator<L: Language> {
    pub(super) iter: cursor::trivia_pieces_iterator::SyntaxTriviaPiecesIterator,
    pub(super) _p: PhantomData<L>,
}
