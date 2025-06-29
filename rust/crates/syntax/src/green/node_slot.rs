use std::fmt::{self, Formatter};

use crate::green::{node::GreenNode, token::GreenToken};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Slot {
    Node {
        rel_offset: u32,
        node: GreenNode,
    },
    Token {
        rel_offset: u32,
        token: GreenToken,
    },
    /// An empty slot for a child that was missing in the source because:
    /// * it's an optional child which is missing for this node
    /// * it's a mandatory child but it's missing because of a syntax error
    Empty {
        rel_offset: u32,
    },
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Slot::Empty { .. } => write!(f, "∅"),
            Slot::Node { node, .. } => std::fmt::Display::fmt(node, f),
            Slot::Token { token, .. } => std::fmt::Display::fmt(token, f),
        }
    }
}
