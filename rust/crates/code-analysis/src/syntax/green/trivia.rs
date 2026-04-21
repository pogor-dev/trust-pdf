//! Green trivia representation with inline PDF trivia bytes.
//!
//! This variant stores per-instance trivia text bytes inline in the green node
//! tail. Trivia text is read from the inline byte slice provided by callers.

use std::{borrow::Borrow, fmt, mem, ops, ptr};

use crate::{
    GreenDiagnostic, SyntaxKind,
    arc::{Arc, HeaderSlice, ThinArc},
    syntax::green::{diagnostics, flags::GreenFlags},
};

use countme::Count;

type Repr = HeaderSlice<GreenTriviaHead, [u8]>;
type ReprThin = HeaderSlice<GreenTriviaHead, [u8; 0]>;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTriviaHead {
    kind: SyntaxKind,
    flags: GreenFlags,
    _c: Count<GreenTrivia>,
}

/// Borrowed trivia view with inline trivia text.
#[repr(transparent)]
pub(crate) struct GreenTriviaData {
    data: ReprThin,
}

impl GreenTriviaData {
    /// Returns the trivia kind (end of line, whitespace, or comment).
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Returns the trivia text as a byte slice.
    #[inline]
    pub fn text(&self) -> &[u8] {
        self.data.slice()
    }

    /// Returns the width of this trivia, which is the length of its text in bytes.
    #[inline]
    pub fn width(&self) -> u8 {
        self.data.slice_len() as u8
    }

    /// Returns the flags associated with this trivia, which may indicate if it's missing or contains diagnostics.
    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }

    /// Returns true if this trivia has diagnostics associated.
    #[inline]
    pub fn contains_diagnostics(&self) -> bool {
        self.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC)
    }

    /// Returns true if this trivia is missing.
    #[inline]
    pub fn is_missing(&self) -> bool {
        !self.flags().contains(GreenFlags::IS_NOT_MISSING)
    }
}

impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.width() == other.width() && self.text() == other.text()
    }
}

impl Eq for GreenTriviaData {}

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
        let text_str: String = self.text().iter().flat_map(|&byte| std::ascii::escape_default(byte)).map(char::from).collect();

        f.debug_struct("GreenTrivia")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .finish()
    }
}

/// Represents a trivia node in the green tree,
/// which is attached to tokens and carries end-of-line, whitespace, or comment text.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, u8>,
}

