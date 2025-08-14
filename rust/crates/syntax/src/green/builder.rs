//! Builder for constructing immutable syntax trees with efficient memory sharing.
//!
//! This module provides tools for building "green" syntax trees - the immutable,
//! cached representation that forms the foundation of the PDF compiler's syntax
//! analysis. The builder pattern allows incremental construction while automatically
//! deduplicating identical subtrees to minimize memory usage.

use std::num::NonZeroUsize;

use crate::{
    GreenNode, GreenTrivia, NodeOrToken,
    cow_mut::CowMut,
    green::{SyntaxKind, element::GreenElement, node_cache::NodeCache},
};

/// A bookmark in the syntax tree construction process, like a save point in a video game.
///
/// Allows you to mark a position during tree building and later decide whether to wrap
/// the constructed nodes in a parent node. This enables lookahead parsing where you
/// start building tokens and only later decide if they should be grouped together.
#[derive(Clone, Copy, Debug)]
pub struct Checkpoint(NonZeroUsize);

impl Checkpoint {
    /// Creates a new checkpoint from an internal position marker.
    fn new(inner: usize) -> Self {
        Self(NonZeroUsize::new(inner + 1).unwrap())
    }

    /// Extracts the internal position marker from the checkpoint.
    fn into_inner(self) -> usize {
        self.0.get() - 1
    }
}

/// A factory for constructing immutable syntax trees (green trees) with optimal memory sharing.
///
/// Think of this as a specialized document builder that constructs PDF syntax trees from
/// tokens and nodes. It maintains a stack of parent nodes being built and efficiently
/// reuses identical subtrees through caching, similar to how string interning works.
/// The "green" tree is the immutable, cached representation that multiple "red" trees
/// can reference without duplication.
#[derive(Default, Debug)]
pub struct GreenNodeBuilder<'cache> {
    cache: CowMut<'cache, NodeCache>,
    parents: Vec<(SyntaxKind, usize)>,
    children: Vec<(u64, GreenElement)>,
}

impl GreenNodeBuilder<'_> {
    /// Creates a new builder with its own cache for constructing syntax trees.
    pub fn new() -> GreenNodeBuilder<'static> {
        GreenNodeBuilder::default()
    }

    /// Creates a builder that shares a cache with other builders for memory efficiency.
    ///
    /// Reusing caches between builders is like having a shared library of common phrases -
    /// when multiple documents use the same PDF constructs, they can reference the same
    /// cached nodes instead of creating duplicates, significantly reducing memory usage.
    pub fn with_cache(cache: &mut NodeCache) -> GreenNodeBuilder<'_> {
        GreenNodeBuilder {
            cache: CowMut::Borrowed(cache),
            parents: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Adds a simple token (like a keyword or operator) to the current syntax tree level.
    #[inline]
    pub fn token(&mut self, kind: SyntaxKind, text: &[u8]) {
        let (hash, token) =
            self.cache
                .token(kind, text, GreenTrivia::new([]), GreenTrivia::new([]));
        self.children.push((hash, token.into()));
    }

    /// Adds a token with surrounding whitespace and comments (trivia) to the syntax tree.
    ///
    /// Trivia includes things like spaces, newlines, and comments that don't affect the
    /// PDF's meaning but are important for preserving the original document formatting.
    #[inline]
    pub fn token_with_trivia(
        &mut self,
        kind: SyntaxKind,
        text: &[u8],
        leading_trivia: GreenTrivia,
        trailing_trivia: GreenTrivia,
    ) {
        let (hash, token) = self
            .cache
            .token(kind, text, leading_trivia, trailing_trivia);
        self.children.push((hash, token.into()));
    }

    /// Begins construction of a new parent node, like opening a new folder to organize files.
    ///
    /// All subsequent tokens and nodes added will become children of this node until
    /// `finish_node()` is called. This enables building nested tree structures that
    /// represent PDF's hierarchical syntax.
    #[inline]
    pub fn start_node(&mut self, kind: SyntaxKind) {
        let len = self.children.len();
        self.parents.push((kind, len));
    }

    /// Completes the current parent node and moves back to the previous level.
    ///
    /// Like closing a folder after adding all its contents - takes all the children
    /// added since `start_node()` and packages them into a single node that gets
    /// added to the parent level.
    #[inline]
    pub fn finish_node(&mut self) {
        let (kind, first_child) = self.parents.pop().unwrap();
        let (hash, node) = self.cache.node(kind, &mut self.children, first_child);
        self.children.push((hash, node.into()));
    }

    /// Creates a bookmark for potential future wrapping, enabling lookahead parsing decisions.
    ///
    /// This is essential for parsing ambiguous PDF syntax where you need to read ahead
    /// to determine the correct structure. Like marking your place in a book before
    /// reading ahead to see if you need to reinterpret what you just read.
    ///
    /// The way wrapping works is that you first of all get a checkpoint,
    /// then you place all tokens you want to wrap, and then *maybe* call
    /// `start_node_at`.
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

    /// Retroactively wraps previously added elements in a new parent node from a checkpoint.
    ///
    /// This enables "I changed my mind" parsing scenarios - you can go back to a saved
    /// position and decide that everything added since then should be grouped under
    /// a new parent. Essential for handling PDF's context-dependent syntax rules.
    #[inline]
    pub fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        let checkpoint = checkpoint.into_inner();
        assert!(
            checkpoint <= self.children.len(),
            "checkpoint no longer valid, was finish_node called early?"
        );

        if let Some(&(_, first_child)) = self.parents.last() {
            assert!(
                checkpoint >= first_child,
                "checkpoint no longer valid, was an unmatched start_node_at called?"
            );
        }

        self.parents.push((kind, checkpoint));
    }

    /// Finalizes tree construction and returns the completed immutable syntax tree.
    ///
    /// Like sealing a document after writing - ensures all parent-child relationships
    /// are properly established and returns the root node representing the entire
    /// parsed PDF structure. The tree becomes immutable and ready for analysis.
    ///
    /// # Panics
    ///
    /// Panics if `start_node_at` and `finish_node` calls are not properly paired,
    /// or if the final result is not a single root node.
    #[inline]
    pub fn finish(mut self) -> GreenNode {
        assert_eq!(self.children.len(), 1);
        match self.children.pop().unwrap().1 {
            NodeOrToken::Node(node) => node,
            NodeOrToken::Token(_) => panic!(),
        }
    }
}
