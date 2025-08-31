use std::borrow::Cow;

use crate::{
    GreenNode,
    green::{GreenList, NodeOrToken, Trivia},
};

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
