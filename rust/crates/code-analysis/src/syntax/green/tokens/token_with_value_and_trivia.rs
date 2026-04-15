//! Green token representation with inline token text, typed value, and optional trivia.
//!
//! This variant combines `GreenTokenWithValue` payload semantics with optional
//! leading/trailing trivia links and cached full width.

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

pub(crate) type GreenTokenWithIntValueAndTrivia = GreenTokenWithValueAndTrivia<i32>;
pub(crate) type GreenTokenWithFloatValueAndTrivia = GreenTokenWithValueAndTrivia<f32>;
pub(crate) type GreenTokenWithStringValueAndTrivia = GreenTokenWithValueAndTrivia<String>;
pub(crate) type GreenTokenWithIntValueAndTriviaData = GreenTokenWithValueAndTriviaData<i32>;
pub(crate) type GreenTokenWithFloatValueAndTriviaData = GreenTokenWithValueAndTriviaData<f32>;
pub(crate) type GreenTokenWithStringValueAndTriviaData = GreenTokenWithValueAndTriviaData<String>;

type Repr<T> = HeaderSlice<GreenTokenWithValueAndTriviaHead<T>, [u8]>;
type ReprThin<T> = HeaderSlice<GreenTokenWithValueAndTriviaHead<T>, [u8; 0]>;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTokenWithValueAndTriviaHead<T> {
    leading_trivia: Option<GreenNode>,           // 8 bytes on 64-bit targets, 4 bytes on 32-bit targets
    trailing_trivia: Option<GreenNode>,          // 8 bytes on 64-bit targets, 4 bytes on 32-bit targets
    value: T,                                    // size depends on T
    full_width: u16,                             // 2 bytes
    kind: SyntaxKind,                            // 2 bytes (`repr(u16)`)
    flags: GreenFlags,                           // 1 byte
    _c: Count<GreenTokenWithValueAndTrivia<()>>, // 0 bytes
}

#[repr(transparent)]
pub(crate) struct GreenTokenWithValueAndTriviaData<T> {
    data: ReprThin<T>,
}

impl<T> GreenTokenWithValueAndTriviaData<T> {
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
        self.data.header.leading_trivia.clone()
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

impl<T> PartialEq for GreenTokenWithValueAndTriviaData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl<T> fmt::Display for GreenTokenWithValueAndTriviaData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl<T> fmt::Debug for GreenTokenWithValueAndTriviaData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text_str = String::from_utf8_lossy(self.text());

        f.debug_struct("GreenTokenWithValueAndTrivia")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .field("full_width", &self.full_width())
            .finish()
    }
}

impl<T: Clone> ToOwned for GreenTokenWithValueAndTriviaData<T> {
    type Owned = GreenTokenWithValueAndTrivia<T>;

    #[inline]
    fn to_owned(&self) -> GreenTokenWithValueAndTrivia<T> {
        let green = unsafe { GreenTokenWithValueAndTrivia::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTokenWithValueAndTrivia::<T>::clone(&green)
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct GreenTokenWithValueAndTrivia<T> {
    ptr: ThinArc<GreenTokenWithValueAndTriviaHead<T>, u8>,
}

impl<T> PartialEq for GreenTokenWithValueAndTrivia<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl<T> Eq for GreenTokenWithValueAndTrivia<T> {}

impl<T> Hash for GreenTokenWithValueAndTrivia<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind().hash(state);
        self.text().hash(state);
    }
}

#[allow(dead_code)]
impl<T> GreenTokenWithValueAndTrivia<T> {
    #[inline]
    pub fn new(
        kind: SyntaxKind,
        text: &[u8],
        value: T,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
    ) -> GreenTokenWithValueAndTrivia<T> {
        Self::create_full(kind, text, value, leading_trivia, trailing_trivia, GreenFlags::IS_NOT_MISSING, Vec::new())
    }

