//! Green token representation with inline token text, typed value, and trailing trivia.
//!
//! This variant avoids storing leading trivia in the header when only trailing
//! trivia is needed, while preserving the same token API surface.

use std::{
    borrow::Borrow,
    fmt,
    hash::{Hash, Hasher},
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    GreenNode,
    arc::{Arc, HeaderSlice, ThinArc},
    syntax::green::{diagnostics, flags::GreenFlags},
};
use countme::Count;

use crate::GreenDiagnostic;
use crate::SyntaxKind;

pub(crate) type GreenTokenWithIntValueAndTrailingTrivia = GreenTokenWithValueAndTrailingTrivia<i32>;
pub(crate) type GreenTokenWithFloatValueAndTrailingTrivia = GreenTokenWithValueAndTrailingTrivia<f32>;
pub(crate) type GreenTokenWithStringValueAndTrailingTrivia = GreenTokenWithValueAndTrailingTrivia<String>;
pub(crate) type GreenTokenWithIntValueAndTrailingTriviaData = GreenTokenWithValueAndTrailingTriviaData<i32>;
pub(crate) type GreenTokenWithFloatValueAndTrailingTriviaData = GreenTokenWithValueAndTrailingTriviaData<f32>;
pub(crate) type GreenTokenWithStringValueAndTrailingTriviaData = GreenTokenWithValueAndTrailingTriviaData<String>;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTokenWithValueAndTrailingTriviaHead<T> {
    trailing_trivia: Option<GreenNode>,                  // 8 bytes on 64-bit targets, 4 bytes on 32-bit targets
    full_width: u16,                                     // 2 bytes
    kind: SyntaxKind,                                    // 2 bytes (`repr(u16)`)
    flags: GreenFlags,                                   // 1 byte
    value: T,                                            // size depends on T
    _c: Count<GreenTokenWithValueAndTrailingTrivia<()>>, // 0 bytes
}

#[repr(transparent)]
pub(crate) struct GreenTokenWithValueAndTrailingTriviaData<T> {
    data: ReprThin<T>,
}

impl<T> GreenTokenWithValueAndTrailingTriviaData<T> {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        self.data.slice()
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        self.write_to(true, true)
    }

    #[inline]
    pub fn value(&self) -> &T {
        &self.data.header.value
    }

    #[inline]
    pub fn width(&self) -> u8 {
        self.data.slice().len() as u8
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

        if trailing
            && let Some(trailing_trivia) = &self.data.header.trailing_trivia {
                bytes.extend_from_slice(&trailing_trivia.full_text());
            }

        bytes
    }
}

impl<T> PartialEq for GreenTokenWithValueAndTrailingTriviaData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl<T> fmt::Display for GreenTokenWithValueAndTrailingTriviaData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl<T> fmt::Debug for GreenTokenWithValueAndTrailingTriviaData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text_str = String::from_utf8_lossy(self.text());

        f.debug_struct("GreenTokenWithValueAndTrailingTrivia")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .field("full_width", &self.full_width())
            .finish()
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct GreenTokenWithValueAndTrailingTrivia<T> {
    ptr: ThinArc<GreenTokenWithValueAndTrailingTriviaHead<T>, u8>,
}

impl<T> PartialEq for GreenTokenWithValueAndTrailingTrivia<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl<T> Eq for GreenTokenWithValueAndTrailingTrivia<T> {}

impl<T> Hash for GreenTokenWithValueAndTrailingTrivia<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind().hash(state);
        self.text().hash(state);
    }
}

