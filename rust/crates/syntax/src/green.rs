mod list;
mod node;
mod token;
mod trivia;

use std::borrow::Cow;

pub use self::{list::SyntaxListWithTwoChildren, node::GreenNode, token::GreenToken, trivia::GreenTrivia};

type Trivia<'a> = ItemOrList<GreenTrivia<'a>, GreenList<'a>>;
type Token<'a> = ItemOrList<GreenToken<'a>, GreenList<'a>>;
type Node<'a> = ItemOrList<GreenElement<'a>, GreenList<'a>>;
type NodeOrToken<'a> = EitherNodeOrToken<Node<'a>, Token<'a>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemOrList<Item, List> {
    /// Single syntax element
    Item(Item),
    /// Collection of syntax elements
    List(List),
}

impl<'a, Item, List> ItemOrList<Item, List>
where
    Item: GreenNode<'a>,
    List: GreenNode<'a>,
{
    pub fn get_first_non_null_child_index(node: &Self) -> u8 {
        for i in 0..node.slot_count() {
            if node.slot(i).is_some() {
                return i;
            }
        }
        0 // If no children found
    }

    pub fn get_last_non_null_child_index(node: &Self) -> u8 {
        for i in (0..node.slot_count()).rev() {
            if node.slot(i).is_some() {
                return i;
            }
        }
        0 // If no children found
    }

    pub fn get_first_terminal(&self) -> Option<&GreenToken<'a>> {
        let mut node: Option<&ItemOrList<Item, List>> = Some(self);

        loop {
            let current = node?;

            // Find first non-null child
            let mut first_child = None;
            let slot_count = current.slot_count();

            for i in 0..slot_count {
                if let Some(child) = current.slot(i) {
                    first_child = Some(child);
                    break;
                }
            }

            node = first_child;

            // Optimization: if no children or reached terminal, stop
            if node.map(|n| n.slot_count()).unwrap_or(0) == 0 {
                break;
            }
        }

        node
    }
}

impl<'a, Item, List> GreenNode<'a> for ItemOrList<Item, List>
where
    Item: GreenNode<'a>,
    List: GreenNode<'a>,
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
pub enum GreenElement<'a> {
    GreenList(GreenList<'a>),
}

impl<'a> GreenNode<'a> for GreenElement<'a> {
    fn kind(&self) -> crate::SyntaxKind {
        todo!()
    }

    fn to_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    fn to_full_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    fn full_width(&self) -> u64 {
        todo!()
    }

    fn slot(&self, _index: u8) -> Option<NodeOrToken<'a>> {
        todo!()
    }

    fn slot_count(&self) -> u8 {
        todo!()
    }

    fn leading_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    fn trailing_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    fn leading_trivia_width(&self) -> u64 {
        todo!()
    }

    fn trailing_trivia_width(&self) -> u64 {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GreenList<'a> {
    ListWithTwoChildren(SyntaxListWithTwoChildren<'a>),
}

impl<'a> GreenNode<'a> for GreenList<'a> {
    fn kind(&self) -> crate::SyntaxKind {
        todo!()
    }

    fn to_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    fn to_full_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    fn full_width(&self) -> u64 {
        todo!()
    }

    fn slot(&self, _index: u8) -> Option<NodeOrToken<'a>> {
        todo!()
    }

    fn slot_count(&self) -> u8 {
        todo!()
    }

    fn leading_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    fn trailing_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    fn leading_trivia_width(&self) -> u64 {
        todo!()
    }

    fn trailing_trivia_width(&self) -> u64 {
        todo!()
    }
}
