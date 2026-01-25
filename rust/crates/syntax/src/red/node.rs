use std::{fmt, ops};

use crate::{GreenNode, Slot, SyntaxKind};

use super::{SyntaxElement, SyntaxToken, SyntaxTrivia};

/// Positioned node in the red tree, providing a view over a green node with position info.
///
/// Red nodes are lightweight cursors that don't duplicate tree data - they reference
/// the underlying immutable green node and track position/parent for navigation.
#[derive(Clone)]
pub struct SyntaxNode {
    green: GreenNode,
    parent: Option<Box<SyntaxNode>>,
    position: u32,
    index: u32,
}

impl SyntaxNode {
    /// Creates a new root syntax node at position 0.
    #[inline]
    pub fn new_root(green: GreenNode) -> Self {
        Self {
            green,
            parent: None,
            position: 0,
            index: 0,
        }
    }

    /// Creates a new child syntax node with parent link.
    #[inline]
    pub(crate) fn new_child(green: GreenNode, parent: SyntaxNode, position: u32, index: u32) -> Self {
        Self {
            green,
            parent: Some(Box::new(parent)),
            position,
            index,
        }
    }

    /// Returns the kind of this node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.green.kind()
    }

    /// Returns a reference to the underlying green node.
    #[inline]
    pub fn green(&self) -> &GreenNode {
        &self.green
    }

    /// Returns a reference to the parent node if it exists.
    #[inline]
    pub fn parent(&self) -> Option<&SyntaxNode> {
        self.parent.as_deref()
    }

    /// Returns the absolute byte position of this node in the source.
    #[inline]
    pub fn position(&self) -> u32 {
        self.position
    }

    /// Returns the index of this node within its parent's children.
    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Returns the full width of this node in bytes (including all descendants).
    #[inline]
    pub fn full_width(&self) -> u32 {
        self.green.full_width()
    }

    /// Returns the byte range span of this node in the source (start..end).
    #[inline]
    pub fn full_span(&self) -> ops::Range<u32> {
        let start = self.position;
        let end = start + self.full_width();
        start..end
    }

    /// Returns the number of direct children (slots) this node has.
    #[inline]
    pub fn child_count(&self) -> usize {
        self.green.slot_count()
    }

    /// Iterates over child elements (nodes, tokens, trivia) with their positions.
    pub fn children(&self) -> ChildIterator {
        ChildIterator::new(self)
    }

    /// Returns diagnostics attached to this node, if any.
    #[inline]
    pub fn diagnostics(&self) -> Option<&crate::GreenDiagnostics> {
        self.green.diagnostics()
    }
}

impl PartialEq for SyntaxNode {
    fn eq(&self, other: &Self) -> bool {
        // Compare by green node identity and position
        std::ptr::eq(&*self.green, &*other.green) && self.position == other.position
    }
}

impl Eq for SyntaxNode {}

impl fmt::Debug for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxNode")
            .field("kind", &self.kind())
            .field("position", &self.position)
            .field("width", &self.full_width())
            .field("children", &self.child_count())
            .finish()
    }
}

/// Iterator over positioned child elements of a syntax node.
pub struct ChildIterator<'a> {
    parent: &'a SyntaxNode,
    slots: crate::green::Slots<'a>,
    position: u32,
    index: u32,
}

impl<'a> ChildIterator<'a> {
    pub(crate) fn new(parent: &'a SyntaxNode) -> Self {
        Self {
            parent,
            slots: parent.green().slots(),
            position: parent.position(),
            index: 0,
        }
    }
}

impl<'a> Iterator for ChildIterator<'a> {
    type Item = SyntaxElement;

    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.slots.next()?;
        let pos = self.position;
        let idx = self.index;

