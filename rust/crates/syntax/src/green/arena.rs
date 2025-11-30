use bumpalo::Bump;
use hashbrown::HashMap;
use triomphe::UniqueArc;

use crate::{
    DiagnosticInfo, SyntaxKind,
    green::{
        GreenElement,
        node::{GreenChild, GreenNodeHead, GreenNodeInTree},
        token::{GreenTokenHead, GreenTokenInTree},
        trivia::{GreenTriviaHead, GreenTriviaInTree, GreenTriviaListHead, GreenTriviaListInTree},
    },
};

pub(crate) struct GreenTree {
    arena: Bump,
    diagnostics: HashMap<GreenElement, Vec<DiagnosticInfo>>,
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
    pub(super) fn alloc_node(&mut self, kind: SyntaxKind, text_len: u32, children_len: u16, children: impl Iterator<Item = GreenChild>) -> GreenNodeInTree {
        // SAFETY: We have mutable access.
        unsafe { self.alloc_node_unchecked(kind, text_len, children_len, children) }
    }

    #[inline]
    pub(super) fn alloc_token(
        &mut self,
        kind: SyntaxKind,
        text: &[u8],
        leading_trivia: GreenTriviaListInTree,
        trailing_trivia: GreenTriviaListInTree,
    ) -> GreenTokenInTree {
        // SAFETY: We have mutable access.
        unsafe { self.alloc_token_unchecked(kind, text, leading_trivia, trailing_trivia) }
    }

    #[inline]
    pub(super) fn alloc_trivia(&mut self, kind: SyntaxKind, text: &[u8]) -> GreenTriviaInTree {
        // SAFETY: We have mutable access.
        unsafe { self.alloc_trivia_unchecked(kind, text) }
    }

    #[inline]
    pub(super) fn alloc_trivia_list(&mut self, pieces: &[GreenTriviaInTree]) -> GreenTriviaListInTree {
        // SAFETY: We have mutable access.
        unsafe { self.alloc_trivia_list_unchecked(pieces) }
    }

    /// # Safety
    ///
    /// You must ensure there is no concurrent allocation.
    #[inline]
    pub(crate) unsafe fn alloc_node_unchecked(
        &self,
        kind: SyntaxKind,
        text_len: u32,
        children_len: u16,
        mut children: impl Iterator<Item = GreenChild>,
    ) -> GreenNodeInTree {
        assert!(children_len as usize <= u16::MAX as usize, "too many children");
        let layout = GreenNodeHead::layout(children_len);
        let token = self.arena.alloc_layout(layout);
        let node = GreenNodeInTree { data: token.cast() };
        // SAFETY: The node is allocated, we don't need it to be initialized for the writing.
        unsafe {
            node.header_ptr_mut().write(GreenNodeHead::new(kind, text_len, children_len));
            let children_ptr = node.children_ptr_mut();
            for child_idx in 0..children_len {
                children_ptr.add(child_idx.into()).write(children.next().expect("too few children"));
            }
        }
        debug_assert!(children.next().is_none(), "too many children");
        node
    }

    /// # Safety
    ///
    /// You must ensure there is no concurrent allocation.
    unsafe fn alloc_token_unchecked(
        &self,
        kind: SyntaxKind,
        text: &[u8],
        leading_trivia: GreenTriviaListInTree,
        trailing_trivia: GreenTriviaListInTree,
    ) -> GreenTokenInTree {
        assert!(text.len() <= u32::MAX as usize, "token text too long");

        let layout = GreenTokenHead::layout(text.len() as u32);
        let token = self.arena.alloc_layout(layout);
        let token = GreenTokenInTree { data: token.cast() };
        let full_width = leading_trivia.full_width() + text.len() as u32 + trailing_trivia.full_width();

        // SAFETY: The token is allocated, we don't need it to be initialized for the writing.
        unsafe {
            token
                .header_ptr_mut()
                .write(GreenTokenHead::new(kind, full_width, leading_trivia, trailing_trivia));
            token.bytes_ptr_mut().copy_from_nonoverlapping(text.as_ptr(), text.len());
        }
        token
    }

    /// # Safety
    ///
    /// You must ensure there is no concurrent allocation.
    unsafe fn alloc_trivia_unchecked(&self, kind: SyntaxKind, text: &[u8]) -> GreenTriviaInTree {
        assert!(text.len() <= u16::MAX.into(), "trivia text too long");

        let layout = GreenTriviaHead::layout(text.len() as u16);
        let trivia = self.arena.alloc_layout(layout);
        let trivia = GreenTriviaInTree { data: trivia.cast() };

        // SAFETY: The trivia is allocated, we don't need it to be initialized for the writing.
        unsafe {
            trivia.header_ptr_mut().write(GreenTriviaHead::new(kind, text));
            trivia.bytes_ptr_mut().copy_from_nonoverlapping(text.as_ptr(), text.len());
        }
        trivia
    }

    // # Safety
    ///
    /// You must ensure there is no concurrent allocation.
    unsafe fn alloc_trivia_list_unchecked(&self, pieces: &[GreenTriviaInTree]) -> GreenTriviaListInTree {
        assert!(pieces.len() <= u16::MAX.into(), "too many trivia pieces");
        let full_width = pieces.iter().map(|p| p.full_width() as u32).sum::<u32>();
        let layout = GreenTriviaListHead::layout(pieces.len() as u16);
        let trivia_list = self.arena.alloc_layout(layout);
        let trivia_list = GreenTriviaListInTree { data: trivia_list.cast() };

        // SAFETY: The trivia list is allocated, we don't need it to be initialized for the writing.
        unsafe {
            trivia_list
                .header_ptr_mut()
                .write(GreenTriviaListHead::new(full_width as u32, pieces.len() as u16));

            trivia_list.pieces_ptr_mut().copy_from_nonoverlapping(pieces.as_ptr(), pieces.len());
        }
        trivia_list
    }
}
