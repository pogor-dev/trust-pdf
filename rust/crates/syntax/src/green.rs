use crate::{
    arc::header_slice::HeaderSlice,
    green::{
        token_head::GreenTokenHead, trivia_child::GreenTriviaChild,
        trivia_child_head::GreenTriviaChildHead, trivia_head::GreenTriviaHead,
    },
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
mod trivia_child;
#[allow(dead_code)]
mod trivia_child_data;
#[allow(dead_code)]
mod trivia_child_head;
#[allow(dead_code)]
mod trivia_data;
#[allow(dead_code)]
mod trivia_head;

#[cfg(test)]
mod token_tests;
#[cfg(test)]
mod trivia_child_tests;
#[cfg(test)]
mod trivia_tests;

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);

type GreenTriviaChildReprThin = HeaderSlice<GreenTriviaChildHead, [u8; 0]>;
type GreenTriviaChildRepr = HeaderSlice<GreenTriviaChildHead, [u8]>;
type GreenTriviaReprThin = HeaderSlice<GreenTriviaHead, [GreenTriviaChild; 0]>;
type GreenTriviaRepr = HeaderSlice<GreenTriviaHead, [GreenTriviaChild]>;
type GreenTokenRepr = HeaderSlice<GreenTokenHead, [u8]>;
type GreenTokenReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;
