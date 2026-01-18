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
        write!(f, "{:?}", self.text())
    }
}

impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenToken").field("kind", &self.kind()).field("text", &self.text()).finish()
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
