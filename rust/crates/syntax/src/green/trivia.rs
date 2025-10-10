use std::{alloc::Layout, fmt, ptr::NonNull, slice};

use countme::Count;

use crate::{SyntaxKind, green::arena::GreenTree};

#[derive(Debug, Default)]
pub struct GreenTriviaList {
    pieces: Vec<GreenTrivia>,
    len: usize,
}

#[repr(C)] // TODO: add a test to ensure size and alignment
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
#[repr(C)] // TODO: add a test to ensure size and alignment
pub(super) struct GreenTriviaData {
    head: GreenTriviaHead, // 4 bytes
    text: [u8; 0],         // 0 bytes, actual text is stored inline after this struct
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GreenTrivia {
    /// INVARIANT: This points at a valid `GreenTriviaInTree` then `str` with len `text_len`,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTriviaData>,
}

impl GreenTrivia {
    /// Creates a freestanding token.
    ///
    /// Note: this is expensive. Prefer building your token directly into the tree with [`GreenNodeBuilder`].
    ///
    /// [`GreenNodeBuilder`]: crate::GreenNodeBuilder
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        let mut arena = GreenTree::new();
        arena.alloc_trivia(kind, text)
    }

    #[inline]
    fn header(&self) -> &GreenTriviaHead {
        // SAFETY: `data`'s invariant.
        unsafe { &*self.header_ptr_mut() }
    }

    #[inline]
    pub fn text(&self) -> &str {
        // SAFETY: `data`'s invariant.
        unsafe { std::str::from_utf8_unchecked(slice::from_raw_parts(self.text_ptr_mut(), self.header().full_width.into())) }
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
        f.debug_struct("GreenTrivia").field("kind", &self.kind()).field("text", &self.text()).finish()
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
        assert_eq!(std::mem::size_of::<GreenTriviaHead>(), 4);
        assert_eq!(std::mem::align_of::<GreenTriviaHead>(), 2);
        assert_eq!(std::mem::size_of::<GreenTriviaData>(), 4);
        assert_eq!(std::mem::align_of::<GreenTriviaData>(), 2);
    }
}
