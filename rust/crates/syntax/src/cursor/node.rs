//! SyntaxNode - The primary cursor for navigating syntax trees.
//!
//! ```text
//!     🌳 SyntaxNode
//!    ┌─────────────┐
//!    │ ┌─────────┐ │   Provides tree navigation:
//!    │ │  Node   │ │   • parent() / children()
//!    │ │ Data    │ │   • siblings() / ancestors()
//!    │ └─────────┘ │   • preorder() traversal
//!    └─────────────┘   • text ranges & offsets
//! ```

use std::{
    borrow::Cow,
    cell::Cell,
    fmt,
    hash::{Hash, Hasher},
    iter,
    ops::Range,
    ptr,
};

use crate::{
    GreenNode, GreenNodeData, NodeOrToken, SyntaxKind,
    cursor::{
        Green, NodeData, free, preorder::Preorder, preorder_with_tokens::PreorderWithTokens,
        element::SyntaxElement, element_children::SyntaxElementChildren,
        node_children::SyntaxNodeChildren, token::SyntaxToken,
    },
    utility_types::{Direction, TokenAtOffset, WalkEvent},
};

pub struct SyntaxNode {
    pub(super) ptr: ptr::NonNull<NodeData>,
}

impl SyntaxNode {
    /// Creates a new immutable root node from a green tree.
    pub fn new_root(green: GreenNode) -> SyntaxNode {
        let green = GreenNode::into_raw(green);
        let green = Green::Node {
            ptr: Cell::new(green),
        };
        SyntaxNode {
            ptr: NodeData::new(None, 0, 0, green, false),
        }
    }

    /// Creates a new mutable root node from a green tree.
    pub fn new_root_mut(green: GreenNode) -> SyntaxNode {
        let green = GreenNode::into_raw(green);
        let green = Green::Node {
            ptr: Cell::new(green),
        };
        SyntaxNode {
            ptr: NodeData::new(None, 0, 0, green, true),
        }
    }

    /// Creates a new child node with the given green data and parent.
    pub(super) fn new_child(
        green: &GreenNodeData,
        parent: SyntaxNode,
        index: u32,
        offset: u32,
    ) -> SyntaxNode {
        let mutable = parent.data().mutable;
        let green = Green::Node {
            ptr: Cell::new(green.into()),
        };
        SyntaxNode {
            ptr: NodeData::new(Some(parent), index, offset, green, mutable),
        }
    }

    /// Returns whether this node is part of a mutable tree.
    pub fn is_mutable(&self) -> bool {
        self.data().mutable
    }

    /// Creates a mutable copy of the entire tree containing this node.
    pub fn clone_for_update(&self) -> SyntaxNode {
        assert!(!self.data().mutable);
        match self.parent() {
            Some(parent) => {
                let parent = parent.clone_for_update();
                SyntaxNode::new_child(self.green_ref(), parent, self.data().index(), self.offset())
            }
            None => SyntaxNode::new_root_mut(self.green_ref().to_owned()),
        }
    }

