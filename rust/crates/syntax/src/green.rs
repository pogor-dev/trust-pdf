mod item_or_list;
mod list;
mod node_or_token;
mod node_trait;
mod token;
mod trivia;

use std::borrow::Cow;

pub use self::{
    item_or_list::ItemOrList, list::SyntaxListWithTwoChildren, node_or_token::EitherNodeOrToken, node_trait::GreenNode, token::GreenToken, trivia::GreenTrivia,
};

type Trivia<'a> = ItemOrList<GreenTrivia<'a>, GreenList<'a>>;
type Token<'a> = ItemOrList<GreenToken<'a>, GreenList<'a>>;
type Node<'a> = ItemOrList<GreenElement<'a>, GreenList<'a>>;
type NodeOrToken<'a> = EitherNodeOrToken<Node<'a>, Token<'a>>;

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
