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
    arc::{Arc, HeaderSlice, ThinArc},
    green::flags::GreenFlags,
};
use countme::Count;

use crate::SyntaxKind;

pub(crate) type GreenTokenWithIntValue = GreenTokenWithValue<u32>;
pub(crate) type GreenTokenWithFloatValue = GreenTokenWithValue<f32>;
pub(crate) type GreenTokenWithStringValue = GreenTokenWithValue<String>;
pub(crate) type GreenTokenWithIntValueData = GreenTokenWithValueData<u32>;
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

type Repr<T> = HeaderSlice<GreenTokenWithValueHead<T>, [u8]>;
type ReprThin<T> = HeaderSlice<GreenTokenWithValueHead<T>, [u8; 0]>;

#[repr(transparent)]
/// Borrowed token view for well-known text tokens.
///
/// The underlying text is not stored in the node; it is derived from
/// `SyntaxKind` at read time.
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

    /// Returns the flags of this token.
    #[inline]
    pub fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }
}

impl<T> PartialEq for GreenTokenWithValueData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl<T: Clone> ToOwned for GreenTokenWithValueData<T> {
    type Owned = GreenTokenWithValue<T>;

    #[inline]
    fn to_owned(&self) -> GreenTokenWithValue<T> {
        let green = unsafe { GreenTokenWithValue::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTokenWithValue::<T>::clone(&green)
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

impl<T> Borrow<GreenTokenWithValueData<T>> for GreenTokenWithValue<T> {
    #[inline]
    fn borrow(&self) -> &GreenTokenWithValueData<T> {
        self
    }
}

impl<T> fmt::Display for GreenTokenWithValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenWithValueData<T> = self;
        fmt::Display::fmt(data, f)
    }
}

impl<T> fmt::Debug for GreenTokenWithValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenWithValueData<T> = self;
        fmt::Debug::fmt(data, f)
    }
}

impl<T> GreenTokenWithValue<T> {
    /// Creates new token.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8], value: T) -> GreenTokenWithValue<T> {
        let flags = GreenFlags::IS_NOT_MISSING; // Tokens created via `new` are always not-missing
        let head = GreenTokenWithValueHead::<T> {
            kind,
            flags,
            value,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenTokenWithValue { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenTokenWithValue<T>) -> ptr::NonNull<GreenTokenWithValueData<T>> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTokenWithValueData<T> = &green;
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
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenWithValueData<T>>) -> GreenTokenWithValue<T> {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin<T>);
            mem::transmute::<Arc<ReprThin<T>>, ThinArc<GreenTokenWithValueHead<T>, u8>>(arc)
        };
        GreenTokenWithValue { ptr: arc }
    }
}

impl<T> ops::Deref for GreenTokenWithValue<T> {
    type Target = GreenTokenWithValueData<T>;

    #[inline]
    fn deref(&self) -> &GreenTokenWithValueData<T> {
        unsafe {
            let repr: &Repr<T> = &*self.ptr;
            let repr: &ReprThin<T> = &*(repr as *const Repr<T> as *const ReprThin<T>);
            mem::transmute::<&ReprThin<T>, &GreenTokenWithValueData<T>>(repr)
        }
    }
}

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
}

#[cfg(test)]
mod green_token_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned_when_pdf_number_expect_equal_token_with_value() {
        let token: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"123", 123);
        let data: &GreenTokenWithValueData<u32> = &*token;
        let owned = data.to_owned();
        assert_eq!(token, owned);
        assert_eq!(owned.value(), &123);
    }

    #[test]
    fn test_eq_when_same_kind_and_same_text_expect_equal() {
        let token1: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"99", 99);
        let token2: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"99", 100);
        let data1: &GreenTokenWithValueData<u32> = &*token1;
        let data2: &GreenTokenWithValueData<u32> = &*token2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_same_kind_and_different_text_expect_not_equal() {
        let token1: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"99", 99);
        let token2: GreenTokenWithIntValue = GreenTokenWithValue::new(SyntaxKind::NumericLiteralToken, b"100", 100);
        let data1: &GreenTokenWithValueData<u32> = &*token1;
        let data2: &GreenTokenWithValueData<u32> = &*token2;
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
