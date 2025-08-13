use std::{cell::Cell, ptr};

use countme::Count;

use crate::{GreenNodeData, GreenTokenData, cursor::Green};
struct _SyntaxElement;

pub(super) struct NodeData {
    _c: Count<_SyntaxElement>,

    rc: Cell<u32>,
    parent: Cell<Option<ptr::NonNull<NodeData>>>,
    index: Cell<u32>,
    green: Green,

    /// Absolute offset for immutable nodes.
    offset: u32,
}
