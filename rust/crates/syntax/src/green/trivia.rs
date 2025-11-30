use std::{alloc::Layout, fmt, hash, ptr::NonNull, slice};

use countme::Count;
use triomphe::Arc;

use crate::{SyntaxKind, green::arena::GreenTree};

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
            .extend(Layout::array::<GreenTriviaInTree>(pieces_len as usize).expect("too big node"))
            .expect("too big node")
            .0
            .pad_to_align()
    }
}

/// This is used to store the trivia list in the arena.
/// The actual pieces are stored inline after the head.
#[repr(C)]
pub(super) struct GreenTriviaListData {
    head: GreenTriviaListHead,      // 8 bytes (with explicit 8-byte alignment)
    pieces: [GreenTriviaInTree; 0], // 0 bytes, actual pieces are stored inline after this struct
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
    pub fn pieces(&self) -> &[GreenTriviaInTree] {
        // SAFETY: `data`'s invariant.
        unsafe { slice::from_raw_parts(self.pieces_ptr_mut().cast::<GreenTriviaInTree>(), self.header().pieces_len.into()) }
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
    pub(super) fn pieces_ptr_mut(&self) -> *mut GreenTriviaInTree {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { (&raw mut (*self.data.as_ptr()).pieces).cast::<GreenTriviaInTree>() }
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
    kind: SyntaxKind,             // 2 bytes
    full_width: u16,              // 2 bytes
    _c: Count<GreenTriviaInTree>, // 0 bytes
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
pub struct GreenTriviaInTree {
    /// INVARIANT: This points at a valid `GreenTriviaData` followed by `text_len` bytes,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTriviaData>,
}

impl GreenTriviaInTree {
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

    #[inline]
    pub(crate) fn to_green_trivia(self, arena: Arc<GreenTree>) -> GreenTrivia {
        GreenTrivia { trivia: self, _arena: arena }
    }
}

impl PartialEq for GreenTriviaInTree {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.bytes() == other.bytes()
    }
}

impl Eq for GreenTriviaInTree {}

impl hash::Hash for GreenTriviaInTree {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.kind().hash(state);
        self.bytes().hash(state);
    }
}

impl fmt::Debug for GreenTriviaInTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: `text` is guaranteed to be valid UTF-8 by the node invariant.
        let text = unsafe { std::str::from_utf8_unchecked(self.bytes()) };
        f.debug_struct("GreenTrivia").field("kind", &self.kind()).field("text", &text).finish()
    }
}

impl fmt::Display for GreenTriviaInTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { std::str::from_utf8_unchecked(self.bytes()) })
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenTriviaInTree {}
unsafe impl Sync for GreenTriviaInTree {}

/// Leaf node in the immutable tree.
#[derive(Clone)]
pub struct GreenTrivia {
    pub(super) trivia: GreenTriviaInTree,
    pub(super) _arena: Arc<GreenTree>,
}

impl GreenTrivia {
    /// Kind of this Trivia.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.trivia.kind()
    }

    /// The bytes of this Trivia.
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        self.trivia.bytes()
    }

    /// The full width of this Trivia.
    #[inline]
    pub fn full_width(&self) -> u16 {
        self.trivia.full_width()
    }

    #[inline]
    pub(crate) fn into_raw_parts(self) -> (GreenTriviaInTree, Arc<GreenTree>) {
        (self.trivia, self._arena)
    }
}

impl PartialEq for GreenTrivia {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.trivia == other.trivia
    }
}

impl Eq for GreenTrivia {}

impl hash::Hash for GreenTrivia {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.trivia.hash(state);
    }
}

impl fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.trivia, f)
    }
}

impl fmt::Display for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.trivia, f)
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenTriviaHead>(), 4); // 4 bytes + 0 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaHead>(), 2); // 2 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTriviaData>(), 4); // 4 bytes + 0 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaData>(), 2); // 2 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTrivia>(), 16); // 16 bytes
        assert_eq!(std::mem::align_of::<GreenTrivia>(), 8); // 8 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTriviaListHead>(), 8); // 8 bytes (aligned to 8)
        assert_eq!(std::mem::align_of::<GreenTriviaListHead>(), 8); // 8 bytes alignment (explicit)

        assert_eq!(std::mem::size_of::<GreenTriviaListData>(), 8); // 8 bytes + 0 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaListData>(), 8); // 8 bytes alignment
    }
}

#[cfg(test)]
mod trivia_tests {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use super::*;
    use crate::green::arena::GreenTree;

    const WHITESPACE_KIND: SyntaxKind = SyntaxKind(1);
    const COMMENT_KIND: SyntaxKind = SyntaxKind(2);

