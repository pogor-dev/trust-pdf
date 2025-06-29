use std::fmt;

use crate::green::{GreenTokenReprThin, kind::RawSyntaxKind, trivia::GreenTrivia};

#[repr(transparent)]
pub(crate) struct GreenTokenData {
    pub(crate) data: GreenTokenReprThin,
}

impl GreenTokenData {
    /// Kind of this Token.
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.data.header.kind
    }

    /// Whole text of this Token, including all trivia.
    #[inline]
    pub fn text(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.slice().as_ptr(), self.data.slice().len()) }
    }

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn text_len(&self) -> u64 {
        self.text().len() as u64
    }

    #[inline]
    pub fn leading_trivia(&self) -> &GreenTrivia {
        &self.data.header.leading
    }

    #[inline]
    pub fn trailing_trivia(&self) -> &GreenTrivia {
        &self.data.header.trailing
    }
}

impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .field("leading", &self.leading_trivia())
            .field("trailing", &self.trailing_trivia())
            .finish()
    }
}

impl fmt::Display for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.text()))
    }
}
