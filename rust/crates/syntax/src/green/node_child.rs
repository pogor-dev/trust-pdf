use std::ops::Range;

use crate::{
    NodeOrToken,
    green::{element::GreenElementRef, node::GreenNode, token::GreenToken},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(crate) enum GreenChild {
    Node { rel_offset: u32, node: GreenNode },
    Token { rel_offset: u32, token: GreenToken },
}

impl GreenChild {
    #[inline]
    pub(crate) fn as_ref(&self) -> GreenElementRef {
        match self {
            GreenChild::Node { node, .. } => NodeOrToken::Node(node),
            GreenChild::Token { token, .. } => NodeOrToken::Token(token),
        }
    }

    #[inline]
    pub(crate) fn rel_offset(&self) -> u32 {
        match self {
            GreenChild::Node { rel_offset, .. } | GreenChild::Token { rel_offset, .. } => {
                *rel_offset
            }
        }
    }

    #[inline]
    pub(crate) fn rel_range(&self) -> Range<u32> {
        let len = self.as_ref().full_width();
        let rel_offset = self.rel_offset();
        rel_offset..(rel_offset + len)
    }
}
