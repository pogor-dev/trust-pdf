mod list;
mod node;
mod token;
mod trivia;

pub use self::{list::SyntaxListWithTwoChildren, node::GreenNode, token::GreenToken, trivia::GreenTrivia};

type Trivia<'a> = ItemOrList<GreenTrivia<'a>, GreenList>;
type Node = ItemOrList<GreenElement, GreenList>;
type NodeOrToken<'a> = EitherNodeOrToken<Node, GreenToken<'a>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemOrList<Item, List> {
    /// Single syntax element
    Item(Item),
    /// Collection of syntax elements
    List(List),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EitherNodeOrToken<Node, Token> {
    /// Leaf token (keywords, literals, operators, names)
    Token(Token),
    /// Structural node with potential children (objects, dictionaries, arrays)
    Node(Node),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GreenElement {
    GreenList(GreenList),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GreenList {
    ListWithTwoChildren,
}
