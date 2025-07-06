use std::{borrow::Borrow, fmt, mem, ops, ptr};

use crate::{
    SyntaxKind,
    arc::{arc_main::Arc, thin_arc::ThinArc},
    green::{
        GreenTokenRepr, GreenTokenReprThin, token_data::GreenTokenData, token_head::GreenTokenHead,
    },
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

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> GreenToken {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const GreenTokenReprThin);
            mem::transmute::<Arc<GreenTokenReprThin>, ThinArc<GreenTokenHead, u8>>(arc)
        };
        GreenToken { ptr: arc }
    }
}

impl Borrow<GreenTokenData> for GreenToken {
    #[inline]
    fn borrow(&self) -> &GreenTokenData {
        self
    }
}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Display::fmt(data, f)
    }
}

impl ops::Deref for GreenToken {
    type Target = GreenTokenData;

    #[inline]
    fn deref(&self) -> &GreenTokenData {
        unsafe {
            let repr: &GreenTokenRepr = &self.ptr;
            let repr: &GreenTokenReprThin =
                &*(repr as *const GreenTokenRepr as *const GreenTokenReprThin);

            mem::transmute::<&GreenTokenReprThin, &GreenTokenData>(repr)
        }
    }
}
