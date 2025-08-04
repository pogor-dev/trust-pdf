use crate::{
    arc::header_slice::HeaderSlice,
    green::{
        node_child::GreenChild, node_head::GreenNodeHead, token_head::GreenTokenHead,
        trivia_child::GreenTriviaChild, trivia_head::GreenTriviaHead,
    },
};

mod element;
mod element_ref;
mod node;
mod node_child;
mod node_children;
mod node_data;
mod node_head;
mod token;
mod token_data;
mod token_head;
mod trivia;
mod trivia_child;
mod trivia_data;
mod trivia_head;

#[cfg(test)]
mod tests;

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);

type GreenTriviaReprThin = HeaderSlice<GreenTriviaHead, [GreenTriviaChild; 0]>;
type GreenTriviaRepr = HeaderSlice<GreenTriviaHead, [GreenTriviaChild]>;
type GreenTokenRepr = HeaderSlice<GreenTokenHead, [u8]>;
type GreenTokenReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;
type GreenNodeRepr = HeaderSlice<GreenNodeHead, [GreenChild]>;
type GreenNodeReprThin = HeaderSlice<GreenNodeHead, [GreenChild; 0]>;
