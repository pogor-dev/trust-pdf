//! SyntaxToken - Cursor for individual tokens in the syntax tree.
//!
//! ```text
//!     📝 SyntaxToken
//!    ┌─────────────┐
//!    │ ┌─────────┐ │   Provides token access:
//!    │ │ "text"  │ │   • text() content
//!    │ │ [kind]  │ │   • kind() type  
//!    │ └─────────┘ │   • prev/next navigation
//!    └─────────────┘   • parent node access
//! ```

use std::{fmt, hash, iter, ops::Range, ptr};

use crate::{
    GreenNode, GreenToken, GreenTokenData, SyntaxKind,
    cursor::{Green, free, node::SyntaxNode, node_data::NodeData, element::SyntaxElement},
    utility_types::Direction,
};

#[derive(Debug)]
pub struct SyntaxToken {
    pub(super) ptr: ptr::NonNull<NodeData>,
}

impl SyntaxToken {
    /// Creates a new SyntaxToken with the given green data and parent.
    pub(super) fn new(
        green: &GreenTokenData,
        parent: SyntaxNode,
        index: u32,
        offset: u32,
    ) -> SyntaxToken {
        let mutable = parent.data().mutable;
        let green = Green::Token { ptr: green.into() };
        SyntaxToken {
            ptr: NodeData::new(Some(parent), index, offset, green, mutable),
        }
    }

    /// Returns a reference to the underlying NodeData.
    #[inline]
    pub(super) fn data(&self) -> &NodeData {
        unsafe { self.ptr.as_ref() }
    }

    /// Returns true if this token's pointer can be safely taken.
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

    /// Replaces this token with a new one, returning the updated parent node.
    pub fn replace_with(&self, replacement: GreenToken) -> GreenNode {
        assert_eq!(self.kind(), replacement.kind());
        let parent = self.parent().unwrap();
        let me: u32 = self.data().index();

        let new_parent = parent
            .green_ref()
            .replace_child(me as usize, replacement.into());
        parent.replace_with(new_parent)
    }

    /// Returns the syntax kind of this token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data().kind()
    }

    /// Returns the text range (excluding trivia) of this token.
    #[inline]
    pub fn span(&self) -> Range<u32> {
        self.data().span()
    }

    /// Returns the full text range (including trivia) of this token.
    #[inline]
    pub fn full_span(&self) -> Range<u32> {
        self.data().full_span()
    }

    /// Returns the index of this token among its siblings.
    #[inline]
    pub fn index(&self) -> usize {
        self.data().index() as usize
    }

    /// Returns the text content of this token (excluding trivia).
    #[inline]
    pub fn text(&self) -> &[u8] {
        match self.data().green().as_token() {
            Some(it) => it.text(),
            None => {
                debug_assert!(
                    false,
                    "corrupted tree: a node thinks it is a token: {:?}",
                    self.data().green().as_node().unwrap().to_string()
                );
                b""
            }
        }
    }

    /// Returns the full text content including leading/trailing trivia.
    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        match self.data().green().as_token() {
            Some(it) => it.full_text(),
            None => {
                debug_assert!(
                    false,
                    "corrupted tree: a node thinks it is a token: {:?}",
                    self.data().green().as_node().unwrap().to_string()
                );
                Vec::new()
            }
        }
    }

    /// Returns the width (byte length) of this token excluding trivia.
    #[inline]
    pub fn width(&self) -> u32 {
        match self.data().green().as_token() {
            Some(it) => it.width(),
            None => {
                debug_assert!(
                    false,
                    "corrupted tree: a node thinks it is a token: {:?}",
                    self.data().green().as_node().unwrap().to_string()
                );
                0
            }
        }
    }

    /// Returns the full width (byte length) including trivia.
    #[inline]
    pub fn full_width(&self) -> u32 {
        match self.data().green().as_token() {
            Some(it) => it.full_width(),
            None => {
                debug_assert!(
                    false,
                    "corrupted tree: a node thinks it is a token: {:?}",
                    self.data().green().as_node().unwrap().to_string()
                );
                0
            }
        }
    }

    /// Returns the underlying green token data.
    #[inline]
    pub fn green(&self) -> &GreenTokenData {
        self.data().green().into_token().unwrap()
    }

    /// Returns the parent node containing this token.
    #[inline]
    pub fn parent(&self) -> Option<SyntaxNode> {
        self.data().parent_node()
    }

    /// Returns an iterator over all ancestor nodes.
    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        std::iter::successors(self.parent(), SyntaxNode::parent)
    }

    /// Returns the next sibling node or token.
    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.data().next_sibling_or_token()
    }

    /// Returns the next sibling matching the given kind predicate.
    pub fn next_sibling_or_token_by_kind(
        &self,
        matcher: &impl Fn(SyntaxKind) -> bool,
    ) -> Option<SyntaxElement> {
        self.data().next_sibling_or_token_by_kind(matcher)
    }

    /// Returns the previous sibling node or token.
    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.data().prev_sibling_or_token()
    }

    /// Returns an iterator over siblings in the given direction.
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

    /// Returns the next token in the tree (depth-first order).
    pub fn next_token(&self) -> Option<SyntaxToken> {
        match self.next_sibling_or_token() {
            Some(element) => element.first_token(),
            None => self
                .ancestors()
                .find_map(|it| it.next_sibling_or_token())
                .and_then(|element| element.first_token()),
        }
    }

    /// Returns the previous token in the tree (reverse depth-first order).
    pub fn prev_token(&self) -> Option<SyntaxToken> {
        match self.prev_sibling_or_token() {
            Some(element) => element.last_token(),
            None => self
                .ancestors()
                .find_map(|it| it.prev_sibling_or_token())
                .and_then(|element| element.last_token()),
        }
    }

    /// Detaches this token from its parent (mutable trees only).
    pub fn detach(&self) {
        assert!(self.data().mutable, "immutable tree: {}", self);
        self.data().detach()
    }
}

impl Clone for SyntaxToken {
    #[inline]
    fn clone(&self) -> Self {
        self.data().inc_rc();
        SyntaxToken { ptr: self.ptr }
    }
}

impl Drop for SyntaxToken {
    #[inline]
    fn drop(&mut self) {
        if self.data().dec_rc() {
            unsafe { free(self.ptr) }
        }
    }
}

// Identity semantics for hash & eq
impl PartialEq for SyntaxToken {
    #[inline]
    fn eq(&self, other: &SyntaxToken) -> bool {
        self.data().key() == other.data().key()
    }
}

impl Eq for SyntaxToken {}

impl hash::Hash for SyntaxToken {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.data().key().hash(state);
    }
}

impl fmt::Display for SyntaxToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.full_text()))
    }
}
