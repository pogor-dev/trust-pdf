use crate::{
    arc::header_slice::HeaderSlice,
    green::{token_head::GreenTokenHead, trivia_head::GreenTriviaHead},
};

#[allow(dead_code)]
mod token;
#[allow(dead_code)]
mod token_data;
#[allow(dead_code)]
mod token_head;
#[allow(dead_code)]
mod trivia;
#[allow(dead_code)]
mod trivia_data;
#[allow(dead_code)]
mod trivia_head;

#[cfg(test)]
mod token_tests;
#[cfg(test)]
mod trivia_tests;

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);

type GreenTriviaReprThin = HeaderSlice<GreenTriviaHead, [u8; 0]>;
type GreenTriviaRepr = HeaderSlice<GreenTriviaHead, [u8]>;
type GreenTokenRepr = HeaderSlice<GreenTokenHead, [u8]>;
type GreenTokenReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;