impl<T> GreenTokenWithValueAndTrailingTrivia<T> {
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8], value: T, trailing_trivia: Option<GreenNode>) -> GreenTokenWithValueAndTrailingTrivia<T> {
        Self::create_full(kind, text, value, trailing_trivia, GreenFlags::IS_NOT_MISSING, Vec::new())
    }

    #[inline]
    pub fn new_with_diagnostic(
        kind: SyntaxKind,
        text: &[u8],
        value: T,
        trailing_trivia: Option<GreenNode>,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> GreenTokenWithValueAndTrailingTrivia<T> {
        Self::create_full(kind, text, value, trailing_trivia, GreenFlags::IS_NOT_MISSING, diagnostics)
    }

    #[inline]
    fn create_full(
        kind: SyntaxKind,
        text: &[u8],
        value: T,
        trailing_trivia: Option<GreenNode>,
        base_flags: GreenFlags,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> GreenTokenWithValueAndTrailingTrivia<T> {
        let has_diagnostics = !diagnostics.is_empty();
        let flags = if has_diagnostics {
            base_flags | GreenFlags::CONTAINS_DIAGNOSTIC
        } else {
            base_flags
        };

        let trailing_width = trailing_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let full_width = text.len() as u16 + trailing_width;

        let head = GreenTokenWithValueAndTrailingTriviaHead::<T> {
            kind,
            flags,
            full_width,
            trailing_trivia,
            value,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        let token = GreenTokenWithValueAndTrailingTrivia { ptr };

        if !has_diagnostics {
            return token;
        }

        let key = token.diagnostics_key();
        diagnostics::insert_diagnostics(key, diagnostics);
        token
    }
}

impl_green_boilerplate!(generic GreenTokenWithValueAndTrailingTriviaHead, GreenTokenWithValueAndTrailingTriviaData, GreenTokenWithValueAndTrailingTrivia, u8);

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use crate::arc::{ArcInner, HeaderSlice};
    use std::mem::offset_of;

    fn expected_heap_allocation_size<T>(text_len: usize) -> usize {
        type ThinRepr<T> = ArcInner<HeaderSlice<GreenTokenWithValueAndTrailingTriviaHead<T>, [u8; 0]>>;
        let inner_to_data_offset = offset_of!(ThinRepr<T>, data);
        let data_to_slice_offset = std::mem::size_of::<HeaderSlice<GreenTokenWithValueAndTrailingTriviaHead<T>, [u8; 0]>>();
        let usable_size = inner_to_data_offset
            .checked_add(data_to_slice_offset)
            .and_then(|v| v.checked_add(text_len))
            .expect("size overflows");
        let align = std::mem::align_of::<ThinRepr<T>>();
        usable_size.wrapping_add(align - 1) & !(align - 1)
    }

    #[test]
    fn test_green_token_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTrailingTriviaHead<u32>>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTrailingTriviaHead<u32>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTrailingTriviaData>(), 32);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTrailingTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTrailingTriviaHead<f32>>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTrailingTriviaHead<f32>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTrailingTriviaData>(), 32);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTrailingTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTrailingTriviaHead<String>>(), 40);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTrailingTriviaHead<String>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTrailingTriviaData>(), 48);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTrailingTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTrailingTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTrailingTrivia>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTrailingTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTrailingTrivia>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTrailingTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTrailingTrivia>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTrailingTriviaHead<u32>>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTrailingTriviaHead<u32>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTrailingTriviaData>(), 20);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTrailingTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTrailingTriviaHead<f32>>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTrailingTriviaHead<f32>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTrailingTriviaData>(), 20);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTrailingTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTrailingTriviaHead<String>>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTrailingTriviaHead<String>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTrailingTriviaData>(), 28);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTrailingTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTrailingTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTrailingTrivia>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTrailingTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTrailingTrivia>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTrailingTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTrailingTrivia>(), 4);
        }
    }

    #[test]
    fn test_expected_heap_allocation_size_when_known_lengths_expect_aligned_sizes() {
        #[cfg(target_pointer_width = "64")]
        {
            let cases_u32: &[(usize, usize)] = &[(0, 40), (1, 48), (8, 48), (9, 56)];
            for (text_len, expected) in cases_u32 {
                assert_eq!(expected_heap_allocation_size::<u32>(*text_len), *expected);
            }

            let cases_f32: &[(usize, usize)] = &[(0, 40), (1, 48), (8, 48), (9, 56)];
            for (text_len, expected) in cases_f32 {
                assert_eq!(expected_heap_allocation_size::<f32>(*text_len), *expected);
            }

            let cases_string: &[(usize, usize)] = &[(0, 56), (1, 64), (8, 64), (9, 72)];
            for (text_len, expected) in cases_string {
                assert_eq!(expected_heap_allocation_size::<String>(*text_len), *expected);
            }
        }

        #[cfg(target_pointer_width = "32")]
        {
            let cases_u32: &[(usize, usize)] = &[(0, 24), (1, 28), (4, 28), (5, 32)];
            for (text_len, expected) in cases_u32 {
                assert_eq!(expected_heap_allocation_size::<u32>(*text_len), *expected);
            }

            let cases_f32: &[(usize, usize)] = &[(0, 24), (1, 28), (4, 28), (5, 32)];
            for (text_len, expected) in cases_f32 {
                assert_eq!(expected_heap_allocation_size::<f32>(*text_len), *expected);
            }

            let cases_string: &[(usize, usize)] = &[(0, 32), (1, 36), (4, 36), (5, 40)];
            for (text_len, expected) in cases_string {
                assert_eq!(expected_heap_allocation_size::<String>(*text_len), *expected);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GreenTrivia;
    use crate::syntax::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    fn trailing_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::EndOfLineTrivia, b"\n").into()],
        ))
    }

    #[test]
    fn test_new_when_numeric_with_trailing_trivia_expect_kind_text_value_and_full_text() {
        let token: GreenTokenWithIntValueAndTrailingTrivia =
            GreenTokenWithValueAndTrailingTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 42, trailing_trivia());

        assert_eq!(token.kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(token.text(), b"42");
        assert_eq!(token.value(), &42);
        assert_eq!(token.width(), 2);
        assert_eq!(token.full_width(), 3);
        assert_eq!(token.full_text(), b"42\n");
        assert!(token.flags().contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_write_to_when_trailing_flag_varies_expect_expected_output() {
        let token: GreenTokenWithStringValueAndTrailingTrivia =
            GreenTokenWithValueAndTrailingTrivia::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string(), trailing_trivia());

        assert_eq!(token.write_to(false, false), b"Type");
        assert_eq!(token.write_to(true, false), b"Type");
        assert_eq!(token.write_to(false, true), b"Type\n");
        assert_eq!(token.write_to(true, true), b"Type\n");
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal_ignoring_value() {
        let token1: GreenTokenWithIntValueAndTrailingTrivia = GreenTokenWithValueAndTrailingTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 1, None);
        let token2: GreenTokenWithIntValueAndTrailingTrivia =
            GreenTokenWithValueAndTrailingTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 2, trailing_trivia());
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_trivia_accessors_when_created_expect_only_trailing_set() {
        let token: GreenTokenWithFloatValueAndTrailingTrivia =
            GreenTokenWithValueAndTrailingTrivia::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5, trailing_trivia());

        assert_eq!(token.leading_trivia(), None);
        assert!(token.trailing_trivia().is_some());
    }

    #[test]
    fn test_into_raw_and_from_raw_when_roundtrip_expect_equal() {
        let token: GreenTokenWithFloatValueAndTrailingTrivia = GreenTokenWithValueAndTrailingTrivia::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5, None);
        let ptr = GreenTokenWithValueAndTrailingTrivia::into_raw(token.clone());
        let reconstructed = unsafe { GreenTokenWithValueAndTrailingTrivia::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow_when_called_expect_data_access() {
        let token: GreenTokenWithStringValueAndTrailingTrivia =
            GreenTokenWithValueAndTrailingTrivia::new(SyntaxKind::NameLiteralToken, b"Catalog", "Catalog".to_string(), trailing_trivia());

        let borrowed: &GreenTokenWithValueAndTrailingTriviaData<String> = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::NameLiteralToken);
        assert_eq!(borrowed.text(), b"Catalog");
        assert_eq!(borrowed.value(), "Catalog");
        assert_eq!(borrowed.leading_trivia(), None);
        assert!(borrowed.trailing_trivia().is_some());
    }

    #[test]
    fn test_new_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "token value trailing diag");
        let key;

        {
            let token: GreenTokenWithIntValueAndTrailingTrivia = GreenTokenWithValueAndTrailingTrivia::new_with_diagnostic(
                SyntaxKind::NumericLiteralToken,
                b"42",
                42,
                trailing_trivia(),
                vec![diagnostic.clone()],
            );
            assert!(token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = token.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*token as *const GreenTokenWithValueAndTrailingTriviaData<i32>) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }
}
