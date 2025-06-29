use std::ptr;

use crate::{
    cursor::{element::GreenElement, green_element::GreenElementRef},
    green::{node_data::GreenNodeData, token_data::GreenTokenData},
    utility_types::node_or_token::NodeOrToken,
};

/// Child SyntaxNodes use "unsafe" weak pointers to refer to their green node.
/// Unlike the safe [std::sync::Weak] these are just a raw pointer: the
/// corresponding [ThinArc](crate::arc::ThinArc) doesn't keep a counter of
/// outstanding weak references or defer the release of the underlying memory
/// until the last `Weak` is dropped. On the other hand, a weak reference to a
/// released green node points to deallocated memory and it is undefined
/// behavior to dereference it, but in the context of `NodeData` this is
/// statically known to never happen
#[derive(Debug, Clone)]
pub(crate) enum WeakGreenElement {
    Node { ptr: ptr::NonNull<GreenNodeData> },
    Token { ptr: ptr::NonNull<GreenTokenData> },
}

impl WeakGreenElement {
    pub(crate) fn new(green: GreenElementRef) -> Self {
        match green {
            NodeOrToken::Node(ptr) => Self::Node {
                ptr: ptr::NonNull::from(ptr),
            },
            NodeOrToken::Token(ptr) => Self::Token {
                ptr: ptr::NonNull::from(ptr),
            },
        }
    }

    pub(crate) fn as_deref(&self) -> GreenElementRef {
        match self {
            WeakGreenElement::Node { ptr } => GreenElementRef::Node(unsafe { ptr.as_ref() }),
            WeakGreenElement::Token { ptr } => GreenElementRef::Token(unsafe { ptr.as_ref() }),
        }
    }

    pub(crate) fn to_owned(&self) -> GreenElement {
        match self {
            WeakGreenElement::Node { ptr } => {
                GreenElement::Node(unsafe { ptr.as_ref().to_owned() })
            }
            WeakGreenElement::Token { ptr } => {
                GreenElement::Token(unsafe { ptr.as_ref().to_owned() })
            }
        }
    }
}
