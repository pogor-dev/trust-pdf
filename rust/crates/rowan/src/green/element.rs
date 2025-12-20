use crate::{
    GreenNode, GreenToken, NodeOrToken, SyntaxKind,
    green::{node::GreenNodeInTree, token::GreenTokenInTree},
};

pub(crate) type GreenElementInTree = NodeOrToken<GreenNodeInTree, GreenTokenInTree>;

impl GreenElementInTree {
    #[inline]
    pub(crate) fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(node) => node.kind(),
            NodeOrToken::Token(token) => token.kind(),
        }
    }

    #[inline]
    pub(crate) fn full_width(&self) -> u32 {
        match self {
            NodeOrToken::Node(node) => node.full_width(),
            NodeOrToken::Token(token) => token.full_width(),
        }
    }
}

impl From<GreenNodeInTree> for GreenElementInTree {
    #[inline]
    fn from(node: GreenNodeInTree) -> Self {
        NodeOrToken::Node(node)
    }
}

impl From<GreenTokenInTree> for GreenElementInTree {
    #[inline]
    fn from(token: GreenTokenInTree) -> Self {
        NodeOrToken::Token(token)
    }
}

pub type GreenElement = NodeOrToken<GreenNode, GreenToken>;

impl GreenElement {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(node) => node.kind(),
            NodeOrToken::Token(token) => token.kind(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            NodeOrToken::Node(node) => node.full_width(),
            NodeOrToken::Token(token) => token.full_width(),
        }
    }
}

impl From<GreenNode> for GreenElement {
    #[inline]
    fn from(node: GreenNode) -> Self {
        NodeOrToken::Node(node)
    }
}

impl From<GreenToken> for GreenElement {
    #[inline]
    fn from(token: GreenToken) -> Self {
        NodeOrToken::Token(token)
    }
}
