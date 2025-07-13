use countme::Count;

use crate::green::trivia::GreenTrivia;

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTriviaHead {
    _c: Count<GreenTrivia>,
}

impl GreenTriviaHead {
    pub(crate) fn new() -> Self {
        Self { _c: Count::new() }
    }
}
