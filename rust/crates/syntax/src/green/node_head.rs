use countme::Count;

use crate::{SyntaxKind, green::node::GreenNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    kind: SyntaxKind,
    text_len: u32,
    _c: Count<GreenNode>,
}
