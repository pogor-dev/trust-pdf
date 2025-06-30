use std::{fmt, marker::PhantomData};

use crate::{Language, cursor};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxToken<L: Language> {
    raw: cursor::SyntaxToken,
    _p: PhantomData<L>,
}

impl<L: Language> SyntaxToken<L> {
    pub fn kind(&self) -> L::Kind {}
    pub fn text(&self) -> &[u8] {}
    pub fn full_text(&self) -> &[u8] {}
    pub fn width(&self) -> usize {}
    pub fn full_width(&self) -> usize {}
    pub fn span(&self) -> Range<isize> {}
    pub fn full_span(&self) -> Range<isize> {}
    pub fn green(&self) -> &GreenTokenData {}
    pub fn parent(&self) -> Option<SyntaxNode<L>> {}
    pub fn leading_trivia(&self) -> SyntaxTrivia<L> {}
    pub fn trailing_trivia(&self) -> SyntaxTrivia<L> {}
}

impl<L: Language> From<cursor::SyntaxToken> for SyntaxToken<L> {
    fn from(raw: cursor::SyntaxToken) -> SyntaxToken<L> {}
}

impl<L: Language> From<SyntaxToken<L>> for cursor::SyntaxToken {
    fn from(token: SyntaxToken<L>) -> cursor::SyntaxToken {}
}

impl<L: Language> fmt::Debug for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl<L: Language> fmt::Display for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}
