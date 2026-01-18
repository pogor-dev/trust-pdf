use crate::{GreenNode, GreenToken, GreenTrivia, SyntaxKind};

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum GreenElement {
    Node(GreenNode),
    Token(GreenToken),
    Trivia(GreenTrivia),
}

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
