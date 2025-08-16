use std::marker::PhantomData;

use crate::{Language, SyntaxNode, cursor};

#[derive(Debug, Clone)]
pub struct SyntaxNodeChildren<L: Language> {
    pub(super) raw: cursor::SyntaxNodeChildren,
    pub(super) _p: PhantomData<L>,
}

impl<L: Language> Iterator for SyntaxNodeChildren<L> {
    type Item = SyntaxNode<L>;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(SyntaxNode::from)
    }
}

impl<L: Language> SyntaxNodeChildren<L> {
    pub fn by_kind(self, matcher: impl Fn(L::Kind) -> bool) -> impl Iterator<Item = SyntaxNode<L>> {
        self.raw
            .by_kind(move |raw_kind| matcher(L::kind_from_raw(raw_kind)))
            .map(SyntaxNode::from)
    }
}
