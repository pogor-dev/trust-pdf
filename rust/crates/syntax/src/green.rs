mod list;
mod node;
mod token;
mod trivia;

use std::borrow::Cow;

pub use self::{list::SyntaxListWithTwoChildren, node::GreenNode, token::GreenToken, trivia::GreenTrivia};

type Trivia<'a> = ItemOrList<GreenTrivia<'a>, GreenList>;
type Token<'a> = ItemOrList<GreenToken<'a>, GreenList>;
type Node<'a> = ItemOrList<GreenElement, GreenList>;
type NodeOrToken<'a> = EitherNodeOrToken<Node<'a>, Token<'a>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemOrList<Item, List>
where
    Item: for<'a> GreenNode<'a>,
    List: for<'a> GreenNode<'a>,
{
    /// Single syntax element
    Item(Item),
    /// Collection of syntax elements
    List(List),
}

impl<'a, Item, List> GreenNode<'a> for ItemOrList<Item, List>
where
    Item: for<'b> GreenNode<'b>,
    List: for<'b> GreenNode<'b>,
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
            ItemOrList::Item(item) => item.full_width(),
            ItemOrList::List(list) => list.full_width(),
        }
    }

    #[inline]
    fn is_token(&self) -> bool {
        if let ItemOrList::Item(item) = self { item.is_token() } else { false }
    }

    #[inline]
    fn is_trivia(&self) -> bool {
        if let ItemOrList::Item(item) = self { item.is_trivia() } else { false }
    }

    #[inline]
    fn is_list(&self) -> bool {
        matches!(self, ItemOrList::List(_))
    }

    #[inline]
    fn slot(&self, index: u8) -> Option<NodeOrToken<'a>> {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EitherNodeOrToken<Node, Token>
where
    Node: for<'a> GreenNode<'a>,
    Token: for<'a> GreenNode<'a>,
{
    /// Leaf token (keywords, literals, operators, names)
    Token(Token),
    /// Structural node with potential children (objects, dictionaries, arrays)
    Node(Node),
}

impl<'a, Node, Token> GreenNode<'a> for EitherNodeOrToken<Node, Token>
where
    Node: for<'b> GreenNode<'b>,
    Token: for<'b> GreenNode<'b>,
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
    fn slot(&self, index: u8) -> Option<NodeOrToken<'a>> {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GreenElement {
    GreenList(GreenList),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GreenList {
    ListWithTwoChildren,
}
