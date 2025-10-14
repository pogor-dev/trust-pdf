use std::{alloc::Layout, fmt, ptr::NonNull, slice};

use countme::Count;

use crate::{SyntaxKind, green::arena::GreenTree};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct GreenTriviaListHead {
    full_width: u32, // 4 bytes
    pieces_len: u16, // 2 bytes
    _c: Count<GreenTriviaList>,
}

impl GreenTriviaListHead {
    #[inline]
    pub(super) fn new(full_width: usize, pieces_len: usize) -> Self {
        Self {
            full_width: full_width as u32,
            pieces_len: pieces_len as u16,
            _c: Count::new(),
        }
    }

    #[inline]
    pub(super) fn layout(pieces_len: usize) -> Layout {
        Layout::new::<GreenTriviaListHead>()
            .extend(Layout::array::<GreenTrivia>(pieces_len).expect("too big node"))
            .expect("too big node")
            .0
            .pad_to_align()
    }
}

/// This is used to store the trivia list in the arena.
/// The actual pieces are stored inline after the head.
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct GreenTriviaListData {
    head: GreenTriviaListHead, // 6 bytes
    pieces: [GreenTrivia; 0],  // 0 bytes, actual pieces are stored inline after this struct
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTriviaList {
    /// INVARIANT: This points at a valid `GreenTriviaListData` followed by `pieces_len` `GreenTrivia`s,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTriviaListData>,
}

impl GreenTriviaList {
    /// Creates a freestanding trivia list.
    ///
    /// Note: this is expensive. Prefer building your trivia list directly into the tree with [`GreenNodeBuilder`].
    ///
    /// [`GreenNodeBuilder`]: crate::GreenNodeBuilder
    #[inline]
    pub fn new(pieces: &[GreenTrivia]) -> GreenTriviaList {
        debug_assert!(pieces.len() <= u16::MAX.into(), "too many trivia pieces");
        let mut arena = GreenTree::new();
        arena.alloc_trivia_list(pieces)
    }

    /// Creates a freestanding single trivia element.
    ///
    /// Note: this is expensive. Prefer building your trivia list directly into the tree with [`GreenNodeBuilder`].
    ///
    /// [`GreenNodeBuilder`]: crate::GreenNodeBuilder
    #[inline]
    pub fn new_single(kind: SyntaxKind, text: &[u8]) -> GreenTriviaList {
        Self::new(&[GreenTrivia::new(kind, text)])
    }

    #[inline]
    fn header(&self) -> &GreenTriviaListHead {
        // SAFETY: `data`'s invariant.
        unsafe { &*self.header_ptr_mut() }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        self.header().full_width as u32
    }

    #[inline]
    pub fn pieces(&self) -> &[GreenTrivia] {
        // SAFETY: `data`'s invariant.
        unsafe { slice::from_raw_parts(self.pieces_ptr_mut().cast::<GreenTrivia>(), self.header().pieces_len.into()) }
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

impl From<GreenTrivia> for GreenTriviaList {
    #[inline]
    fn from(trivia: GreenTrivia) -> Self {
        Self::new(&[trivia])
    }
}

impl fmt::Debug for GreenTriviaList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTriviaList").field("full_width", &self.full_width()).finish()
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenTriviaList {}
unsafe impl Sync for GreenTriviaList {}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
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
    pub(super) fn layout(text_len: usize) -> Layout {
        Layout::new::<GreenTriviaHead>()
            .extend(Layout::array::<u8>(text_len).expect("too big node"))
            .expect("too big node")
            .0
            .pad_to_align()
    }
}

/// This is used to store the trivia in the arena.
/// The actual text is stored inline after the head.
#[derive(Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub(super) struct GreenTriviaData {
    head: GreenTriviaHead, // 4 bytes
    text: [u8; 0],         // 0 bytes, actual text is stored inline after this struct
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTrivia {
    /// INVARIANT: This points at a valid `GreenTriviaInTree` followed by `text_len` bytes,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTriviaData>,
}

impl GreenTrivia {
    /// Creates a freestanding trivia.
    ///
    /// Note: this is expensive. Prefer building your trivia directly into the tree with [`GreenNodeBuilder`].
    ///
    /// [`GreenNodeBuilder`]: crate::GreenNodeBuilder
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        debug_assert!(text.len() <= u16::MAX.into(), "text too long");
        let mut arena = GreenTree::new();
        arena.alloc_trivia(kind, text)
    }

    #[inline]
    fn header(&self) -> &GreenTriviaHead {
        // SAFETY: `data`'s invariant.
        unsafe { &*self.header_ptr_mut() }
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        // SAFETY: `data`'s invariant.
        unsafe { slice::from_raw_parts(self.text_ptr_mut(), self.header().full_width.into()) }
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.header().kind
    }

    #[inline]
    pub fn full_width(&self) -> usize {
        self.header().full_width.into()
    }

    /// Does not require the pointer to be valid.
    #[inline]
    pub(super) fn header_ptr_mut(&self) -> *mut GreenTriviaHead {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { &raw mut (*self.data.as_ptr()).head }
    }

    #[inline]
    pub(super) fn text_ptr_mut(&self) -> *mut u8 {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { (&raw mut (*self.data.as_ptr()).text).cast::<u8>() }
    }
}

impl fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: `text` is guaranteed to be valid UTF-8 by the node invariant.
        let text = unsafe { std::str::from_utf8_unchecked(self.text()) };
        f.debug_struct("GreenTrivia").field("kind", &self.kind()).field("text", &text).finish()
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenTrivia {}
unsafe impl Sync for GreenTrivia {}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenTriviaHead>(), 4); // 4 bytes + 0 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaHead>(), 2); // 2 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTriviaData>(), 4); // 4 bytes + 0 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaData>(), 2); // 2 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTriviaListHead>(), 8); // 6 bytes + 2 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaListHead>(), 4); // 4 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTriviaListData>(), 8); // 8 bytes + 0 bytes padding
        assert_eq!(std::mem::align_of::<GreenTriviaListData>(), 8); // 8 bytes alignment
    }
}
