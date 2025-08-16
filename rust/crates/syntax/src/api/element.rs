use std::{iter, ops::Range};

use crate::{Language, NodeOrToken, SyntaxNode, SyntaxToken, cursor};

pub type SyntaxElement<L> = NodeOrToken<SyntaxNode<L>, SyntaxToken<L>>;

impl<L: Language> SyntaxElement<L> {
    pub fn span(&self) -> Range<u32> {
        match self {
            NodeOrToken::Node(it) => it.span(),
            NodeOrToken::Token(it) => it.span(),
        }
    }

    pub fn full_span(&self) -> Range<u32> {
        match self {
            NodeOrToken::Node(it) => it.full_span(),
            NodeOrToken::Token(it) => it.full_span(),
        }
    }

    pub fn index(&self) -> usize {
        match self {
            NodeOrToken::Node(it) => it.index(),
            NodeOrToken::Token(it) => it.index(),
        }
    }

    pub fn kind(&self) -> L::Kind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    pub fn parent(&self) -> Option<SyntaxNode<L>> {
        match self {
            NodeOrToken::Node(it) => it.parent(),
            NodeOrToken::Token(it) => it.parent(),
        }
    }

    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode<L>> + use<L> {
        let first = match self {
            NodeOrToken::Node(it) => Some(it.clone()),
            NodeOrToken::Token(it) => it.parent(),
        };
        iter::successors(first, SyntaxNode::parent)
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        match self {
            NodeOrToken::Node(it) => it.next_sibling_or_token(),
            NodeOrToken::Token(it) => it.next_sibling_or_token(),
        }
    }
    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        match self {
            NodeOrToken::Node(it) => it.prev_sibling_or_token(),
            NodeOrToken::Token(it) => it.prev_sibling_or_token(),
        }
    }
    pub fn detach(&self) {
        match self {
            NodeOrToken::Node(it) => it.detach(),
            NodeOrToken::Token(it) => it.detach(),
        }
    }
}

impl<L: Language> From<cursor::SyntaxElement> for SyntaxElement<L> {
    fn from(raw: cursor::SyntaxElement) -> SyntaxElement<L> {
        match raw {
            NodeOrToken::Node(it) => NodeOrToken::Node(it.into()),
            NodeOrToken::Token(it) => NodeOrToken::Token(it.into()),
        }
    }
}
