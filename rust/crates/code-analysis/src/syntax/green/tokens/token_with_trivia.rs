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

type Repr = HeaderSlice<GreenTokenWithTriviaHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenWithTriviaHead, [u8; 0]>;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTokenWithTriviaHead {
    leading_trivia: Option<GreenNode>,  // 8 bytes on 64-bit targets, 4 bytes on 32-bit targets
    trailing_trivia: Option<GreenNode>, // 8 bytes on 64-bit targets, 4 bytes on 32-bit targets
    full_width: u16,                    // 2 bytes
    kind: SyntaxKind,                   // 2 bytes
    flags: GreenFlags,                  // 1 byte
    _c: Count<GreenTokenWithTrivia>,    // 0 bytes
}

/// Borrowed token view for well-known text tokens.
///
/// The underlying text is not stored in the node; it is derived from
/// `SyntaxKind` at read time.
#[repr(transparent)]
pub(crate) struct GreenTokenWithTriviaData {
    data: ReprThin,
}

impl GreenTokenWithTriviaData {
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

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        self.write_to(true, true)
    }

    /// Returns the length of the text covered by this token.
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
        self.data.header.leading_trivia.clone()
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        self.data.header.trailing_trivia.clone()
    }

    /// Returns the flags of this token.
    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }

    #[inline]
    pub(crate) fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.full_width() as usize);
        if leading && let Some(leading_trivia) = &self.data.header.leading_trivia {
            bytes.extend_from_slice(&leading_trivia.full_text());
        }

        bytes.extend_from_slice(self.text());

        if trailing && let Some(trailing_trivia) = &self.data.header.trailing_trivia {
            bytes.extend_from_slice(&trailing_trivia.full_text());
        }
        bytes
    }
}

impl PartialEq for GreenTokenWithTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl fmt::Display for GreenTokenWithTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GreenTokenWithTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.text();
        let text_str = String::from_utf8_lossy(text);

        f.debug_struct("GreenTokenWithTrivia")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .finish()
    }
}

impl ToOwned for GreenTokenWithTriviaData {
    type Owned = GreenTokenWithTrivia;

    #[inline]
    fn to_owned(&self) -> GreenTokenWithTrivia {
        let green = unsafe { GreenTokenWithTrivia::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTokenWithTrivia::clone(&green)
    }
}

/// Leaf node in the immutable tree.
///
/// Represents a token whose text is well-known for its `SyntaxKind` and can be
/// reconstructed without storing token bytes in the node payload.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenTokenWithTrivia {
    ptr: ThinArc<GreenTokenWithTriviaHead, u8>,
}

#[allow(dead_code)]
impl GreenTokenWithTrivia {
    /// Creates a present (non-missing) token with optional trivia.
    #[inline]
    pub fn new(kind: SyntaxKind, leading_trivia: Option<GreenNode>, trailing_trivia: Option<GreenNode>) -> Self {
        Self::create_full(kind, leading_trivia, trailing_trivia, GreenFlags::IS_NOT_MISSING, Vec::new())
    }

    #[inline]
    pub fn new_with_diagnostic(
        kind: SyntaxKind,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        Self::create_full(kind, leading_trivia, trailing_trivia, GreenFlags::IS_NOT_MISSING, diagnostics)
    }

    /// Creates a missing (synthetic) token for error recovery.
    ///
    /// Missing tokens are parser-inserted placeholders when expected syntax is
    /// absent. They do **not** set `GreenFlags::IS_NOT_MISSING`.
    #[inline]
    pub fn new_missing(kind: SyntaxKind, leading_trivia: Option<GreenNode>, trailing_trivia: Option<GreenNode>) -> Self {
        Self::create_full(kind, leading_trivia, trailing_trivia, GreenFlags::NONE, Vec::new())
    }

    #[inline]
    pub fn new_missing_with_diagnostic(
        kind: SyntaxKind,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        Self::create_full(kind, leading_trivia, trailing_trivia, GreenFlags::NONE, diagnostics)
    }

    #[inline]
    fn create_full(
        kind: SyntaxKind,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
        base_flags: GreenFlags,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let has_diagnostics = !diagnostics.is_empty();
        let flags = match has_diagnostics {
            true => base_flags | GreenFlags::CONTAINS_DIAGNOSTIC,
            false => base_flags,
        };

        let first_leading_width = leading_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let last_trailing_width = trailing_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let full_width = kind.get_text().len() as u16 + first_leading_width + last_trailing_width;

        let head = GreenTokenWithTriviaHead {
            kind,
            flags,
            full_width,
            leading_trivia,
            trailing_trivia,
            _c: Count::new(),
        };

        let ptr = ThinArc::from_header_and_iter(head, std::iter::empty());
        let token = GreenTokenWithTrivia { ptr };

        if has_diagnostics {
            let key = token.diagnostics_key();
            diagnostics::insert_diagnostics(key, diagnostics);
        }

        token
    }
}

