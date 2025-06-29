use std::marker::PhantomData;

use crate::syntax::language::Language;

#[derive(Clone)]
pub struct SyntaxTriviaPiecesIterator<L: Language> {
    iter: cursor::SyntaxTriviaPiecesIterator,
    _p: PhantomData<L>,
}
