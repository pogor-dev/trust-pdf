//! Green trivia representation with inline PDF trivia bytes.
//!
//! This variant stores per-instance trivia text bytes inline in the green node
//! tail. Trivia text is read from the inline byte slice provided by callers.

use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    arc::{Arc, HeaderSlice, ThinArc},
    syntax::green::{diagnostics, flags::GreenFlags},
};
use countme::Count;

use crate::GreenDiagnostic;
use crate::SyntaxKind;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTriviaHead {
    kind: SyntaxKind,       // 2 bytes
    flags: GreenFlags,      // 1 byte
    _c: Count<GreenTrivia>, // 0 bytes
}

/// Borrowed trivia view with inline trivia text.
#[repr(transparent)]
pub(crate) struct GreenTriviaData {
    data: ReprThin,
}

impl GreenTriviaData {
    /// Kind of this trivia.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Text of this trivia.
    #[inline]
    pub fn text(&self) -> &[u8] {
        self.data.slice()
    }

    /// Returns the length of the text covered by this trivia.
    #[inline]
    pub fn width(&self) -> u8 {
        self.data.slice().len() as u8
    }

    /// Returns the flags of this trivia.
    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }
}

impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl fmt::Display for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.text();
        let text_str = String::from_utf8_lossy(text);

        f.debug_struct("GreenTrivia")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .finish()
    }
}

/// Leaf node in the immutable tree.
///
/// Represents trivia with caller-provided text bytes stored inline.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, u8>,
}

impl GreenTrivia {
    /// Creates new trivia.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        Self::create_full(kind, text, GreenFlags::IS_NOT_MISSING, Vec::new())
    }

    #[inline]
    pub fn new_with_diagnostic(kind: SyntaxKind, text: &[u8], diagnostics: Vec<GreenDiagnostic>) -> GreenTrivia {
        Self::create_full(kind, text, GreenFlags::IS_NOT_MISSING, diagnostics)
    }

    #[inline]
    fn create_full(kind: SyntaxKind, text: &[u8], base_flags: GreenFlags, diagnostics: Vec<GreenDiagnostic>) -> GreenTrivia {
        let has_diagnostics = !diagnostics.is_empty();
        let flags = match has_diagnostics {
            true => base_flags | GreenFlags::CONTAINS_DIAGNOSTIC,
            false => base_flags,
        };

        let head = GreenTriviaHead { kind, flags, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        let trivia = GreenTrivia { ptr };

        if has_diagnostics {
            let key = trivia.diagnostics_key();
            diagnostics::insert_diagnostics(key, diagnostics);
        }

        trivia
    }
}

impl_green_boilerplate!(GreenTriviaHead, GreenTriviaData, GreenTrivia, u8);

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use crate::arc::{ArcInner, HeaderSlice};
    use std::mem::offset_of;

    fn expected_heap_allocation_size(text_len: usize) -> usize {
        type ThinRepr = ArcInner<HeaderSlice<GreenTriviaHead, [u8; 0]>>;

        // Mirror ThinArc::from_header_and_iter allocation math:
        // slice_offset + payload, rounded up to allocation alignment.
        let inner_to_data_offset = offset_of!(ThinRepr, data);
        // `slice` is private to `arc.rs`; for `[u8; 0]` use the sized prefix.
        let data_to_slice_offset = std::mem::size_of::<HeaderSlice<GreenTriviaHead, [u8; 0]>>();
        let slice_offset = inner_to_data_offset + data_to_slice_offset;

        let usable_size = slice_offset.checked_add(text_len).expect("size overflows");
        let align = std::mem::align_of::<ThinRepr>();
        usable_size.wrapping_add(align - 1) & !(align - 1)
    }

    #[test]
    fn test_green_trivia_head_memory_layout() {
        // GreenTriviaHead: kind (2 bytes) + flags (1 byte) + _c (0 bytes)
        // Expected: 2 + 1 + 1 padding for alignment = 4 bytes
        assert_eq!(std::mem::size_of::<GreenTriviaHead>(), 4);
        assert_eq!(std::mem::align_of::<GreenTriviaHead>(), 2);
    }

    #[test]
    fn test_green_trivia_data_memory_layout() {
        // GreenTriviaData on 64-bit targets:
        // header (4 bytes) + padding (4 bytes) + length (8 bytes) = 16 bytes
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTriviaData>(), 16);
            assert_eq!(std::mem::align_of::<GreenTriviaData>(), 8);
        }

        // GreenTriviaData on 32-bit targets:
        // header (4 bytes) + length (4 bytes) = 8 bytes
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTriviaData>(), 8);
            assert_eq!(std::mem::align_of::<GreenTriviaData>(), 4);
        }
    }

    #[test]
    fn test_green_trivia_memory_layout() {
        // GreenTrivia wraps a ThinArc pointer.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTrivia>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTrivia>(), 4);
        }
    }

    #[test]
    fn test_expected_heap_allocation_size_when_known_lengths_expect_aligned_sizes() {
        #[cfg(target_pointer_width = "64")]
        {
            let cases: &[(usize, usize)] = &[(0, 24), (1, 32), (8, 32), (9, 40)];
            for (text_len, expected) in cases {
                assert_eq!(expected_heap_allocation_size(*text_len), *expected);
            }
        }

        #[cfg(target_pointer_width = "32")]
        {
            let cases: &[(usize, usize)] = &[(0, 12), (1, 16), (4, 16), (5, 20)];
            for (text_len, expected) in cases {
                assert_eq!(expected_heap_allocation_size(*text_len), *expected);
            }
        }
    }

    #[test]
    fn test_expected_heap_allocation_size_when_created_trivia_expect_matches_case_table() {
        #[cfg(target_pointer_width = "64")]
        {
            let cases: [(&[u8], usize); 4] = [(b"", 24), (b" ", 32), (b"12345678", 32), (b"123456789", 40)];
            for (text, expected) in cases {
                let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, text);
                assert_eq!(expected_heap_allocation_size(trivia.width() as usize), expected);
            }
        }

        #[cfg(target_pointer_width = "32")]
        {
            let cases: [(&[u8], usize); 4] = [(b"", 12), (b" ", 16), (b"1234", 16), (b"12345", 20)];
            for (text, expected) in cases {
                let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, text);
                assert_eq!(expected_heap_allocation_size(trivia.width() as usize), expected);
            }
        }
    }
}

