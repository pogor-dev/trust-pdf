mod elements;
mod item_or_list;
mod list;
mod node_or_token;
mod node_trait;
mod token;
mod trivia;

use std::borrow::Cow;

pub use self::{
    elements::GreenElement,
    item_or_list::ItemOrList,
    list::{GreenList, SyntaxList, SyntaxListWithTwoChildren},
    node_or_token::EitherNodeOrToken,
    node_trait::GreenNode,
    token::GreenToken,
    trivia::GreenTrivia,
};

type Trivia<'a> = ItemOrList<GreenTrivia<'a>, GreenList<'a>>;
type Token<'a> = ItemOrList<GreenToken<'a>, GreenList<'a>>;
type Node<'a> = ItemOrList<GreenElement<'a>, GreenList<'a>>;
type NodeOrToken<'a> = EitherNodeOrToken<Node<'a>, Token<'a>>;
