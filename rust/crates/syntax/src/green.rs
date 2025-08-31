mod item_or_list;
mod list;
mod node;
mod node_or_token;
mod node_trait;
mod token;
mod trivia;

use std::borrow::Cow;

pub use self::{
    item_or_list::{IsGreenList, ItemOrList},
    list::{GreenList, SyntaxList, SyntaxListWithTwoChildren},
    node::GreenNode,
    node_or_token::EitherNodeOrToken,
    node_trait::GreenNodeTrait,
    token::GreenToken,
    trivia::GreenTrivia,
};

type Trivia<'a> = ItemOrList<GreenTrivia<'a>, GreenList<'a>>;
type Node<'a> = ItemOrList<GreenNode<'a>, GreenList<'a>>;
type NodeOrToken<'a> = EitherNodeOrToken<Node<'a>, GreenToken<'a>>;
