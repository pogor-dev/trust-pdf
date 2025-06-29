pub(crate) mod green_node;
pub(crate) mod syntax_kind;
pub(crate) mod trivia;
pub(crate) mod trivia_data;

use crate::{
    arc::header_slice::HeaderSlice, green::trivia_data::GreenTriviaHead,
    syntax::trivia_piece::TriviaPiece,
};

type ReprThin = HeaderSlice<GreenTriviaHead, [TriviaPiece; 0]>;
