use countme::Count;

use crate::{SyntaxKind, green::trivia::GreenTrivia};

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTriviaHead {
    pub(crate) kind: SyntaxKind,
    _c: Count<GreenTrivia>,
}

impl GreenTriviaHead {
    pub(crate) fn new(kind: SyntaxKind) -> Self {
        Self {
            kind,
            _c: Count::new(),
        }
    }
}