impl Borrow<GreenTokenWithTriviaData> for GreenTokenWithTrivia {
    #[inline]
    fn borrow(&self) -> &GreenTokenWithTriviaData {
        self
    }
}

impl fmt::Display for GreenTokenWithTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenWithTriviaData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Debug for GreenTokenWithTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenWithTriviaData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenTokenWithTrivia {
    /// Consumes the handle and returns a raw non-null pointer to the data.
    #[inline]
    pub(crate) fn into_raw(this: GreenTokenWithTrivia) -> ptr::NonNull<GreenTokenWithTriviaData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTokenWithTriviaData = &green;
        ptr::NonNull::from(green)
    }

    /// Reconstructs an owned handle from a raw pointer.
    ///
    /// # Safety
    ///
    /// The raw pointer must have been produced by `into_raw` and not yet
    /// consumed. The underlying `Arc` allocation must still be live.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenWithTriviaData>) -> GreenTokenWithTrivia {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTokenWithTriviaHead, u8>>(arc)
        };
        GreenTokenWithTrivia { ptr: arc }
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
        let data: &GreenTokenWithTriviaData = self;
        data as *const GreenTokenWithTriviaData as usize
    }
}

impl Drop for GreenTokenWithTrivia {
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

impl ops::Deref for GreenTokenWithTrivia {
    type Target = GreenTokenWithTriviaData;

    #[inline]
    fn deref(&self) -> &GreenTokenWithTriviaData {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTokenWithTriviaData>(repr)
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use crate::arc::{ArcInner, HeaderSlice};
    use std::mem::offset_of;

    fn expected_heap_allocation_size(payload_len: usize) -> usize {
        type ThinRepr = ArcInner<HeaderSlice<GreenTokenWithTriviaHead, [u8; 0]>>;
        let inner_to_data_offset = offset_of!(ThinRepr, data);
        let data_to_slice_offset = std::mem::size_of::<HeaderSlice<GreenTokenWithTriviaHead, [u8; 0]>>();
        let usable_size = inner_to_data_offset
            .checked_add(data_to_slice_offset)
            .and_then(|v| v.checked_add(payload_len))
            .expect("size overflows");
        let align = std::mem::align_of::<ThinRepr>();
        usable_size.wrapping_add(align - 1) & !(align - 1)
    }

    #[test]
    fn test_green_token_head_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithTriviaHead>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithTriviaHead>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithTriviaHead>(), 12);
            assert_eq!(std::mem::align_of::<GreenTokenWithTriviaHead>(), 4);
        }
    }

    #[test]
    fn test_green_token_data_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithTriviaData>(), 32);
            assert_eq!(std::mem::align_of::<GreenTokenWithTriviaData>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithTriviaData>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithTriviaData>(), 4);
        }
    }

    #[test]
    fn test_green_token_memory_layout() {
        // GreenTokenWithTrivia wraps a ThinArc pointer.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithTrivia>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithTrivia>(), 4);
        }
    }

    #[test]
    fn test_expected_heap_allocation_size_when_zero_payload_expect_header_only_allocation() {
        #[cfg(target_pointer_width = "64")]
        assert_eq!(expected_heap_allocation_size(0), 40);

        #[cfg(target_pointer_width = "32")]
        assert_eq!(expected_heap_allocation_size(0), 20);
    }

    #[test]
    fn test_expected_heap_allocation_size_when_created_tokens_expect_zero_payload_allocation() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let token_missing = GreenTokenWithTrivia::new_missing(SyntaxKind::FalseKeyword, None, None);

        #[cfg(target_pointer_width = "64")]
        let expected = 40;

        #[cfg(target_pointer_width = "32")]
        let expected = 20;

        let actuals = [token.kind(), token_missing.kind()];

        for _ in actuals {
            assert_eq!(expected_heap_allocation_size(0), expected);
        }
    }
}

