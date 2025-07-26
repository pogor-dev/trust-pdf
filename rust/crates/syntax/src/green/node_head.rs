use countme::Count;

use crate::{SyntaxKind, green::node::GreenNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    pub(super) kind: SyntaxKind,
    pub(super) width: u32,
    pub(super) full_width: u32,
    _c: Count<GreenNode>,
}

impl GreenNodeHead {
    pub(super) fn new(kind: SyntaxKind, width: u32, full_width: u32) -> GreenNodeHead {
        GreenNodeHead {
            kind,
            width,
            full_width,
            _c: Count::new(),
        }
    }
}
