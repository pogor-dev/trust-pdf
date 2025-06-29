use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    arc::{arc::Arc, thin_arc::ThinArc},
    green::{
        GreenTokenRepr, GreenTokenReprThin, kind::RawSyntaxKind, token_data::GreenTokenData,
        token_head::GreenTokenHead,
    },
};

/// Leaf node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenToken {
    ptr: ThinArc<GreenTokenHead, u8>,
}

impl GreenToken {
    /// Kind of this Token.
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.data.header.kind
    }

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn text_len(&self) -> u64 {
        self.text().len() as u64
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> GreenToken {
        let arc = unsafe { Arc::from_raw(&ptr.as_ref().data as *const GreenTokenReprThin) };
        let arc =
            unsafe { mem::transmute::<Arc<GreenTokenReprThin>, ThinArc<GreenTokenHead, u8>>(arc) };
        GreenToken { ptr: arc }
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

impl ToOwned for GreenTokenData {
    type Owned = GreenToken;

    #[inline]
    fn to_owned(&self) -> GreenToken {
        unsafe {
            let green = GreenToken::from_raw(ptr::NonNull::from(self));
            let green = ManuallyDrop::new(green);
            GreenToken::clone(&green)
        }
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
