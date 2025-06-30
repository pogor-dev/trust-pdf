use crate::{arc::header_slice::HeaderSlice, green::trivia_head::GreenTriviaHead};

// mod child;
// mod node;
// mod node_cache;
// mod node_data;
// mod node_head;
// mod token;
// mod token_data;
// mod token_head;
mod trivia;
mod trivia_data;
mod trivia_head;

#[cfg(test)]
mod trivia_tests;

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);

type GreenTriviaReprThin = HeaderSlice<GreenTriviaHead, [u8; 0]>;
type GreenTriviaRepr = HeaderSlice<GreenTriviaHead, [u8]>;
