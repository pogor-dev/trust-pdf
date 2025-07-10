use crate::{
    arc::header_slice::HeaderSlice,
    green::{token_head::GreenTokenHead, trivia_child_head::GreenTriviaChildHead},
};

#[allow(dead_code)]
mod token;
#[allow(dead_code)]
mod token_data;
#[allow(dead_code)]
mod token_head;
#[allow(dead_code)]
mod trivia_child;
#[allow(dead_code)]
mod trivia_child_data;
#[allow(dead_code)]
mod trivia_child_head;

#[cfg(test)]
mod token_tests;
#[cfg(test)]
mod trivia_child_tests;

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);

type GreenTriviaReprThin = HeaderSlice<GreenTriviaChildHead, [u8; 0]>;
type GreenTriviaRepr = HeaderSlice<GreenTriviaChildHead, [u8]>;
type GreenTokenRepr = HeaderSlice<GreenTokenHead, [u8]>;
type GreenTokenReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;
