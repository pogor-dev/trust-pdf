use std::{cell::Cell, ptr};

use countme::Count;
struct _SyntaxElement;

enum Green {
    Node {
        ptr: Cell<ptr::NonNull<GreenNodeData>>,
    },
    Token {
        ptr: ptr::NonNull<GreenTokenData>,
    },
    // TODO: trivia?
}

struct NodeData {
    _c: Count<_SyntaxElement>,

    rc: Cell<u32>,
    parent: Cell<Option<ptr::NonNull<NodeData>>>,
    index: Cell<u32>,
    green: Green,
}