#[allow(dead_code)]
impl GreenTrivia {
    /// Creates new trivia.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        Self::create_full(kind, text, Vec::new())
    }

    #[inline]
    pub fn new_with_diagnostic(kind: SyntaxKind, text: &[u8], diagnostics: Vec<GreenDiagnostic>) -> GreenTrivia {
        Self::create_full(kind, text, diagnostics)
    }

    fn create_full(kind: SyntaxKind, text: &[u8], diagnostics: Vec<GreenDiagnostic>) -> GreenTrivia {
        let has_diagnostics = !diagnostics.is_empty();
        let flags = GreenFlags::IS_NOT_MISSING;
        let flags = match has_diagnostics {
            true => flags | GreenFlags::CONTAINS_DIAGNOSTIC,
            false => flags,
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

    #[inline]
    pub(crate) fn diagnostics(&self) -> Option<Vec<GreenDiagnostic>> {
        diagnostics::get_diagnostics(self.diagnostics_key())
    }

    #[inline]
    fn diagnostics_key(&self) -> usize {
        let data: &GreenTriviaData = self;
        data as *const GreenTriviaData as usize
    }
}

impl Borrow<GreenTriviaData> for GreenTrivia {
    #[inline]
    fn borrow(&self) -> &GreenTriviaData {
        self
    }
}

impl fmt::Display for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl Drop for GreenTrivia {
    #[inline]
    fn drop(&mut self) {
        // Clear side-table diagnostics only for the final owner.
        // This avoids duplicate removals while cloned green handles are
        // still alive and keeps diagnostics lifetime tied to green data.
        let should_clear = self.ptr.with_arc(|arc| arc.is_unique());
        if should_clear {
            diagnostics::remove_diagnostics(self.diagnostics_key());
        }
    }
}

impl ops::Deref for GreenTrivia {
    type Target = GreenTriviaData;

    #[inline]
    fn deref(&self) -> &GreenTriviaData {
        unsafe {
            let repr: &Repr = &*self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTriviaData>(repr)
        }
    }
}

#[cfg(test)]
mod green_trivia_tests {
    use super::*;
    use crate::syntax::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    fn assert_eq_trait<T: Eq + ?Sized>(_: &T) {}

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
    fn test_contains_diagnostics_when_no_diagnostics_expect_false() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert!(!trivia.contains_diagnostics());
    }

    #[test]
    fn test_contains_diagnostics_when_diagnostics_exist_expect_true() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "trivia diag");
        let trivia = GreenTrivia::new_with_diagnostic(SyntaxKind::WhitespaceTrivia, b" ", vec![diagnostic]);
        assert!(trivia.contains_diagnostics());
    }

    #[test]
    fn test_is_missing_when_created_trivia_expect_false() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert!(!trivia.is_missing());
    }

    #[test]
    fn test_eq_when_same_kind_and_text_via_deref_data_expect_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert_eq!(&*trivia1, &*trivia2);
    }

    #[test]
    fn test_eq_when_different_text_via_deref_data_expect_not_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n");
        assert_ne!(&*trivia1, &*trivia2);
    }

    #[test]
    fn test_eq_trait_when_deref_data_expect_implemented() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert_eq_trait(&*trivia);
    }

    #[test]
    fn test_debug() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let debug_str = format!("{:?}", trivia);
        let expected = "GreenTrivia { kind: WhitespaceTrivia, text: \" \", width: 1 }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_debug_when_text_contains_control_characters_expect_escaped_output() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n\t");
        let debug_str = format!("{:?}", trivia);
        let expected = "GreenTrivia { kind: WhitespaceTrivia, text: \"\\\\n\\\\t\", width: 2 }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_display() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" \n\t");
        assert_eq!(trivia.to_string(), " \n\t");
    }

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
    fn test_eq_trait_when_owned_trivia_expect_implemented() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert_eq_trait(&trivia);
    }

    #[test]
    fn test_borrow() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let borrowed: &GreenTriviaData = trivia.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::WhitespaceTrivia);
        assert_eq!(borrowed.text(), b" ");
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
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use crate::arc::{ArcInner, HeaderSlice};
    use std::mem::offset_of;

    #[test]
    fn test_green_trivia_head_memory_layout() {
        // GreenTriviaHead: kind (1 byte) + flags (1 byte) + _c (0 bytes)
        assert_eq!(std::mem::size_of::<GreenTriviaHead>(), 2);
        assert_eq!(std::mem::align_of::<GreenTriviaHead>(), 1);
    }

    #[test]
    fn test_green_trivia_data_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        let cases: &[(usize, usize)] = &[(16, 8)];

        #[cfg(target_pointer_width = "32")]
        let cases: &[(usize, usize)] = &[(8, 4)];

        for (expected_size, expected_align) in cases {
            assert_eq!(std::mem::size_of::<GreenTriviaData>(), *expected_size);
            assert_eq!(std::mem::align_of::<GreenTriviaData>(), *expected_align);
        }
    }

    #[test]
    fn test_green_trivia_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        let cases: &[(usize, usize)] = &[(8, 8)];

        #[cfg(target_pointer_width = "32")]
        let cases: &[(usize, usize)] = &[(4, 4)];

        for (expected_size, expected_align) in cases {
            assert_eq!(std::mem::size_of::<GreenTrivia>(), *expected_size);
            assert_eq!(std::mem::align_of::<GreenTrivia>(), *expected_align);
        }
    }
}
