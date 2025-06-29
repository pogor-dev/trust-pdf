use countme::Count;

use crate::green::{kind::RawSyntaxKind, token::GreenToken, trivia::GreenTrivia};

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTokenHead {
    pub(crate) kind: RawSyntaxKind,
    pub(crate) leading: GreenTrivia,
    pub(crate) trailing: GreenTrivia,
    _c: Count<GreenToken>,
}
