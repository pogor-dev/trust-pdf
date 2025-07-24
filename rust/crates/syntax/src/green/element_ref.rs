use crate::{
    NodeOrToken, SyntaxKind,
    green::{
        element::GreenElement, node::GreenNode, node_data::GreenNodeData, token::GreenToken,
        token_data::GreenTokenData,
    },
};

pub(crate) type GreenElementRef<'a> = NodeOrToken<&'a GreenNodeData, &'a GreenTokenData>;

impl GreenElementRef<'_> {
    /// Returns kind of this element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    /// Returns the length of the text covered by this element.
    #[inline]
    pub fn width(self) -> u32 {
        match self {
            NodeOrToken::Node(it) => it.width(),
            NodeOrToken::Token(it) => it.width(),
        }
    }

    #[inline]
    pub fn full_width(self) -> u32 {
        match self {
            NodeOrToken::Node(it) => it.full_width(),
            NodeOrToken::Token(it) => it.full_width(),
        }
    }
}

impl<'a> From<&'a GreenNode> for GreenElementRef<'a> {
    #[inline]
    fn from(node: &'a GreenNode) -> GreenElementRef<'a> {
        NodeOrToken::Node(node)
    }
}

impl<'a> From<&'a GreenToken> for GreenElementRef<'a> {
    #[inline]
    fn from(token: &'a GreenToken) -> GreenElementRef<'a> {
        NodeOrToken::Token(token)
    }
}

impl GreenElementRef<'_> {
    pub fn to_owned(self) -> GreenElement {
        match self {
            NodeOrToken::Node(it) => NodeOrToken::Node(it.to_owned()),
            NodeOrToken::Token(it) => NodeOrToken::Token(it.to_owned()),
        }
    }
}
