//! Green token representation for well-known PDF token text.
//!
//! This variant stores per-instance text bytes inline in the green node tail.
//! The token text is read from the inline byte slice and may differ from
//! `SyntaxKind::get_text()` when callers provide explicit payload text.

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
    green::{diagnostics, flags::GreenFlags},
};
use countme::Count;

use crate::GreenDiagnostic;
use crate::SyntaxKind;

pub(crate) type GreenTokenWithIntValue = GreenTokenWithValue<i32>;
pub(crate) type GreenTokenWithFloatValue = GreenTokenWithValue<f32>;
pub(crate) type GreenTokenWithStringValue = GreenTokenWithValue<String>;
pub(crate) type GreenTokenWithIntValueData = GreenTokenWithValueData<i32>;
pub(crate) type GreenTokenWithFloatValueData = GreenTokenWithValueData<f32>;
pub(crate) type GreenTokenWithStringValueData = GreenTokenWithValueData<String>;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTokenWithValueHead<T> {
    kind: SyntaxKind,                   // 2 bytes
    flags: GreenFlags,                  // 1 byte
    value: T,                           // X bytes (depends on the type of the value)
    _c: Count<GreenTokenWithValue<()>>, // 0 bytes
}

/// Borrowed token view for tokens with inline text and typed values.
///
/// The underlying text is stored inline in the node tail with an associated
/// typed value (int, float, or string).
#[repr(transparent)]
pub(crate) struct GreenTokenWithValueData<T> {
    data: ReprThin<T>,
}

impl<T> GreenTokenWithValueData<T> {
    /// Kind of this token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Text of this token.
    #[inline]
    pub fn text(&self) -> &[u8] {
        self.data.slice()
    }

    /// Value of this token.
    #[inline]
    pub fn value(&self) -> &T {
        &self.data.header.value
    }

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn width(&self) -> u8 {
        self.data.slice().len() as u8
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        self.text().to_vec()
    }

    #[inline]
    pub fn full_width(&self) -> u8 {
        self.width()
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenNode> {
        None
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        None
    }

    #[inline]
    pub(crate) fn write_to(&self, _leading: bool, _trailing: bool) -> Vec<u8> {
        self.text().to_vec()
    }

    /// Returns the flags of this token.
    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }
}

impl<T> PartialEq for GreenTokenWithValueData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl<T> fmt::Display for GreenTokenWithValueData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl<T> fmt::Debug for GreenTokenWithValueData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.text();
        let text_str = String::from_utf8_lossy(text);

        f.debug_struct("GreenTokenWithValue")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .finish()
    }
}

/// Leaf node in the immutable tree.
///
/// Represents a token whose text is well-known for its `SyntaxKind` and can be
/// reconstructed without storing token bytes in the node payload.
#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct GreenTokenWithValue<T> {
    ptr: ThinArc<GreenTokenWithValueHead<T>, u8>,
}

impl<T> PartialEq for GreenTokenWithValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl<T> Eq for GreenTokenWithValue<T> {}

impl<T> Hash for GreenTokenWithValue<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind().hash(state);
        self.text().hash(state);
    }
}

impl<T> GreenTokenWithValue<T> {
    /// Creates new token.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8], value: T) -> GreenTokenWithValue<T> {
        Self::create_full(kind, text, value, GreenFlags::IS_NOT_MISSING, Vec::new())
    }

    #[inline]
    pub fn new_with_diagnostic(kind: SyntaxKind, text: &[u8], value: T, diagnostics: Vec<GreenDiagnostic>) -> GreenTokenWithValue<T> {
        Self::create_full(kind, text, value, GreenFlags::IS_NOT_MISSING, diagnostics)
    }

    #[inline]
    fn create_full(kind: SyntaxKind, text: &[u8], value: T, base_flags: GreenFlags, diagnostics: Vec<GreenDiagnostic>) -> GreenTokenWithValue<T> {
        let has_diagnostics = !diagnostics.is_empty();
        let flags = match has_diagnostics {
            true => base_flags | GreenFlags::CONTAINS_DIAGNOSTIC,
            false => base_flags,
        };

        let head = GreenTokenWithValueHead::<T> {
            kind,
            flags,
            value,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        let token = GreenTokenWithValue { ptr };

        if has_diagnostics {
            let key = token.diagnostics_key();
            diagnostics::insert_diagnostics(key, diagnostics);
        }

        token
    }
}

impl_green_boilerplate!(generic GreenTokenWithValueHead, GreenTokenWithValueData, GreenTokenWithValue, u8);

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_token_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValueHead<u32>>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueHead<u32>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueData>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueHead<f32>>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueHead<f32>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueData>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueHead<String>>(), 32);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueHead<String>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueData>(), 40);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueData>(), 8);

            // GreenTokenWithValue wraps a ThinArc pointer.
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValue>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValue>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValue>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValue>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValue>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValue>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueHead>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueHead>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueData>(), 12);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueHead>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueHead>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueData>(), 12);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueHead>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueHead>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueData>(), 20);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueData>(), 4);

            // GreenTokenWithValue wraps a ThinArc pointer.
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValue>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValue>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValue>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValue>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValue>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValue>(), 4);
        }
    }
}

