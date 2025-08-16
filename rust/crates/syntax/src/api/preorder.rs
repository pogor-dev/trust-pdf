use std::marker::PhantomData;

use crate::{Language, SyntaxNode, WalkEvent, cursor};

#[derive(Debug, Clone)]
pub struct Preorder<L: Language> {
    pub(super) raw: cursor::Preorder,
    pub(super) _p: PhantomData<L>,
}

impl<L: Language> Preorder<L> {
    pub fn skip_subtree(&mut self) {
        self.raw.skip_subtree()
    }
}

impl<L: Language> Iterator for Preorder<L> {
    type Item = WalkEvent<SyntaxNode<L>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|it| it.map(SyntaxNode::from))
    }
}
