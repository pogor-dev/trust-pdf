use std::borrow::Cow;

use crate::{
    GreenNode,
    green::{NodeOrToken, Trivia},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EitherNodeOrToken<Node, Token> {
    /// Leaf token (keywords, literals, operators, names)
    Token(Token),
    /// Structural node with potential children (objects, dictionaries, arrays)
    Node(Node),
}

impl<'a, Node, Token> GreenNode<'a> for EitherNodeOrToken<Node, Token>
where
    Node: GreenNode<'a>,
    Token: GreenNode<'a>,
{
    #[inline]
    fn kind(&self) -> crate::SyntaxKind {
        todo!()
    }

    #[inline]
    fn to_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    #[inline]
    fn to_full_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    #[inline]
    fn full_width(&self) -> u64 {
        match self {
            EitherNodeOrToken::Token(token) => token.full_width(),
            EitherNodeOrToken::Node(node) => node.full_width(),
        }
    }

    #[inline]
    fn is_token(&self) -> bool {
        matches!(self, EitherNodeOrToken::Token(_))
    }

    #[inline]
    // TODO: abstraction?
    fn slot(&self, _index: u8) -> Option<NodeOrToken<'a>> {
        todo!()
    }

    #[inline]
    fn slot_count(&self) -> u8 {
        todo!()
    }

    #[inline]
    // TODO: abstraction?
    fn leading_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    #[inline]
    // TODO: abstraction?
    fn trailing_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    #[inline]
    fn leading_trivia_width(&self) -> u64 {
        todo!()
    }

    #[inline]
    fn trailing_trivia_width(&self) -> u64 {
        todo!()
    }
}
