pub(crate) mod kind;
pub(crate) mod node;
pub(crate) mod node_data;
pub(crate) mod node_head;
pub(crate) mod node_slot;
pub(crate) mod node_slots;
pub(crate) mod token;
pub(crate) mod token_data;
pub(crate) mod token_head;
pub(crate) mod trivia;
pub(crate) mod trivia_data;

use crate::{
    arc::header_slice::HeaderSlice,
    green::{
        node_head::GreenNodeHead, node_slot::Slot, token_head::GreenTokenHead,
        trivia_data::GreenTriviaHead,
    },
    syntax::trivia_piece::TriviaPiece,
};

type GreenTriviaReprThin = HeaderSlice<GreenTriviaHead, [TriviaPiece; 0]>;

type GreenTokenRepr = HeaderSlice<GreenTokenHead, [u8]>;
type GreenTokenReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;

type GreenNodeRepr = HeaderSlice<GreenNodeHead, [Slot]>;
type GreenNodeReprThin = HeaderSlice<GreenNodeHead, [Slot; 0]>;
