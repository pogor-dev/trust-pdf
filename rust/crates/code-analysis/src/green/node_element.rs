use crate::{
    SyntaxKind,
    green::{GreenTokenElement, GreenTokenElementRef, NodeOrTokenOrTrivia},
};

pub type GreenElement = NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia>;
pub(crate) type GreenElementRef<'a> = NodeOrTokenOrTrivia<&'a GreenNodeData, &'a GreenTokenElementRef<'a>, &'a GreenTriviaData>;

impl GreenElement {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenElement::Node(n) => n.kind(),
            GreenElement::Token(t) => t.kind(),
            GreenElement::Trivia(tr) => tr.kind(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenElement::Node(n) => n.width(),
            GreenElement::Token(t) => t.width(),
            GreenElement::Trivia(tr) => tr.width(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            GreenElement::Node(n) => n.full_width(),
            GreenElement::Token(t) => t.full_width(),
            GreenElement::Trivia(tr) => tr.width(),
        }
    }
}

impl From<GreenToken> for GreenElement {
    #[inline]
    fn from(token: GreenToken) -> GreenElement {
        NodeOrTokenOrTrivia::Token(token)
    }
}

impl From<GreenNode> for GreenElement {
    #[inline]
    fn from(node: GreenNode) -> GreenElement {
        NodeOrTokenOrTrivia::Node(node)
    }
}

impl From<GreenTrivia> for GreenElement {
    #[inline]
    fn from(trivia: GreenTrivia) -> GreenElement {
        NodeOrTokenOrTrivia::Trivia(trivia)
    }
}
