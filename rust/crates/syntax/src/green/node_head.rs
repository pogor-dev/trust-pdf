use countme::Count;

use crate::green::{kind::RawSyntaxKind, node::GreenNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    pub(super) kind: RawSyntaxKind,
    pub(super) text_len: u64,
    _c: Count<GreenNode>,
}
