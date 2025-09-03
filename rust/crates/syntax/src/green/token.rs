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
        let mut combined = Vec::new();
        combined
        // let leading = self.data.header.leading_token.as_ref().map(|t| t.text()).unwrap_or(&[]);
        // let trailing = self.data.header.trailing_token.as_ref().map(|t| t.text()).unwrap_or(&[]);
        // let text = self.text();

        // let total_len = leading.len() + text.len() + trailing.len();
        // let mut combined = Vec::with_capacity(total_len);

        // if total_len == 0 {
        //     return combined;
        // }

        // combined.extend_from_slice(leading);
        // combined.extend_from_slice(text);
        // combined.extend_from_slice(trailing);
        // combined
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
        f.debug_struct("GreenToken").field("kind", &self.kind()).field("text", &self.text()).finish()
    }
}

impl fmt::Display for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", byte_to_string(self.text()))
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
    pub fn new_with_token(kind: SyntaxKind, text: &[u8], leading_token: GreenTrivia, trailing_token: GreenTrivia) -> GreenToken {
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
