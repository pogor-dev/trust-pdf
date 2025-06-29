use countme::Count;

use crate::cursor::syntax_node_kind::NodeKind;

#[derive(Debug)]
pub(crate) struct NodeData {
    _c: Count<_SyntaxElement>,

    kind: NodeKind,
    slot: u32,
    /// Absolute offset for immutable nodes, unused for mutable nodes.
    offset: u32,
}

#[derive(Debug)]
struct _SyntaxElement;

pub(crate) fn has_live() -> bool {
    countme::get::<_SyntaxElement>().live > 0
}
