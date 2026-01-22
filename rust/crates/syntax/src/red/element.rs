use std::ops;

use crate::SyntaxKind;

use super::{SyntaxNode, SyntaxToken, SyntaxTrivia};

/// Sum type over the three kinds of positioned syntax elements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxElement {
    Node(SyntaxNode),
    Token(SyntaxToken),
    Trivia(SyntaxTrivia),
}

impl SyntaxElement {
    /// Returns the position of this element.
    #[inline]
    pub fn position(&self) -> u32 {
        match self {
            SyntaxElement::Node(n) => n.position(),
            SyntaxElement::Token(t) => t.position(),
            SyntaxElement::Trivia(tr) => tr.position(),
        }
    }

    /// Returns the text range of this element.
    #[inline]
    pub fn full_span(&self) -> ops::Range<u32> {
        match self {
            SyntaxElement::Node(n) => n.full_span(),
            SyntaxElement::Token(t) => t.full_span(),
            SyntaxElement::Trivia(tr) => tr.full_span(),
        }
    }

    /// Returns the kind of this element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            SyntaxElement::Node(n) => n.kind(),
            SyntaxElement::Token(t) => t.kind(),
            SyntaxElement::Trivia(tr) => tr.kind(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{SyntaxKind, SyntaxNode, tree};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_element_kind() {
        let green = tree! {
            SyntaxKind::List => {
                (SyntaxKind::NumericLiteralToken, b"1"),
                (SyntaxKind::NumericLiteralToken, b" "),
                SyntaxKind::List => {
                    (SyntaxKind::NumericLiteralToken, b"2")
                }
            }
        };

        let root = SyntaxNode::new_root(green);
        let children: Vec<_> = root.children().collect();

        assert_eq!(children[0].kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(children[1].kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(children[2].kind(), SyntaxKind::List);
    }
}
