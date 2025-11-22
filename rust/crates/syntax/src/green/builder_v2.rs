use std::num::NonZeroUsize;

use crate::{
    DiagnosticInfo, GreenTrivia, NodeOrToken,
    green::{GreenNode, SyntaxKind, element::GreenElementInTree, node_cache::NodeCache},
};

/// A checkpoint for maybe wrapping a node. See `GreenNodeBuilder::checkpoint` for details.
#[derive(Clone, Copy, Debug)]
pub struct Checkpoint(NonZeroUsize);

impl Checkpoint {
    fn new(inner: usize) -> Self {
        Self(NonZeroUsize::new(inner + 1).unwrap())
    }

    fn into_inner(self) -> usize {
        self.0.get() - 1
    }
}

/// Builder for constructing tokens with trivia without intermediate allocations.
///
/// Trivia added before calling `text()` becomes leading trivia.
/// Trivia added after calling `text()` becomes trailing trivia.
/// The order of trivia is preserved as added.
pub struct TokenBuilder<'a> {
    cache: &'a mut NodeCache,
    kind: SyntaxKind,
    text: Option<Vec<u8>>,
    leading_trivia: Vec<GreenTrivia>,
    trailing_trivia: Vec<GreenTrivia>,
    text_set: bool,
    diagnostics: Vec<(ErrorCode, DiagnosticContext)>,
    current_offset: usize,
    text_offset: usize,
    text_length: usize,
}

#[derive(Debug, Clone, Copy)]
enum DiagnosticContext {
    /// Diagnostic attached right after text() - applies to token text only
    TokenOnly,
    /// Diagnostic attached after trivia - applies to that trivia
    Trivia { offset: usize, length: usize },
    /// Diagnostic attached at token level - includes all trivia
    TokenWithTrivia,
}

impl<'a> TokenBuilder<'a> {
    fn new(cache: &'a mut NodeCache, kind: SyntaxKind) -> Self {
        Self {
            cache,
            kind,
            text: None,
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
            text_set: false,
            diagnostics: Vec::new(),
            current_offset: 0,
            text_offset: 0,
            text_length: 0,
        }
    }

    /// Set the token text content. After this, any trivia added becomes trailing trivia.
    pub fn text(mut self, text: &[u8]) -> Self {
        self.text_offset = self.current_offset;
        self.text_length = text.len();
        self.current_offset += text.len();
        self.text = Some(text.to_vec());
        self.text_set = true;
        self
    }

    /// Add trivia. If `text()` hasn't been called yet, this is leading trivia.
    /// If `text()` has been called, this is trailing trivia.
    pub fn trivia(mut self, kind: SyntaxKind, text: &[u8]) -> Self {
        let (_, trivia) = self.cache.trivia(kind, std::str::from_utf8(text).unwrap());
        let trivia_len = text.len();

        if self.text_set {
            self.trailing_trivia.push(trivia);
        } else {
            self.leading_trivia.push(trivia);
        }

        self.current_offset += trivia_len;
        self
    }

    /// Convenience: add a space trivia (leading or trailing depending on context)
    pub fn space(self) -> Self {
        self.trivia(SyntaxKind::Whitespace, b" ")
    }

    /// Convenience: add a newline trivia (leading or trailing depending on context)
    pub fn newline(self) -> Self {
        self.trivia(SyntaxKind::Whitespace, b"\n")
    }

    /// Convenience: add a comment trivia (leading or trailing depending on context)
    pub fn comment(self, text: &[u8]) -> Self {
        self.trivia(SyntaxKind::Comment, text)
    }

    /// Attach a diagnostic (error, warning, etc.) to this token.
    ///
    /// Smart context detection:
    /// - After `text()`: diagnostic applies to token text only (excludes trivia)
    /// - After `trivia()`: diagnostic applies to the last added trivia
    /// - Before `text()`: diagnostic includes all trivia
    pub fn diagnostic(mut self, code: ErrorCode) -> Self {
        let context = if self.text_set && !self.trailing_trivia.is_empty() {
            // After trivia - get the last trivia's length
            let last_trivia_len = self.current_offset - self.text_offset - self.text_length;
            DiagnosticContext::Trivia {
                offset: self.text_offset + self.text_length,
                length: last_trivia_len,
            }
        } else if self.text_set {
            // Right after text() - token only
            DiagnosticContext::TokenOnly
        } else {
            // Before text() - full token with trivia
            DiagnosticContext::TokenWithTrivia
        };

        self.diagnostics.push((code, context));
        self
    }

    /// Attach a diagnostic that explicitly applies to the full token including all trivia.
    /// Use this when you want to ensure the diagnostic spans the entire token regardless of position.
    pub fn diagnostic_full(mut self, code: ErrorCode) -> Self {
        self.diagnostics.push((code, DiagnosticContext::TokenWithTrivia));
        self
    }

    /// Attach multiple diagnostics to this token
    pub fn diagnostics(mut self, codes: impl IntoIterator<Item = ErrorCode>) -> Self {
        for code in codes {
            self = self.diagnostic(code);
        }
        self
    }
}

impl<'a> From<TokenBuilder<'a>> for (u64, GreenElementInTree, Vec<DiagnosticInfo>) {
    fn from(builder: TokenBuilder<'a>) -> (u64, GreenElementInTree, Vec<DiagnosticInfo>) {
        let text = builder.text.expect("Token text must be set before conversion");
        let (hash, token) = builder
            .cache
            .token_with_trivia(builder.kind, &text, &builder.leading_trivia, &builder.trailing_trivia);

        // Calculate leading trivia length
        let leading_len: usize = builder.leading_trivia.iter().map(|t| t.text_len()).sum();
        let trailing_len: usize = builder.trailing_trivia.iter().map(|t| t.text_len()).sum();
        let full_len = leading_len + builder.text_length + trailing_len;

        // Convert diagnostics with context to DiagnosticInfo
        let diagnostics = builder
            .diagnostics
            .into_iter()
            .map(|(code, context)| {
                let (offset, length) = match context {
                    DiagnosticContext::TokenOnly => (leading_len, builder.text_length),
                    DiagnosticContext::Trivia { offset, length } => (offset, length),
                    DiagnosticContext::TokenWithTrivia => (0, full_len),
                };
                DiagnosticInfo::new_with_offset_and_length(code, offset, length)
            })
            .collect();

        (hash, token.into(), diagnostics)
    }
}