#[cfg(test)]
mod green_token_tests {
    use super::*;
    use crate::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_when_pdf_number_expect_kind_text_and_value() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        assert_eq!(token.kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(token.text(), b"42");
        assert_eq!(token.value(), &42);
    }

    #[test]
    fn test_new_when_pdf_number_expect_is_not_missing_flag_set() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        assert!(token.flags().contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_kind_when_pdf_name_expect_name_kind() {
        let token: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string());
        assert_eq!(token.kind(), SyntaxKind::NameLiteralToken);
    }

    #[test]
    fn test_text_when_pdf_string_expect_original_string_token_text() {
        let token: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::StringLiteralToken, b"(hello)", "hello".to_string());
        assert_eq!(token.text(), b"(hello)");
    }

    #[test]
    fn test_width_when_pdf_number_expect_width_matches_text() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"456", 456);
        assert_eq!(token.width(), 3);
    }

    #[test]
    fn test_full_text_and_full_width_when_value_token_expect_text_equivalence() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"456", 456);
        assert_eq!(token.full_text(), token.text());
        assert_eq!(token.full_width(), token.width());
    }

    #[test]
    fn test_trivia_accessors_when_value_token_expect_none() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"456", 456);
        assert_eq!(token.leading_trivia(), None);
        assert_eq!(token.trailing_trivia(), None);
    }

    #[test]
    fn test_write_to_when_value_token_expect_text_ignoring_flags() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"456", 456);
        assert_eq!(token.write_to(false, false), token.text());
        assert_eq!(token.write_to(true, true), token.text());
    }

    #[test]
    fn test_eq_when_same_kind_expect_equal() {
        let token1: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        let token2: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string());
        let token2: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::HexStringLiteralToken, b"<54797065>", "Type".to_string());
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_clone_when_pdf_string_expect_value_preserved() {
        let token1: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::StringLiteralToken, b"(test)", "test".to_string());
        let token2 = token1.clone();
        assert_eq!(token1, token2);
        assert_eq!(token2.kind(), SyntaxKind::StringLiteralToken);
        assert_eq!(token2.text(), b"(test)");
        assert_eq!(token2.value(), "test");
    }

    #[test]
    fn test_display_when_pdf_number_expect_text_rendering() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"999", 999);
        assert_eq!(token.to_string(), "999");
    }

    #[test]
    fn test_debug_when_pdf_number_expect_kind_text_width() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42", 42);
        let debug_str = format!("{:?}", token);
        let expected = "GreenTokenWithValue { kind: NumericLiteralToken, text: \"42\", width: 2 }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_empty_text_when_pdf_string_expect_zero_width() {
        let token: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::StringLiteralToken, b"", String::new());
        assert_eq!(token.text(), b"");
        assert_eq!(token.width(), 0);
        assert_eq!(token.value(), "");
    }

    #[test]
    fn test_into_raw_and_from_raw_when_pdf_number_expect_roundtrip() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"777", 777);
        let ptr = GreenTokenWithValue::into_raw(token.clone());
        let reconstructed = unsafe { GreenTokenWithValue::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow_when_pdf_name_expect_access_kind_text_value() {
        let token: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::NameLiteralToken, b"Catalog", "Catalog".to_string());
        let borrowed: &GreenTokenWithValueData<String> = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::NameLiteralToken);
        assert_eq!(borrowed.text(), b"Catalog");
        assert_eq!(borrowed.value(), "Catalog");
    }

    #[test]
    fn test_value_when_pdf_hexstring_expect_decoded_payload_value() {
        let token: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::HexStringLiteralToken, b"<48656C6C6F>", "Hello".to_string());
        assert_eq!(token.value(), "Hello");
    }

    #[test]
    fn test_new_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "token value diag");
        let key;

        {
            let token: GreenTokenWithIntValue = GreenTokenWithValue::new_with_diagnostic(SyntaxKind::NumericLiteralToken, b"42", 42, vec![diagnostic.clone()]);
            assert!(token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = token.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*token as *const GreenTokenWithValueData<i32>) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }
}

#[cfg(test)]
mod green_token_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned_when_pdf_number_expect_equal_token_with_value() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"123", 123);
        let data: &GreenTokenWithValueData<i32> = &*token;
        let owned = data.to_owned();
        assert_eq!(token, owned);
        assert_eq!(owned.value(), &123);
    }

    #[test]
    fn test_eq_when_same_kind_and_same_text_expect_equal() {
        let token1: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"99", 99);
        let token2: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"99", 100);
        let data1: &GreenTokenWithValueData<i32> = &*token1;
        let data2: &GreenTokenWithValueData<i32> = &*token2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_same_kind_and_different_text_expect_not_equal() {
        let token1: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"99", 99);
        let token2: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"100", 100);
        let data1: &GreenTokenWithValueData<i32> = &*token1;
        let data2: &GreenTokenWithValueData<i32> = &*token2;
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::StringLiteralToken, b"(A)", "A".to_string());
        let token2: GreenTokenWithStringValue = GreenTokenWithValue::new(SyntaxKind::HexStringLiteralToken, b"<41>", "A".to_string());
        let data1: &GreenTokenWithValueData<String> = &*token1;
        let data2: &GreenTokenWithValueData<String> = &*token2;
        assert_ne!(data1, data2);
    }
}
