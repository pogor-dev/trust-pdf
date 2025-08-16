use std::marker::PhantomData;

use crate::{Language, NodeOrToken, SyntaxElement, cursor};

#[derive(Debug, Clone)]
pub struct SyntaxElementChildren<L: Language> {
    pub(super) raw: cursor::SyntaxElementChildren,
    pub(super) _p: PhantomData<L>,
}

impl<L: Language> Iterator for SyntaxElementChildren<L> {
    type Item = SyntaxElement<L>;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(NodeOrToken::from)
    }
}

impl<L: Language> SyntaxElementChildren<L> {
    pub fn by_kind(
        self,
        matcher: impl Fn(L::Kind) -> bool,
    ) -> impl Iterator<Item = SyntaxElement<L>> {
        self.raw
            .by_kind(move |raw_kind| matcher(L::kind_from_raw(raw_kind)))
            .map(NodeOrToken::from)
    }
}