/// A builder for a green tree.
#[derive(Default)]
pub struct GreenNodeBuilder {
    cache: NodeCache,
    parents: Vec<(SyntaxKind, usize)>,
    children: Vec<(u64, GreenElementInTree)>,
    diagnostics: Vec<DiagnosticInfo>,
    pending_node_diagnostics: Vec<ErrorCode>,
    node_offsets: Vec<usize>,
    current_offset: usize,
    /// Token currently being built with start_token/finish_token
    token_in_progress: Option<TokenInProgress>,
}

/// Temporary storage for a token being built incrementally
struct TokenInProgress {
    kind: SyntaxKind,
    text: Option<Vec<u8>>,
    leading_trivia: Vec<GreenTrivia>,
    trailing_trivia: Vec<GreenTrivia>,
    text_set: bool,
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

    /// Adds new token to the current branch (simple version without trivia).
    #[inline]
    pub fn token(&mut self, kind: SyntaxKind, text: &[u8]) {
        let (hash, token) = self.cache.token(kind, std::str::from_utf8(text).unwrap());
        self.children.push((hash, token.into()));
    }

    /// Create a token builder for adding trivia
    #[inline]
    pub fn token_builder(&mut self, kind: SyntaxKind) -> TokenBuilder<'_> {
        TokenBuilder::new(&mut self.cache, kind)
    }

    /// Add a token with trivia (accepts TokenBuilder via Into)
    #[inline]
    pub fn add_token(&mut self, token: impl Into<(u64, GreenElementInTree, Vec<DiagnosticInfo>)>) {
        let (hash, element, token_diagnostics) = token.into();

        // Update offset tracking
        let element_len = match &element {
            NodeOrToken::Token(t) => t.text_len(),
            NodeOrToken::Node(n) => n.text_len(),
        };
        self.current_offset += element_len;

        self.children.push((hash, element));
        self.diagnostics.extend(token_diagnostics);
    }

    /// Start building a token with trivia incrementally (similar to start_node).
    ///
    /// Use this when you want to build tokens imperatively rather than using
    /// the fluent `token_builder()` API. Call `finish_token()` when done.
    ///
    /// # Example
    /// ```ignore
    /// builder.start_token(SyntaxKind::Number);
    /// builder.add_leading_trivia(SyntaxKind::Whitespace, b" ");
    /// builder.set_token_text(b"42");
    /// builder.add_trailing_trivia(SyntaxKind::Whitespace, b"\n");
    /// builder.finish_token();
    /// ```
    #[inline]
    pub fn start_token(&mut self, kind: SyntaxKind) {
        assert!(self.token_in_progress.is_none(), "finish_token() must be called before start_token()");
        self.token_in_progress = Some(TokenInProgress {
            kind,
            text: None,
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
            text_set: false,
        });
    }

    /// Add leading trivia to the token being built.
    /// Must be called after `start_token()` and before `set_token_text()`.
    #[inline]
    pub fn add_leading_trivia(&mut self, kind: SyntaxKind, text: &[u8]) {
        let token = self.token_in_progress.as_mut().expect("start_token() must be called first");
        assert!(!token.text_set, "add_leading_trivia() must be called before set_token_text()");
        let (_, trivia) = self.cache.trivia(kind, std::str::from_utf8(text).unwrap());
        token.leading_trivia.push(trivia);
    }

    /// Set the token's text content.
    /// Must be called after `start_token()`.
    #[inline]
    pub fn set_token_text(&mut self, text: &[u8]) {
        let token = self.token_in_progress.as_mut().expect("start_token() must be called first");
        token.text = Some(text.to_vec());
        token.text_set = true;
    }

    /// Add trailing trivia to the token being built.
    /// Must be called after `set_token_text()`.
    #[inline]
    pub fn add_trailing_trivia(&mut self, kind: SyntaxKind, text: &[u8]) {
        let token = self.token_in_progress.as_mut().expect("start_token() must be called first");
        assert!(token.text_set, "add_trailing_trivia() must be called after set_token_text()");
        let (_, trivia) = self.cache.trivia(kind, std::str::from_utf8(text).unwrap());
        token.trailing_trivia.push(trivia);
    }

    /// Finish building the current token and add it to the tree.
    /// Must be paired with `start_token()`.
    #[inline]
    pub fn finish_token(&mut self) {
        let token = self.token_in_progress.take().expect("start_token() must be called first");
        let text = token.text.expect("set_token_text() must be called before finish_token()");

        let (hash, green_token) = self
            .cache
            .token_with_trivia(token.kind, std::str::from_utf8(&text).unwrap(), &token.leading_trivia, &token.trailing_trivia);

        let element_len = green_token.text_len();
        self.current_offset += element_len;
        self.children.push((hash, green_token.into()));
    }

    /// Attach a diagnostic to the current node being built.
    /// The diagnostic will span the entire node including all children and trivia.
    #[inline]
    pub fn diagnostic(&mut self, code: ErrorCode) {
        self.pending_node_diagnostics.push(code);
    }

    /// Attach multiple diagnostics to the current node being built
    #[inline]
    pub fn diagnostics(&mut self, codes: impl IntoIterator<Item = ErrorCode>) {
        self.pending_node_diagnostics.extend(codes);
    }

    /// Start new node and make it current.
    #[inline]
    pub fn start_node(&mut self, kind: SyntaxKind) {
        let len = self.children.len();
        self.parents.push((kind, len));
        self.node_offsets.push(self.current_offset);
    }

    /// Finish current branch and restore previous
    /// branch as current.
    #[inline]
    pub fn finish_node(&mut self) {
        let (kind, first_child) = self.parents.pop().unwrap();
        let node_start_offset = self.node_offsets.pop().unwrap();
        let (hash, node) = self.cache.node(kind, &mut self.children, first_child);

        let node_length = self.current_offset - node_start_offset;

        // Convert pending node diagnostics with full node span
        if !self.pending_node_diagnostics.is_empty() {
            for code in self.pending_node_diagnostics.drain(..) {
                self.diagnostics
                    .push(DiagnosticInfo::new_with_offset_and_length(code, node_start_offset, node_length));
            }
        }

        self.children.push((hash, node.into()));
    }

    /// Prepare for maybe wrapping the next node.
    /// The way wrapping works is that you first of all get a checkpoint,
    /// then you place all tokens you want to wrap, and then *maybe* call
    /// `start_node_at`.
    ///
    /// Example:
    /// ```rust
    /// # use syntax::{GreenNodeBuilder, SyntaxKind};
    /// # const PLUS: SyntaxKind = SyntaxKind(0);
    /// # const OPERATION: SyntaxKind = SyntaxKind(1);
    /// # struct Parser;
    /// # impl Parser {
    /// #     fn peek(&self) -> Option<SyntaxKind> { None }
    /// #     fn parse_expr(&mut self) {}
    /// # }
    /// # let mut builder = GreenNodeBuilder::new();
    /// # let mut parser = Parser;
    /// let checkpoint = builder.checkpoint();
    /// parser.parse_expr();
    /// if parser.peek() == Some(PLUS) {
    ///   // 1 + 2 = Add(1, 2)
    ///   builder.start_node_at(checkpoint, OPERATION);
    ///   parser.parse_expr();
    ///   builder.finish_node();
    /// }
    /// ```
    #[inline]
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint::new(self.children.len())
    }

    /// Wrap the previous branch marked by `checkpoint` in a new branch and
    /// make it current.
    #[inline]
    pub fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        let checkpoint = checkpoint.into_inner();
        assert!(checkpoint <= self.children.len(), "checkpoint no longer valid, was finish_node called early?");

        if let Some(&(_, first_child)) = self.parents.last() {
            assert!(checkpoint >= first_child, "checkpoint no longer valid, was an unmatched start_node_at called?");
        }

        self.parents.push((kind, checkpoint));
    }

    /// Complete tree building. Make sure that
    /// `start_node` and `finish_node` calls
    /// are paired!
    #[inline]
    pub fn finish(mut self) -> (GreenNode, Vec<DiagnosticInfo>) {
        let arena = self.cache.arena.shareable();
        assert_eq!(self.children.len(), 1);
        let tree = match self.children.pop().unwrap().1 {
            NodeOrToken::Node(node) => GreenNode { node, arena },
            NodeOrToken::Token(_) => panic!("Expected node, got token"),
        };
        (tree, self.diagnostics)
    }

    /// Get all accumulated diagnostics without finishing the builder
    #[inline]
    pub fn diagnostics(&self) -> &[DiagnosticInfo] {
        &self.diagnostics
    }
}

