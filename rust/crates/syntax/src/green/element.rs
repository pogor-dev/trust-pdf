use std::borrow::Cow;

use crate::{
    NodeOrToken, SyntaxKind,
    green::{node::GreenNode, node_data::GreenNodeData, token::GreenToken},
};

pub(super) type GreenElement = NodeOrToken<GreenNode, GreenNode>;

impl GreenElement {
    /// Returns kind of this element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.as_deref().kind()
    }

    /// Returns the length of the text covered by this element.
    #[inline]
    pub fn width(&self) -> u32 {
        self.as_deref().width()
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        self.as_deref().full_width()
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
