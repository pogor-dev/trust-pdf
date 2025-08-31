use std::{
    borrow, fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    SyntaxKind,
    arc::{Arc, HeaderSlice, ThinArc},
    green::byte_to_string,
};
use countme::Count;

type Repr = HeaderSlice<GreenTriviaPieceHead, [u8]>;
type ReprThin = HeaderSlice<GreenTriviaPieceHead, [u8; 0]>;

#[derive(PartialEq, Eq, Hash)]
struct GreenTriviaPieceHead {
    kind: SyntaxKind,
    _c: Count<GreenTriviaPiece>,
}

#[repr(transparent)]
pub struct GreenTriviaPieceData {
    data: ReprThin,
}

impl GreenTriviaPieceData {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        self.data.slice()
    }

    /// Returns the full length of the trivia.
    /// It is expected to have up to 65535 bytes (e.g. long comments)
    #[inline]
    pub fn full_len(&self) -> u16 {
        self.text().len() as u16
    }
}

impl ToOwned for GreenTriviaPieceData {
    type Owned = GreenTriviaPiece;

    #[inline]
    fn to_owned(&self) -> GreenTriviaPiece {
        let green = unsafe { GreenTriviaPiece::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTriviaPiece::clone(&green)
    }
}

impl fmt::Debug for GreenTriviaPieceData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTriviaPiece")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .finish()
    }
}

impl fmt::Display for GreenTriviaPieceData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        byte_to_string(self.text(), f)
    }
}

// TODO: check if trivia piece is not too fragmented
#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTriviaPiece {
    ptr: ThinArc<GreenTriviaPieceHead, u8>,
}

impl GreenTriviaPiece {
    /// Creates new Token.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTriviaPiece {
        let head = GreenTriviaPieceHead { kind, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenTriviaPiece { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenTriviaPiece) -> ptr::NonNull<GreenTriviaPieceData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaPieceData = &green;
        ptr::NonNull::from(green)
    }

    /// # Safety
    ///
    /// This function uses `unsafe` code to create an `Arc` from a raw pointer and then transmutes it into a `ThinArc`.
    ///
    /// - The raw pointer must be valid and correctly aligned for the type `ReprThin`.
    /// - The lifetime of the raw pointer must outlive the lifetime of the `Arc` created from it.
    /// - The transmute operation must be safe, meaning that the memory layout of `Arc<ReprThin>` must be compatible with `ThinArc<GreenTokenHead, u8>`.
    ///
    /// Failure to uphold these invariants can lead to undefined behavior.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaPieceData>) -> GreenTriviaPiece {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaPieceHead, u8>>(arc)
        };
        GreenTriviaPiece { ptr: arc }
    }
}

impl fmt::Debug for GreenTriviaPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaPieceData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenTriviaPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaPieceData = self;
        fmt::Display::fmt(data, f)
    }
}

impl borrow::Borrow<GreenTriviaPieceData> for GreenTriviaPiece {
    #[inline]
    fn borrow(&self) -> &GreenTriviaPieceData {
        self
    }
}

impl ops::Deref for GreenTriviaPiece {
    type Target = GreenTriviaPieceData;

    #[inline]
    fn deref(&self) -> &GreenTriviaPieceData {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTriviaPieceData>(repr)
        }
    }
}
