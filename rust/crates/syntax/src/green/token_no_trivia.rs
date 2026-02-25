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
    arc::{Arc, HeaderSlice, ThinArc},
    green::flags::GreenFlags,
};
use countme::Count;

use crate::SyntaxKind;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTokenNoTriviaHead {
    kind: SyntaxKind,              // 2 bytes
    flags: GreenFlags,             // 1 byte
    _c: Count<GreenTokenNoTrivia>, // 0 bytes
}

type Repr = HeaderSlice<GreenTokenNoTriviaHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenNoTriviaHead, [u8; 0]>;

#[repr(transparent)]
/// Borrowed token view for well-known text tokens.
///
/// The underlying text is not stored in the node; it is derived from
/// `SyntaxKind` at read time.
pub struct GreenTokenNoTriviaData {
    data: ReprThin,
}

impl GreenTokenNoTriviaData {
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

    /// Returns the flags of this token.
    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }
}

impl PartialEq for GreenTokenNoTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl ToOwned for GreenTokenNoTriviaData {
    type Owned = GreenTokenNoTrivia;

    #[inline]
    fn to_owned(&self) -> GreenTokenNoTrivia {
        let green = unsafe { GreenTokenNoTrivia::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTokenNoTrivia::clone(&green)
    }
}

impl fmt::Display for GreenTokenNoTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GreenTokenNoTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.text();
        let text_str = String::from_utf8_lossy(text);

        f.debug_struct("GreenTokenNoTrivia")
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
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTokenNoTrivia {
    ptr: ThinArc<GreenTokenNoTriviaHead, u8>,
}

impl Borrow<GreenTokenNoTriviaData> for GreenTokenNoTrivia {
    #[inline]
    fn borrow(&self) -> &GreenTokenNoTriviaData {
        self
    }
}

impl fmt::Display for GreenTokenNoTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenNoTriviaData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Debug for GreenTokenNoTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenNoTriviaData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenTokenNoTrivia {
    /// Creates new token.
    #[inline]
    pub fn new(kind: SyntaxKind) -> GreenTokenNoTrivia {
        let flags = GreenFlags::IS_NOT_MISSING; // Tokens created via `new` are always not-missing
        let head = GreenTokenNoTriviaHead { kind, flags, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, std::iter::empty());
        GreenTokenNoTrivia { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenTokenNoTrivia) -> ptr::NonNull<GreenTokenNoTriviaData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTokenNoTriviaData = &green;
        ptr::NonNull::from(green)
    }

    /// # Safety
    ///
    /// This function uses `unsafe` code to create an `Arc` from a raw pointer and then transmutes it into a `ThinArc`.
    ///
    /// - The raw pointer must be valid and correctly aligned for the type `ReprThin`.
    /// - The lifetime of the raw pointer must outlive the lifetime of the `Arc` created from it.
    /// - The transmute operation must be safe, meaning that the memory layout of `Arc<ReprThin>` must be compatible with `ThinArc<GreenTokenNoTriviaHead, u8>`.
    ///
    /// Failure to uphold these invariants can lead to undefined behavior.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenNoTriviaData>) -> GreenTokenNoTrivia {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTokenNoTriviaHead, u8>>(arc)
        };
        GreenTokenNoTrivia { ptr: arc }
    }
}

impl ops::Deref for GreenTokenNoTrivia {
    type Target = GreenTokenNoTriviaData;

    #[inline]
    fn deref(&self) -> &GreenTokenNoTriviaData {
        unsafe {
            let repr: &Repr = &*self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTokenNoTriviaData>(repr)
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_token_head_memory_layout() {
        // GreenTokenNoTriviaHead: kind (2 bytes) + flags (1 byte) + _c (0 bytes)
        // Expected: 2 + 1 + 1 padding for alignment = 4 bytes
        assert_eq!(std::mem::size_of::<GreenTokenNoTriviaHead>(), 4);
        assert_eq!(std::mem::align_of::<GreenTokenNoTriviaHead>(), 2);
    }

    #[test]
    fn test_green_token_data_memory_layout() {
        // GreenTokenNoTriviaData on 64-bit targets:
        // header (4 bytes) + padding (4 bytes) + length (8 bytes) = 16 bytes
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenNoTriviaData>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenNoTriviaData>(), 8);
        }

        // GreenTokenNoTriviaData on 32-bit targets:
        // header (4 bytes) + length (4 bytes) = 8 bytes
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenNoTriviaData>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenNoTriviaData>(), 4);
        }
    }

    #[test]
    fn test_green_token_memory_layout() {
        // GreenTokenNoTrivia wraps a ThinArc pointer.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenNoTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenNoTrivia>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenNoTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenNoTrivia>(), 4);
        }
    }
}

#[cfg(test)]
mod green_token_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_token() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(token.text(), b"true");
    }

    #[test]
    fn test_new_when_created_expect_is_not_missing_flag_set() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        assert!(token.flags().contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_kind() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.kind(), SyntaxKind::TrueKeyword);
    }

    #[test]
    fn test_text() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.text(), b"true");
    }

    #[test]
    fn test_width() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.width(), 4);
    }

    #[test]
    fn test_eq_when_same_kind_expect_equal() {
        let token1 = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let token2 = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let token2 = GreenTokenNoTrivia::new(SyntaxKind::FalseKeyword);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_clone() {
        let token1 = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let token2 = token1.clone();
        assert_eq!(token1, token2);
        assert_eq!(token2.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(token2.text(), b"true");
    }

    #[test]
    fn test_display() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        assert_eq!(token.to_string(), "true");
    }

    #[test]
    fn test_debug() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let debug_str = format!("{:?}", token);
        let expected = "GreenTokenNoTrivia { kind: TrueKeyword, text: \"true\", width: 4 }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_empty_text() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::NameLiteralToken);
        assert_eq!(token.text(), b"");
        assert_eq!(token.width(), 0);
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let ptr = GreenTokenNoTrivia::into_raw(token.clone());
        let reconstructed = unsafe { GreenTokenNoTrivia::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let borrowed: &GreenTokenNoTriviaData = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::TrueKeyword);
        assert_eq!(borrowed.text(), b"true");
    }
}

#[cfg(test)]
mod green_token_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned() {
        let token = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let data: &GreenTokenNoTriviaData = &*token;
        let owned = data.to_owned();
        assert_eq!(token, owned);
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal() {
        let token1 = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let token2 = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let data1: &GreenTokenNoTriviaData = &*token1;
        let data2: &GreenTokenNoTriviaData = &*token2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenTokenNoTrivia::new(SyntaxKind::TrueKeyword);
        let token2 = GreenTokenNoTrivia::new(SyntaxKind::FalseKeyword);
        let data1: &GreenTokenNoTriviaData = &*token1;
        let data2: &GreenTokenNoTriviaData = &*token2;
        assert_ne!(data1, data2);
    }
}