    #[inline]
    pub fn new_with_diagnostic(
        kind: SyntaxKind,
        text: &[u8],
        value: T,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> GreenTokenWithValueAndTrivia<T> {
        Self::create_full(kind, text, value, leading_trivia, trailing_trivia, GreenFlags::IS_NOT_MISSING, diagnostics)
    }

    #[inline]
    fn create_full(
        kind: SyntaxKind,
        text: &[u8],
        value: T,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
        base_flags: GreenFlags,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> GreenTokenWithValueAndTrivia<T> {
        let has_diagnostics = !diagnostics.is_empty();
        let flags = match has_diagnostics {
            true => base_flags | GreenFlags::CONTAINS_DIAGNOSTIC,
            false => base_flags,
        };

        let first_leading_width = leading_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let last_trailing_width = trailing_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let full_width = text.len() as u16 + first_leading_width + last_trailing_width;

        let head = GreenTokenWithValueAndTriviaHead::<T> {
            kind,
            flags,
            full_width,
            leading_trivia,
            trailing_trivia,
            value,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        let token = GreenTokenWithValueAndTrivia { ptr };

        if has_diagnostics {
            let key = token.diagnostics_key();
            diagnostics::insert_diagnostics(key, diagnostics);
        }

        token
    }
}

impl<T> Borrow<GreenTokenWithValueAndTriviaData<T>> for GreenTokenWithValueAndTrivia<T> {
    #[inline]
    fn borrow(&self) -> &GreenTokenWithValueAndTriviaData<T> {
        self
    }
}

impl<T> fmt::Display for GreenTokenWithValueAndTrivia<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenWithValueAndTriviaData<T> = self;
        fmt::Display::fmt(data, f)
    }
}

impl<T> fmt::Debug for GreenTokenWithValueAndTrivia<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenWithValueAndTriviaData<T> = self;
        fmt::Debug::fmt(data, f)
    }
}

impl<T> GreenTokenWithValueAndTrivia<T> {
    /// Consumes the handle and returns a raw non-null pointer to the data.
    #[inline]
    pub(crate) fn into_raw(this: GreenTokenWithValueAndTrivia<T>) -> ptr::NonNull<GreenTokenWithValueAndTriviaData<T>> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTokenWithValueAndTriviaData<T> = &green;
        ptr::NonNull::from(green)
    }

    /// Reconstructs an owned handle from a raw pointer.
    ///
    /// # Safety
    ///
    /// The raw pointer must have been produced by `into_raw` and not yet
    /// consumed. The underlying `Arc` allocation must still be live.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenWithValueAndTriviaData<T>>) -> GreenTokenWithValueAndTrivia<T> {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin<T>);
            mem::transmute::<Arc<ReprThin<T>>, ThinArc<GreenTokenWithValueAndTriviaHead<T>, u8>>(arc)
        };
        GreenTokenWithValueAndTrivia { ptr: arc }
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
        let data: &GreenTokenWithValueAndTriviaData<T> = self;
        data as *const GreenTokenWithValueAndTriviaData<T> as usize
    }
}

impl<T> Drop for GreenTokenWithValueAndTrivia<T> {
    #[inline]
    fn drop(&mut self) {
        // Same rationale as non-generic variant: remove diagnostics on
        // last-owner drop so cleanup is deterministic and race-free.
        let should_clear = self.ptr.with_arc(|arc| arc.is_unique());
        if should_clear {
            self.clear_diagnostics();
        }
    }
}

impl<T> ops::Deref for GreenTokenWithValueAndTrivia<T> {
    type Target = GreenTokenWithValueAndTriviaData<T>;