// ============================================================================
// Roslyn-style Fluent API (Alternative to macro)
// ============================================================================

/// Arena-based builder for constructing green trees with Roslyn-style fluent API.
///
/// This provides an alternative to both the macro and the imperative builder pattern,
/// offering a fluent API similar to Roslyn's `SyntaxFactory` in C#.
///
/// # Overview
///
/// - **Arena**: `GreenTreeArena` - Central arena that manages the node cache
/// - **Node Builder**: `ArenaNodeBuilder` - Fluent builder for composing nodes
/// - **Token construction**: Via closures (zero-allocation) or `ArenaTokenBuilder` (diagnostics)
///
/// # Key Design Principles (Roslyn-aligned)
///
/// 1. **No intermediate allocations**: `.with_token()` accepts closures that work directly with cache
/// 2. **Trivia as parameters**: Pass trivia lists when creating tokens, not as builder methods
/// 3. **Diagnostic support**: Use `ArenaTokenBuilder` when you need diagnostics
/// 4. **Explicit construction**: Tokens created via `cache.token_with_trivia()`, not chained methods
///
/// # Basic Usage (Zero-allocation style)
///
/// ```ignore
/// let mut arena = GreenTreeArena::new();
///
/// // Roslyn-style: trivia passed as parameters
/// let tree = arena.node(Expression)
///     .with_token(|cache| {
///         let (_, space) = cache.trivia(Whitespace, " ");
///         cache.token_with_trivia(
///             Number,
///             "42",
///             &[space],  // Leading trivia
///             &[]        // Trailing trivia
///         )
///     })
///     .with_token(|cache| {
///         cache.token(Operator, "+")  // No trivia
///     })
///     .build();
/// ```
///
/// # Comparison with Roslyn C#
///
/// **Roslyn C#**:
/// ```csharp
/// CompilationUnit()
///     .WithMembers(
///         SingletonList<MemberDeclarationSyntax>(
///             GlobalStatement(
///                 LocalDeclarationStatement(
///                     VariableDeclaration(
///                         IdentifierName(
///                             Identifier(
///                                 TriviaList(),           // Leading
///                                 SyntaxKind.VarKeyword,
///                                 "var",
///                                 "var",
///                                 TriviaList(Space))))))) // Trailing
/// ```
///
/// **trust-pdf Rust** (closure-based, zero-allocation):
/// ```ignore
/// arena.node(CompilationUnit)
///     .with_node(|cache| {
///         let (_, space) = cache.trivia(Whitespace, " ");
///         cache.node(GlobalStatement, &mut [
///             (hash, cache.node(LocalDeclarationStatement, &mut [
///                 {
///                     let (h, tok) = cache.token_with_trivia(
///                         VarKeyword,
///                         "var",
///                         &[],       // Leading trivia
///                         &[space]   // Trailing trivia
///                     );
///                     (h, NodeOrToken::Token(tok))
///                 }
///             ]))
///         ])
///     })
///     .build()
/// ```
///
/// # With Diagnostics (Builder style)
///
/// When you need diagnostics, use `ArenaTokenBuilder` instead of closures:
///
/// ```ignore
/// arena.node(Expression)
///     .with_token(
///         arena.token(Number)
///             .text(b"invalid")
///             .diagnostic(ErrorCode::UnexpectedToken)
///     )
///     .build()
/// ```
///
/// # Nested Nodes
///
/// ```ignore
/// let tree = arena.node(Object)
///     .with_node(|cache| {
///         cache.node(Header, &mut [
///             (h1, NodeOrToken::Token(cache.token(Number, "1").1)),
///             (h2, NodeOrToken::Token(cache.token(Keyword, "obj").1))
///         ])
///     })
///     .build();
/// ```
pub struct GreenTreeArena {
    cache: NodeCache,
}

