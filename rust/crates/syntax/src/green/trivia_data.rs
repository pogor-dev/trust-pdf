use std::{fmt, mem::ManuallyDrop, ptr};

use crate::green::{
    GreenTriviaReprThin, trivia::GreenTrivia, trivia_child::GreenTriviaChild,
    trivia_head::GreenTriviaHead,
};

#[repr(transparent)]
pub(crate) struct GreenTriviaData {
    /// Underlying thin representation providing access to both header and body
    pub(crate) data: GreenTriviaReprThin,
}

impl GreenTriviaData {
    #[inline]
    pub(crate) fn header(&self) -> &GreenTriviaHead {
        &self.data.header
    }

    #[inline]
    pub fn children(&self) -> &[GreenTriviaChild] {
        self.data.slice()
    }
}

impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.children() == other.children()
    }
}

impl ToOwned for GreenTriviaData {
    type Owned = GreenTrivia;

    #[inline]
    fn to_owned(&self) -> GreenTrivia {
        let green = unsafe { GreenTrivia::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTrivia::clone(&green)
    }
}

impl fmt::Debug for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.children().iter()).finish()
    }
}

impl fmt::Display for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in self.children() {
            match std::str::from_utf8(child.text()) {
                Ok(text) => write!(f, "{}", text)?,
                Err(_) => write!(f, "{:?}", child.text())?,
            }
        }
        Ok(())
    }
}
