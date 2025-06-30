use crate::{arc::thin_arc::ThinArc, green::trivia_head::GreenTriviaHead};

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, u8>,
}
