use crate::{
    GreenNodeData, GreenToken, GreenTokenElement, GreenTokenElementRef, GreenTokenWithFloatValue, GreenTokenWithIntValue, GreenTokenWithStringValue,
    GreenTrivia, GreenTriviaData, SyntaxKind, green::NodeOrTokenOrTrivia,
};

pub type GreenNodeElement = NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia>;
pub(crate) type GreenNodeElementRef<'a> = NodeOrTokenOrTrivia<&'a GreenNodeData, &'a GreenTokenElementRef<'a>, &'a GreenTriviaData>;

impl GreenNodeElement {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenNodeElement::Node(n) => n.kind(),
            GreenNodeElement::Token(t) => t.kind(),
            GreenNodeElement::Trivia(tr) => tr.kind(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenNodeElement::Node(n) => n.width(),
            GreenNodeElement::Token(t) => t.width(),
            GreenNodeElement::Trivia(tr) => tr.width().into(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            GreenNodeElement::Node(n) => n.full_width(),
            GreenNodeElement::Token(t) => t.full_width(),
            GreenNodeElement::Trivia(tr) => tr.width().into(),
        }
    }
}

impl From<GreenToken> for GreenNodeElement {
    #[inline]
    fn from(token: GreenToken) -> GreenNodeElement {
        NodeOrTokenOrTrivia::Token(GreenTokenElement::Token(token))
    }
}

impl From<GreenTokenWithIntValue> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenWithIntValue) -> GreenNodeElement {
        NodeOrTokenOrTrivia::Token(GreenTokenElement::TokenWithIntValue(token))
    }
}

impl From<GreenTokenWithFloatValue> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenWithFloatValue) -> GreenNodeElement {
        NodeOrTokenOrTrivia::Token(GreenTokenElement::TokenWithFloatValue(token))
    }
}

impl From<GreenTokenWithStringValue> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenWithStringValue) -> GreenNodeElement {
        NodeOrTokenOrTrivia::Token(GreenTokenElement::TokenWithStringValue(token))
    }
}

impl From<GreenNode> for GreenNodeElement {
    #[inline]
    fn from(node: GreenNode) -> GreenNodeElement {
        NodeOrTokenOrTrivia::Node(node)
    }
}

impl From<GreenTrivia> for GreenNodeElement {
    #[inline]
    fn from(trivia: GreenTrivia) -> GreenNodeElement {
        NodeOrTokenOrTrivia::Trivia(trivia)
    }
}
