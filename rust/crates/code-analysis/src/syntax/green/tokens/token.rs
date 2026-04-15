//! Green token representation for well-known PDF token text.
//!
//! This variant stores no per-instance text bytes. The token text is inferred
//! directly from `SyntaxKind` via `SyntaxKind::get_text()`, which matches the
//! fixed-text token pattern used for punctuation/keywords.

use std::{
    borrow::Borrow,
    fmt,
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

type Repr = HeaderSlice<GreenTokenHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTokenHead {
    kind: SyntaxKind,      // 2 bytes
    flags: GreenFlags,     // 1 byte
    _c: Count<GreenToken>, // 0 bytes
}

/// Borrowed token view for well-known text tokens.
///
/// The underlying text is not stored in the node; it is derived from
/// `SyntaxKind` at read time.
#[repr(transparent)]
pub(crate) struct GreenTokenData {
    data: ReprThin,
}

impl GreenTokenData {
    /// Kind of this token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Text of this token.
    #[inline]
    pub fn text(&self) -> &[u8] {
        self.kind().get_text()
    }

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn width(&self) -> u8 {
        self.kind().get_text().len() as u8
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

impl PartialEq for GreenTokenData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl fmt::Display for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.text();
        let text_str = String::from_utf8_lossy(text);

        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .finish()
    }
}

impl ToOwned for GreenTokenData {
    type Owned = GreenToken;

    #[inline]
    fn to_owned(&self) -> GreenToken {
        let green = unsafe { GreenToken::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenToken::clone(&green)
    }
}

/// Leaf node in the immutable tree.
///
/// Represents a token whose text is well-known for its `SyntaxKind` and can be
/// reconstructed without storing token bytes in the node payload.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenToken {
    ptr: ThinArc<GreenTokenHead, u8>,
}

#[allow(dead_code)]
impl GreenToken {
    /// Creates a present (non-missing) token.
    #[inline]
    pub fn new(kind: SyntaxKind) -> GreenToken {
        Self::create_full(kind, GreenFlags::IS_NOT_MISSING, Vec::new())
    }

    #[inline]
    pub fn new_with_diagnostic(kind: SyntaxKind, diagnostics: Vec<GreenDiagnostic>) -> GreenToken {
        Self::create_full(kind, GreenFlags::IS_NOT_MISSING, diagnostics)
    }

    /// Creates a missing (synthetic) token for error recovery.
    ///
    /// Missing tokens are parser-inserted placeholders when expected syntax is
    /// absent. They do **not** set `GreenFlags::IS_NOT_MISSING`.
    #[inline]
    pub fn new_missing(kind: SyntaxKind) -> GreenToken {
        Self::create_full(kind, GreenFlags::NONE, Vec::new())
    }

    #[inline]
    pub fn new_missing_with_diagnostic(kind: SyntaxKind, diagnostics: Vec<GreenDiagnostic>) -> GreenToken {
        Self::create_full(kind, GreenFlags::NONE, diagnostics)
    }

    #[inline]
    fn create_full(kind: SyntaxKind, base_flags: GreenFlags, diagnostics: Vec<GreenDiagnostic>) -> GreenToken {
        let has_diagnostics = !diagnostics.is_empty();
        let flags = match has_diagnostics {
            true => base_flags | GreenFlags::CONTAINS_DIAGNOSTIC,
            false => base_flags,
        };

        let head = GreenTokenHead { kind, flags, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, std::iter::empty());
        let token = GreenToken { ptr };

        if has_diagnostics {
            let key = token.diagnostics_key();
            diagnostics::insert_diagnostics(key, diagnostics);
        }

        token
    }
}

impl Borrow<GreenTokenData> for GreenToken {
    #[inline]
    fn borrow(&self) -> &GreenTokenData {
        self
    }
}

impl fmt::Display for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenToken {
    /// Consumes the handle and returns a raw non-null pointer to the data.
    #[inline]
    pub(crate) fn into_raw(this: GreenToken) -> ptr::NonNull<GreenTokenData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTokenData = &green;
        ptr::NonNull::from(green)
    }

    /// Reconstructs an owned handle from a raw pointer.
    ///
    /// # Safety
    ///
    /// The raw pointer must have been produced by `into_raw` and not yet
    /// consumed. The underlying `Arc` allocation must still be live.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> GreenToken {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTokenHead, u8>>(arc)
        };
        GreenToken { ptr: arc }
    }

    #[inline]
    pub(crate) fn diagnostics(&self) -> Option<Vec<crate::GreenDiagnostic>> {
        use crate::syntax::green::diagnostics;

        diagnostics::get_diagnostics(self.diagnostics_key())
    }

    #[inline]
    fn clear_diagnostics(&self) {
        use crate::syntax::green::diagnostics;

        diagnostics::remove_diagnostics(self.diagnostics_key());
    }

    #[inline]
    fn diagnostics_key(&self) -> usize {
        let data: &GreenTokenData = self;
        data as *const GreenTokenData as usize
    }
}