#[cfg(test)]
mod green_token_tests {
    use super::*;
    use crate::GreenTrivia;
    use crate::syntax::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    fn leading_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ").into()],
        ))
    }

    fn trailing_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::EndOfLineTrivia, b"\n").into()],
        ))
    }

    #[test]
    fn test_new_token() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        assert_eq!(token.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(token.text(), b"true");
    }

    #[test]
    fn test_new_when_created_expect_is_not_missing_flag_set() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        assert!(token.flags().contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_kind() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        assert_eq!(token.kind(), SyntaxKind::TrueKeyword);
    }

    #[test]
    fn test_text() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        assert_eq!(token.text(), b"true");
    }

    #[test]
    fn test_width() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        assert_eq!(token.width(), 4);
    }

    #[test]
    fn test_full_width_when_trivia_present_expect_includes_trivia() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia());
        assert_eq!(token.full_width(), 6);
    }

    #[test]
    fn test_width_when_trivia_present_expect_token_width_only() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia());
        assert_eq!(token.width(), 4);
    }

    #[test]
    fn test_full_text_when_trivia_present_expect_includes_trivia_and_text() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia());
        assert_eq!(token.full_text(), b" true\n");
    }

    #[test]
    fn test_write_to_when_flags_vary_expect_expected_bytes() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia());
        assert_eq!(token.write_to(false, false), b"true");
        assert_eq!(token.write_to(true, false), b" true");
        assert_eq!(token.write_to(false, true), b"true\n");
        assert_eq!(token.write_to(true, true), b" true\n");
    }

    #[test]
    fn test_eq_when_same_kind_expect_equal() {
        let token1 = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let token2 = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let token2 = GreenTokenWithTrivia::new(SyntaxKind::FalseKeyword, None, None);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_clone() {
        let token1 = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let token2 = token1.clone();
        assert_eq!(token1, token2);
        assert_eq!(token2.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(token2.text(), b"true");
    }

    #[test]
    fn test_display() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        assert_eq!(token.to_string(), "true");
    }

    #[test]
    fn test_debug() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let debug_str = format!("{:?}", token);
        let expected = "GreenTokenWithTrivia { kind: TrueKeyword, text: \"true\", width: 4 }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_empty_text() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::NameLiteralToken, None, None);
        assert_eq!(token.text(), b"");
        assert_eq!(token.width(), 0);
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let ptr = GreenTokenWithTrivia::into_raw(token.clone());
        let reconstructed = unsafe { GreenTokenWithTrivia::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let borrowed: &GreenTokenWithTriviaData = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(borrowed.text(), b"true");
    }

    #[test]
    fn test_new_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "token trivia diag");
        let key;

        {
            let token = GreenTokenWithTrivia::new_with_diagnostic(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia(), vec![diagnostic.clone()]);
            assert!(token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = token.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*token as *const GreenTokenWithTriviaData) as usize;
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
    fn test_to_owned() {
        let token = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let data: &GreenTokenWithTriviaData = &*token;
        let owned = data.to_owned();
        assert_eq!(token, owned);
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal() {
        let token1 = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let token2 = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let data1: &GreenTokenWithTriviaData = &*token1;
        let data2: &GreenTokenWithTriviaData = &*token2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenTokenWithTrivia::new(SyntaxKind::TrueKeyword, None, None);
        let token2 = GreenTokenWithTrivia::new(SyntaxKind::FalseKeyword, None, None);
        let data1: &GreenTokenWithTriviaData = &*token1;
        let data2: &GreenTokenWithTriviaData = &*token2;
        assert_ne!(data1, data2);
    }
}

#[cfg(test)]
mod green_missing_token_tests {
    use super::*;
    use crate::GreenTrivia;
    use crate::syntax::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    fn leading_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ").into()],
        ))
    }

    fn trailing_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::EndOfLineTrivia, b"\n").into()],
        ))
    }

    #[test]
    fn test_new_missing_when_created_expect_missing_flag_state() {
        let token = GreenTokenWithTrivia::new_missing(SyntaxKind::TrueKeyword, None, None);
        assert!(!token.flags().contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(token.flags(), GreenFlags::NONE);
    }

    #[test]
    fn test_new_missing_full_text_when_trivia_present_expect_includes_trivia_and_text() {
        let token = GreenTokenWithTrivia::new_missing(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia());
        assert_eq!(token.full_text(), b" true\n");
        assert_eq!(token.width(), 4);
        assert_eq!(token.full_width(), 6);
    }

    #[test]
    fn test_new_missing_write_to_when_flags_vary_expect_expected_bytes() {
        let token = GreenTokenWithTrivia::new_missing(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia());
        assert_eq!(token.write_to(false, false), b"true");
        assert_eq!(token.write_to(true, false), b" true");
        assert_eq!(token.write_to(false, true), b"true\n");
        assert_eq!(token.write_to(true, true), b" true\n");
    }

    #[test]
    fn test_new_missing_into_raw_and_from_raw_when_roundtrip_expect_equal() {
        let token = GreenTokenWithTrivia::new_missing(SyntaxKind::TrueKeyword, None, None);
        let ptr = GreenTokenWithTrivia::into_raw(token.clone());
        let reconstructed = unsafe { GreenTokenWithTrivia::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_new_missing_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "token trivia missing diag");
        let key;

        {
            let token =
                GreenTokenWithTrivia::new_missing_with_diagnostic(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia(), vec![diagnostic.clone()]);
            assert!(!token.flags().contains(GreenFlags::IS_NOT_MISSING));
            assert!(token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = token.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*token as *const GreenTokenWithTriviaData) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }
}