    #[test]
    fn test_kind() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b" ").to_green_trivia(arena.shareable());
        assert_eq!(trivia.kind(), WHITESPACE_KIND);
    }

    #[test]
    fn test_bytes() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b"   ").to_green_trivia(arena.shareable());
        assert_eq!(trivia.bytes(), b"   ");
    }

    #[test]
    fn test_full_width() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b"\n\t").to_green_trivia(arena.shareable());
        assert_eq!(trivia.full_width(), 2);
    }

    #[test]
    fn test_into_raw_parts() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b" ").to_green_trivia(arena.shareable());
        let (trivia_in_tree, _) = trivia.into_raw_parts();

        assert_eq!(trivia_in_tree.kind(), WHITESPACE_KIND);
        assert_eq!(trivia_in_tree.bytes(), b" ");
    }

    #[test]
    fn test_hash() {
        let cases = [
            (COMMENT_KIND, b"% Comment".to_vec(), COMMENT_KIND, b"% Comment".to_vec(), true),
            (COMMENT_KIND, b"% Comment".to_vec(), WHITESPACE_KIND, b"% Comment".to_vec(), false),
            (COMMENT_KIND, b"% Comment".to_vec(), COMMENT_KIND, b"% Different".to_vec(), false),
        ];

        for (kind1, text1, kind2, text2, should_be_equal) in cases {
            let mut arena = GreenTree::new();

            let trivia1 = arena.alloc_trivia(kind1, text1.as_slice());
            let trivia2 = arena.alloc_trivia(kind2, text2.as_slice());
            let shareable = arena.shareable();
            let trivia1 = trivia1.to_green_trivia(shareable.clone());
            let trivia2 = trivia2.to_green_trivia(shareable);

            let mut hasher1 = DefaultHasher::new();
            trivia1.hash(&mut hasher1);
            let hash1 = hasher1.finish();

            let mut hasher2 = DefaultHasher::new();
            trivia2.hash(&mut hasher2);
            let hash2 = hasher2.finish();

            if should_be_equal {
                assert_eq!(hash1, hash2);
            } else {
                assert_ne!(hash1, hash2);
            }
        }
    }

    #[test]
    fn test_eq() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia3 = arena.alloc_trivia(WHITESPACE_KIND, b"\n");

        let shareable = arena.shareable();
        let trivia1 = trivia1.to_green_trivia(shareable.clone());
        let trivia2 = trivia2.to_green_trivia(shareable.clone());
        let trivia3 = trivia3.to_green_trivia(shareable);

        assert_eq!(trivia1, trivia2);
        assert_ne!(trivia1, trivia3);
    }

    #[test]
    fn test_display() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b" \n\t").to_green_trivia(arena.shareable());
        assert_eq!(trivia.to_string(), " \n\t");
    }

    #[test]
    fn test_debug() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b" \n\t").to_green_trivia(arena.shareable());
        let debug_str = format!("{:?}", trivia);
        assert_eq!(debug_str, "GreenTrivia { kind: SyntaxKind(1), text: \" \\n\\t\" }");
    }
}

#[cfg(test)]
mod trivia_list_tests {

    use crate::green::arena::GreenTree;

    use super::*;

    const WHITESPACE_KIND: SyntaxKind = SyntaxKind(1);
    const COMMENT_KIND: SyntaxKind = SyntaxKind(2);

    #[test]
    fn test_full_width() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2]);
        assert_eq!(trivia_list.full_width(), 10);
    }

    #[test]
    fn test_pieces() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2]);
        let pieces = trivia_list.pieces();
        assert_eq!(pieces, &[trivia1, trivia2]);
    }

    #[test]
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

    #[test]
    fn test_display() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2]);
        assert_eq!(trivia_list.to_string(), " % comment");
    }

    #[test]
    fn test_debug() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2]);
        let debug_str = format!("{:?}", trivia_list);
        assert_eq!(debug_str, "GreenTriviaList { full_width: 10 }");
    }

    #[test]
    fn test_full_bytes_when_single_piece_expect_single_piece_bytes() {
        let mut arena = GreenTree::new();
        let trivia = arena.alloc_trivia(WHITESPACE_KIND, b"  \t");
        let trivia_list = arena.alloc_trivia_list(&[trivia]);
        assert_eq!(trivia_list.full_bytes(), b"  \t");
    }

    #[test]
    fn test_full_bytes_when_multiple_pieces_expect_concatenated_bytes() {
        let mut arena = GreenTree::new();
        let trivia1 = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let trivia2 = arena.alloc_trivia(COMMENT_KIND, b"% comment");
        let trivia3 = arena.alloc_trivia(WHITESPACE_KIND, b"\n");
        let trivia_list = arena.alloc_trivia_list(&[trivia1, trivia2, trivia3]);
        assert_eq!(trivia_list.full_bytes(), b" % comment\n");
    }

    #[test]
    fn test_full_bytes_when_empty_list_expect_empty_vec() {
        let mut arena = GreenTree::new();
        let trivia_list = arena.alloc_trivia_list(&[]);
        assert_eq!(trivia_list.full_bytes(), b"");
    }
}
