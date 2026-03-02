//! Green token representation for well-known PDF token text with trailing trivia.
//!
//! This variant stores no per-instance text bytes and keeps only an optional
//! trailing trivia node to reduce per-token overhead when leading trivia is
//! guaranteed to be absent.

use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    GreenNode,
    arc::{Arc, HeaderSlice, ThinArc},
    green::{diagnostics, flags::GreenFlags},
};
use countme::Count;

use crate::GreenDiagnostic;
use crate::SyntaxKind;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTokenWithTrailingTriviaHead {
    trailing_trivia: Option<GreenNode>,      // 8 bytes on 64-bit targets, 4 bytes on 32-bit targets
    full_width: u16,                         // 2 bytes
    kind: SyntaxKind,                        // 2 bytes (`repr(u16)`)
    flags: GreenFlags,                       // 1 byte
    _c: Count<GreenTokenWithTrailingTrivia>, // 0 bytes
}

#[repr(transparent)]
pub(crate) struct GreenTokenWithTrailingTriviaData {
    data: ReprThin,
}

impl GreenTokenWithTrailingTriviaData {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        self.kind().get_text()
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        self.write_to(true, true)
    }

    #[inline]
    pub fn width(&self) -> u8 {
        self.kind().get_text().len() as u8
    }

    #[inline]
    pub fn full_width(&self) -> u16 {
        self.data.header.full_width
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenNode> {
        None
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        self.data.header.trailing_trivia.clone()
    }

    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }

    #[inline]
    pub(crate) fn write_to(&self, _leading: bool, trailing: bool) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.full_width() as usize);
        bytes.extend_from_slice(self.text());
        if trailing {
            if let Some(trailing_trivia) = &self.data.header.trailing_trivia {
                bytes.extend_from_slice(&trailing_trivia.full_text());
            }
        }
        bytes
    }
}

impl PartialEq for GreenTokenWithTrailingTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl fmt::Display for GreenTokenWithTrailingTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GreenTokenWithTrailingTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.text();
        let text_str = String::from_utf8_lossy(text);

        f.debug_struct("GreenTokenWithTrailingTrivia")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .finish()
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenTokenWithTrailingTrivia {
    ptr: ThinArc<GreenTokenWithTrailingTriviaHead, u8>,
}

impl GreenTokenWithTrailingTrivia {
    #[inline]
    pub fn new(kind: SyntaxKind, trailing_trivia: Option<GreenNode>) -> Self {
        Self::create_full(kind, trailing_trivia, GreenFlags::IS_NOT_MISSING, Vec::new())
    }

    #[inline]
    pub fn new_with_diagnostic(kind: SyntaxKind, trailing_trivia: Option<GreenNode>, diagnostics: Vec<GreenDiagnostic>) -> Self {
        Self::create_full(kind, trailing_trivia, GreenFlags::IS_NOT_MISSING, diagnostics)
    }

    #[inline]
    pub fn new_missing(kind: SyntaxKind, trailing_trivia: Option<GreenNode>) -> Self {
        Self::create_full(kind, trailing_trivia, GreenFlags::NONE, Vec::new())
    }

    #[inline]
    pub fn new_missing_with_diagnostic(kind: SyntaxKind, trailing_trivia: Option<GreenNode>, diagnostics: Vec<GreenDiagnostic>) -> Self {
        Self::create_full(kind, trailing_trivia, GreenFlags::NONE, diagnostics)
    }

    #[inline]
    fn create_full(kind: SyntaxKind, trailing_trivia: Option<GreenNode>, base_flags: GreenFlags, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let has_diagnostics = !diagnostics.is_empty();
        let flags = if has_diagnostics {
            base_flags | GreenFlags::CONTAINS_DIAGNOSTIC
        } else {
            base_flags
        };

        let trailing_width = trailing_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let full_width = kind.get_text().len() as u16 + trailing_width;

        let head = GreenTokenWithTrailingTriviaHead {
            kind,
            flags,
            full_width,
            trailing_trivia,
            _c: Count::new(),
        };

        let ptr = ThinArc::from_header_and_iter(head, std::iter::empty());
        let token = GreenTokenWithTrailingTrivia { ptr };

        if !has_diagnostics {
            return token;
        }

        let key = token.diagnostics_key();
        diagnostics::insert_diagnostics(key, diagnostics);
        token
    }
}

