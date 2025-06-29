use std::fmt::{self, Formatter};
use std::mem;

use countme::Count;

use crate::arc::arc::Arc;
use crate::green::ReprThin;
use crate::green::trivia_data::GreenTriviaData;
use crate::syntax::trivia_piece::TriviaPiece;
use crate::{arc::thin_arc::ThinArc, green::trivia_data::GreenTriviaHead};

/// List of trivia. Used to store either the leading or trailing trivia of a token.
/// The identity of a trivia is defined by the kinds and lengths of its items but not by
/// the texts of an individual piece. That means, that `\r` and `\n` can both be represented
/// by the same trivia, a trivia with a single `LINEBREAK` piece with the length 1.
/// This is safe because the text is stored on the token to which the trivia belongs and
/// `a\n` and `a\r` never resolve to the same tokens. Thus, they only share the trivia but are
/// otherwise two different tokens.
#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenTrivia {
    pointer: Option<ThinArc<GreenTriviaHead, TriviaPiece>>,
}

impl fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.pieces(), f)
    }
}

impl GreenTrivia {
    /// Creates a new trivia containing the passed in pieces
    pub fn new<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = TriviaPiece>,
        I::IntoIter: ExactSizeIterator,
    {
        let data =
            ThinArc::from_header_and_iter(GreenTriviaHead { _c: Count::new() }, pieces.into_iter());

        GreenTrivia {
            pointer: Some(data),
        }
    }

    /// Creates an empty trivia
    pub fn empty() -> Self {
        GreenTrivia { pointer: None }
    }

    /// Returns the pieces of the trivia
    pub fn pieces(&self) -> &[TriviaPiece] {
        match &self.pointer {
            None => &[],
            Some(ptr) => ptr.slice(),
        }
    }

    pub(crate) unsafe fn from_raw(ptr: *mut GreenTriviaData) -> Self {
        if let Some(pointer) = unsafe { ptr.as_ref() } {
            let arc = unsafe { Arc::from_raw(&pointer.data as *const ReprThin) };
            let arc = unsafe {
                mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaHead, TriviaPiece>>(arc)
            };
            Self { pointer: Some(arc) }
        } else {
            Self { pointer: None }
        }
    }
}
