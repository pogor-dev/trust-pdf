mod list;
mod node;
mod token;
mod trivia;

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

impl<Item, List> ItemOrList<Item, List>
where
    Item: for<'a> GreenNode<'a>,
    List: for<'a> GreenNode<'a>,
{
    #[inline]
    pub fn is_item(&self) -> bool {
        matches!(self, ItemOrList::Item(_))
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, ItemOrList::List(_))
    }

    #[inline]
    pub fn full_width(&self) -> u64 {
        match self {
            ItemOrList::Item(item) => item.full_width(),
            ItemOrList::List(list) => list.full_width(),
        }
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

impl<Node, Token> EitherNodeOrToken<Node, Token>
where
    Node: for<'a> GreenNode<'a>,
    Token: for<'a> GreenNode<'a>,
{
    #[inline]
    pub fn is_token(&self) -> bool {
        matches!(self, EitherNodeOrToken::Token(_))
    }

    #[inline]
    pub fn is_node(&self) -> bool {
        matches!(self, EitherNodeOrToken::Node(_))
    }

    #[inline]
    pub fn full_width(&self) -> u64 {
        match self {
            EitherNodeOrToken::Token(token) => token.full_width(),
            EitherNodeOrToken::Node(node) => node.full_width(),
        }
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
