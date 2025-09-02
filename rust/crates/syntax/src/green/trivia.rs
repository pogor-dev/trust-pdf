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

type Repr = HeaderSlice<GreenTriviaHead, [GreenTriviaPiece]>;
type ReprThin = HeaderSlice<GreenTriviaHead, [GreenTriviaPiece; 0]>;

#[derive(PartialEq, Eq, Hash)]
struct GreenTriviaHead {
    text_len: u32,
    _c: Count<GreenTrivia>,
}

#[repr(transparent)]
pub struct GreenTriviaData {
    data: ReprThin,
}

impl GreenTriviaData {
    #[inline]
    pub fn text(&self) -> &[u8] {
        // TODO: fix
        &[]
        // self.data.slice()
    }

    /// Returns the full length of the trivia.
    /// It is expected to have up to 65535 bytes (e.g. long comments)
    #[inline]
    pub fn full_len(&self) -> u32 {
        self.data.header.text_len.into()
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
        f.debug_struct("GreenTrivia").field("text", &self.text()).finish()
    }
}

impl fmt::Display for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        byte_to_string(self.text(), f)
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, GreenTriviaPiece>,
}

impl GreenTrivia {
    /// Creates a new trivia containing the passed in pieces
    pub fn new_list<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = GreenTriviaPiece>,
        I::IntoIter: ExactSizeIterator,
    {
        let pieces_vec: Vec<GreenTriviaPiece> = pieces.into_iter().collect();
        let text_len = pieces_vec.iter().map(|p| p.len() as u32).sum();
        let head = GreenTriviaHead { text_len, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, pieces_vec.into_iter());
        GreenTrivia { ptr }
    }

    /// Creates a single piece of trivia from the given text.
    pub fn new_single(kind: SyntaxKind, text: &[u8]) -> Self {
        let text_len = text.len() as u32;
        let head = GreenTriviaHead { text_len, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, std::iter::once(GreenTriviaPiece::new(kind, text)));
        GreenTrivia { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenTrivia) -> ptr::NonNull<GreenTriviaData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaData = &green;
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
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaData>) -> GreenTrivia {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaHead, GreenTriviaPiece>>(arc)
        };
        GreenTrivia { ptr: arc }
    }
}

impl fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaData = self;
        fmt::Display::fmt(data, f)
    }
}

impl borrow::Borrow<GreenTriviaData> for GreenTrivia {
    #[inline]
    fn borrow(&self) -> &GreenTriviaData {
        self
    }
}

impl ops::Deref for GreenTrivia {
    type Target = GreenTriviaData;

    #[inline]
    fn deref(&self) -> &GreenTriviaData {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTriviaData>(repr)
        }
    }
}

type TriviaPieceRepr = HeaderSlice<GreenTriviaPieceHead, [u8]>;
type TriviaPieceReprThin = HeaderSlice<GreenTriviaPieceHead, [u8; 0]>;

#[derive(PartialEq, Eq, Hash)]
struct GreenTriviaPieceHead {
    kind: SyntaxKind,
    _c: Count<GreenTriviaPiece>,
}

#[repr(transparent)]
pub struct GreenTriviaPieceData {
    data: TriviaPieceReprThin,
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

    /// Returns the full length of the trivia piece.
    /// It is expected to have up to 65535 bytes (e.g. long comments)
    #[inline]
    pub fn len(&self) -> u16 {
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
            let arc = Arc::from_raw(&ptr.as_ref().data as *const TriviaPieceReprThin);
            mem::transmute::<Arc<TriviaPieceReprThin>, ThinArc<GreenTriviaPieceHead, u8>>(arc)
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
            let repr: &TriviaPieceRepr = &self.ptr;
            let repr: &TriviaPieceReprThin = &*(repr as *const TriviaPieceRepr as *const TriviaPieceReprThin);
            mem::transmute::<&TriviaPieceReprThin, &GreenTriviaPieceData>(repr)
        }
    }
}
