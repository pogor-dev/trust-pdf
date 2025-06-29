use std::borrow::Cow;

use crate::{
    green::{kind::RawSyntaxKind, node::GreenNode, node_data::GreenNodeData, token::GreenToken},
    utility_types::node_or_token::NodeOrToken,
};

pub(crate) type GreenElement = NodeOrToken<GreenNode, GreenToken>;

impl GreenElement {
    /// Returns kind of this element.
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        match self {
            NodeOrToken::Node(node) => node.kind(),
            NodeOrToken::Token(token) => token.kind(),
        }
    }

    /// Returns the length of the text covered by this element.
    #[inline]
    pub fn text_len(&self) -> u64 {
        match self {
            NodeOrToken::Token(token) => token.text_len(),
            NodeOrToken::Node(node) => node.text_len(),
        }
    }
}

impl From<GreenNode> for GreenElement {
    #[inline]
    fn from(node: GreenNode) -> GreenElement {
        NodeOrToken::Node(node)
    }
}

impl From<GreenToken> for GreenElement {
    #[inline]
    fn from(token: GreenToken) -> GreenElement {
        NodeOrToken::Token(token)
    }
}

impl From<Cow<'_, GreenNodeData>> for GreenElement {
    #[inline]
    fn from(cow: Cow<'_, GreenNodeData>) -> Self {
        NodeOrToken::Node(cow.into_owned())
    }
}