impl GreenTreeArena {
    pub fn new() -> Self {
        Self { cache: NodeCache::default() }
    }

    /// Create a token builder for the given syntax kind
    pub fn token(&mut self, kind: SyntaxKind) -> ArenaTokenBuilder<'_> {
        ArenaTokenBuilder {
            cache: &mut self.cache,
            kind,
            text: None,
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
            text_set: false,
            diagnostics: Vec::new(),
            current_offset: 0,
            text_offset: 0,
            text_length: 0,
        }
    }

    /// Create a node builder for the given syntax kind
    pub fn node(&mut self, kind: SyntaxKind) -> ArenaNodeBuilder<'_> {
        ArenaNodeBuilder {
            cache: &mut self.cache,
            kind,
            children: Vec::new(),
        }
    }
}

impl Default for GreenTreeArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Token builder that borrows from arena
///
/// Created via `arena.token(kind)`. Provides fluent API for building tokens with trivia.
///
/// # Methods
///
/// - `text(bytes)` - Set token content (required)
/// - `space()` - Add space trivia (smart: leading before text, trailing after)
/// - `leading_space()` - Explicitly add leading space
/// - `trailing_space()` - Explicitly add trailing space
/// - `newline()` - Add newline trivia (smart)
/// - `comment(text)` - Add comment trivia (smart)
/// - `diagnostic(code)` - Attach diagnostic (smart context)
/// - `build()` - Explicitly build (or use implicit `Into`)
pub struct ArenaTokenBuilder<'a> {
    cache: &'a mut NodeCache,
    kind: SyntaxKind,
    text: Option<Vec<u8>>,
    leading_trivia: Vec<GreenTrivia>,
    trailing_trivia: Vec<GreenTrivia>,
    text_set: bool,
    diagnostics: Vec<(ErrorCode, DiagnosticContext)>,
    current_offset: usize,
    text_offset: usize,
    text_length: usize,
}

impl<'a> ArenaTokenBuilder<'a> {
    /// Set the token text content
    pub fn text(mut self, text: &[u8]) -> Self {
        self.text_offset = self.current_offset;
        self.text_length = text.len();
        self.current_offset += text.len();
        self.text = Some(text.to_vec());
        self.text_set = true;
        self
    }

    /// Add leading trivia (before text is set) or trailing trivia (after text is set)
    pub fn space(mut self) -> Self {
        let (_, trivia) = self.cache.trivia(SyntaxKind::Whitespace, " ");
        if self.text_set {
            self.trailing_trivia.push(trivia);
        } else {
            self.leading_trivia.push(trivia);
        }
        self.current_offset += 1;
        self
    }

    /// Add leading space (explicit)
    pub fn leading_space(mut self) -> Self {
        let (_, trivia) = self.cache.trivia(SyntaxKind::Whitespace, " ");
        self.leading_trivia.push(trivia);
        self.current_offset += 1;
        self
    }

    /// Add trailing space (explicit)
    pub fn trailing_space(mut self) -> Self {
        let (_, trivia) = self.cache.trivia(SyntaxKind::Whitespace, " ");
        self.trailing_trivia.push(trivia);
        self.current_offset += 1;
        self
    }

    /// Add newline
    pub fn newline(mut self) -> Self {
        let (_, trivia) = self.cache.trivia(SyntaxKind::Whitespace, "\n");
        if self.text_set {
            self.trailing_trivia.push(trivia);
        } else {
            self.leading_trivia.push(trivia);
        }
        self.current_offset += 1;
        self
    }

    /// Add comment
    pub fn comment(mut self, text: &[u8]) -> Self {
        let (_, trivia) = self.cache.trivia(SyntaxKind::Comment, std::str::from_utf8(text).unwrap());
        let len = text.len();
        if self.text_set {
            self.trailing_trivia.push(trivia);
        } else {
            self.leading_trivia.push(trivia);
        }
        self.current_offset += len;
        self
    }

    /// Attach diagnostic (smart context)
    pub fn diagnostic(mut self, code: ErrorCode) -> Self {
        let context = if self.text_set && !self.trailing_trivia.is_empty() {
            let last_trivia_len = self.current_offset - self.text_offset - self.text_length;
            DiagnosticContext::Trivia {
                offset: self.text_offset + self.text_length,
                length: last_trivia_len,
            }
        } else if self.text_set {
            DiagnosticContext::TokenOnly
        } else {
            DiagnosticContext::TokenWithTrivia
        };
        self.diagnostics.push((code, context));
        self
    }

