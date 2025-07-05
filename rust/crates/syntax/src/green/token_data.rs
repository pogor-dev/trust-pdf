use std::fmt;

use crate::{
    SyntaxKind,
    green::{GreenTokenReprThin, token::GreenToken},
};

#[repr(transparent)]
pub struct GreenTokenData {
    data: GreenTokenReprThin,
}

impl GreenTokenData {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    // /// Text of this Token.
    // #[inline]
    // pub fn text(&self) -> &[u8] {}

    // /// Returns the width of the text covered by this token.
    // #[inline]
    // pub fn width(&self) -> u32 {}
}

// impl PartialEq for GreenTokenData {
//     fn eq(&self, other: &Self) -> bool {}
// }

// impl ToOwned for GreenTokenData {
//     type Owned = GreenToken;

//     #[inline]
//     fn to_owned(&self) -> GreenToken {}
// }

// impl fmt::Debug for GreenTokenData {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
// }

// impl fmt::Display for GreenTokenData {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
// }
