//! Green token representation for well-known PDF token text.
//!
//! This variant stores per-instance text bytes inline in the green node tail.
//! The token text is read from the inline byte slice and may differ from
//! `SyntaxKind::get_text()` when callers provide explicit payload text.

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
struct GreenTokenWithValueHead {
    kind: SyntaxKind,               // 2 bytes
    flags: GreenFlags,              // 1 byte
    _c: Count<GreenTokenWithValue>, // 0 bytes
}

type Repr = HeaderSlice<GreenTokenWithValueHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenWithValueHead, [u8; 0]>;

#[repr(transparent)]
/// Borrowed token view for well-known text tokens.
///
/// The underlying text is not stored in the node; it is derived from
/// `SyntaxKind` at read time.
pub(crate) struct GreenTokenWithValueData {
    data: ReprThin,
}

impl GreenTokenWithValueData {
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

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn width(&self) -> u8 {
        self.data.slice().len() as u8
    }

    /// Returns the flags of this token.
    #[inline]
    pub fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }
}

impl PartialEq for GreenTokenWithValueData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.flags() == other.flags()
    }
}

impl ToOwned for GreenTokenWithValueData {
    type Owned = GreenTokenWithValue;

    #[inline]
    fn to_owned(&self) -> GreenTokenWithValue {
        let green = unsafe { GreenTokenWithValue::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTokenWithValue::clone(&green)
    }
}

impl fmt::Display for GreenTokenWithValueData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GreenTokenWithValueData {
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
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenTokenWithValue {
    ptr: ThinArc<GreenTokenWithValueHead, u8>,
}

impl Borrow<GreenTokenWithValueData> for GreenTokenWithValue {
    #[inline]
    fn borrow(&self) -> &GreenTokenWithValueData {
        self
    }
}

impl fmt::Display for GreenTokenWithValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenWithValueData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Debug for GreenTokenWithValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenWithValueData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenTokenWithValue {
    /// Creates new token.
    #[inline]
    pub fn new(kind: SyntaxKind, value: &[u8]) -> GreenTokenWithValue {
        let flags = GreenFlags::IS_NOT_MISSING; // Tokens created via `new` are always not-missing
        let head = GreenTokenWithValueHead { kind, flags, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, value.iter().copied());
        GreenTokenWithValue { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenTokenWithValue) -> ptr::NonNull<GreenTokenWithValueData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTokenWithValueData = &green;
        ptr::NonNull::from(green)
    }

    /// # Safety
    ///
    /// This function uses `unsafe` code to create an `Arc` from a raw pointer and then transmutes it into a `ThinArc`.
    ///
    /// - The raw pointer must be valid and correctly aligned for the type `ReprThin`.
    /// - The lifetime of the raw pointer must outlive the lifetime of the `Arc` created from it.
    /// - The transmute operation must be safe, meaning that the memory layout of `Arc<ReprThin>` must be compatible with `ThinArc<GreenTokenWithValueHead, u8>`.
    ///
    /// Failure to uphold these invariants can lead to undefined behavior.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenWithValueData>) -> GreenTokenWithValue {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTokenWithValueHead, u8>>(arc)
        };
        GreenTokenWithValue { ptr: arc }
    }
}

impl ops::Deref for GreenTokenWithValue {
    type Target = GreenTokenWithValueData;

    #[inline]
    fn deref(&self) -> &GreenTokenWithValueData {
        unsafe {
            let repr: &Repr = &*self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTokenWithValueData>(repr)
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_token_head_memory_layout() {
        // GreenTokenWithValueHead: kind (2 bytes) + flags (1 byte) + _c (0 bytes)
        // Expected: 2 + 1 + 1 padding for alignment = 4 bytes
        assert_eq!(std::mem::size_of::<GreenTokenWithValueHead>(), 4);
        assert_eq!(std::mem::align_of::<GreenTokenWithValueHead>(), 2);
    }

    #[test]
    fn test_green_token_data_memory_layout() {
        // GreenTokenWithValueData on 64-bit targets:
        // header (4 bytes) + padding (4 bytes) + length (8 bytes) = 16 bytes
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValueData>(), 16);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueData>(), 8);
        }

        // GreenTokenWithValueData on 32-bit targets:
        // header (4 bytes) + length (4 bytes) = 8 bytes
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValueData>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueData>(), 4);
        }
    }

    #[test]
    fn test_green_token_memory_layout() {
        // GreenTokenWithValue wraps a ThinArc pointer.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValue>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithValue>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValue>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithValue>(), 4);
        }
    }
}

#[cfg(test)]
mod green_token_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_token() {
        let token = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42");
        assert_eq!(token.kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(token.text(), b"42");
    }

    #[test]
    fn test_new_when_created_expect_is_not_missing_flag_set() {
        let token = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42");
        assert!(token.flags().contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_kind() {
        let token = GreenTokenWithValue::new(SyntaxKind::NameLiteralToken, b"Type");
        assert_eq!(token.kind(), SyntaxKind::NameLiteralToken);
    }

    #[test]
    fn test_text() {
        let token = GreenTokenWithValue::new(SyntaxKind::StringLiteralToken, b"hello");
        assert_eq!(token.text(), b"hello");
    }

    #[test]
    fn test_width() {
        let token = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"456");
        assert_eq!(token.width(), 3);
    }

    #[test]
    fn test_eq_when_same_kind_expect_equal() {
        let token1 = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42");
        let token2 = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42");
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42");
        let token2 = GreenTokenWithValue::new(SyntaxKind::NameLiteralToken, b"42");
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_clone() {
        let token1 = GreenTokenWithValue::new(SyntaxKind::StringLiteralToken, b"test");
        let token2 = token1.clone();
        assert_eq!(token1, token2);
        assert_eq!(token2.kind(), SyntaxKind::StringLiteralToken);
        assert_eq!(token2.text(), b"test");
    }

    #[test]
    fn test_display() {
        let token = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"999");
        assert_eq!(token.to_string(), "999");
    }

    #[test]
    fn test_debug() {
        let token = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42");
        let debug_str = format!("{:?}", token);
        let expected = "GreenTokenWithValue { kind: NumericLiteralToken, text: \"42\", width: 2 }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_empty_text() {
        let token = GreenTokenWithValue::new(SyntaxKind::StringLiteralToken, b"");
        assert_eq!(token.text(), b"");
        assert_eq!(token.width(), 0);
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let token = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"777");
        let ptr = GreenTokenWithValue::into_raw(token.clone());
        let reconstructed = unsafe { GreenTokenWithValue::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow() {
        let token = GreenTokenWithValue::new(SyntaxKind::NameLiteralToken, b"abc");
        let borrowed: &GreenTokenWithValueData = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::NameLiteralToken);
        assert_eq!(borrowed.text(), b"abc");
    }
}

#[cfg(test)]
mod green_token_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned() {
        let token = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"123");
        let data: &GreenTokenWithValueData = &*token;
        let owned = data.to_owned();
        assert_eq!(token, owned);
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal() {
        let token1 = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"99");
        let token2 = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"99");
        let data1: &GreenTokenWithValueData = &*token1;
        let data2: &GreenTokenWithValueData = &*token2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"42");
        let token2 = GreenTokenWithValue::new(SyntaxKind::NameLiteralToken, b"42");
        let data1: &GreenTokenWithValueData = &*token1;
        let data2: &GreenTokenWithValueData = &*token2;
        assert_ne!(data1, data2);
    }
}
