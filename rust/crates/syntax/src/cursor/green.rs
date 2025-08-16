//! Green tree reference wrapper for cursors.
//!
//! ```text
//!     🌲 Green Reference
//!    ┌─────────────┐
//!    │ Node OR     │   Points to green tree:
//!    │ Token       │   • immutable reference
//!    │ Pointer     │   • shared green data
//!    └─────────────┘   • memory efficient
//! ```

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
