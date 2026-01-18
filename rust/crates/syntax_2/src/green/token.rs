use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    GreenNode,
    arc::{Arc, HeaderSlice, ThinArc},
};
use countme::Count;

use crate::SyntaxKind;

#[derive(PartialEq, Eq, Hash)]
struct GreenTokenHead {
    kind: SyntaxKind,
    full_width: u32,
    leading_trivia: GreenNode,
    trailing_trivia: GreenNode,
    _c: Count<GreenToken>,
}

type Repr = HeaderSlice<GreenTokenHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;

#[repr(transparent)]
pub struct GreenTokenData {
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
        self.data.slice()
    }

    /// Full text of this token.
    #[inline]
    pub fn full_text(&self) -> &[u8] {
        self.text()
    }

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn width(&self) -> u32 {
        self.data.header.full_width - self.leading_trivia().full_width() - self.trailing_trivia().full_width()
    }

    /// Returns the full width of this token.
    #[inline]
    pub fn full_width(&self) -> u32 {
        self.data.header.full_width
    }

    /// The leading trivia of this token.
    #[inline]
    pub fn leading_trivia(&self) -> GreenNode {
        self.data.header.leading_trivia.clone()
    }

    /// The trailing trivia of this token.
    #[inline]
    pub fn trailing_trivia(&self) -> GreenNode {
        self.data.header.trailing_trivia.clone()
    }

    /// Writes the token to a byte vector with conditional trivia inclusion
    ///
    /// # Parameters
    /// * `leading` - If true, include the leading trivia
    /// * `trailing` - If true, include the trailing trivia
    pub(super) fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        let mut output = Vec::new();

        if leading {
            output.extend_from_slice(&self.leading_trivia().full_text());
        }

        let text = self.text();
        output.extend_from_slice(text);

        if trailing {
            output.extend_from_slice(&self.trailing_trivia().full_text());
        }

        output
    }
}

impl PartialEq for GreenTokenData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
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
        let full_text = self.full_text();
        let text_str = String::from_utf8_lossy(text);
        let full_text_str = String::from_utf8_lossy(full_text);

        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .field("full_text", &full_text_str)
            .field("full_width", &self.full_width())
            .field("leading_trivia", &self.leading_trivia())
            .field("trailing_trivia", &self.trailing_trivia())
            .finish()
    }
}

