use std::{fmt, marker::PhantomData};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxTrivia<L: Language> {
    raw: cursor::SyntaxTrivia,
    _p: PhantomData<L>,
}

impl<L: Language> SyntaxTrivia<L> {
    pub fn kind(&self) -> L::Kind {}
    pub fn text(&self) -> &[u8] {}
    pub fn width(&self) -> usize {}
    pub fn span(&self) -> Range<usize> {}
    pub fn green(&self) -> &GreenTriviaData {}
    pub fn parent(&self) -> Option<SyntaxToken<L>> {}
}

impl<L: Language> From<cursor::SyntaxTrivia> for SyntaxTrivia<L> {
    fn from(raw: cursor::SyntaxTrivia) -> SyntaxTrivia<L> {}
}

impl<L: Language> From<SyntaxTrivia<L>> for cursor::SyntaxTrivia {
    fn from(token: SyntaxTrivia<L>) -> cursor::SyntaxTrivia {}
}

impl<L: Language> fmt::Debug for SyntaxTrivia<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl<L: Language> fmt::Display for SyntaxTrivia<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}
