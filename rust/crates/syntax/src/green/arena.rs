use bumpalo::Bump;
use hashbrown::HashMap;
use triomphe::UniqueArc;

use crate::{
    DiagnosticInfo, GreenTrivia, GreenTriviaList, SyntaxKind,
    green::trivia::{GreenTriviaHead, GreenTriviaListHead},
};

pub(crate) struct GreenTree {
    arena: Bump,
    diagnostics: HashMap<*const u8, Vec<DiagnosticInfo>>,
}

// SAFETY: We only mutate when having mutable access, and mutating doesn't invalidate existing pointers.
unsafe impl Sync for GreenTree {}

impl GreenTree {
    // This needs to be inside `UniqueArc` because otherwise the `verify_origin()` comparisons
    // are messed up.
    #[inline]
    pub(crate) fn new() -> UniqueArc<Self> {
        UniqueArc::new(Self {
            arena: Bump::new(),
            diagnostics: HashMap::default(),
        })
    }

    #[inline]
    pub(super) fn alloc_trivia(&mut self, kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        // SAFETY: We have mutable access.
        unsafe { self.alloc_trivia_unchecked(kind, text) }
    }

    #[inline]
    pub(super) fn alloc_trivia_list(&mut self, pieces: &[GreenTrivia]) -> GreenTriviaList {
        // SAFETY: We have mutable access.
        unsafe { self.alloc_trivia_list_unchecked(pieces) }
    }

    /// # Safety
    ///
    /// You must ensure there is no concurrent allocation.
    unsafe fn alloc_trivia_unchecked(&self, kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        assert!(text.len() <= u16::MAX.into());

        let layout = GreenTriviaHead::layout(text.len());
        let trivia = self.arena.alloc_layout(layout);
        let trivia = GreenTrivia { data: trivia.cast() };

        // SAFETY: The trivia is allocated, we don't need it to be initialized for the writing.
        unsafe {
            trivia.header_ptr_mut().write(GreenTriviaHead::new(kind, text));
            trivia.text_ptr_mut().copy_from_nonoverlapping(text.as_ptr(), text.len());
        }
        trivia
    }

    // # Safety
    ///
    /// You must ensure there is no concurrent allocation.
    unsafe fn alloc_trivia_list_unchecked(&self, pieces: &[GreenTrivia]) -> GreenTriviaList {
        assert!(pieces.len() <= u16::MAX.into());
        let full_width = pieces.iter().map(|p| p.full_width() as usize).sum::<usize>();
        let layout = GreenTriviaListHead::layout(pieces.len());
        let trivia_list = self.arena.alloc_layout(layout);
        let trivia_list = GreenTriviaList { data: trivia_list.cast() };

        // SAFETY: The trivia list is allocated, we don't need it to be initialized for the writing.
        unsafe {
            trivia_list.header_ptr_mut().write(GreenTriviaListHead::new(full_width, pieces.len()));
            trivia_list.pieces_ptr_mut().copy_from_nonoverlapping(pieces.as_ptr(), pieces.len());
        }
        trivia_list
    }
}