/// Leaf node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenToken {
    ptr: ThinArc<GreenTokenHead, u8>,
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
    /// Creates new token.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8], leading_trivia: GreenNode, trailing_trivia: GreenNode) -> GreenToken {
        assert!(text.len() <= u32::MAX as usize, "token text length exceeds u32::MAX");
        let full_width = text.len() as u32 + leading_trivia.full_width() + trailing_trivia.full_width();
        let head = GreenTokenHead {
            kind,
            full_width,
            leading_trivia,
            trailing_trivia,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenToken { ptr }
    }
    #[inline]
    pub(crate) fn into_raw(this: GreenToken) -> ptr::NonNull<GreenTokenData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTokenData = &green;
        ptr::NonNull::from(green)
    }

    /// # Safety
    ///
    /// This function uses `unsafe` code to create an `Arc` from a raw pointer and then transmutes it into a `ThinArc`.
    ///
    /// - The raw pointer must be valid and correctly aligned for the type `ReprThin`.
    /// - The lifetime of the raw pointer must outlive the lifetime of the `Arc` created from it.
    /// - The transmute operation must be safe, meaning that the memory layout of `Arc<ReprThin>` must be compatible with `ThinArc<GreenTokenHead, u8>`.
    ///
    /// Failure to uphold these invariants can lead to undefined behavior.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> GreenToken {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTokenHead, u8>>(arc)
        };
        GreenToken { ptr: arc }
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

    #[test]
    fn test_green_token_head_memory_layout() {
        // GreenTokenHead: kind (2 bytes) + full_width (4 bytes) + 2 GreenNode pointers + _c (0 bytes)
        assert!(std::mem::size_of::<GreenTokenHead>() >= 6);
    }

    #[test]
    fn test_green_token_data_memory_layout() {
        // GreenTokenData is transparent wrapper around ReprThin
        assert!(std::mem::size_of::<GreenTokenData>() >= std::mem::size_of::<GreenTokenHead>());
    }

    #[test]
    fn test_green_token_memory_layout() {
        // GreenToken wraps ThinArc pointer (8 bytes on 64-bit)
        assert_eq!(std::mem::size_of::<GreenToken>(), std::mem::size_of::<usize>());
        assert_eq!(std::mem::align_of::<GreenToken>(), std::mem::align_of::<usize>());
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn empty_trivia_list() -> GreenNode {
        GreenNode::new(SyntaxKind::List, vec![])
    }

    #[test]
    fn test_new_token() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        assert_eq!(token.kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(token.text(), b"42");
    }

    #[test]
    fn test_kind() {
        let token = GreenToken::new(SyntaxKind::NameLiteralToken, b"foo", empty_trivia_list(), empty_trivia_list());
        assert_eq!(token.kind(), SyntaxKind::NameLiteralToken);
    }

    #[test]
    fn test_text() {
        let token = GreenToken::new(SyntaxKind::StringLiteralToken, b"hello", empty_trivia_list(), empty_trivia_list());
        assert_eq!(token.text(), b"hello");
    }

    #[test]
    fn test_full_text() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"123", empty_trivia_list(), empty_trivia_list());
        assert_eq!(token.full_text(), b"123");
    }

    #[test]
    fn test_width() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"456", empty_trivia_list(), empty_trivia_list());
        assert_eq!(token.width(), 3);
    }

    #[test]
    fn test_full_width() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"789", empty_trivia_list(), empty_trivia_list());
        assert_eq!(token.full_width(), 3);
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_eq_when_different_text_expect_not_equal() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"43", empty_trivia_list(), empty_trivia_list());
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NameLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_clone() {
        let token1 = GreenToken::new(SyntaxKind::StringLiteralToken, b"test", empty_trivia_list(), empty_trivia_list());
        let token2 = token1.clone();
        assert_eq!(token1, token2);
        assert_eq!(token2.kind(), SyntaxKind::StringLiteralToken);
        assert_eq!(token2.text(), b"test");
    }

    #[test]
    fn test_display() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"999", empty_trivia_list(), empty_trivia_list());
        assert_eq!(token.to_string(), "999");
    }

    #[test]
    fn test_debug() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let debug_str = format!("{:?}", token);
        let expected = "GreenToken { kind: NumericLiteralToken, text: \"42\", width: 2, full_text: \"42\", full_width: 2, leading_trivia: GreenNode { kind: List, full_width: 0, slot_count: 0 }, trailing_trivia: GreenNode { kind: List, full_width: 0, slot_count: 0 } }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_empty_text() {
        let token = GreenToken::new(SyntaxKind::NameLiteralToken, b"", empty_trivia_list(), empty_trivia_list());
        assert_eq!(token.text(), b"");
        assert_eq!(token.width(), 0);
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"777", empty_trivia_list(), empty_trivia_list());
        let ptr = GreenToken::into_raw(token.clone());
        let reconstructed = unsafe { GreenToken::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow() {
        let token = GreenToken::new(SyntaxKind::NameLiteralToken, b"abc", empty_trivia_list(), empty_trivia_list());
        let borrowed: &GreenTokenData = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::NameLiteralToken);
        assert_eq!(borrowed.text(), b"abc");
    }

    #[test]
    fn test_to_owned() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"123", empty_trivia_list(), empty_trivia_list());
        let data: &GreenTokenData = &*token;
        let owned = data.to_owned();
        assert_eq!(token, owned);
    }

    #[test]
    fn test_green_token_data_eq_when_same_kind_and_text_expect_equal() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"99", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"99", empty_trivia_list(), empty_trivia_list());
        let data1: &GreenTokenData = &*token1;
        let data2: &GreenTokenData = &*token2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_green_token_data_eq_when_different_text_expect_not_equal() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"11", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"22", empty_trivia_list(), empty_trivia_list());
        let data1: &GreenTokenData = &*token1;
        let data2: &GreenTokenData = &*token2;
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_green_token_data_eq_when_different_kind_expect_not_equal() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NameLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let data1: &GreenTokenData = &*token1;
        let data2: &GreenTokenData = &*token2;
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_write_to_with_no_trivia() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let data: &GreenTokenData = &*token;
        assert_eq!(data.write_to(false, false), b"42");
        assert_eq!(data.write_to(true, true), b"42");
    }
}
