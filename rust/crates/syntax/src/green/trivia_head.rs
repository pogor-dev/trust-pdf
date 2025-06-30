use countme::Count;

use crate::{SyntaxKind, arc::header_slice::HeaderSlice, green::trivia::GreenTrivia};

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTriviaHead {
    kind: SyntaxKind,
    _c: Count<GreenTrivia>,
}
