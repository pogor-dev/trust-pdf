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
type TriviaPieceRepr = HeaderSlice<GreenTriviaPieceHead, [u8]>;
type TriviaPieceReprThin = HeaderSlice<GreenTriviaPieceHead, [u8; 0]>;

#[derive(PartialEq, Eq, Hash)]
pub(super) struct GreenTriviaHead {
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
        let piece = GreenTriviaPiece::new(kind, text);
        GreenTrivia::new_list(std::iter::once(piece))
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

#[derive(PartialEq, Eq, Hash)]
pub(super) struct GreenTriviaPieceHead {
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_green_trivia_piece() {
        let kind = SyntaxKind(1);
        let text = b"% test trivia";
        let trivia_piece = GreenTriviaPiece::new(kind, text);

        assert_eq!(trivia_piece.kind(), kind);
        assert_eq!(trivia_piece.text(), text);
        assert_eq!(trivia_piece.len() as usize, text.len());
    }

    #[rstest]
    #[allow(useless_ptr_null_checks)]
    fn test_into_raw_when_valid_piece_expect_non_null_ptr() {
        let kind = SyntaxKind(2);
        let text = b"% raw trivia";
        let trivia_piece = GreenTriviaPiece::new(kind, text);
        let ptr = GreenTriviaPiece::into_raw(trivia_piece.clone());
        assert!(!ptr.as_ptr().is_null());
    }

    #[rstest]
    fn test_from_raw_when_ptr_from_piece_expect_equivalent_piece() {
        let kind = SyntaxKind(3);
        let text = b"from raw trivia";
        let trivia_piece = GreenTriviaPiece::new(kind, text);
        let ptr = GreenTriviaPiece::into_raw(trivia_piece.clone());
        let recovered = unsafe { GreenTriviaPiece::from_raw(ptr) };
        assert_eq!(recovered.kind(), kind);
        assert_eq!(recovered.text(), text);
    }

    #[rstest]
    fn test_fmt_debug_when_valid_piece_expect_struct_debug_output() {
        let kind = SyntaxKind(4);
        let text = b"% debug trivia";
        let trivia_piece = GreenTriviaPiece::new(kind, text);
        let debug_str = format!("{:?}", trivia_piece);
        let expected = format!("GreenTriviaPiece {{ kind: {:?}, text: {:?} }}", kind, text);
        assert_eq!(debug_str, expected);
    }

    #[rstest]
    #[case::ascii(SyntaxKind(5), b"% display trivia", "% display trivia")]
    #[case::emoji(SyntaxKind(6), b"\xF0\x9F\x98\x80", "😀")]
    #[case::checkmark(SyntaxKind(7), b"\xE2\x9C\x94", "✔")]
    #[case::normal_ascii(SyntaxKind(8), b"normal ascii", "normal ascii")]
    #[case::copyright(SyntaxKind(9), b"\xC2\xA9 copyright", "© copyright")]
    #[case::invalid_utf8(SyntaxKind(10), b"\xFF\xFEinvalid", "\\xFF\\xFEinvalid")]
    #[case::printable_ascii(SyntaxKind(11), b"!AZaz09~", "!AZaz09~")]
    #[case::space(SyntaxKind(12), b" ", " ")]
    #[case::newline(SyntaxKind(13), b"\n", "\n")]
    #[case::carriage_return(SyntaxKind(14), b"\r", "\r")]
    #[case::tab(SyntaxKind(15), b"\t", "\t")]
    #[case::non_printable(SyntaxKind(16), b"\x01\x7F", "\u{1}\u{7f}")]
    #[case::invalid_ascii(SyntaxKind(17), b"\xFF!AZaz09~\xFE", "\\xFF!AZaz09~\\xFE")]
    #[case::invalid_space(SyntaxKind(18), b"\xFF \xFE", "\\xFF \\xFE")]
    #[case::invalid_newline(SyntaxKind(19), b"\xFF\n\xFE", "\\xFF\\n\\xFE")]
    #[case::invalid_cr(SyntaxKind(20), b"\xFF\r\xFE", "\\xFF\\r\\xFE")]
    #[case::invalid_tab(SyntaxKind(21), b"\xFF\t\xFE", "\\xFF\\t\\xFE")]
    #[case::invalid_non_printable(SyntaxKind(22), b"\xFF\xFE\x01\x7F", "\\xFF\\xFE\\x01\\x7F")]
    fn test_fmt_display_when_valid_piece_expect_text_output(#[case] kind: SyntaxKind, #[case] text: &'static [u8], #[case] expected: &'static str) {
        let trivia_piece = GreenTriviaPiece::new(kind, text);
        let display_str = format!("{}", trivia_piece);
        assert_eq!(display_str, expected);
    }

    #[rstest]
    fn test_borrow_for_trivia_piece() {
        use std::borrow::Borrow;
        let kind = SyntaxKind(42);
        let text = b"borrow trivia";
        let trivia_piece = GreenTriviaPiece::new(kind, text);
        let borrowed: &GreenTriviaPieceData = trivia_piece.borrow();
        assert_eq!(borrowed.kind(), kind);
        assert_eq!(borrowed.text(), text);
    }
}