    /// Build the token (explicit)
    pub fn build(self) -> (u64, GreenElementInTree, Vec<DiagnosticInfo>) {
        self.into()
    }
}

impl<'a> From<ArenaTokenBuilder<'a>> for (u64, GreenElementInTree, Vec<DiagnosticInfo>) {
    fn from(builder: ArenaTokenBuilder<'a>) -> (u64, GreenElementInTree, Vec<DiagnosticInfo>) {
        let text = builder.text.expect("Token text must be set before conversion");
        let (hash, token) = builder.cache.token_with_trivia(
            builder.kind,
            std::str::from_utf8(&text).unwrap(),
            &builder.leading_trivia,
            &builder.trailing_trivia,
        );

        let leading_len: usize = builder.leading_trivia.iter().map(|t| t.text_len()).sum();
        let trailing_len: usize = builder.trailing_trivia.iter().map(|t| t.text_len()).sum();
        let full_len = leading_len + builder.text_length + trailing_len;

        let diagnostics = builder
            .diagnostics
            .into_iter()
            .map(|(code, context)| {
                let (offset, length) = match context {
                    DiagnosticContext::TokenOnly => (leading_len, builder.text_length),
                    DiagnosticContext::Trivia { offset, length } => (offset, length),
                    DiagnosticContext::TokenWithTrivia => (0, full_len),
                };
                DiagnosticInfo::new_with_offset_and_length(code, offset, length)
            })
            .collect();

        (hash, token.into(), diagnostics)
    }
}

/// Node builder that borrows from arena
///
/// Created via `arena.node(kind)`. Provides fluent API for composing nodes from children.
///
/// # Methods
///
/// - `with_token(token)` - Add a token child (accepts `ArenaTokenBuilder` or built token)
/// - `with_node(node)` - Add a node child
/// - `build()` - Build the node (or use implicit `Into`)
pub struct ArenaNodeBuilder<'a> {
    cache: &'a mut NodeCache,
    kind: SyntaxKind,
    children: Vec<(u64, GreenElementInTree)>,
}

impl<'a> ArenaNodeBuilder<'a> {
    /// Add a child token
    ///
    /// Accepts either:
    /// - A closure `|cache| -> (u64, GreenToken)` for zero-allocation construction
    /// - An `ArenaTokenBuilder` for diagnostic support via fluent API
    ///
    /// # Example (closure style - zero allocation)
    /// ```ignore
    /// arena.node(Expression)
    ///     .with_token(|cache| {
    ///         let (_, space) = cache.trivia(Whitespace, " ");
    ///         cache.token_with_trivia(Number, "42", &[space], &[])
    ///     })
    /// ```
    ///
    /// # Example (builder style - with diagnostics)
    /// ```ignore
    /// arena.node(Expression)
    ///     .with_token(
    ///         arena.token(Number)
    ///             .text(b"42")
    ///             .diagnostic(ErrorCode::UnexpectedToken)
    ///     )
    /// ```
    pub fn with_token(mut self, token: impl TokenProvider<'a>) -> Self {
        let (hash, element, _diagnostics) = token.provide_token(self.cache);
        self.children.push((hash, element));
        self
    }

    /// Add a child node
    ///
    /// Accepts a closure that receives a mutable reference to the cache.
    pub fn with_node<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut NodeCache) -> (u64, GreenNodeData),
    {
        let (hash, node) = f(self.cache);
        self.children.push((hash, NodeOrToken::Node(node)));
        self
    }

    /// Build the node (explicit)
    pub fn build(mut self) -> GreenNode {
        let (_, node) = self.cache.node(self.kind, &mut self.children, 0);
        let arena = self.cache.arena.shareable();
        GreenNode { node, arena }
    }
}

impl<'a> From<ArenaNodeBuilder<'a>> for GreenNode {
    fn from(builder: ArenaNodeBuilder<'a>) -> GreenNode {
        builder.build()
    }
}

/// Trait for providing tokens to node builders
///
/// This allows `.with_token()` to accept both closures (zero-allocation)
/// and `ArenaTokenBuilder` (for diagnostic support).
pub trait TokenProvider<'a> {
    fn provide_token(self, cache: &mut NodeCache) -> (u64, GreenElementInTree, Vec<DiagnosticInfo>);
}

// Closure implementation (zero-allocation)
impl<'a, F> TokenProvider<'a> for F
where
    F: FnOnce(&mut NodeCache) -> (u64, GreenToken),
{
    fn provide_token(self, cache: &mut NodeCache) -> (u64, GreenElementInTree, Vec<DiagnosticInfo>) {
        let (hash, token) = self(cache);
        (hash, NodeOrToken::Token(token), Vec::new())
    }
}

// ArenaTokenBuilder implementation (with diagnostics)
impl<'a> TokenProvider<'a> for ArenaTokenBuilder<'a> {
    fn provide_token(self, _cache: &mut NodeCache) -> (u64, GreenElementInTree, Vec<DiagnosticInfo>) {
        self.into()
    }
}

// ============================================================================
// Macro Examples and Usage
// ============================================================================

