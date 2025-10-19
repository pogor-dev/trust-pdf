use crate::{
    GreenTrivia, NodeOrToken,
    green::{GreenNode, SyntaxKind, cache::GreenCache, element::GreenElement},
};

/// A builder for a green tree.
#[derive(Default)]
pub struct GreenNodeBuilder {
    cache: GreenCache,
    parents: Vec<(SyntaxKind, usize)>,
    children: Vec<(u64, GreenElement)>,
}

impl std::fmt::Debug for GreenNodeBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GreenNodeBuilder").finish_non_exhaustive()
    }
}

impl GreenNodeBuilder {
    /// Creates new builder.
    #[inline]
    pub fn new() -> GreenNodeBuilder {
        GreenNodeBuilder::default()
    }

    /// Adds new token to the current branch.
    #[inline]
    pub fn token(&mut self, kind: SyntaxKind, text: &[u8], leading_trivia: &[GreenTrivia], trailing_trivia: &[GreenTrivia]) {
        let (hash, token) = self.cache.token(kind, text, leading_trivia, trailing_trivia);
        self.children.push((hash, token.into()));
    }

    /// Start new node and make it current.
    #[inline]
    pub fn start_node(&mut self, kind: SyntaxKind) {
        let len = self.children.len();
        self.parents.push((kind, len));
    }

    /// Finish current branch and restore previous
    /// branch as current.
    #[inline]
    pub fn finish_node(&mut self) {
        let (kind, first_child) = self.parents.pop().unwrap();
        let (hash, node) = self.cache.node(kind, &mut self.children, first_child);
        self.children.push((hash, node.into()));
    }

    /// Complete tree building. Make sure that
    /// `start_node_at` and `finish_node` calls
    /// are paired!
    #[inline]
    pub fn finish(mut self) -> GreenNode {
        assert_eq!(self.children.len(), 1);
        match self.children.pop().unwrap().1 {
            NodeOrToken::Node(node) => node,
            NodeOrToken::Token(_) => panic!(),
        }
    }
}