#[cfg(test)]
mod green_trivia_tests {
    use super::*;
    use crate::syntax::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_trivia() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert_eq!(trivia.kind(), SyntaxKind::WhitespaceTrivia);
        assert_eq!(trivia.text(), b" ");
    }

    #[test]
    fn test_new_when_created_expect_is_not_missing_flag_set() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert!(trivia.flags().contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_kind() {
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, b"% comment");
        assert_eq!(trivia.kind(), SyntaxKind::CommentTrivia);
    }

    #[test]
    fn test_text() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"   ");
        assert_eq!(trivia.text(), b"   ");
    }

    #[test]
    fn test_width() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n\t");
        assert_eq!(trivia.width(), 2);
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert_eq!(trivia1, trivia2);
    }

    #[test]
    fn test_eq_when_different_text_expect_not_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n");
        assert_ne!(trivia1, trivia2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::CommentTrivia, b" ");
        assert_ne!(trivia1, trivia2);
    }

    #[test]
    fn test_clone() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" \n\t");
        let trivia2 = trivia1.clone();
        assert_eq!(trivia1, trivia2);
        assert_eq!(trivia2.kind(), SyntaxKind::WhitespaceTrivia);
        assert_eq!(trivia2.text(), b" \n\t");
    }

    #[test]
    fn test_display() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" \n\t");
        assert_eq!(trivia.to_string(), " \n\t");
    }

    #[test]
    fn test_debug() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let debug_str = format!("{:?}", trivia);
        let expected = "GreenTrivia { kind: WhitespaceTrivia, text: \" \", width: 1 }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_empty_text() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"");
        assert_eq!(trivia.text(), b"");
        assert_eq!(trivia.width(), 0);
    }

    #[test]
    fn test_multiline_comment_text() {
        let text = b"% line1\n% line2\n% line3";
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, text);
        assert_eq!(trivia.text(), text);
        assert_eq!(trivia.width(), text.len() as u8);
    }

    #[test]
    fn test_unicode_comment_text() {
        let text = b"% \xE4\xBD\xA0\xE5\xA5\xBD\xE4\xB8\x96\xE7\x95\x8C";
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, text);
        assert_eq!(trivia.text(), text);
        assert_eq!(trivia.width(), text.len() as u8);
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let ptr = GreenTrivia::into_raw(trivia.clone());
        let reconstructed = unsafe { GreenTrivia::from_raw(ptr) };
        assert_eq!(trivia, reconstructed);
    }

    #[test]
    fn test_borrow() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let borrowed: &GreenTriviaData = trivia.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::WhitespaceTrivia);
        assert_eq!(borrowed.text(), b" ");
    }

    #[test]
    fn test_new_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "trivia diag");
        let key;

        {
            let trivia = GreenTrivia::new_with_diagnostic(SyntaxKind::WhitespaceTrivia, b" ", vec![diagnostic.clone()]);
            assert!(trivia.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = trivia.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*trivia as *const GreenTriviaData) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }

    #[test]
    fn test_new_with_diagnostic_when_empty_expect_same_as_new_without_diagnostic_flag() {
        let trivia = GreenTrivia::new_with_diagnostic(SyntaxKind::WhitespaceTrivia, b" ", vec![]);
        assert!(trivia.flags().contains(GreenFlags::IS_NOT_MISSING));
        assert!(!trivia.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
        assert!(trivia.diagnostics().is_none());
    }
}

#[cfg(test)]
mod green_trivia_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let data: &GreenTriviaData = &*trivia;
        let owned = data.to_owned();
        assert_eq!(trivia, owned);
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let data1: &GreenTriviaData = &*trivia1;
        let data2: &GreenTriviaData = &*trivia2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_text_expect_not_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n");
        let data1: &GreenTriviaData = &*trivia1;
        let data2: &GreenTriviaData = &*trivia2;
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::CommentTrivia, b" ");
        let data1: &GreenTriviaData = &*trivia1;
        let data2: &GreenTriviaData = &*trivia2;
        assert_ne!(data1, data2);
    }
}

