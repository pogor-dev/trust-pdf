use std::{borrow::Borrow, fmt, ops, ptr};

use crate::{
    SyntaxKind,
    arc::thin_arc::ThinArc,
    green::{token_data::GreenTokenData, token_head::GreenTokenHead},
};

/// Leaf node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenToken {
    ptr: ThinArc<GreenTokenHead, u8>,
}

impl GreenToken {
    /// Creates new Token.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenToken {
        let head = GreenTokenHead::new(kind);
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenToken { ptr }
    }

    // #[inline]
    // pub(crate) fn into_raw(this: GreenToken) -> ptr::NonNull<GreenTokenData> {}

    // #[inline]
    // pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> GreenToken {}
}

// impl Borrow<GreenTokenData> for GreenToken {
//     #[inline]
//     fn borrow(&self) -> &GreenTokenData {
//         self
//     }
// }

// impl fmt::Debug for GreenToken {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
// }

// impl fmt::Display for GreenToken {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
// }

// impl ops::Deref for GreenToken {
//     type Target = GreenTokenData;

//     #[inline]
//     fn deref(&self) -> &GreenTokenData {}
// }
