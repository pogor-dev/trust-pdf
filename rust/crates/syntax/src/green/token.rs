use std::{
    borrow, fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    GreenTrivia, SyntaxKind,
    arc::{Arc, HeaderSlice, ThinArc},
    green::byte_to_string,
};
use countme::Count;

type Repr = HeaderSlice<GreenTokenHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;

#[derive(PartialEq, Eq, Hash)]
pub(super) struct GreenTokenHead {
    kind: SyntaxKind,
    full_text_len: u32,
    leading_token: Option<GreenTrivia>,
    trailing_token: Option<GreenTrivia>,
    _c: Count<GreenToken>,
}

#[repr(transparent)]
pub struct GreenTokenData {
    data: ReprThin,
}

impl GreenTokenData {
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
        let leading = self.data.header.leading_token.as_ref().map(|t| t.text());
        let trailing = self.data.header.trailing_token.as_ref().map(|t| t.text());
        let text = self.text();

        let full_text_len = self.full_text_len() as usize;
        let mut combined = Vec::with_capacity(full_text_len);

        if full_text_len == 0 {
            return combined;
        }

        if let Some(leading) = leading {
            combined.extend_from_slice(leading.as_slice());
        }

        combined.extend_from_slice(text);

        if let Some(trailing) = trailing {
            combined.extend_from_slice(trailing.as_slice());
        }

