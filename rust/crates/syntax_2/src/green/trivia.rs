use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::arc::{Arc, HeaderSlice, ThinArc};
use countme::Count;

use crate::SyntaxKind;

#[derive(PartialEq, Eq, Hash)]
struct GreenTriviaHead {
    kind: SyntaxKind,
    _c: Count<GreenTrivia>,
}

type Repr = HeaderSlice<GreenTriviaHead, [u8]>;
type ReprThin = HeaderSlice<GreenTriviaHead, [u8; 0]>;

#[repr(transparent)]
pub struct GreenTriviaData {
    data: ReprThin,
}

impl GreenTriviaData {
    /// Kind of this trivia.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Text of this trivia.
    #[inline]
    pub fn text(&self) -> &[u8] {
        self.data.slice()
    }

    /// Returns the length of the text covered by this trivia.
    #[inline]
    pub fn text_len(&self) -> u32 {
        self.text().len() as u32
    }
}

impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl ToOwned for GreenTriviaData {
    type Owned = GreenTrivia;

    #[inline]
    fn to_owned(&self) -> GreenTrivia {
        let green = unsafe { GreenTrivia::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTrivia::clone(&green)
    }
}

impl fmt::Display for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.text())
    }
}

impl fmt::Debug for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTrivia").field("kind", &self.kind()).field("text", &self.text()).finish()
    }
}

/// Leaf node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, u8>,
}

impl Borrow<GreenTriviaData> for GreenTrivia {
    #[inline]
    fn borrow(&self) -> &GreenTriviaData {
        self
    }
}

impl fmt::Display for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenTrivia {
    /// Creates new trivia.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &str) -> GreenTrivia {
        let head = GreenTriviaHead { kind, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, text.bytes());
        GreenTrivia { ptr }
    }
    #[inline]
    pub(crate) fn into_raw(this: GreenTrivia) -> ptr::NonNull<GreenTriviaData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaData = &green;
        ptr::NonNull::from(green)
    }

    /// # Safety
    ///
    /// This function uses `unsafe` code to create an `Arc` from a raw pointer and then transmutes it into a `ThinArc`.
    ///
    /// - The raw pointer must be valid and correctly aligned for the type `ReprThin`.
    /// - The lifetime of the raw pointer must outlive the lifetime of the `Arc` created from it.
    /// - The transmute operation must be safe, meaning that the memory layout of `Arc<ReprThin>` must be compatible with `ThinArc<GreenTriviaHead, u8>`.
    ///
    /// Failure to uphold these invariants can lead to undefined behavior.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaData>) -> GreenTrivia {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaHead, u8>>(arc)
        };
        GreenTrivia { ptr: arc }
    }
}

impl ops::Deref for GreenTrivia {
    type Target = GreenTriviaData;

    #[inline]
    fn deref(&self) -> &GreenTriviaData {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTriviaData>(repr)
        }
    }
}