/// Macro for building syntax trees using builder pattern with automatic `.into()` conversion
///
/// # Syntax
///
/// Nodes: `Kind => { children }`
/// Tokens: `(Kind) => { trivia/text items }`
/// Token text: `text("content")` or `text(b"content")`
/// Trivia: Added before `text()` becomes leading trivia, after becomes trailing trivia
/// Trivia helpers: `space()`, `newline()`, `comment("text")`, `trivia(Kind, "text")`
/// Diagnostics:
///   - Token: `diagnostic(ErrorCode)` - smart context-aware diagnostics
///     - After `text()`: applies to token only (excludes trivia)
///     - After `trivia()`: applies to that trivia
///     - Before `text()`: applies to entire token with all trivia
///   - Token: `@diagnostic(ErrorCode)` - explicit full token (includes all trivia)
///   - Node: `@diagnostic(ErrorCode)` - applies to entire node span
///
/// # Examples
///
/// ```ignore
/// let tree = tree! {
///     builder,
///     Object => {
///         (Number) => {
///             space(),              // Leading trivia
///             space(),              // Leading trivia
///             text("42"),           // Token content divides leading from trailing
///             space(),              // Trailing trivia
///             comment("% count"),   // Trailing trivia
///             newline()             // Trailing trivia
///         },
///         (Operator) => { space(), text("+"), space() },
///         (Number) => { text("10") }
///     }
/// };
///
/// // With diagnostics (smart context)
/// let tree = tree! {
///     builder,
///     Expression => {
///         @diagnostic(ErrorCode::UnexpectedToken),  // Node-level: spans entire Expression
///         (Number) => {
///             text(b"invalid"),
///             diagnostic(ErrorCode::UnexpectedToken)  // Token-level: applies to token text only
///         },
///         (Number) => {
///             @diagnostic(ErrorCode::UnexpectedToken),  // Token-level: full token with all trivia
///             space(),
///             text(b"42"),
///             newline()
///         },
///         (Name) => {
///             space(),
///             text(b"/Type"),
///             @diagnostic(ErrorCode::UnexpectedToken)  // Token-level: applies to the newline trivia
///             newline(),
///         }
///     }
/// };
/// ```
#[allow(unused_macros)]
macro_rules! tree {
    ($builder:ident, $($tt:tt)*) => {{
        $crate::green::builder_v2::__tree_inner!(@node $builder, $($tt)*)
    }};
}

#[doc(hidden)]
#[allow(unused_macros)]
macro_rules! __tree_inner {
    // Node with children: Kind => { children }
    (@node $builder:ident, $kind:expr => { $($children:tt)* }) => {{
        $builder.start_node($kind);
        $($crate::green::builder_v2::__tree_inner!(@child $builder, $children);)*
        $builder.finish_node();
    }};

    // Node diagnostic: @diagnostic(ErrorCode)
    (@child $builder:ident, @diagnostic($code:expr)) => {{
        $builder.diagnostic($code);
    }};

    // Token as child: (Kind) => { trivia/text items }
    (@child $builder:ident, ($kind:expr) => { $($token_items:tt)* }) => {{
        let mut token_builder = $builder.token_builder($kind);
        $(token_builder = $crate::green::builder_v2::__tree_inner!(@token token_builder, $token_items);)*
        $builder.add_token(token_builder);
    }};

    // Nested node as child
    (@child $builder:ident, $kind:expr => { $($children:tt)* }) => {{
        $crate::green::builder_v2::__tree_inner!(@node $builder, $kind => { $($children)* });
    }};

    // Inside token: text("content")
    (@token $token_builder:ident, text($text:expr)) => {{
        $token_builder.text($text)
    }};

    // Inside token: trivia(Kind, "text") - automatically leading or trailing based on context
    (@token $token_builder:ident, trivia($kind:expr, $text:expr)) => {{
        $token_builder.trivia($kind, $text.as_bytes())
    }};

    // Inside token: space() - automatically leading or trailing based on context
    (@token $token_builder:ident, space()) => {{
        $token_builder.space()
    }};

    // Inside token: newline() - automatically leading or trailing based on context
    (@token $token_builder:ident, newline()) => {{
        $token_builder.newline()
    }};

    // Inside token: comment("text") - automatically leading or trailing based on context
    (@token $token_builder:ident, comment($text:expr)) => {{
        $token_builder.comment($text.as_bytes())
    }};

    // Inside token: diagnostic(ErrorCode)
    (@token $token_builder:ident, diagnostic($code:expr)) => {{
        $token_builder.diagnostic($code)
    }};

    // Inside token: @diagnostic(ErrorCode) - explicit full token with trivia
    (@token $token_builder:ident, @diagnostic($code:expr)) => {{
        $token_builder.diagnostic_full($code)
    }};

    // Inside token: @diagnostic(ErrorCode) - explicit full token with trivia
    (@token $token_builder:ident, @diagnostic($code:expr)) => {{
        $token_builder.diagnostic($code)
    }};
}

#[cfg(test)]
mod usage_examples {
    use super::*;