        combined
    }

    /// Returns the length of the token, excluding leading or trailing trivia.
    #[inline]
    pub fn text_len(&self) -> u32 {
        self.text().len() as u32
    }

    /// Returns the full length of the token, including leading or trailing trivia.
    #[inline]
    pub fn full_text_len(&self) -> u32 {
        self.data.header.full_text_len
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<&GreenTrivia> {
        self.data.header.leading_token.as_ref()
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<&GreenTrivia> {
        self.data.header.trailing_token.as_ref()
    }

    #[inline]
    pub fn leading_trivia_width(&self) -> u32 {
        self.leading_trivia().map_or(0, |t| t.text_len())
    }

    #[inline]
    pub fn trailing_trivia_width(&self) -> u32 {
        self.trailing_trivia().map_or(0, |t| t.text_len())
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

impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("text", &byte_to_string(&self.text()))
            .field("full_text", &byte_to_string(&self.full_text()))
            .finish()
    }
}

impl fmt::Display for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", byte_to_string(&self.full_text()))
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenToken {
    ptr: ThinArc<GreenTokenHead, u8>,
}

impl GreenToken {
    /// Creates new Token.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenToken {
        let head = GreenTokenHead {
            kind,
            full_text_len: text.len() as u32,
            leading_token: None,
            trailing_token: None,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenToken { ptr }
    }

    /// Creates new Token.
    #[inline]
    pub fn new_with_leading_token(kind: SyntaxKind, text: &[u8], leading_token: GreenTrivia) -> GreenToken {
        let head = GreenTokenHead {
            kind,
            full_text_len: (text.len() as u32) + (leading_token.text_len() as u32),
            leading_token: Some(leading_token),
            trailing_token: None,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenToken { ptr }
    }

    /// Creates new Token.
    #[inline]
    pub fn new_with_trailing_token(kind: SyntaxKind, text: &[u8], trailing_token: GreenTrivia) -> GreenToken {
        let head = GreenTokenHead {
            kind,
            full_text_len: (text.len() as u32) + (trailing_token.text_len() as u32),
            leading_token: None,
            trailing_token: Some(trailing_token),
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenToken { ptr }
    }

    /// Creates new Token.
    #[inline]
    pub fn new_with_trivia(kind: SyntaxKind, text: &[u8], leading_token: GreenTrivia, trailing_token: GreenTrivia) -> GreenToken {
        let head = GreenTokenHead {
            kind,
            full_text_len: (text.len() as u32) + (leading_token.text_len() as u32) + (trailing_token.text_len() as u32),
            leading_token: Some(leading_token),
            trailing_token: Some(trailing_token),
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

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Display::fmt(data, f)
    }
}

impl borrow::Borrow<GreenTokenData> for GreenToken {
    #[inline]
    fn borrow(&self) -> &GreenTokenData {
        self
    }
}

impl ops::Deref for GreenToken {
    type Target = GreenTokenData;

    #[inline]
    fn deref(&self) -> &GreenTokenData {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTokenData>(repr)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_new() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let token = GreenToken::new(kind, text);
        assert_eq!(token.kind(), kind);
        assert_eq!(token.text(), text);
    }

    #[rstest]
    fn test_with_leading_trivia() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let token = GreenToken::new_with_leading_token(kind, text, leading_trivia.clone());
        assert_eq!(token.kind(), kind);
        assert_eq!(token.text(), text);
        assert_eq!(token.leading_trivia(), Some(&leading_trivia));
    }

    #[rstest]
    fn test_with_trailing_trivia() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trailing_token(kind, text, trailing_trivia.clone());
        assert_eq!(token.kind(), kind);
        assert_eq!(token.text(), text);
        assert_eq!(token.trailing_trivia(), Some(&trailing_trivia));
    }

    #[rstest]
    fn test_with_trivia() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());
        assert_eq!(token.kind(), kind);
        assert_eq!(token.text(), text);
        assert_eq!(token.leading_trivia(), Some(&leading_trivia));
        assert_eq!(token.trailing_trivia(), Some(&trailing_trivia));
    }

    #[rstest]
    #[allow(useless_ptr_null_checks)]
    fn test_into_raw_pointer() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());
        let ptr: ptr::NonNull<GreenTokenData> = GreenToken::into_raw(token.clone());
        assert!(!ptr.as_ptr().is_null());
    }

    #[rstest]
    fn test_from_raw_pointer() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());
        let ptr: ptr::NonNull<GreenTokenData> = GreenToken::into_raw(token.clone());
        let recovered = unsafe { GreenToken::from_raw(ptr) };
        assert_eq!(recovered.kind(), kind);
        assert_eq!(recovered.text(), text);
        assert_eq!(recovered.leading_trivia(), Some(&leading_trivia));
        assert_eq!(recovered.trailing_trivia(), Some(&trailing_trivia));
    }

    #[rstest]
    fn test_fmt_debug() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());
        let debug_str = format!("{:?}", token);
        assert_eq!(debug_str, "GreenToken { kind: SyntaxKind(1), text: \"test\", full_text: \" test\\n\" }");
    }

    #[rstest]
    fn test_fmt_display() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());
        let display_str = format!("{}", token);
        assert_eq!(display_str, " test\n");
    }

    #[rstest]
    fn test_borrowing() {
        use std::borrow::Borrow;

        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());

        let borrowed = token.borrow();
        let data: &GreenTokenData = &borrowed;
        let owned = data.to_owned();
        assert_eq!(owned.text(), borrowed.text());
    }

    #[rstest]
    fn test_kind() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());

        assert_eq!(token.kind(), kind);
    }

    #[rstest]
    fn test_text() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());

        assert_eq!(token.text(), text);
    }

    #[rstest]
    #[case::token_with_trivia(b" ", b"test", b"\n", b" test\n")]
    #[case::token_without_trivia(b"", b"test", b"", b"test")]
    #[case::token_empty(b"", b"", b"", b"")]
    #[case::token_with_leading_trivia(b" ", b"test", b"", b" test")]
    #[case::token_with_trailing_trivia(b"", b"test", b"\n", b"test\n")]
    fn test_full_text(#[case] leading: &[u8], #[case] text: &[u8], #[case] trailing: &[u8], #[case] expected: &[u8]) {
        let kind = SyntaxKind(1);
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), leading);
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), trailing);
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());
        assert_eq!(token.full_text(), expected);
    }

    #[rstest]
    fn test_text_len() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());

        assert_eq!(token.text_len(), 4);
    }

    #[rstest]
    #[case::token_with_trivia(b" ", b"test", b"\n", 6)]
    #[case::token_without_trivia(b"", b"test", b"", 4)]
    #[case::token_empty(b"", b"", b"", 0)]
    #[case::token_with_leading_trivia(b" ", b"test", b"", 5)]
    #[case::token_with_trailing_trivia(b"", b"test", b"\n", 5)]
    fn test_full_text_length(#[case] leading: &[u8], #[case] text: &[u8], #[case] trailing: &[u8], #[case] expected_len: u32) {
        let kind = SyntaxKind(1);
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), leading);
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), trailing);
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());
        assert_eq!(token.full_text_len() as u32, expected_len);
    }

    #[rstest]
    fn test_leading_trivia() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());

        assert_eq!(token.leading_trivia(), Some(&leading_trivia));
    }

    #[rstest]
    fn test_trailing_trivia() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());

        assert_eq!(token.trailing_trivia(), Some(&trailing_trivia));
    }

    #[rstest]
    fn test_leading_trivia_width() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());

        assert_eq!(token.leading_trivia_width(), 1);
    }

    #[rstest]
    fn test_trailing_trivia_width() {
        let kind = SyntaxKind(1);
        let text = b"test";
        let leading_trivia = GreenTrivia::new_single(SyntaxKind(2), b" ");
        let trailing_trivia = GreenTrivia::new_single(SyntaxKind(3), b"\n");
        let token = GreenToken::new_with_trivia(kind, text, leading_trivia.clone(), trailing_trivia.clone());

        assert_eq!(token.trailing_trivia_width(), 1);
    }
}