        let (element, width) = match slot {
            Slot::Node { node, .. } => {
                let width = node.full_width();
                let child = SyntaxNode::new_child(node.clone(), self.parent.clone(), pos, idx);
                (SyntaxElement::Node(child), width)
            }
            Slot::Token { token, .. } => {
                let width = token.full_width();
                let child = SyntaxToken::new_child(token.clone(), self.parent.clone(), pos, idx);
                (SyntaxElement::Token(child), width)
            }
            Slot::Trivia { trivia, .. } => {
                let width = trivia.text().len() as u32;
                let child = SyntaxTrivia::new_child(trivia.clone(), self.parent.clone(), pos, idx);
                (SyntaxElement::Trivia(child), width)
            }
        };

        self.position += width;
        self.index += 1;
        Some(element)
    }
}

#[cfg(test)]
mod tests {
    use crate::{SyntaxElement, SyntaxKind, SyntaxNode, tree};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_root_node() {
        let green = tree! {
            SyntaxKind::List => {
                (SyntaxKind::NumericLiteralToken, b"42")
            }
        };

        let root = SyntaxNode::new_root(green);
        assert_eq!(root.kind(), SyntaxKind::List);
        assert_eq!(root.position(), 0);
        assert_eq!(root.full_width(), 2);
        assert!(root.parent().is_none());
    }

    #[test]
    fn test_children_iteration() {
        let green = tree! {
            SyntaxKind::List => {
                (SyntaxKind::NumericLiteralToken, b"42"),
                (SyntaxKind::NumericLiteralToken, b" "),
                (SyntaxKind::NumericLiteralToken, b"99")
            }
        };

        let root = SyntaxNode::new_root(green);
        let children: Vec<_> = root.children().collect();

        assert_eq!(children.len(), 3);

        match &children[0] {
            SyntaxElement::Token(t) => {
                assert_eq!(t.kind(), SyntaxKind::NumericLiteralToken);
                assert_eq!(t.position(), 0);
                assert_eq!(t.text(), b"42");
            }
            _ => panic!("Expected token"),
        }

        match &children[1] {
            SyntaxElement::Token(t) => {
                assert_eq!(t.kind(), SyntaxKind::NumericLiteralToken);
                assert_eq!(t.position(), 2);
                assert_eq!(t.text(), b" ");
            }
            _ => panic!("Expected token"),
        }

        match &children[2] {
            SyntaxElement::Token(t) => {
                assert_eq!(t.kind(), SyntaxKind::NumericLiteralToken);
                assert_eq!(t.position(), 3);
                assert_eq!(t.text(), b"99");
            }
            _ => panic!("Expected token"),
        }
    }

    #[test]
    fn test_full_span() {
        let green = tree! {
            SyntaxKind::List => {
                (SyntaxKind::NumericLiteralToken, b"123")
            }
        };

        let root = SyntaxNode::new_root(green);
        let range = root.full_span();
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 3);
    }

    #[test]
    fn test_nested_nodes() {
        let green = tree! {
            SyntaxKind::List => {
                SyntaxKind::List => {
                    (SyntaxKind::NumericLiteralToken, b"42")
                }
            }
        };

        let root = SyntaxNode::new_root(green);
        assert_eq!(root.child_count(), 1);

        let children: Vec<_> = root.children().collect();
        match &children[0] {
            SyntaxElement::Node(n) => {
                assert_eq!(n.kind(), SyntaxKind::List);
                assert_eq!(n.position(), 0);
                assert_eq!(n.full_width(), 2);
                assert!(n.parent().is_some());
            }
            _ => panic!("Expected node"),
        }
    }

    #[test]
    fn test_child_positions() {
        let green = tree! {
            SyntaxKind::List => {
                (SyntaxKind::NumericLiteralToken, b"1"),
                (SyntaxKind::NumericLiteralToken, b"22"),
                (SyntaxKind::NumericLiteralToken, b"333")
            }
        };

        let root = SyntaxNode::new_root(green);
        let children: Vec<_> = root.children().collect();

        assert_eq!(children[0].position(), 0); // "1" starts at 0
        assert_eq!(children[1].position(), 1); // "22" starts at 1
        assert_eq!(children[2].position(), 3); // "333" starts at 3
    }
}
