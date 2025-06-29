use crate::cursor::trivia::SyntaxTrivia;

#[derive(Clone)]
pub struct SyntaxTriviaPiecesIterator {
    pub(crate) raw: SyntaxTrivia,
    pub(crate) next_index: usize,
    pub(crate) next_offset: u64,
    pub(crate) end_index: usize,
    pub(crate) end_offset: u64,
}
