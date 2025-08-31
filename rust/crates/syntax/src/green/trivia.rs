use std::{
    borrow, fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    SyntaxKind,
    arc::{Arc, HeaderSlice, ThinArc},
    green::byte_to_string,
};
use countme::Count;

type Repr = HeaderSlice<GreenTriviaHead, [u8]>;
type ReprThin = HeaderSlice<GreenTriviaHead, [u8; 0]>;

#[derive(PartialEq, Eq, Hash)]
struct GreenTriviaHead {
    kind: SyntaxKind,
    _c: Count<GreenTrivia>,
}

#[repr(transparent)]
pub struct GreenTriviaData {
    data: ReprThin,
}

impl GreenTriviaData {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        self.data.slice()
    }

    /// Returns the full length of the trivia.
    /// It is expected to have up to 65535 bytes (e.g. long comments)
    #[inline]
    pub fn full_len(&self) -> u16 {
        self.text().len() as u16
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

impl fmt::Debug for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTrivia").field("kind", &self.kind()).field("text", &self.text()).finish()
    }
}

impl fmt::Display for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        byte_to_string(self.text(), f)
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, u8>,
}

impl GreenTrivia {
    /// Creates new Token.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        let head = GreenTriviaHead { kind, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
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
    /// - The transmute operation must be safe, meaning that the memory layout of `Arc<ReprThin>` must be compatible with `ThinArc<GreenTokenHead, u8>`.
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

impl fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaData = self;
        fmt::Display::fmt(data, f)
    }
}

impl borrow::Borrow<GreenTriviaData> for GreenTrivia {
    #[inline]
    fn borrow(&self) -> &GreenTriviaData {
        self
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