    /// Creates a new independent tree rooted at this node.
    pub fn clone_subtree(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green().into())
    }

    /// Returns a reference to the underlying NodeData.
    #[inline]
    pub(super) fn data(&self) -> &NodeData {
        unsafe { self.ptr.as_ref() }
    }

    /// Returns true if this node's pointer can be safely taken.
    #[inline]
    pub(super) fn can_take_ptr(&self) -> bool {
        self.data().rc.get() == 1 && !self.data().mutable
    }

    /// Takes ownership of the underlying pointer (for optimization).
    #[inline]
    pub(super) fn take_ptr(self) -> ptr::NonNull<NodeData> {
        assert!(self.can_take_ptr());
        let ret = self.ptr;
        // don't change the refcount when self gets dropped
        std::mem::forget(self);
        ret
    }

    /// Replaces this node with a new one, returning the updated root.
    pub fn replace_with(&self, replacement: GreenNode) -> GreenNode {
        assert_eq!(self.kind(), replacement.kind());
        match &self.parent() {
            None => replacement,
            Some(parent) => {
                let new_parent = parent
                    .green_ref()
                    .replace_child(self.data().index() as usize, replacement.into());
                parent.replace_with(new_parent)
            }
        }
    }

    /// Returns the syntax kind of this node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data().kind()
    }

    /// Returns the absolute byte offset of this node in the text.
    #[inline]
    pub(super) fn offset(&self) -> u32 {
        self.data().offset()
    }

    /// Returns the text range (excluding trivia) of this node.
    #[inline]
    pub fn span(&self) -> Range<u32> {
        self.data().span()
    }

    /// Returns the full text range (including trivia) of this node.
    #[inline]
    pub fn full_span(&self) -> Range<u32> {
        self.data().full_span()
    }

    /// Returns the index of this node among its siblings.
    #[inline]
    pub fn index(&self) -> usize {
        self.data().index() as usize
    }

    #[inline]
    pub fn full_text(&self) -> crate::SyntaxText {
        crate::SyntaxText::new(self.clone())
    }

    /// Returns the underlying green node data.
    #[inline]
    pub fn green(&self) -> Cow<'_, GreenNodeData> {
        let green_ref = self.green_ref();
        match self.data().mutable {
            false => Cow::Borrowed(green_ref),
            true => Cow::Owned(green_ref.to_owned()),
        }
    }

    /// Returns a reference to the underlying green node data.
    #[inline]
    pub(super) fn green_ref(&self) -> &GreenNodeData {
        self.data().green().into_node().unwrap()
    }

    /// Returns the parent node containing this node.
    #[inline]
    pub fn parent(&self) -> Option<SyntaxNode> {
        self.data().parent_node()
    }

    /// Returns an iterator over all ancestor nodes.
    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        iter::successors(Some(self.clone()), SyntaxNode::parent)
    }

    /// Returns an iterator over direct child nodes.
    #[inline]
    pub fn children(&self) -> SyntaxNodeChildren {
        SyntaxNodeChildren::new(self.clone())
    }

    /// Returns an iterator over direct child elements (nodes and tokens).
    #[inline]
    pub fn children_with_tokens(&self) -> SyntaxElementChildren {
        SyntaxElementChildren::new(self.clone())
    }

    /// Returns the first child node.
    pub fn first_child(&self) -> Option<SyntaxNode> {
        self.green_ref()
            .children()
            .raw
            .enumerate()
            .find_map(|(index, child)| {
                child.as_ref().into_node().map(|green| {
                    SyntaxNode::new_child(
                        green,
                        self.clone(),
                        index as u32,
                        self.offset() + child.rel_offset(),
                    )
                })
            })
    }

    /// Returns the first child node matching the given kind predicate.
    pub fn first_child_by_kind(&self, matcher: &impl Fn(SyntaxKind) -> bool) -> Option<SyntaxNode> {
        self.green_ref()
            .children()
            .raw
            .enumerate()
            .find_map(|(index, child)| {
                if !matcher(child.as_ref().kind()) {
                    return None;
                }
                child.as_ref().into_node().map(|green| {
                    SyntaxNode::new_child(
                        green,
                        self.clone(),
                        index as u32,
                        self.offset() + child.rel_offset(),
                    )
                })
            })
    }

    /// Returns the last child node.
    pub fn last_child(&self) -> Option<SyntaxNode> {
        self.green_ref()
            .children()
            .raw
            .enumerate()
            .rev()
            .find_map(|(index, child)| {
                child.as_ref().into_node().map(|green| {
                    SyntaxNode::new_child(
                        green,
                        self.clone(),
                        index as u32,
                        self.offset() + child.rel_offset(),
                    )
                })
            })
    }

    /// Returns the first child element (node or token).
    pub fn first_child_or_token(&self) -> Option<SyntaxElement> {
        self.green_ref().children().raw.next().map(|child| {
            SyntaxElement::new(
                child.as_ref(),
                self.clone(),
                0,
                self.offset() + child.rel_offset(),
            )
        })
    }

    /// Returns the first child element matching the given kind predicate.
    pub fn first_child_or_token_by_kind(
        &self,
        matcher: &impl Fn(SyntaxKind) -> bool,
    ) -> Option<SyntaxElement> {
        self.green_ref()
            .children()
            .raw
            .enumerate()
            .find_map(|(index, child)| {
                if !matcher(child.as_ref().kind()) {
                    return None;
                }
                Some(SyntaxElement::new(
                    child.as_ref(),
                    self.clone(),
                    index as u32,
                    self.offset() + child.rel_offset(),
                ))
            })
    }

    /// Returns the last child element (node or token).
    pub fn last_child_or_token(&self) -> Option<SyntaxElement> {
        self.green_ref()
            .children()
            .raw
            .enumerate()
            .next_back()
            .map(|(index, child)| {
                SyntaxElement::new(
                    child.as_ref(),
                    self.clone(),
                    index as u32,
                    self.offset() + child.rel_offset(),
                )
            })
    }

    // if possible (i.e. unshared), consume self and advance it to point to the next sibling
    // this way, we can reuse the previously allocated buffer
    pub fn to_next_sibling(self) -> Option<SyntaxNode> {
        if !self.can_take_ptr() {
            // cannot mutate in-place
            return self.next_sibling();
        }

        let mut ptr = self.take_ptr();
        let data = unsafe { ptr.as_mut() };
        assert!(data.rc.get() == 1);

        let parent = data.parent_node()?;
        let parent_offset = parent.offset();
        let siblings = parent.green_ref().children().raw.enumerate();
        let index = data.index() as usize;

        siblings
            .skip(index + 1)
            .find_map(|(index, child)| {
                child
                    .as_ref()
                    .into_node()
                    .map(|green| (green, index as u32, child.rel_offset()))
            })
            .map(|(green, index, rel_offset)| {
                data.index.set(index);
                data.offset = parent_offset + rel_offset;
                data.green = Green::Node {
                    ptr: Cell::new(green.into()),
                };
                SyntaxNode { ptr }
            })
            .or_else(|| {
                data.dec_rc();
                unsafe { free(ptr) };
                None
            })
    }

    /// Returns the next sibling node.
    pub fn next_sibling(&self) -> Option<SyntaxNode> {
        self.data().next_sibling()
    }

    /// Returns the next sibling node matching the given kind predicate.
    pub fn next_sibling_by_kind(
        &self,
        matcher: &impl Fn(SyntaxKind) -> bool,
    ) -> Option<SyntaxNode> {
        self.data().next_sibling_by_kind(matcher)
    }

    /// Returns the previous sibling node.
    pub fn prev_sibling(&self) -> Option<SyntaxNode> {
        self.data().prev_sibling()
    }

    /// Returns the next sibling element (node or token).
    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.data().next_sibling_or_token()
    }

    /// Returns the next sibling element matching the given kind predicate.
    pub fn next_sibling_or_token_by_kind(
        &self,
        matcher: &impl Fn(SyntaxKind) -> bool,
    ) -> Option<SyntaxElement> {
        self.data().next_sibling_or_token_by_kind(matcher)
    }

    /// Returns the previous sibling element (node or token).
    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.data().prev_sibling_or_token()
    }

    /// Returns the first token within this node's subtree.
    pub fn first_token(&self) -> Option<SyntaxToken> {
        self.first_child_or_token()?.first_token()
    }

    /// Returns the last token within this node's subtree.
    pub fn last_token(&self) -> Option<SyntaxToken> {
        self.last_child_or_token()?.last_token()
    }

    /// Returns an iterator over sibling nodes in the given direction.
    #[inline]
    pub fn siblings(&self, direction: Direction) -> impl Iterator<Item = SyntaxNode> + use<> {
        iter::successors(Some(self.clone()), move |node| match direction {
            Direction::Next => node.next_sibling(),
            Direction::Prev => node.prev_sibling(),
        })
    }

    #[inline]
    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElement> + use<> {
        let me: SyntaxElement = self.clone().into();
        iter::successors(Some(me), move |el| match direction {
            Direction::Next => el.next_sibling_or_token(),
            Direction::Prev => el.prev_sibling_or_token(),
        })
    }

    #[inline]
    pub fn descendants(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        self.preorder().filter_map(|event| match event {
            WalkEvent::Enter(node) => Some(node),
            WalkEvent::Leave(_) => None,
        })
    }

    #[inline]
    pub fn descendants_with_tokens(&self) -> impl Iterator<Item = SyntaxElement> + use<> {
        self.preorder_with_tokens().filter_map(|event| match event {
            WalkEvent::Enter(it) => Some(it),
            WalkEvent::Leave(_) => None,
        })
    }

    /// Returns a preorder iterator over child nodes only.
    #[inline]
    pub fn preorder(&self) -> Preorder {
        Preorder::new(self.clone())
    }

    /// Returns a preorder iterator over all elements (nodes and tokens).
    #[inline]
    pub fn preorder_with_tokens(&self) -> PreorderWithTokens {
        PreorderWithTokens::new(self.clone())
    }

    pub fn token_at_offset(&self, offset: u32) -> TokenAtOffset<SyntaxToken> {
        // TODO: this could be faster if we first drill-down to node, and only
        // then switch to token search. We should also replace explicit
        // recursion with a loop.
        let range = self.full_span();
        assert!(
            range.start <= offset && offset <= range.end,
            "Bad offset: range {:?} offset {:?}",
            range,
            offset
        );
        if range.is_empty() {
            return TokenAtOffset::None;
        }

        let mut children = self.children_with_tokens().filter(|child| {
            let child_range = child.full_span();
            !child_range.is_empty() && (child_range.start <= offset && offset <= child_range.end)
        });

        let left = children.next().unwrap();
        let right = children.next();
        assert!(children.next().is_none());

        if let Some(right) = right {
            match (left.token_at_offset(offset), right.token_at_offset(offset)) {
                (TokenAtOffset::Single(left), TokenAtOffset::Single(right)) => {
                    TokenAtOffset::Between(left, right)
                }
                _ => unreachable!(),
            }
        } else {
            left.token_at_offset(offset)
        }
    }

    pub fn covering_element(&self, range: Range<u32>) -> SyntaxElement {
        let mut res: SyntaxElement = self.clone().into();
        loop {
            assert!(
                res.full_span().start <= range.start && res.full_span().end >= range.end,
                "Bad range: node range {:?}, range {:?}",
                res.full_span(),
                range,
            );

            res = match &res {
                NodeOrToken::Token(_) => return res,
                NodeOrToken::Node(node) => match node.child_or_token_at_range(range.clone()) {
                    Some(it) => it,
                    None => return res,
                },
            };
        }
    }

    pub fn child_or_token_at_range(&self, range: Range<u32>) -> Option<SyntaxElement> {
        let rel_range = (range.start - self.offset())..(range.end - self.offset());
        self.green_ref()
            .child_at_range(rel_range)
            .map(|(index, rel_offset, green)| {
                SyntaxElement::new(
                    green,
                    self.clone(),
                    index as u32,
                    self.offset() + rel_offset,
                )
            })
    }

    pub fn splice_children<I: IntoIterator<Item = SyntaxElement>>(
        &self,
        to_delete: Range<usize>,
        to_insert: I,
    ) {
        assert!(self.data().mutable, "immutable tree: {}", self);
        for (i, child) in self.children_with_tokens().enumerate() {
            if to_delete.contains(&i) {
                child.detach();
            }
        }
        let mut index = to_delete.start;
        for child in to_insert {
            self.attach_child(index, child);
            index += 1;
        }
    }

    pub fn detach(&self) {
        assert!(self.data().mutable, "immutable tree: {}", self);
        self.data().detach()
    }

    fn attach_child(&self, index: usize, child: SyntaxElement) {
        assert!(self.data().mutable, "immutable tree: {}", self);
        child.detach();
        let data = match &child {
            NodeOrToken::Node(it) => it.data(),
            NodeOrToken::Token(it) => it.data(),
        };
        self.data().attach_child(index, data)
    }
}

impl Clone for SyntaxNode {
    #[inline]
    fn clone(&self) -> Self {
        self.data().inc_rc();
        SyntaxNode { ptr: self.ptr }
    }
}

impl Drop for SyntaxNode {
    #[inline]
    fn drop(&mut self) {
        if self.data().dec_rc() {
            unsafe { free(self.ptr) }
        }
    }
}

// Identity semantics for hash & eq
impl PartialEq for SyntaxNode {
    #[inline]
    fn eq(&self, other: &SyntaxNode) -> bool {
        self.data().key() == other.data().key()
    }
}

impl Eq for SyntaxNode {}

impl Hash for SyntaxNode {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data().key().hash(state);
    }
}

impl fmt::Debug for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxNode")
            .field("kind", &self.kind())
            .field("full_span", &self.full_span())
            .finish()
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.preorder_with_tokens()
            .filter_map(|event| match event {
                WalkEvent::Enter(NodeOrToken::Token(token)) => Some(token),
                _ => None,
            })
            .try_for_each(|it| fmt::Display::fmt(&it, f))
    }
}
