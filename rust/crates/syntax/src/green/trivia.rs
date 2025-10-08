use std::{ptr::NonNull, sync::Arc};

use countme::Count;

use crate::{SyntaxKind, green::arena::GreenTree};

#[derive(Debug, Default)]
pub struct GreenTriviaList<'trivia> {
    pieces: Vec<GreenTrivia<'trivia>>,
    full_width: usize,
}

#[repr(packed)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct GreenTriviaHead {
    kind: SyntaxKind,       // 2 bytes
    full_width: u16,        // 2 bytes
    _c: Count<GreenTrivia>, // 0 bytes
}

/// This is used to store the trivia in the arena.
/// The actual text is stored inline after the head.
#[derive(Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub(super) struct GreenTriviaData {
    head: GreenTriviaHead,
    text: [u8; 0],
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GreenTrivia {
    token: GreenTriviaInTree,
    arena: Arc<GreenTree>,
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