    #[inline]
    fn deref(&self) -> &GreenTokenWithValueAndTriviaData<T> {
        unsafe {
            let repr: &Repr<T> = &*self.ptr;
            let repr: &ReprThin<T> = &*(repr as *const Repr<T> as *const ReprThin<T>);
            mem::transmute::<&ReprThin<T>, &GreenTokenWithValueAndTriviaData<T>>(repr)
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use crate::arc::{ArcInner, HeaderSlice};
    use std::mem::offset_of;

    fn expected_heap_allocation_size<T>(text_len: usize) -> usize {
        type ThinRepr<T> = ArcInner<HeaderSlice<GreenTokenWithValueAndTriviaHead<T>, [u8; 0]>>;
        let inner_to_data_offset = offset_of!(ThinRepr<T>, data);
        let data_to_slice_offset = std::mem::size_of::<HeaderSlice<GreenTokenWithValueAndTriviaHead<T>, [u8; 0]>>();
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
            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<u32>>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<u32>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTriviaData>(), 32);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<f32>>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<f32>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTriviaData>(), 32);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<String>>(), 48);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<String>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTriviaData>(), 56);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTrivia>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTrivia>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTrivia>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<u32>>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<u32>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTriviaData>(), 20);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<f32>>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<f32>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTriviaData>(), 20);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<String>>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<String>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTriviaData>(), 28);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTrivia>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTrivia>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTrivia>(), 4);
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

            let cases_string: &[(usize, usize)] = &[(0, 64), (1, 72), (8, 72), (9, 80)];
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
    fn test_new_when_numeric_with_trivia_expect_kind_text_value_and_full_text() {
        let token: GreenTokenWithIntValueAndTrivia =
            GreenTokenWithValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 42, leading_trivia(), trailing_trivia());
        assert_eq!(token.kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(token.text(), b"42");
        assert_eq!(token.value(), &42);
        assert_eq!(token.width(), 2);
        assert_eq!(token.full_width(), 4);
        assert_eq!(token.full_text(), b" 42\n");
    }

    #[test]
    fn test_write_to_when_trivia_flags_vary_expect_expected_output() {
        let token: GreenTokenWithStringValueAndTrivia =
            GreenTokenWithValueAndTrivia::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string(), leading_trivia(), trailing_trivia());

        assert_eq!(token.write_to(false, false), b"Type");
        assert_eq!(token.write_to(true, false), b" Type");
        assert_eq!(token.write_to(false, true), b"Type\n");
        assert_eq!(token.write_to(true, true), b" Type\n");
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal_ignoring_value() {
        let token1: GreenTokenWithIntValueAndTrivia = GreenTokenWithValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 1, None, None);
        let token2: GreenTokenWithIntValueAndTrivia = GreenTokenWithValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 2, None, None);
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_into_raw_and_from_raw_when_roundtrip_expect_equal() {
        let token: GreenTokenWithFloatValueAndTrivia = GreenTokenWithValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5, None, None);
        let ptr = GreenTokenWithValueAndTrivia::into_raw(token.clone());
        let reconstructed = unsafe { GreenTokenWithValueAndTrivia::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow_when_name_with_trivia_expect_access_data() {
        let token: GreenTokenWithStringValueAndTrivia = GreenTokenWithValueAndTrivia::new(
            SyntaxKind::NameLiteralToken,
            b"Catalog",
            "Catalog".to_string(),
            leading_trivia(),
            trailing_trivia(),
        );

        let borrowed: &GreenTokenWithValueAndTriviaData<String> = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::NameLiteralToken);
        assert_eq!(borrowed.text(), b"Catalog");
        assert_eq!(borrowed.value(), "Catalog");
        assert!(borrowed.leading_trivia().is_some());
        assert!(borrowed.trailing_trivia().is_some());
    }

    #[test]
    fn test_new_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "token value trivia diag");
        let key;

        {
            let token: GreenTokenWithIntValueAndTrivia = GreenTokenWithValueAndTrivia::new_with_diagnostic(
                SyntaxKind::NumericLiteralToken,
                b"42",
                42,
                leading_trivia(),
                trailing_trivia(),
                vec![diagnostic.clone()],
            );
            assert!(token.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = token.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*token as *const GreenTokenWithValueAndTriviaData<i32>) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }
}
