//! SyntaxElement - Unified cursor for nodes and tokens.
//!
//! ```text
//!     🔗 SyntaxElement
//!    ┌─────────────┐
//!    │  Node OR    │   Unified interface:
//!    │  Token      │   • pattern matching
//!    │ ┌─────────┐ │   • common operations
//!    │ │ Element │ │   • polymorphic access
//!    │ └─────────┘ │   
//!    └─────────────┘   
//! ```

use std::{cell::Cell, iter, ops::Range, ptr};

use crate::{
    NodeOrToken, SyntaxKind,
    cursor::{Green, NodeData, free, node::SyntaxNode, token::SyntaxToken},
    green::GreenElementRef,
    utility_types::TokenAtOffset,
};

pub type SyntaxElement = NodeOrToken<SyntaxNode, SyntaxToken>;

impl SyntaxElement {
    /// Creates a new SyntaxElement from a green element reference.
    pub(super) fn new(
        element: GreenElementRef<'_>,
        parent: SyntaxNode,
        index: u32,
        offset: usize,
    ) -> SyntaxElement {
        match element {
            NodeOrToken::Node(node) => SyntaxNode::new_child(node, parent, index, offset).into(),
            NodeOrToken::Token(token) => SyntaxToken::new(token, parent, index, offset).into(),
        }
    }

    /// Returns the text range (excluding trivia) of this element.
    #[inline]
    pub fn span(&self) -> Range<usize> {
        match self {
            NodeOrToken::Node(it) => it.span(),
            NodeOrToken::Token(it) => it.span(),
        }
    }

    /// Returns the full text range (including trivia) of this element.
    #[inline]
    pub fn full_span(&self) -> Range<usize> {
        match self {
            NodeOrToken::Node(it) => it.full_span(),
            NodeOrToken::Token(it) => it.full_span(),
        }
    }

    /// Returns the index of this element among its siblings.
    #[inline]
    pub fn index(&self) -> usize {
        match self {
            NodeOrToken::Node(it) => it.index(),
            NodeOrToken::Token(it) => it.index(),
        }
    }

    /// Returns the syntax kind of this element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    /// Returns the parent node containing this element.
    #[inline]
    pub fn parent(&self) -> Option<SyntaxNode> {
        match self {
            NodeOrToken::Node(it) => it.parent(),
            NodeOrToken::Token(it) => it.parent(),
        }
    }

    /// Returns an iterator over all ancestor nodes.
    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        let first = match self {
            NodeOrToken::Node(it) => Some(it.clone()),
            NodeOrToken::Token(it) => it.parent(),
        };
        iter::successors(first, SyntaxNode::parent)
    }

    /// Returns the first token within this element's subtree.
    pub fn first_token(&self) -> Option<SyntaxToken> {
        match self {
            NodeOrToken::Node(it) => it.first_token(),
            NodeOrToken::Token(it) => Some(it.clone()),
        }
    }

    /// Returns the last token within this element's subtree.
    pub fn last_token(&self) -> Option<SyntaxToken> {
        match self {
            NodeOrToken::Node(it) => it.last_token(),
            NodeOrToken::Token(it) => Some(it.clone()),
        }
    }

    /// Returns the next sibling element (node or token).
    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        match self {
            NodeOrToken::Node(it) => it.next_sibling_or_token(),
            NodeOrToken::Token(it) => it.next_sibling_or_token(),
        }
    }

    fn can_take_ptr(&self) -> bool {
        match self {
            NodeOrToken::Node(it) => it.can_take_ptr(),
            NodeOrToken::Token(it) => it.can_take_ptr(),
        }
    }

    fn take_ptr(self) -> ptr::NonNull<NodeData> {
        match self {
            NodeOrToken::Node(it) => it.take_ptr(),
            NodeOrToken::Token(it) => it.take_ptr(),
        }
    }

    // if possible (i.e. unshared), consume self and advance it to point to the next sibling
    // this way, we can reuse the previously allocated buffer
    pub fn to_next_sibling_or_token(self) -> Option<SyntaxElement> {
        if !self.can_take_ptr() {
            // cannot mutate in-place
            return self.next_sibling_or_token();
        }

        let mut ptr = self.take_ptr();
        let data = unsafe { ptr.as_mut() };

        let parent = data.parent_node()?;
        let parent_offset = parent.offset();
        let siblings = parent.green_ref().children().raw.enumerate();
        let index = data.index() as usize;

        siblings
            .skip(index + 1)
            .map(|(index, green)| {
                data.index.set(index as u32);
                data.offset = parent_offset + green.rel_offset();

                match green.as_ref() {
                    NodeOrToken::Node(node) => {
                        data.green = Green::Node {
                            ptr: Cell::new(node.into()),
                        };
                        Some(SyntaxElement::Node(SyntaxNode { ptr }))
                    }
                    NodeOrToken::Token(token) => {
                        data.green = Green::Token { ptr: token.into() };
                        Some(SyntaxElement::Token(SyntaxToken { ptr }))
                    }
                }
            })
            .next()
            .flatten()
            .or_else(|| {
                data.dec_rc();
                unsafe { free(ptr) };
                None
            })
    }

    pub fn next_sibling_or_token_by_kind(
        &self,
        matcher: &impl Fn(SyntaxKind) -> bool,
    ) -> Option<SyntaxElement> {
        match self {
            NodeOrToken::Node(it) => it.next_sibling_or_token_by_kind(matcher),
            NodeOrToken::Token(it) => it.next_sibling_or_token_by_kind(matcher),
        }
    }

    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        match self {
            NodeOrToken::Node(it) => it.prev_sibling_or_token(),
            NodeOrToken::Token(it) => it.prev_sibling_or_token(),
        }
    }

    /// Returns the token at the given byte offset within this element.
    pub(super) fn token_at_offset(&self, offset: usize) -> TokenAtOffset<SyntaxToken> {
        assert!(self.full_span().start <= offset && offset <= self.full_span().end);
        match self {
            NodeOrToken::Token(token) => TokenAtOffset::Single(token.clone()),
            NodeOrToken::Node(node) => node.token_at_offset(offset),
        }
    }

    pub fn detach(&self) {
        match self {
            NodeOrToken::Node(it) => it.detach(),
            NodeOrToken::Token(it) => it.detach(),
        }
    }
}

impl From<SyntaxNode> for SyntaxElement {
    #[inline]
    fn from(node: SyntaxNode) -> SyntaxElement {
        NodeOrToken::Node(node)
    }
}

impl From<SyntaxToken> for SyntaxElement {
    #[inline]
    fn from(token: SyntaxToken) -> SyntaxElement {
        NodeOrToken::Token(token)
    }
}
