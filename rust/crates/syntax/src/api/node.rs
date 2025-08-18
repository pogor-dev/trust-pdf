use std::{borrow::Cow, fmt, marker::PhantomData, ops::Range};

use crate::{
    Direction, GreenNode, GreenNodeData, Language, NodeOrToken, SyntaxElement,
    SyntaxElementChildren, SyntaxNodeChildren, SyntaxText, SyntaxToken, TokenAtOffset, WalkEvent,
    api::{Preorder, PreorderWithTokens},
    cursor,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxNode<L: Language> {
    raw: cursor::SyntaxNode,
    _p: PhantomData<L>,
}

impl<L: Language> SyntaxNode<L> {
    pub fn new_root(green: GreenNode) -> SyntaxNode<L> {
        SyntaxNode::from(cursor::SyntaxNode::new_root(green))
    }
    pub fn new_root_mut(green: GreenNode) -> SyntaxNode<L> {
        SyntaxNode::from(cursor::SyntaxNode::new_root_mut(green))
    }

    /// Returns a green tree, equal to the green tree this node
    /// belongs to, except with this node substituted. The complexity
    /// of the operation is proportional to the depth of the tree.
    pub fn replace_with(&self, replacement: GreenNode) -> GreenNode {
        self.raw.replace_with(replacement)
    }

    pub fn kind(&self) -> L::Kind {
        L::kind_from_raw(self.raw.kind())
    }

    pub fn span(&self) -> Range<usize> {
        self.raw.span()
    }

    pub fn full_span(&self) -> Range<usize> {
        self.raw.full_span()
    }

    pub fn index(&self) -> usize {
        self.raw.index()
    }

    pub fn text(&self) -> SyntaxText {
        self.raw.text()
    }

    pub fn full_text(&self) -> SyntaxText {
        self.raw.full_text()
    }

    pub fn green(&self) -> Cow<'_, GreenNodeData> {
        self.raw.green()
    }

    pub fn parent(&self) -> Option<SyntaxNode<L>> {
        self.raw.parent().map(Self::from)
    }

    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode<L>> + use<L> {
        self.raw.ancestors().map(SyntaxNode::from)
    }

    pub fn children(&self) -> SyntaxNodeChildren<L> {
        SyntaxNodeChildren {
            raw: self.raw.children(),
            _p: PhantomData,
        }
    }

    pub fn children_with_tokens(&self) -> SyntaxElementChildren<L> {
        SyntaxElementChildren {
            raw: self.raw.children_with_tokens(),
            _p: PhantomData,
        }
    }

    pub fn first_child(&self) -> Option<SyntaxNode<L>> {
        self.raw.first_child().map(Self::from)
    }

    pub fn first_child_by_kind(&self, matcher: &impl Fn(L::Kind) -> bool) -> Option<SyntaxNode<L>> {
        self.raw
            .first_child_by_kind(&|raw_kind| matcher(L::kind_from_raw(raw_kind)))
            .map(Self::from)
    }

    pub fn last_child(&self) -> Option<SyntaxNode<L>> {
        self.raw.last_child().map(Self::from)
    }

    pub fn first_child_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.first_child_or_token().map(NodeOrToken::from)
    }

    pub fn first_child_or_token_by_kind(
        &self,
        matcher: &impl Fn(L::Kind) -> bool,
    ) -> Option<SyntaxElement<L>> {
        self.raw
            .first_child_or_token_by_kind(&|raw_kind| matcher(L::kind_from_raw(raw_kind)))
            .map(NodeOrToken::from)
    }

    pub fn last_child_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.last_child_or_token().map(NodeOrToken::from)
    }

    pub fn next_sibling(&self) -> Option<SyntaxNode<L>> {
        self.raw.next_sibling().map(Self::from)
    }

    pub fn next_sibling_by_kind(
        &self,
        matcher: &impl Fn(L::Kind) -> bool,
    ) -> Option<SyntaxNode<L>> {
        self.raw
            .next_sibling_by_kind(&|raw_kind| matcher(L::kind_from_raw(raw_kind)))
            .map(Self::from)
    }

    pub fn prev_sibling(&self) -> Option<SyntaxNode<L>> {
        self.raw.prev_sibling().map(Self::from)
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.next_sibling_or_token().map(NodeOrToken::from)
    }

    pub fn next_sibling_or_token_by_kind(
        &self,
        matcher: &impl Fn(L::Kind) -> bool,
    ) -> Option<SyntaxElement<L>> {
        self.raw
            .next_sibling_or_token_by_kind(&|raw_kind| matcher(L::kind_from_raw(raw_kind)))
            .map(NodeOrToken::from)
    }

    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.prev_sibling_or_token().map(NodeOrToken::from)
    }

    /// Return the leftmost token in the subtree of this node.
    pub fn first_token(&self) -> Option<SyntaxToken<L>> {
        self.raw.first_token().map(SyntaxToken::from)
    }
    /// Return the rightmost token in the subtree of this node.
    pub fn last_token(&self) -> Option<SyntaxToken<L>> {
        self.raw.last_token().map(SyntaxToken::from)
    }

    pub fn siblings(&self, direction: Direction) -> impl Iterator<Item = SyntaxNode<L>> + use<L> {
        self.raw.siblings(direction).map(SyntaxNode::from)
    }

    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElement<L>> {
        self.raw
            .siblings_with_tokens(direction)
            .map(SyntaxElement::from)
    }

    pub fn descendants(&self) -> impl Iterator<Item = SyntaxNode<L>> + use<L> {
        self.raw.descendants().map(SyntaxNode::from)
    }

    pub fn descendants_with_tokens(&self) -> impl Iterator<Item = SyntaxElement<L>> + use<L> {
        self.raw.descendants_with_tokens().map(NodeOrToken::from)
    }

    /// Traverse the subtree rooted at the current node (including the current
    /// node) in preorder, excluding tokens.
    pub fn preorder(&self) -> Preorder<L> {
        Preorder {
            raw: self.raw.preorder(),
            _p: PhantomData,
        }
    }

    /// Traverse the subtree rooted at the current node (including the current
    /// node) in preorder, including tokens.
    pub fn preorder_with_tokens(&self) -> PreorderWithTokens<L> {
        PreorderWithTokens {
            raw: self.raw.preorder_with_tokens(),
            _p: PhantomData,
        }
    }

    /// Find a token in the subtree corresponding to this node, which covers the offset.
    /// Precondition: offset must be within node's range.
    pub fn token_at_offset(&self, offset: usize) -> TokenAtOffset<SyntaxToken<L>> {
        self.raw.token_at_offset(offset).map(SyntaxToken::from)
    }

    /// Return the deepest node or token in the current subtree that fully
    /// contains the range. If the range is empty and is contained in two leaf
    /// nodes, either one can be returned. Precondition: range must be contained
    /// within the current node
    pub fn covering_element(&self, range: Range<usize>) -> SyntaxElement<L> {
        NodeOrToken::from(self.raw.covering_element(range))
    }

    /// Finds a [`SyntaxElement`] which intersects with a given `range`. If
    /// there are several intersecting elements, any one can be returned.
    ///
    /// The method uses binary search internally, so it's complexity is
    /// `O(log(N))` where `N = self.children_with_tokens().count()`.
    pub fn child_or_token_at_range(&self, range: Range<usize>) -> Option<SyntaxElement<L>> {
        self.raw
            .child_or_token_at_range(range)
            .map(SyntaxElement::from)
    }

    /// Returns an independent copy of the subtree rooted at this node.
    ///
    /// The parent of the returned node will be `None`, the start offset will be
    /// zero, but, otherwise, it'll be equivalent to the source node.
    pub fn clone_subtree(&self) -> SyntaxNode<L> {
        SyntaxNode::from(self.raw.clone_subtree())
    }

    pub fn clone_for_update(&self) -> SyntaxNode<L> {
        SyntaxNode::from(self.raw.clone_for_update())
    }

    pub fn is_mutable(&self) -> bool {
        self.raw.is_mutable()
    }

    pub fn detach(&self) {
        self.raw.detach()
    }

    pub fn splice_children<I: IntoIterator<Item = SyntaxElement<L>>>(
        &self,
        to_delete: Range<usize>,
        to_insert: I,
    ) {
        let to_insert = to_insert.into_iter().map(cursor::SyntaxElement::from);
        self.raw.splice_children(to_delete, to_insert)
    }
}

impl<L: Language> fmt::Debug for SyntaxNode<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let mut level = 0;
            for event in self.preorder_with_tokens() {
                match event {
                    WalkEvent::Enter(element) => {
                        for _ in 0..level {
                            write!(f, "  ")?;
                        }
                        match element {
                            NodeOrToken::Node(node) => writeln!(f, "{:?}", node)?,
                            NodeOrToken::Token(token) => writeln!(f, "{:?}", token)?,
                        }
                        level += 1;
                    }
                    WalkEvent::Leave(_) => level -= 1,
                }
            }
            assert_eq!(level, 0);
            Ok(())
        } else {
            write!(f, "{:?}@{:?}", self.kind(), self.full_span())
        }
    }
}

impl<L: Language> fmt::Display for SyntaxNode<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw, f)
    }
}

impl<L: Language> From<cursor::SyntaxNode> for SyntaxNode<L> {
    fn from(raw: cursor::SyntaxNode) -> SyntaxNode<L> {
        SyntaxNode {
            raw,
            _p: PhantomData,
        }
    }
}

impl<L: Language> From<SyntaxNode<L>> for cursor::SyntaxNode {
    fn from(node: SyntaxNode<L>) -> cursor::SyntaxNode {
        node.raw
    }
}
