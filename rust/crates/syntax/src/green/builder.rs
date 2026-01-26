use crate::{
    GreenNode, GreenToken, GreenTrait, GreenTrivia, NodeCache, NodeOrTokenOrTrivia, Slot, SyntaxKind,
    cow_mut::CowMut,
    green::{DiagnosticSeverity, GreenDiagnostic, GreenDiagnostics, element::GreenElement},
};

/// A builder for a green tree.
#[derive(Default, Debug)]
pub struct GreenNodeBuilder<'cache> {
    cache: CowMut<'cache, NodeCache>,
    parents: Vec<(SyntaxKind, usize)>,
    children: Vec<(u64, GreenElement)>,
    current_token: Option<TokenBuilder>,
    pending_diagnostics: Vec<GreenDiagnostic>,
}

impl<'cache> GreenNodeBuilder<'cache> {
    /// Creates a new empty builder.
    #[inline]
    pub fn new() -> GreenNodeBuilder<'static> {
        GreenNodeBuilder::default()
    }

    /// Reusing `NodeCache` between different `GreenNodeBuilder`s saves memory.
    /// It allows to structurally share underlying trees.
    pub fn with_cache(cache: &mut NodeCache) -> GreenNodeBuilder<'_> {
        GreenNodeBuilder {
            cache: CowMut::Borrowed(cache),
            parents: Vec::new(),
            children: Vec::new(),
            current_token: None,
            pending_diagnostics: Vec::new(),
        }
    }

    /// Start new token and make it current.
    #[inline]
    pub fn start_token(&mut self, kind: SyntaxKind) {
        assert!(self.current_token.is_none(), "Cannot start a new token while another is active");
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
        let token_builder = self.current_token.as_mut().expect("No current token");
        assert!(!token_builder.text_set, "Token text already set");
        token_builder.text = Some(text.to_vec());
        token_builder.text_set = true;
    }

    /// Add trivia to the current token.
    /// Trivia before `token_text()` becomes leading trivia, after becomes trailing trivia.
    #[inline]
    pub fn trivia(&mut self, kind: SyntaxKind, text: &[u8]) {
        let token_builder = self.current_token.as_mut().expect("No current token");
        let trivia = GreenTrivia::new(kind, text);

        if token_builder.text_set {
            // After text() call - trailing trivia
            token_builder.trailing_trivia.push(trivia);
        } else {
            // Before text() call - leading trivia
            token_builder.leading_trivia.push(trivia);
        }
    }

    /// Finish current token and restore previous branch as current.
    #[inline]
    pub fn finish_token(&mut self) {
        let token_builder = self.current_token.take().expect("No current token to finish");
        let text = token_builder.text.expect("Token text must be set before finishing the token");

        let leading_trivia = match token_builder.leading_trivia.is_empty() {
            true => None,
            false => Some(GreenNode::new(
                SyntaxKind::List,
                token_builder.leading_trivia.into_iter().map(GreenElement::Trivia),
                None,
            )),
        };

        let trailing_trivia = match token_builder.trailing_trivia.is_empty() {
            true => None,
            false => Some(GreenNode::new(
                SyntaxKind::List,
                token_builder.trailing_trivia.into_iter().map(GreenElement::Trivia),
                None,
            )),
        };

        let diagnostics = match self.pending_diagnostics.is_empty() {
            true => None,
            false => Some(GreenDiagnostics::new(&self.pending_diagnostics)),
        };

        let (hash, token) = self.cache.token(token_builder.kind, &text, leading_trivia, trailing_trivia, diagnostics);
        self.children.push((hash, token.into()));
        self.pending_diagnostics.clear();
    }

    /// Add a diagnostic to the most recently added element.
    /// Returns an error if no element has been added yet.
    #[inline]
    pub fn add_diagnostic(&mut self, severity: DiagnosticSeverity, code: u16, message: &str) -> Result<(), &'static str> {
        if self.children.is_empty() {
            return Err("Cannot add diagnostic: no elements have been added yet");
        }

        let last_idx = self.children.len() - 1;
        let diagnostic = GreenDiagnostic::new(code, severity, message);

        match &mut self.children[last_idx] {
            (_, GreenElement::Token(token)) => {
                // Create new token with diagnostic added
                let existing_diags = token.diagnostics().map(|d| d.diagnostics().to_vec()).unwrap_or_default();
                let mut all_diags = existing_diags;
                all_diags.push(diagnostic);
                let new_diagnostics = Some(GreenDiagnostics::new(&all_diags));

                *token = GreenToken::new(
                    token.kind(),
                    token.text(),
                    token.leading_trivia().clone(),
                    token.trailing_trivia().clone(),
                    new_diagnostics,
                );
            }
            (_, GreenElement::Node(node)) => {
                // Create new node with diagnostic added
                let existing_diags = node.diagnostics().map(|d| d.diagnostics().to_vec()).unwrap_or_default();
                let mut all_diags = existing_diags;
                all_diags.push(diagnostic);
                let new_diagnostics = Some(GreenDiagnostics::new(&all_diags));

                // Reconstruct node with new diagnostics
                let slots: Vec<_> = node
                    .slots()
                    .cloned()
                    .map(|s| match s {
                        Slot::Node { node, .. } => GreenElement::Node(node),
                        Slot::Token { token, .. } => GreenElement::Token(token),
                        Slot::Trivia { trivia, .. } => GreenElement::Trivia(trivia),
                    })
                    .collect();

                *node = GreenNode::new(node.kind(), slots, new_diagnostics);
            }
            (_, GreenElement::Trivia(_)) => {
                return Err("Cannot add diagnostics to trivia");
            }
        }

        Ok(())
    }

    /// Start new node and make it current.
    #[inline]
    pub fn start_node(&mut self, kind: SyntaxKind) {
        let len = self.children.len();
        self.parents.push((kind, len));
    }

    /// Finish current branch and restore previous branch as current.
    #[inline]
    pub fn finish_node(&mut self) {
        let (kind, first_child) = self.parents.pop().expect("No current node to finish");

        let diagnostics = match self.pending_diagnostics.is_empty() {
            true => None,
            false => Some(GreenDiagnostics::new(&self.pending_diagnostics)),
        };

        let (hash, node) = self.cache.node(kind, &mut self.children, first_child, diagnostics);
        self.children.push((hash, node.into()));
        self.pending_diagnostics.clear();
    }

    /// Complete tree building. Make sure that `start_node` and `finish_node` calls are paired!
    #[inline]
    pub fn finish(mut self) -> GreenNode {
        assert_eq!(self.children.len(), 1, "Builder should have exactly one root element");
        match self.children.pop().unwrap().1 {
            NodeOrTokenOrTrivia::Node(node) => node,
            NodeOrTokenOrTrivia::Token(_) => panic!(),
            NodeOrTokenOrTrivia::Trivia(_) => panic!(),
        }
    }
}

#[derive(Debug)]
struct TokenBuilder {
    kind: SyntaxKind,
    text: Option<Vec<u8>>,
    text_set: bool,
    leading_trivia: Vec<GreenTrivia>,
    trailing_trivia: Vec<GreenTrivia>,
}
