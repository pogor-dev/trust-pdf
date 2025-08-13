use std::{cell::Cell, ptr};

use crate::{GreenNodeData, GreenTokenData};

mod node;
mod node_data;
mod token;

enum Green {
    Node {
        ptr: Cell<ptr::NonNull<GreenNodeData>>,
    },
    Token {
        ptr: ptr::NonNull<GreenTokenData>,
    },
}
