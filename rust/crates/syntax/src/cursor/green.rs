use std::{cell::Cell, ptr};

use crate::{GreenNodeData, GreenTokenData};

pub(super) enum Green {
    Node {
        ptr: Cell<ptr::NonNull<GreenNodeData>>,
    },
    Token {
        ptr: ptr::NonNull<GreenTokenData>,
    },
}
