use std::borrow::Cow;

use crate::{
    GreenNodeTrait, SyntaxKind,
    green::{NodeOrToken, Trivia},
};

/// Marker trait to indicate that a type represents a list node
///
/// This trait should only be implemented by types where `is_list()` returns `true`
pub trait IsGreenList<'a>: GreenNodeTrait<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemOrList<Item, List> {
    /// Single syntax element
    Item(Item),
    /// Collection of syntax elements
    List(List),
}

impl<'a, Item, List> GreenNodeTrait<'a> for ItemOrList<Item, List>
where
    Item: GreenNodeTrait<'a>,
    List: IsGreenList<'a>,
{
    #[inline]
    fn kind(&self) -> crate::SyntaxKind {
        match self {
            ItemOrList::Item(item) => item.kind(),
            ItemOrList::List(_) => SyntaxKind::List,
        }
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
            ItemOrList::Item(item) => item.full_width(),
            ItemOrList::List(list) => list.full_width(),
        }
    }

    #[inline]
    fn is_token(&self) -> bool {
        match self {
            ItemOrList::Item(item) => item.is_token(),
            ItemOrList::List(_) => false,
        }
    }

    #[inline]
    fn is_trivia(&self) -> bool {
        match self {
            ItemOrList::Item(item) => item.is_trivia(),
            ItemOrList::List(_) => false,
        }
    }

    #[inline]
    fn is_list(&self) -> bool {
        match self {
            ItemOrList::Item(_) => false,
            ItemOrList::List(_) => true,
        }
    }

    #[inline]
    fn slot(&self, index: u8) -> Option<NodeOrToken<'a>> {
        match self {
            ItemOrList::Item(item) => item.slot(index),
            ItemOrList::List(list) => list.slot(index),
        }
    }

    #[inline]
    fn slot_count(&self) -> u8 {
        match self {
            ItemOrList::Item(item) => item.slot_count(),
            ItemOrList::List(list) => list.slot_count(),
        }
    }

    #[inline]
    fn leading_trivia(&self) -> Option<Trivia<'a>> {
        match self {
            ItemOrList::Item(item) => item.leading_trivia(),
            ItemOrList::List(list) => list.leading_trivia(),
        }
    }

    #[inline]
    fn trailing_trivia(&self) -> Option<Trivia<'a>> {
        match self {
            ItemOrList::Item(item) => item.trailing_trivia(),
            ItemOrList::List(list) => list.trailing_trivia(),
        }
    }

    #[inline]
    fn leading_trivia_width(&self) -> u64 {
        match self {
            ItemOrList::Item(item) => item.leading_trivia_width(),
            ItemOrList::List(list) => list.leading_trivia_width(),
        }
    }

    #[inline]
    fn trailing_trivia_width(&self) -> u64 {
        match self {
            ItemOrList::Item(item) => item.trailing_trivia_width(),
            ItemOrList::List(list) => list.trailing_trivia_width(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EitherNodeOrToken<Node, Token> {
    /// Leaf token (keywords, literals, operators, names)
    Token(Token),
    /// Structural node with potential children (objects, dictionaries, arrays)
    Node(Node),
}

impl<'a, Node, Token> GreenNodeTrait<'a> for EitherNodeOrToken<Node, Token>
where
    Node: GreenNodeTrait<'a>,
    Token: GreenNodeTrait<'a>,
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
    fn slot(&self, _index: u8) -> Option<NodeOrToken<'a>> {
        todo!()
    }

    #[inline]
    fn slot_count(&self) -> u8 {
        todo!()
    }

    #[inline]
    fn leading_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    #[inline]
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
