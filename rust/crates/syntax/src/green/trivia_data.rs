use std::{
    fmt::{self, Formatter},
    mem::ManuallyDrop,
};

use countme::Count;

use crate::green::{ReprThin, trivia::GreenTrivia};
use crate::syntax::trivia_piece::TriviaPiece;

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTriviaHead {
    pub(super) _c: Count<GreenTrivia>,
}

#[repr(transparent)]
pub(crate) struct GreenTriviaData {
    pub(crate) data: ReprThin,
}

impl GreenTriviaData {
    #[allow(unused)]
    #[inline]
    pub fn header(&self) -> &GreenTriviaHead {
        &self.data.header
    }

    #[inline]
    pub fn pieces(&self) -> &[TriviaPiece] {
        self.data.slice()
    }

    #[inline]
    pub(crate) fn to_owned(&self) -> GreenTrivia {
        unsafe {
            let green = GreenTrivia::from_raw(self as *const _ as *mut _);
            let green = ManuallyDrop::new(green);
            GreenTrivia::clone(&green)
        }
    }
}

impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.pieces() == other.pieces()
    }
}

impl fmt::Debug for GreenTriviaData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.pieces().iter()).finish()
    }
}

pub(crate) fn has_live() -> bool {
    countme::get::<GreenTrivia>().live > 0
}
