use crate::{GreenNode, GreenToken, NodeOrToken, SyntaxKind};

pub(crate) type GreenElement = NodeOrToken<GreenNode, GreenToken>;

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

impl GreenElement {
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
