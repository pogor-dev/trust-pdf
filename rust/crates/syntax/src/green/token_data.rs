use std::{fmt, mem::ManuallyDrop, ptr};

use crate::{
    SyntaxKind,
    green::{GreenTokenReprThin, token::GreenToken},
};

#[repr(transparent)]
pub struct GreenTokenData {
    pub(crate) data: GreenTokenReprThin,
}

impl GreenTokenData {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        let slice = self.data.slice();
        unsafe { std::slice::from_raw_parts(slice.as_ptr(), slice.len()) }
    }

    #[inline]
    pub(crate) fn width(&self) -> u64 {
        self.text().len() as u64
    }
}

impl PartialEq for GreenTokenData {
    fn eq(&self, other: &Self) -> bool {
        // TODO: trivia equality?
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl ToOwned for GreenTokenData {
    type Owned = GreenToken;

    #[inline]
    fn to_owned(&self) -> GreenToken {
        let green = unsafe { GreenToken::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenToken::clone(&green)
    }
}

impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTokenData")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .finish()
    }
}

impl fmt::Display for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = String::from_utf8_lossy(self.text());
        write!(f, "{}", text)
    }
}
