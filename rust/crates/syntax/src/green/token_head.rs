use countme::Count;

use crate::{SyntaxKind, green::token::GreenToken};

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTokenHead {
    pub(crate) kind: SyntaxKind,
    _c: Count<GreenToken>,
}

impl GreenTokenHead {
    pub(crate) fn new(kind: SyntaxKind) -> Self {
        GreenTokenHead {
            kind,
            _c: Count::new(),
        }
    }
}
