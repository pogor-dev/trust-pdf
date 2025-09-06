use std::ptr;

use crate::{
    NodeOrToken, SyntaxKind,
    green::GreenElementRef,
    red::{NodeData, SyntaxNode, SyntaxToken},
};

pub type SyntaxElement = NodeOrToken<SyntaxNode, SyntaxToken>;

impl SyntaxElement {
    fn new(element: GreenElementRef<'_>, parent: SyntaxNode, index: u32, offset: u64) -> SyntaxElement {
        match element {
            NodeOrToken::Node(node) => SyntaxNode::new_child(node, parent, index, offset).into(),
            NodeOrToken::Token(token) => SyntaxToken::new(token, parent, index, offset).into(),
        }
    }

    #[inline]
    pub fn index(&self) -> u32 {
        match self {
            NodeOrToken::Node(it) => it.index(),
            NodeOrToken::Token(it) => it.index(),
        }
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    #[inline]
    pub fn parent(&self) -> Option<SyntaxNode> {
        match self {
            NodeOrToken::Node(it) => it.parent(),
            NodeOrToken::Token(it) => it.parent(),
        }
    }

    fn can_take_ptr(&self) -> bool {
        match self {
            NodeOrToken::Node(it) => it.can_take_ptr(),
            NodeOrToken::Token(it) => it.can_take_ptr(),
        }
    }

    fn take_ptr(self) -> ptr::NonNull<NodeData> {
        match self {
            NodeOrToken::Node(it) => it.take_ptr(),
            NodeOrToken::Token(it) => it.take_ptr(),
        }
    }
}

impl From<SyntaxNode> for SyntaxElement {
    #[inline]
    fn from(node: SyntaxNode) -> SyntaxElement {
        NodeOrToken::Node(node)
    }
}

impl From<SyntaxToken> for SyntaxElement {
    #[inline]
    fn from(token: SyntaxToken) -> SyntaxElement {
        NodeOrToken::Token(token)
    }
}
