use std::{alloc::Layout, fmt, ptr::NonNull, slice};

use countme::Count;

use crate::SyntaxKind;

#[repr(C, align(8))]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct GreenTriviaListHead {
    full_width: u32,            // 4 bytes
    pieces_len: u16,            // 2 bytes
    _padding: u16,              // 2 bytes padding to ensure 8-byte total size
    _c: Count<GreenTriviaList>, // 0 bytes
}

impl GreenTriviaListHead {
    #[inline]
    pub(super) fn new(full_width: u32, pieces_len: u16) -> Self {
        Self {
            full_width,
            pieces_len,
            _padding: 0,
            _c: Count::new(),
        }
    }

    #[inline]
    pub(super) fn layout(pieces_len: u16) -> Layout {
        Layout::new::<GreenTriviaListHead>()
            .extend(Layout::array::<GreenTrivia>(pieces_len as usize).expect("too big node"))
            .expect("too big node")
            .0
            .pad_to_align()
    }
}

/// This is used to store the trivia list in the arena.
/// The actual pieces are stored inline after the head.
#[repr(C)]
pub(super) struct GreenTriviaListData {
    head: GreenTriviaListHead, // 8 bytes (with explicit 8-byte alignment)
    pieces: [GreenTrivia; 0],  // 0 bytes, actual pieces are stored inline after this struct
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct GreenTriviaList {
    /// INVARIANT: This points at a valid `GreenTriviaListData` followed by `pieces_len` `GreenTrivia`s,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTriviaListData>,
}

impl GreenTriviaList {
    /// Returns the full bytes of all trivia pieces concatenated
    #[inline]
    pub fn full_bytes(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(self.full_width() as usize);
        for piece in self.pieces() {
            output.extend_from_slice(piece.bytes());
        }
        output
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        self.header().full_width
    }

    #[inline]
    pub fn pieces(&self) -> &[GreenTrivia] {
        // SAFETY: `data`'s invariant.
        unsafe { slice::from_raw_parts(self.pieces_ptr_mut().cast::<GreenTrivia>(), self.header().pieces_len.into()) }
    }

    #[inline]
    fn header(&self) -> &GreenTriviaListHead {
        // SAFETY: `data`'s invariant.
        unsafe { &*self.header_ptr_mut() }
    }

    /// Does not require the pointer to be valid.
    #[inline]
    pub(super) fn header_ptr_mut(&self) -> *mut GreenTriviaListHead {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { &raw mut (*self.data.as_ptr()).head }
    }

    #[inline]
    pub(super) fn pieces_ptr_mut(&self) -> *mut GreenTrivia {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { (&raw mut (*self.data.as_ptr()).pieces).cast::<GreenTrivia>() }
    }
}

impl PartialEq for GreenTriviaList {
    fn eq(&self, other: &Self) -> bool {
        // Early exit on different widths for performance
        self.full_width() == other.full_width() && self.pieces() == other.pieces()
    }
}

impl Eq for GreenTriviaList {}

impl fmt::Debug for GreenTriviaList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTriviaList").field("full_width", &self.full_width()).finish()
    }
}

impl fmt::Display for GreenTriviaList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for piece in self.pieces() {
            write!(f, "{}", piece)?;
        }
        Ok(())
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenTriviaList {}
unsafe impl Sync for GreenTriviaList {}

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct GreenTriviaHead {
    kind: SyntaxKind,       // 2 bytes
    full_width: u16,        // 2 bytes
    _c: Count<GreenTrivia>, // 0 bytes
}

impl GreenTriviaHead {
    #[inline]
    pub(super) fn new(kind: SyntaxKind, text: &[u8]) -> Self {
        Self {
            kind,
            full_width: text.len() as u16,
            _c: Count::new(),
        }
    }

    #[inline]
    pub(super) fn layout(text_len: u16) -> Layout {
        Layout::new::<GreenTriviaHead>()
            .extend(Layout::array::<u8>(text_len as usize).expect("too big node"))
            .expect("too big node")
            .0
            .pad_to_align()
    }
}

/// This is used to store the trivia in the arena.
/// The actual text is stored inline after the head.
#[repr(C)]
pub(super) struct GreenTriviaData {
    head: GreenTriviaHead, // 4 bytes
    text: [u8; 0],         // 0 bytes, actual text is stored inline after this struct
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct GreenTrivia {
    /// INVARIANT: This points at a valid `GreenTriviaData` followed by `text_len` bytes,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTriviaData>,
}

impl GreenTrivia {
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        // SAFETY: `data`'s invariant.
        unsafe { slice::from_raw_parts(self.bytes_ptr_mut(), self.header().full_width.into()) }
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.header().kind
    }

    #[inline]
    pub fn full_width(&self) -> u16 {
        self.header().full_width
    }

    #[inline]
    fn header(&self) -> &GreenTriviaHead {
        // SAFETY: `data`'s invariant.
        unsafe { &*self.header_ptr_mut() }
    }

