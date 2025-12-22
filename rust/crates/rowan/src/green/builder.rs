use crate::{
    DiagnosticInfo, GreenNode, NodeOrToken,
    diagnostics::DiagnosticSeverity,
    green::{SyntaxKind, cache::GreenCache, element::GreenElementInTree, trivia::GreenTriviaInTree},
};

/// A builder for a green tree.
#[derive(Default)]
pub struct GreenNodeBuilder {
    cache: GreenCache,
    parents: Vec<(SyntaxKind, usize)>,
    children: Vec<(u64, GreenElementInTree)>,
    current_token: Option<TokenBuilder>,
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

    /// Attaches diagnostic to the most recently added element (token or node).
    ///
    /// This method stores a diagnostic for the last element added to the tree.
    /// Multiple diagnostics can be attached to a single element.
    ///
    /// Note: Actual diagnostic storage and retrieval is handled through the arena.
    #[inline]
    pub fn add_diagnostic(&mut self, severity: DiagnosticSeverity, code: u16, message: &'static str) {
        let diagnostic = DiagnosticInfo::new(code, message, severity);
        self.cache.diagnostic(diagnostic);
    }

    /// Attaches new trivia to the current token.
    #[inline]
    pub fn trivia(&mut self, kind: SyntaxKind, text: &[u8]) {
        let token_builder = self.current_token.as_mut().expect("No current token to add trivia to");
        let (_hash, trivia) = self.cache.trivia(kind, text);

        if token_builder.text_set {
            token_builder.trailing_trivia.push(trivia);
        } else {
            token_builder.leading_trivia.push(trivia);
        }
    }

    /// Adds new token to the current branch.
    #[inline]
    pub fn token(&mut self, kind: SyntaxKind, text: &[u8], leading_trivia: &[GreenTriviaInTree], trailing_trivia: &[GreenTriviaInTree]) {
        let leading_trivia = self.cache.trivia_list(leading_trivia).1;
        let trailing_trivia = self.cache.trivia_list(trailing_trivia).1;
        let (hash, token) = self.cache.token(kind, text, leading_trivia, trailing_trivia);
        self.children.push((hash, token.into()));
    }

    /// Start new token and make it current.
    #[inline]
    pub fn start_token(&mut self, kind: SyntaxKind) {
        assert!(self.current_token.is_none(), "Nested tokens are not allowed");

        self.current_token = Some(TokenBuilder {
            kind,
            text: None,
            text_set: false,
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        });
    }

    /// Set text for the current token.
    #[inline]
    pub fn token_text(&mut self, text: &[u8]) {
        let token_builder = self.current_token.as_mut().expect("No current token to set text for");
        assert!(!token_builder.text_set, "Token text can only be set once");

        token_builder.text = Some(text.to_vec());
        token_builder.text_set = true;
    }

    /// Finish current token and restore previous
    /// branch as current.
    #[inline]
    pub fn finish_token(&mut self) {
        let token_builder = self.current_token.take().expect("No current token to finish");
        let text = token_builder.text.expect("Token text must be set before finishing the token");
        let leading_trivia = self.cache.trivia_list(&token_builder.leading_trivia).1;
        let trailing_trivia = self.cache.trivia_list(&token_builder.trailing_trivia).1;

        let (hash, token) = self.cache.token(token_builder.kind, &text, leading_trivia, trailing_trivia);
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
    /// `start_node` and `finish_node` calls
    /// are paired!
    ///
    /// Returns the root node and the arena that owns all the allocated data.
    /// The arena must be kept alive as long as any nodes from the tree are in use.
    #[inline]
    pub fn finish(mut self) -> GreenNode {
        let arena = self.cache.arena.shareable();
        assert_eq!(self.children.len(), 1);
        match self.children.pop().unwrap().1 {
            NodeOrToken::Node(node) => GreenNode { node, arena },
            NodeOrToken::Token(_) => {
                panic!("Expected root node to be a GreenNode, but got a Token. This usually indicates mismatched start_node/finish_node calls.")
            }
        }
    }
}

struct TokenBuilder {
    kind: SyntaxKind,
    text: Option<Vec<u8>>,
    text_set: bool,
    leading_trivia: Vec<GreenTriviaInTree>,
    trailing_trivia: Vec<GreenTriviaInTree>,
}
