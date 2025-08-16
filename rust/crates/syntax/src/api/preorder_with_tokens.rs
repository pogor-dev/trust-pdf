use std::marker::PhantomData;

use crate::{Language, SyntaxElement, WalkEvent, cursor};

#[derive(Debug, Clone)]
pub struct PreorderWithTokens<L: Language> {
    pub(super) raw: cursor::PreorderWithTokens,
    pub(super) _p: PhantomData<L>,
}

impl<L: Language> PreorderWithTokens<L> {
    pub fn skip_subtree(&mut self) {
        self.raw.skip_subtree()
    }
}

impl<L: Language> Iterator for PreorderWithTokens<L> {
    type Item = WalkEvent<SyntaxElement<L>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|it| it.map(SyntaxElement::from))
    }
}