impl Drop for GreenToken {
    #[inline]
    fn drop(&mut self) {
        // Clear side-table diagnostics only for the final owner.
        // This avoids duplicate removals while cloned green handles are
        // still alive and keeps diagnostics lifetime tied to green data.
        let should_clear = self.ptr.with_arc(|arc| arc.is_unique());
        if should_clear {
            self.clear_diagnostics();
        }
    }
}

impl ops::Deref for GreenToken {
    type Target = GreenTokenData;

    #[inline]
    fn deref(&self) -> &GreenTokenData {
        unsafe {
            let repr: &Repr = &*self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTokenData>(repr)
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use crate::arc::{ArcInner, HeaderSlice};
    use std::mem::offset_of;

    fn expected_heap_allocation_size(payload_len: usize) -> usize {
        type ThinRepr = ArcInner<HeaderSlice<GreenTokenHead, [u8; 0]>>;
        let inner_to_data_offset = offset_of!(ThinRepr, data);
        let data_to_slice_offset = std::mem::size_of::<HeaderSlice<GreenTokenHead, [u8; 0]>>();
        let usable_size = inner_to_data_offset
            .checked_add(data_to_slice_offset)
            .and_then(|v| v.checked_add(payload_len))
            .expect("size overflows");
        let align = std::mem::align_of::<ThinRepr>();
        usable_size.wrapping_add(align - 1) & !(align - 1)
    }

    #[test]
    fn test_green_token_head_memory_layout() {
        // GreenTokenHead: kind (1 byte) + flags (1 byte) + _c (0 bytes)
        assert_eq!(std::mem::size_of::<GreenTokenHead>(), 2);
        assert_eq!(std::mem::align_of::<GreenTokenHead>(), 1);
    }

    #[test]
    fn test_green_token_data_memory_layout() {
        // GreenTokenData on 64-bit targets:
        // header (2 bytes) + padding (6 bytes) + length (8 bytes) = 16 bytes
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenData>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenData>(), 8);
        }

        // GreenTokenData on 32-bit targets:
        // header (2 bytes) + padding (2 bytes) + length (4 bytes) = 8 bytes
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenData>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenData>(), 4);
        }
    }

    #[test]
    fn test_green_token_memory_layout() {
        // GreenToken wraps a ThinArc pointer.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenToken>(), 8);
            assert_eq!(std::mem::align_of::<GreenToken>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenToken>(), 4);
            assert_eq!(std::mem::align_of::<GreenToken>(), 4);
        }
    }

    #[test]
    fn test_expected_heap_allocation_size_when_zero_payload_expect_header_only_allocation() {
        #[cfg(target_pointer_width = "64")]
        assert_eq!(expected_heap_allocation_size(0), 24);

        #[cfg(target_pointer_width = "32")]
        assert_eq!(expected_heap_allocation_size(0), 12);
    }

    #[test]
    fn test_expected_heap_allocation_size_when_created_tokens_expect_zero_payload_allocation() {
        let tokens = [
            GreenToken::new(SyntaxKind::TrueKeyword),
            GreenToken::new_missing(SyntaxKind::FalseKeyword),
            GreenToken::new_with_diagnostic(SyntaxKind::NullKeyword, vec![]),
        ];

        #[cfg(target_pointer_width = "64")]
        let expected = 24;

        #[cfg(target_pointer_width = "32")]
        let expected = 12;

        for token in tokens {
            let _ = token.kind();
            assert_eq!(expected_heap_allocation_size(0), expected);
        }
    }
}