    // Example usage without macros
    #[test]
    #[ignore = "just for documentation"]
    fn example_simple_usage() {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind(100)); // Object
        builder.token(SyntaxKind(1), b"42");
        builder.token(SyntaxKind(2), b"+");
        builder.token(SyntaxKind(1), b"10");
        builder.finish_node();
        let (_tree, _diagnostics) = builder.finish();
    }

    #[test]
    #[ignore = "just for documentation"]
    fn example_with_trivia() {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind(100)); // Expression

        // Token with leading and trailing trivia
        // Everything before text() is leading, everything after is trailing
        builder.add_token(
            builder
                .token_builder(SyntaxKind(1)) // Number
                .space() // Leading space
                .space() // Leading space
                .text(b"42") // Token content - divides leading from trailing
                .space() // Trailing space
                .comment(b"% first number") // Trailing comment
                .newline(), // Trailing newline
        );

        // Token with spaces on both sides
        builder.add_token(
            builder
                .token_builder(SyntaxKind(2)) // Operator
                .space() // Leading space
                .text(b"+") // Token content
                .space(), // Trailing space
        );

        // Simple token without trivia
        builder.add_token(
            builder
                .token_builder(SyntaxKind(1)) // Number
                .text(b"10"),
        );

        builder.finish_node();
        let (_tree, _diagnostics) = builder.finish();
    }

    #[test]
    #[ignore = "just for documentation"]
    fn example_with_diagnostics() {
        use crate::{DiagnosticInfo, ErrorCode};

        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind(100)); // Expression

        // Token with error diagnostic (applies to token text only)
        builder.add_token(builder.token_builder(SyntaxKind(1)).text(b"invalid").diagnostic(ErrorCode::UnexpectedToken));

        // Token with trivia diagnostic
        builder.add_token(
            builder
                .token_builder(SyntaxKind(2))
                .space() // Leading
                .text(b"+")
                .newline() // Trailing
                .diagnostic(ErrorCode::UnexpectedToken), // Applies to the newline trivia
        );

        // Token with full token diagnostic (explicit)
        builder.add_token(
            builder
                .token_builder(SyntaxKind(3))
                .space() // Leading
                .text(b"/Type")
                .newline() // Trailing
                .diagnostic_full(ErrorCode::UnexpectedToken), // Applies to full token with all trivia
        );

        // Attach diagnostic to node (spans entire node)
        builder.diagnostic(ErrorCode::UnexpectedToken);

        builder.finish_node();
        let (_tree, diagnostics) = builder.finish();

        assert_eq!(diagnostics.len(), 4); // Token diagnostic, trivia diagnostic, full token diagnostic, node diagnostic
    }

    #[test]
    #[ignore = "just for documentation"]
    fn example_checkpoint_pattern() {
        let mut builder = GreenNodeBuilder::new();
        let checkpoint = builder.checkpoint();

        builder.add_token(builder.token_builder(SyntaxKind(1)).text(b"1"));

        // Decide to wrap in binary expression
        builder.start_node_at(checkpoint, SyntaxKind(10)); // BinaryExpr
        builder.add_token(
            builder
                .token_builder(SyntaxKind(2))
                .space() // Leading
                .text(b"+")
                .space(), // Trailing
        );
        builder.add_token(builder.token_builder(SyntaxKind(1)).text(b"2"));
        builder.finish_node();

        let (_tree, _diagnostics) = builder.finish();
    }

    #[test]
    #[ignore = "just for documentation"]
    fn example_roslyn_arena_style() {
        let mut arena = GreenTreeArena::new();

        // Roslyn-style: closures receive cache, no intermediate allocations
        // "  42 % first\n + 10"
        let tree = arena
            .node(SyntaxKind(100)) // Expression
            .with_token(|cache| {
                let (_, space) = cache.trivia(SyntaxKind(0), " ");
                let (_, newline) = cache.trivia(SyntaxKind(0), "\n");
                let (_, comment) = cache.trivia(SyntaxKind(0), "% first");
                cache.token_with_trivia(
                    SyntaxKind(1), // Number
                    "42",
                    &[space, space],            // Leading trivia
                    &[space, comment, newline], // Trailing trivia
                )
            })
            .with_token(|cache| {
                let (_, space) = cache.trivia(SyntaxKind(0), " ");
                cache.token_with_trivia(
                    SyntaxKind(2), // Operator
                    "+",
                    &[space], // Leading
                    &[space], // Trailing
                )
            })
            .with_token(|cache| {
                cache.token(SyntaxKind(1), "10") // Number, no trivia
            })
            .build();

        let _final_tree = tree;
    }

    #[test]
    #[ignore = "just for documentation"]
    fn example_roslyn_with_diagnostics() {
        use crate::ErrorCode;

        let mut arena = GreenTreeArena::new();

        // For diagnostics, use the fluent ArenaTokenBuilder
        // (closure style doesn't support diagnostics)
        let tree = arena
            .node(SyntaxKind(100))
            .with_token(
                arena.token(SyntaxKind(1)).text(b"invalid").diagnostic(ErrorCode::UnexpectedToken), // Token only
            )
            .with_token(
                arena.token(SyntaxKind(2)).space().text(b"+").newline().diagnostic(ErrorCode::UnexpectedToken), // Trivia (newline)
            )
            .build();

        let _final_tree = tree;
    }
}

/*
Roslyn-style arena API usage:

use crate::green::builder_v2::GreenTreeArena;

let mut arena = GreenTreeArena::new();

// Simple expression: "  42 % first\n + 10"
let tree = arena.node(SyntaxKind::Expression)
    .with_token(
        arena.token(SyntaxKind::Number)
            .leading_space()
            .leading_space()
            .text(b"42")
            .space()
            .comment(b"% first")
            .newline()
            .build()  // Explicit build
    )
    .with_token(
        arena.token(SyntaxKind::Operator)
            .space()
            .text(b"+")
            .space()  // Implicit Into conversion
    )
    .with_token(
        arena.token(SyntaxKind::Number)
            .text(b"10")
    )
    .build();

// Complex PDF structure
let mut arena = GreenTreeArena::new();

let tree = arena.node(SyntaxKind::Object)
    .with_node(
        arena.node(SyntaxKind::IndirectObject)
            .with_token(arena.token(SyntaxKind::Number).text(b"1"))
            .with_token(arena.token(SyntaxKind::Number).space().text(b"0"))
            .with_token(
                arena.token(SyntaxKind::Keyword)
                    .space()
                    .text(b"obj")
                    .newline()
            )
            .build()
    )
    .with_node(
        arena.node(SyntaxKind::Dictionary)
            .with_token(
                arena.token(SyntaxKind::Delimiter)
                    .newline()
                    .text(b"<<")
                    .newline()
            )
            .with_token(
                arena.token(SyntaxKind::Name)
                    .leading_space()
                    .leading_space()
                    .text(b"/Type")
                    .trailing_space()
            )
            .with_token(
                arena.token(SyntaxKind::Name)
                    .text(b"/Catalog")
                    .newline()
            )
            .with_token(
                arena.token(SyntaxKind::Delimiter)
                    .text(b">>")
                    .newline()
            )
            .build()
    )
    .with_token(
        arena.token(SyntaxKind::Keyword)
            .text(b"endobj")
            .newline()
    )
    .build();
*/