impl_green_boilerplate!(
    GreenTokenWithTrailingTriviaHead,
    GreenTokenWithTrailingTriviaData,
    GreenTokenWithTrailingTrivia,
    u8
);

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_token_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithTrailingTriviaHead>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithTrailingTriviaHead>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithTrailingTriviaData>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithTrailingTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithTrailingTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithTrailingTrivia>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithTrailingTriviaHead>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithTrailingTriviaHead>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithTrailingTriviaData>(), 12);
            assert_eq!(std::mem::align_of::<GreenTokenWithTrailingTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithTrailingTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithTrailingTrivia>(), 4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GreenTrivia;
    use crate::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    fn trailing_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::EndOfLineTrivia, b"\n").into()],
        ))
    }

    #[test]
    fn test_new_when_created_expect_is_not_missing_flag_set() {
        let token = GreenTokenWithTrailingTrivia::new(SyntaxKind::TrueKeyword, trailing_trivia());
        assert!(token.flags().contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_new_missing_when_created_expect_missing_flag_state() {
        let token = GreenTokenWithTrailingTrivia::new_missing(SyntaxKind::TrueKeyword, trailing_trivia());
        assert!(!token.flags().contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(token.flags(), GreenFlags::NONE);
    }

    #[test]
    fn test_full_width_when_trivia_present_expect_includes_trivia() {
        let token = GreenTokenWithTrailingTrivia::new(SyntaxKind::TrueKeyword, trailing_trivia());
        assert_eq!(token.width(), 4);
        assert_eq!(token.full_width(), 5);
        assert_eq!(token.full_text(), b"true\n");
    }

    #[test]
    fn test_write_to_when_trailing_flag_varies_expect_expected_bytes() {
        let token = GreenTokenWithTrailingTrivia::new(SyntaxKind::TrueKeyword, trailing_trivia());
        assert_eq!(token.write_to(false, false), b"true");
        assert_eq!(token.write_to(true, false), b"true");
        assert_eq!(token.write_to(false, true), b"true\n");
        assert_eq!(token.write_to(true, true), b"true\n");
    }

    #[test]
    fn test_trivia_accessors_when_created_expect_only_trailing_set() {
        let token = GreenTokenWithTrailingTrivia::new(SyntaxKind::TrueKeyword, trailing_trivia());
        assert_eq!(token.leading_trivia(), None);
        assert!(token.trailing_trivia().is_some());
    }

    #[test]
    fn test_into_raw_and_from_raw_when_roundtrip_expect_equal() {
        let token = GreenTokenWithTrailingTrivia::new(SyntaxKind::FalseKeyword, trailing_trivia());
        let ptr = GreenTokenWithTrailingTrivia::into_raw(token.clone());
        let reconstructed = unsafe { GreenTokenWithTrailingTrivia::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow_when_called_expect_data_access() {
        let token = GreenTokenWithTrailingTrivia::new(SyntaxKind::NullKeyword, trailing_trivia());
        let borrowed: &GreenTokenWithTrailingTriviaData = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::NullKeyword);
        assert_eq!(borrowed.text(), b"null");
        assert_eq!(borrowed.leading_trivia(), None);
        assert!(borrowed.trailing_trivia().is_some());
    }

    #[test]
    fn test_new_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "token trailing diag");
        let key;

        {
            let token = GreenTokenWithTrailingTrivia::new_with_diagnostic(SyntaxKind::TrueKeyword, trailing_trivia(), vec![diagnostic.clone()]);
            assert!(token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = token.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*token as *const GreenTokenWithTrailingTriviaData) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }

    #[test]
    fn test_new_missing_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "token trailing missing diag");
        let key;

        {
            let token = GreenTokenWithTrailingTrivia::new_missing_with_diagnostic(SyntaxKind::TrueKeyword, trailing_trivia(), vec![diagnostic.clone()]);
            assert!(!token.flags().contains(GreenFlags::IS_NOT_MISSING));
            assert!(token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = token.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*token as *const GreenTokenWithTrailingTriviaData) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }
}