#[cfg(test)]
mod green_token_tests {
    use super::*;
    use crate::syntax::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_token() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(token.text(), b"true");
    }

    #[test]
    fn test_new_when_created_expect_is_not_missing_flag_set() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        assert!(token.flags().contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_kind() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.kind(), SyntaxKind::TrueKeyword);
    }

    #[test]
    fn test_text() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.text(), b"true");
    }

    #[test]
    fn test_width() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.width(), 4);
    }

    #[test]
    fn test_full_text_and_full_width_when_plain_token_expect_text_equivalence() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.full_text(), token.text());
        assert_eq!(token.full_width(), token.width());
    }

    #[test]
    fn test_trivia_accessors_when_plain_token_expect_none() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.leading_trivia(), None);
        assert_eq!(token.trailing_trivia(), None);
    }

    #[test]
    fn test_write_to_when_plain_token_expect_text_ignoring_flags() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.write_to(false, false), token.text());
        assert_eq!(token.write_to(true, true), token.text());
    }

    #[test]
    fn test_eq_when_same_kind_expect_equal() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::TrueKeyword);
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::FalseKeyword);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_clone() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = token1.clone();
        assert_eq!(token1, token2);
        assert_eq!(token2.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(token2.text(), b"true");
    }

    #[test]
    fn test_display() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.to_string(), "true");
    }

    #[test]
    fn test_debug() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        let debug_str = format!("{:?}", token);
        let expected = "GreenToken { kind: TrueKeyword, text: \"true\", width: 4 }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_empty_text() {
        let token = GreenToken::new(SyntaxKind::NameLiteralToken);
        assert_eq!(token.text(), b"");
        assert_eq!(token.width(), 0);
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        let ptr = GreenToken::into_raw(token.clone());
        let reconstructed = unsafe { GreenToken::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        let borrowed: &GreenTokenData = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(borrowed.text(), b"true");
    }

    #[test]
    fn test_new_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "token diag");
        let key;

        {
            let token = GreenToken::new_with_diagnostic(SyntaxKind::TrueKeyword, vec![diagnostic.clone()]);
            assert!(token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = token.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*token as *const GreenTokenData) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }

    #[test]
    fn test_new_with_diagnostic_when_empty_expect_same_as_new_without_diagnostic_flag() {
        let token = GreenToken::new_with_diagnostic(SyntaxKind::TrueKeyword, vec![]);
        assert!(token.flags().contains(GreenFlags::IS_NOT_MISSING));
        assert!(!token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
        assert!(token.diagnostics().is_none());
    }
}

#[cfg(test)]
mod green_token_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        let data: &GreenTokenData = &*token;
        let owned = data.to_owned();
        assert_eq!(token, owned);
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::TrueKeyword);
        let data1: &GreenTokenData = &*token1;
        let data2: &GreenTokenData = &*token2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::FalseKeyword);
        let data1: &GreenTokenData = &*token1;
        let data2: &GreenTokenData = &*token2;
        assert_ne!(data1, data2);
    }
}

#[cfg(test)]
mod green_missing_token_tests {
    use super::*;
    use crate::syntax::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_missing_when_created_expect_missing_flag_state() {
        let token = GreenToken::new_missing(SyntaxKind::TrueKeyword);
        assert!(!token.flags().contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(token.flags(), GreenFlags::NONE);
    }

    #[test]
    fn test_new_missing_when_created_expect_same_text_and_width() {
        let token = GreenToken::new_missing(SyntaxKind::TrueKeyword);
        assert_eq!(token.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(token.text(), b"true");
        assert_eq!(token.width(), 4);
    }

    #[test]
    fn test_new_missing_into_raw_and_from_raw_when_roundtrip_expect_equal() {
        let token = GreenToken::new_missing(SyntaxKind::TrueKeyword);
        let ptr = GreenToken::into_raw(token.clone());
        let reconstructed = unsafe { GreenToken::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_new_missing_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "token missing diag");
        let key;

        {
            let token = GreenToken::new_missing_with_diagnostic(SyntaxKind::TrueKeyword, vec![diagnostic.clone()]);
            assert!(!token.flags().contains(GreenFlags::IS_NOT_MISSING));
            assert!(token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = token.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*token as *const GreenTokenData) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }

    #[test]
    fn test_new_missing_with_diagnostic_when_empty_expect_same_as_new_missing_without_diagnostic_flag() {
        let token = GreenToken::new_missing_with_diagnostic(SyntaxKind::TrueKeyword, vec![]);
        assert!(!token.flags().contains(GreenFlags::IS_NOT_MISSING));
        assert!(!token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
        assert!(token.diagnostics().is_none());
    }
}
