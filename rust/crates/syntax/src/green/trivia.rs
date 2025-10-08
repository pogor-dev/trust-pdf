use std::ptr::NonNull;

use countme::Count;

use crate::SyntaxKind;

#[derive(Debug, Default)]
pub struct GreenTriviaList<'trivia> {
    pieces: Vec<GreenTrivia<'trivia>>,
    full_width: usize,
}

#[repr(packed)] // Makes this 4 bytes instead of 8 bytes.
pub(super) struct GreenTriviaHead {
    kind: SyntaxKind,       // 2 bytes
    full_width: u16,        // 2 bytes
    _c: Count<GreenTrivia>, // 0 bytes
}

/// This is used to store the trivia in the arena.
/// The actual text is stored inline after the head.
pub(super) struct GreenTriviaData {
    head: GreenTriviaHead,
    text: [u8; 0],
}

#[derive(Clone, Copy)]
pub(crate) struct GreenTriviaInTree {
    /// INVARIANT: This points at a valid `GreenTriviaInTree` then `str` with len `text_len`,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTriviaData>,
}

impl GreenTriviaInTree {
    pub(crate) unsafe fn alloc_token_unchecked(arena: &bumpalo::Bump, kind: SyntaxKind, text: &[u8]) -> GreenTriviaInTree {}
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenTriviaInTree {}
unsafe impl Sync for GreenTriviaInTree {}

// TODO: no need for memory allocation for EOL, space
// TODO: this can't be done here if we want to keep this crate abstracted like rowan,
// TODO: we don't know the SyntaxKind here
