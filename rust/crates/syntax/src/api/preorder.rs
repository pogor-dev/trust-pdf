use crate::{Language, SyntaxElement, SyntaxNode, WalkEvent, red};
use std::marker::PhantomData;

/// Preorder iterator for traversing syntax nodes.
#[derive(Debug, Clone)]
pub struct Preorder<L: Language> {
    pub(super) raw: red::Preorder,
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

/// Preorder iterator for traversing syntax nodes and tokens.
#[derive(Debug, Clone)]
pub struct PreorderWithTokens<L: Language> {
    pub(super) raw: red::PreorderWithTokens,
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
