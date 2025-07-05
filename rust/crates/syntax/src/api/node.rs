use std::{borrow::Cow, fmt, marker::PhantomData};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxNode<L: Language> {
    raw: cursor::SyntaxNode,
    _p: PhantomData<L>,
}

impl<L: Language> SyntaxNode<L> {
    pub fn new_root(green: GreenNode) -> SyntaxNode<L> {}
    pub fn kind(&self) -> L::Kind {}
    pub fn text(&self) -> SyntaxText {}
    pub fn green(&self) -> Cow<'_, GreenNodeData> {}
    pub fn parent(&self) -> Option<SyntaxNode<L>> {}
}

impl<L: Language> fmt::Debug for SyntaxNode<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl<L: Language> fmt::Display for SyntaxNode<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl<L: Language> From<cursor::SyntaxNode> for SyntaxNode<L> {
    fn from(raw: cursor::SyntaxNode) -> SyntaxNode<L> {}
}

impl<L: Language> From<SyntaxNode<L>> for cursor::SyntaxNode {
    fn from(node: SyntaxNode<L>) -> cursor::SyntaxNode {}
}
