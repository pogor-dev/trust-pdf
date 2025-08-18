//! Implementation of the cursors -- API for convenient access to syntax trees.
//!
//! Functional programmers will recognize that this module implements a zipper
//! for a purely functional (green) tree.
//!
//! A cursor node (`SyntaxNode`) points to a `GreenNode` and a parent
//! `SyntaxNode`. This allows cursor to provide iteration over both ancestors
//! and descendants, as well as a cheep access to absolute offset of the node in
//! file.
//!
//! By default `SyntaxNode`s are immutable, but you can get a mutable copy of
//! the tree by calling `clone_for_update`. Mutation is based on interior
//! mutability and doesn't need `&mut`. You can have two `SyntaxNode`s pointing
//! at different parts of the same tree; mutations via the first node will be
//! reflected in the other.

// Implementation notes:
//
// The implementation is utterly and horribly unsafe. This whole module is an
// unsafety boundary. It is believed that the API here is, in principle, sound,
// but the implementation might have bugs.
//
// The core type is `NodeData` -- a heap-allocated reference counted object,
// which points to a green node or a green token, and to the parent `NodeData`.
// Publicly-exposed `SyntaxNode` and `SyntaxToken` own a reference to
// `NodeData`.
//
// `NodeData`s are transient, and are created and destroyed during tree
// traversals. In general, only currently referenced nodes and their ancestors
// are alive at any given moment.
//
// More specifically, `NodeData`'s ref count is equal to the number of
// outstanding `SyntaxNode` and `SyntaxToken` plus the number of children with
// non-zero ref counts. For example, if the user has only a single `SyntaxNode`
// pointing somewhere in the middle of the tree, then all `NodeData` on the path
// from that point towards the root have ref count equal to one.
//
// `NodeData` which doesn't have a parent (is a root) owns the corresponding
// green node or token, and is responsible for freeing it.
//
// That's mostly it for the immutable subset of the API. Mutation is fun though,
// you'll like it!
//
// Mutability is a run-time property of a tree of `NodeData`. The whole tree is
// either mutable or immutable. `clone_for_update` clones the whole tree of
// `NodeData`s, making it mutable (note that the green tree is re-used).
//
// If the tree is mutable, then all live `NodeData` are additionally liked to
// each other via intrusive liked lists. Specifically, there are two pointers to
// siblings, as well as a pointer to the first child. Note that only live nodes
// are considered. If the user only has `SyntaxNode`s for  the first and last
// children of some particular node, then their `NodeData` will point at each
// other.
//
// The links are used to propagate mutations across the tree. Specifically, each
// `NodeData` remembers it's index in parent. When the node is detached from or
// attached to the tree, we need to adjust the indices of all subsequent
// siblings. That's what makes the `for c in node.children() { c.detach() }`
// pattern work despite the apparent iterator invalidation.
//
// This code is encapsulated into the sorted linked list (`sll`) module.
//
// The actual mutation consist of functionally "mutating" (creating a
// structurally shared copy) the green node, and then re-spinning the tree. This
// is a delicate process: `NodeData` point directly to the green nodes, so we
// must make sure that those nodes don't move. Additionally, during mutation a
// node might become or might stop being a root, so we must take care to not
// double free / leak its green node.
//
// Because we can change green nodes using only shared references, handing out
// references into green nodes in the public API would be unsound. We don't do
// that, but we do use such references internally a lot. Additionally, for
// tokens the underlying green token actually is immutable, so we can, and do
// return `&str`.
//
// Invariants [must not leak outside of the module]:
//    - Mutability is the property of the whole tree. Intermixing elements that
//      differ in mutability is not allowed.
//    - Mutability property is persistent.
//    - References to the green elements' data are not exposed into public API
//      when the tree is mutable.
//    - TBD

use std::ptr;

use crate::{
    GreenNode, GreenToken,
    cursor::{green::Green, node_data::NodeData},
    sll,
};

mod element;
mod element_children;
mod green;
pub(super) mod node;
mod node_children;
mod node_data;
mod preorder;
mod preorder_with_tokens;
pub(super) mod token;
pub(super) mod trivia;

#[cfg(test)]
#[path = "cursor/tests/lib.rs"]
mod tests;

pub(crate) use self::{
    element::SyntaxElement, element_children::SyntaxElementChildren, node::SyntaxNode,
    node_children::SyntaxNodeChildren, preorder::Preorder,
    preorder_with_tokens::PreorderWithTokens, token::SyntaxToken, trivia::SyntaxTrivia,
};

#[inline(never)]
unsafe fn free(mut data: ptr::NonNull<NodeData>) {
    unsafe {
        loop {
            debug_assert_eq!(data.as_ref().rc.get(), 0);
            debug_assert!(data.as_ref().first.get().is_null());
            let node = Box::from_raw(data.as_ptr());
            match node.parent.take() {
                Some(parent) => {
                    debug_assert!(parent.as_ref().rc.get() > 0);
                    if node.mutable {
                        sll::unlink(&parent.as_ref().first, &*node)
                    }
                    if parent.as_ref().dec_rc() {
                        data = parent;
                    } else {
                        break;
                    }
                }
                None => {
                    match &node.green {
                        Green::Node { ptr } => {
                            let _ = GreenNode::from_raw(ptr.get());
                        }
                        Green::Token { ptr } => {
                            let _ = GreenToken::from_raw(*ptr);
                        }
                    }
                    break;
                }
            }
        }
    }
}