/*
Macro usage examples (when macros are exported):

// Simple tree
let mut builder = GreenNodeBuilder::new();
let tree = tree! {
    builder,
    Object => {
        (Number) => { text(b"42") },
        (Operator) => { text(b"+") },
        (Number) => { text(b"10") }
    }
};

// With trivia - order determines leading vs trailing
let mut builder = GreenNodeBuilder::new();
let tree = tree! {
    builder,
    Expression => {
        (Number) => {
            space(),                    // Leading (before text())
            space(),                    // Leading (before text())
            text(b"42"),                // Divider - everything before is leading, after is trailing
            space(),                    // Trailing (after text())
            comment(b"% first number"), // Trailing (after text())
            newline()                   // Trailing (after text())
        },
        (Operator) => {
            space(),      // Leading
            text(b"+"),
            space()       // Trailing
        },
        (Number) => { text(b"10") }
    }
};

// Complex PDF structure
let mut builder = GreenNodeBuilder::new();
let tree = tree! {
    builder,
    Object => {
        Indirect => {
            (Number) => { text(b"1") },
            (Number) => {
                space(),       // Leading
                text(b"0")
            },
            (Keyword) => {
                space(),       // Leading
                text(b"obj"),
                newline()      // Trailing
            }
        },
        Dictionary => {
            (Delimiter) => {
                newline(),     // Leading
                text(b"<<"),
                newline()      // Trailing
            },
            (Name) => {
                space(),       // Leading
                space(),       // Leading
                text(b"/Type"),
                space()        // Trailing
            },
            (Name) => {
                text(b"/Catalog"),
                newline()      // Trailing
            },
            (Delimiter) => {
                text(b">>"),
                newline()      // Trailing
            }
        },
        (Keyword) => {
            text(b"endobj"),
            newline()          // Trailing
        }
    }
};

// Nested nodes
let mut builder = GreenNodeBuilder::new();
let (tree, diagnostics) = tree! {
    builder,
    Document => {
        Header => {
            (Comment) => { text(b"%PDF-1.7"), newline() }
        },
        Body => {
            Object => {
                (Number) => { text(b"1") },
                (Number) => { space(), text(b"0") },
                (Keyword) => { space(), text(b"obj"), newline() }
            }
        }
    }
};

// With smart diagnostics in macro
use crate::ErrorCode;

let mut builder = GreenNodeBuilder::new();
let (tree, diagnostics) = tree! {
    builder,
    Expression => {
        @diagnostic(ErrorCode::UnexpectedToken),  // Node-level: spans entire Expression node
        (Number) => {
            text(b"invalid"),
            diagnostic(ErrorCode::UnexpectedToken)  // Token-level: applies to "invalid" token only
        },
        (Operator) => {
            space(),      // Leading trivia
            text(b"+"),
            space()       // Trailing trivia
        },
        (Number) => {
            space(),      // Leading trivia
            text(b"42"),  // Token text
            newline(),    // Trailing trivia
            diagnostic(ErrorCode::UnexpectedToken)  // Token-level: applies to newline trivia only
        },
        (Name) => {
            @diagnostic(ErrorCode::UnexpectedToken),  // Explicit full token: space + "/Type" + newline
            space(),
            text(b"/Type"),
            newline()
        }
    }
};
// diagnostics[0]: offset 0, length = full Expression node span (node-level)
// diagnostics[1]: offset at "invalid", length 7 (token-level, token only)
// diagnostics[2]: offset at newline after "42", length 1 (token-level, trivia only)
// diagnostics[3]: offset at space before "/Type", length = full token with all trivia

// Node-level diagnostics using @diagnostic in macro
use crate::ErrorCode;

let mut builder = GreenNodeBuilder::new();
let (tree, diagnostics) = tree! {
    builder,
    Object => {
        @diagnostic(ErrorCode::UnexpectedToken),  // Spans entire Object node
        (Keyword) => {
            space(),
            text(b"token"),
            newline()
        }
    }
};
// diagnostic: offset 0, length = space + "token" + newline = full node span

// Multiple node diagnostics
let mut builder = GreenNodeBuilder::new();
let (tree, diagnostics) = tree! {
    builder,
    Document => {
        @diagnostic(ErrorCode::UnexpectedToken),  // First node diagnostic
        Header => {
            @diagnostic(ErrorCode::UnexpectedToken),  // Nested node diagnostic
            (Comment) => { text(b"%PDF-1.7") }
        },
        @diagnostic(ErrorCode::UnexpectedToken)  // Second node diagnostic on Document
    }
};
*/

macro_rules! tree {
    // Base case: no more tokens to process
    (@node $builder:ident,) => {};

    // Token with trivia: (Kind) => { trivia_items }
    (@node $builder:ident, ($kind:ident) => { $($trivia:tt)* }) => {{
        // TODO: handle trivia and text
    }};

    // Token with trivia, followed by more: (Kind) => { trivia_items }, rest
    (@node $builder:ident, ($kind:ident) => { $($trivia:tt)* }, $($rest:tt)*) => {{
        // TODO: handle trivia and text
        tree!(@node $builder, $($rest)*);
    }};

    // Node with children: Kind => { children }
    (@node $builder:ident, $kind:ident => { $($children:tt)* }) => {{
        $builder.start_node($crate::green::SyntaxKind::$kind);
        tree!(@node $builder, $($children)*);
        $builder.finish_node();
    }};

    // Multiple nodes: process first, then rest
    (@node $builder:ident, $kind:ident => { $($children:tt)* }, $($rest:tt)*) => {{
        $builder.start_node($crate::green::SyntaxKind::$kind);
        tree!(@node $builder, $($children)*);
        $builder.finish_node();
        tree!(@node $builder, $($rest)*);
    }};

    // Public entry point: creates builder and starts recursion
    ($($tt:tt)*) => {{
        let mut builder = $crate::green::builder::GreenNodeBuilder::new();
        tree!(@node builder, $($tt)*);
        builder.finish()
    }};
}
