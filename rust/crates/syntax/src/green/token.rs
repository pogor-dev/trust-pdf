use std::{fmt, ptr::NonNull, slice};

use crate::{GreenTriviaList, SyntaxKind, green::arena::GreenTree};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct GreenTokenHead {
    leading_trivia: GreenTriviaList,  // 8 bytes
    trailing_trivia: GreenTriviaList, // 8 bytes
    full_width: u32,                  // 4 bytes
    kind: SyntaxKind,                 // 2 bytes
}

impl GreenTokenHead {
    #[inline]
    pub(super) fn new(leading: GreenTriviaList, trailing: GreenTriviaList, full_width: u32, kind: SyntaxKind) -> Self {
        Self {
            leading_trivia: leading,
            trailing_trivia: trailing,
            full_width,
            kind,
        }
    }

    #[inline]
    pub(super) fn layout(text_len: u32) -> std::alloc::Layout {
        std::alloc::Layout::new::<GreenTokenHead>()
            .extend(std::alloc::Layout::array::<u8>(text_len as usize).expect("too big token"))
            .expect("too big token")
            .0
            .pad_to_align()
    }
}

/// This is used to store the token in the arena.
/// The actual text is stored inline after the head.
#[derive(Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub(super) struct GreenTokenData {
    head: GreenTokenHead, // 24 bytes
    text: [u8; 0],        // 0 bytes, actual text is stored inline after this struct
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenToken {
    /// INVARIANT: This points at a valid `GreenTokenData` followed by `text_len` bytes,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTokenData>,
}

impl GreenToken {
    /// Creates a freestanding trivia.
    ///
    /// Note: this is expensive. Prefer building your trivia directly into the tree with [`GreenNodeBuilder`].
    ///
    /// [`GreenNodeBuilder`]: crate::GreenNodeBuilder
    #[inline]
    pub fn new(leading: GreenTriviaList, trailing: GreenTriviaList, kind: SyntaxKind, text: &[u8]) -> GreenToken {
        debug_assert!(text.len() <= u32::MAX as usize);
        let mut arena = GreenTree::new();
        arena.alloc_token(leading, trailing, kind, text)
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.header().kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        // SAFETY: `data`'s invariant.
        unsafe { slice::from_raw_parts(self.text_ptr_mut(), self.header().full_width as usize) }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        self.header().full_width
    }

    #[inline]
    fn header(&self) -> &GreenTokenHead {
        // SAFETY: The invariant on `data` ensures this is valid for reads.
        unsafe { &self.data.as_ref().head }
    }

    /// Does not require the pointer to be valid.
    #[inline]
    pub(super) fn header_ptr_mut(&self) -> *mut GreenTokenHead {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { &raw mut (*self.data.as_ptr()).head }
    }

    #[inline]
    pub(super) fn text_ptr_mut(&self) -> *mut u8 {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { (&raw mut (*self.data.as_ptr()).text).cast::<u8>() }
    }
}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("text", &String::from_utf8_lossy(self.text()))
            .finish()
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenToken {}
unsafe impl Sync for GreenToken {}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenTokenHead>(), 24); // 22 bytes + 2 bytes padding
        assert_eq!(std::mem::align_of::<GreenTokenHead>(), 8); // 8 bytes alignment
    }
}