    /// Does not require the pointer to be valid.
    #[inline]
    pub(super) fn header_ptr_mut(&self) -> *mut GreenTriviaHead {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { &raw mut (*self.data.as_ptr()).head }
    }

    #[inline]
    pub(super) fn bytes_ptr_mut(&self) -> *mut u8 {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { (&raw mut (*self.data.as_ptr()).text).cast::<u8>() }
    }
}

impl PartialEq for GreenTrivia {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.bytes() == other.bytes()
    }
}

impl Eq for GreenTrivia {}

impl fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: `text` is guaranteed to be valid UTF-8 by the node invariant.
        let text = unsafe { std::str::from_utf8_unchecked(self.bytes()) };
        f.debug_struct("GreenTrivia").field("kind", &self.kind()).field("text", &text).finish()
    }
}

impl fmt::Display for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { std::str::from_utf8_unchecked(self.bytes()) })
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenTrivia {}
unsafe impl Sync for GreenTrivia {}

#[cfg(test)]
mod memory_layout_tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenTriviaHead>(), 4); // 4 bytes + 0 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaHead>(), 2); // 2 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTriviaData>(), 4); // 4 bytes + 0 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaData>(), 2); // 2 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTriviaListHead>(), 8); // 8 bytes (aligned to 8)
        assert_eq!(std::mem::align_of::<GreenTriviaListHead>(), 8); // 8 bytes alignment (explicit)

        assert_eq!(std::mem::size_of::<GreenTriviaListData>(), 8); // 8 bytes + 0 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaListData>(), 8); // 8 bytes alignment
    }
}

#[cfg(test)]
mod trivia_tests {
    use rstest::rstest;

    use crate::green::arena::GreenTree;

    use super::*;

    const WHITESPACE_KIND: SyntaxKind = SyntaxKind(1);

    #[rstest]
    fn test_kind() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        assert_eq!(trivia.kind(), WHITESPACE_KIND);
    }

    #[rstest]
    fn test_bytes() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b"   ");
        assert_eq!(trivia.bytes(), b"   ");
    }

    #[rstest]
    fn test_full_width() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b"\n\t");
        assert_eq!(trivia.full_width(), 2);
    }

    #[rstest]
    fn test_eq() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia3 = arena.alloc_trivia(WHITESPACE_KIND, b"\n");

        assert_eq!(trivia1, trivia2);
        assert_ne!(trivia1, trivia3);
    }

    #[rstest]
    fn test_display() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b" \n\t");
        assert_eq!(trivia.to_string(), " \n\t");
    }

    #[rstest]
    fn test_debug() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b" \n\t");
        let debug_str = format!("{:?}", trivia);
        assert_eq!(debug_str, "GreenTrivia { kind: SyntaxKind(1), text: \" \\n\\t\" }");
    }
}

#[cfg(test)]
mod trivia_list_tests {
    use rstest::rstest;

    use crate::green::arena::GreenTree;

    use super::*;

    const WHITESPACE_KIND: SyntaxKind = SyntaxKind(1);
    const COMMENT_KIND: SyntaxKind = SyntaxKind(2);

    #[rstest]
    fn test_full_width() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2]);
        assert_eq!(trivia_list.full_width(), 10);
    }

    #[rstest]
    fn test_pieces() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2]);
        let pieces = trivia_list.pieces();
        assert_eq!(pieces, &[trivia1, trivia2]);
    }

    #[rstest]
    fn test_eq() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia_list1 = arena.alloc_trivia_list(&[trivia1, trivia2]);
        let trivia_list2 = arena.alloc_trivia_list(&[trivia1, trivia2]);
        let trivia_list3 = arena.alloc_trivia_list(&[trivia2, trivia1]);

        assert_eq!(trivia_list1, trivia_list2);
        assert_ne!(trivia_list1, trivia_list3);
    }

    #[rstest]
    fn test_display() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2]);
        assert_eq!(trivia_list.to_string(), " % comment");
    }

    #[rstest]
    fn test_debug() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2]);
        let debug_str = format!("{:?}", trivia_list);
        assert_eq!(debug_str, "GreenTriviaList { full_width: 10 }");
    }

    #[rstest]
    fn test_full_bytes_when_single_piece_expect_single_piece_bytes() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b"  \t");
        let trivia_list = arena.alloc_trivia_list(&[trivia]);
        assert_eq!(trivia_list.full_bytes(), b"  \t");
    }

    #[rstest]
    fn test_full_bytes_when_multiple_pieces_expect_concatenated_bytes() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia3 = arena.alloc_trivia(WHITESPACE_KIND, b"\n");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2, trivia3]);
        assert_eq!(trivia_list.full_bytes(), b" % comment\n");
    }

    #[rstest]
    fn test_full_bytes_when_empty_list_expect_empty_vec() {
        let mut arena = GreenTree::new();
        let trivia_list = arena.alloc_trivia_list(&[]);
        assert_eq!(trivia_list.full_bytes(), b"");
    }
}
