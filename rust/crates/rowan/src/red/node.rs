use std::{fmt, ops};

use crate::{GreenNode, SyntaxKind};

/// A positioned node in the red tree, wrapping a green node with position information.
///
/// Provides access to the underlying green node and its position in the source file.
#[repr(C)]
#[derive(Clone, Default)]
pub struct SyntaxNode<'a> {
    underlying_node: Option<GreenNode>, // 16 bytes
    parent: Option<&'a SyntaxNode<'a>>, // 8 bytes
    position: u64,                      // 8 bytes
    index: u16,                         // 2 bytes
}

impl<'a> SyntaxNode<'a> {
    /// Creates a new `SyntaxNode` with the given properties.
    #[inline]
    pub fn new(parent: Option<&'a SyntaxNode<'a>>, underlying_node: Option<GreenNode>, position: u64, index: u16) -> Self {
        Self {
            parent,
            underlying_node,
            position,
            index,
        }
    }

    /// Returns the kind of this node.
    #[inline]
    pub fn kind(&self) -> Option<SyntaxKind> {
        self.underlying_node.as_ref().map(|n| n.kind())
    }

    /// Returns a reference to the parent node if it exists.
    #[inline]
    pub fn parent(&self) -> Option<&SyntaxNode<'a>> {
        self.parent
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
        self.underlying_node.as_ref().map_or(0, |n| n.full_width())
    }

    /// Returns the width of this node (excluding trivia).
    #[inline]
    fn width(&self) -> u32 {
        self.underlying_node.as_ref().map_or(0, |n| n.width())
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
        let leading_width = self
            .underlying_node
            .as_ref()
            .and_then(|n| n.leading_trivia())
            .map_or(0, |t| t.full_width() as u64);
        let start = self.position + leading_width;
        let end = start + self.width() as u64;
        start..end
    }

    /// Returns the number of children this node has.
    #[inline]
    pub fn children_len(&self) -> u16 {
        self.underlying_node.as_ref().map_or(0, |n| n.children_len())
    }
}

impl<'a> PartialEq for SyntaxNode<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.underlying_node == other.underlying_node && self.position == other.position && self.index == other.index
    }
}

impl<'a> Eq for SyntaxNode<'a> {}

impl<'a> fmt::Debug for SyntaxNode<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxNode")
            .field("kind", &self.kind())
            .field("position", &self.position)
            .field("index", &self.index)
            .finish()
    }
}

impl<'a> fmt::Display for SyntaxNode<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.underlying_node.as_ref() {
            Some(node) => write!(f, "{}", String::from_utf8_lossy(&node.full_bytes())),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_memory_layout() {
        // 16 (GreenNode) + 8 (parent) + 8 (position) + 2 (index) = 34 bytes
        // + 6 bytes padding for 8-byte alignment = 40 bytes total
        assert_eq!(std::mem::size_of::<SyntaxNode>(), 40);
        assert_eq!(std::mem::align_of::<SyntaxNode>(), 8);
    }
}
