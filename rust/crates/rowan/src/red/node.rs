use std::{fmt, ops};

use crate::{GreenNode, SyntaxKind};

/// A semantic node in the red tree, wrapping a green node with position information.
///
/// Provides access to the underlying green node and its position in the source file.
#[derive(Clone)]
pub struct SyntaxNode {
    parent: Option<Box<SyntaxNode>>,
    underlying_node: GreenNode,
    position: u64,
    index: u16,
}

impl SyntaxNode {
    /// Creates a new `SyntaxNode` with the given properties.
    #[inline]
    pub fn new(parent: Option<Box<SyntaxNode>>, underlying_node: GreenNode, position: u64, index: u16) -> Self {
        Self {
            parent,
            underlying_node,
            position,
            index,
        }
    }

    /// Returns the kind of this node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.underlying_node.kind()
    }

    /// Returns a reference to the parent node if it exists.
    #[inline]
    pub fn parent(&self) -> Option<&SyntaxNode> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    /// Returns the position of this node in the source.
    #[inline]
    fn position(&self) -> u64 {
        self.position
    }

    /// Returns the index of this node within its parent.
    #[inline]
    fn index(&self) -> u16 {
        self.index
    }

    /// Returns the full width of this node (including trivia).
    #[inline]
    fn full_width(&self) -> u32 {
        self.underlying_node.full_width()
    }

    /// Returns the width of this node (excluding trivia).
    #[inline]
    fn width(&self) -> u32 {
        self.underlying_node.width()
    }

    /// Returns the span of this node in the source (including trivia).
    #[inline]
    pub fn full_span(&self) -> ops::Range<u64> {
        let start = self.position;
        let end = start + self.full_width() as u64;
        start..end
    }

    /// Returns the span of this node without trivia.
    #[inline]
    pub fn span(&self) -> ops::Range<u64> {
        let start = self.position + self.underlying_node.leading_trivia().map(|t| t.full_width() as u64).unwrap_or(0);
        let end = start + self.width() as u64;
        start..end
    }

    /// Returns the number of children this node has.
    #[inline]
    pub fn children_len(&self) -> u16 {
        self.underlying_node.children_len()
    }
}

impl PartialEq for SyntaxNode {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.underlying_node == other.underlying_node && self.position == other.position && self.index == other.index
    }
}

impl Eq for SyntaxNode {}

impl fmt::Debug for SyntaxNode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxNode")
            .field("kind", &self.kind())
            .field("position", &self.position)
            .field("index", &self.index)
            .finish()
    }
}

impl fmt::Display for SyntaxNode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.underlying_node.full_bytes()))
    }
}
