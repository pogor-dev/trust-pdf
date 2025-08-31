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

type Repr = HeaderSlice<GreenTriviaListHead, [GreenTrivia]>;
type ReprThin = HeaderSlice<GreenTriviaListHead, [GreenTrivia; 0]>;

#[derive(PartialEq, Eq, Hash)]
struct GreenTriviaListHead {
    text_len: u32,
    _c: Count<GreenTriviaList>,
}

#[repr(transparent)]
pub struct GreenTriviaListData {
    data: ReprThin,
}

impl GreenTriviaListData {
    #[inline]
    pub fn text(&self) -> &[u8] {
        // TODO: fix
        &[]
        // self.data.slice()
    }

    /// Returns the full length of the trivia.
    /// It is expected to have up to 65535 bytes (e.g. long comments)
    #[inline]
    pub fn full_len(&self) -> u32 {
        self.data.header.text_len.into()
    }
}

impl ToOwned for GreenTriviaListData {
    type Owned = GreenTriviaList;

    #[inline]
    fn to_owned(&self) -> GreenTriviaList {
        let green = unsafe { GreenTriviaList::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTriviaList::clone(&green)
    }
}

impl fmt::Debug for GreenTriviaListData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTriviaList").field("text", &self.text()).finish()
    }
}

impl fmt::Display for GreenTriviaListData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        byte_to_string(self.text(), f)
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTriviaList {
    ptr: ThinArc<GreenTriviaListHead, GreenTrivia>,
}

impl GreenTriviaList {
    /// Creates a new trivia containing the passed in pieces
    pub fn new_list<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = GreenTrivia>,
        I::IntoIter: ExactSizeIterator,
    {
        let pieces_vec: Vec<GreenTrivia> = pieces.into_iter().collect();
        let text_len = pieces_vec.iter().map(|p| p.full_len() as u32).sum();
        let head = GreenTriviaListHead { text_len, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, pieces_vec.into_iter());
        GreenTriviaList { ptr }
    }

    /// Creates a single piece of trivia from the given text.
    pub fn new_single(kind: SyntaxKind, text: &[u8]) -> Self {
        let text_len = text.len() as u32;
        let head = GreenTriviaListHead { text_len, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, std::iter::once(GreenTrivia::new(kind, text)));
        GreenTriviaList { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenTriviaList) -> ptr::NonNull<GreenTriviaListData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaListData = &green;
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
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaListData>) -> GreenTriviaList {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaListHead, GreenTrivia>>(arc)
        };
        GreenTriviaList { ptr: arc }
    }
}

impl fmt::Debug for GreenTriviaList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaListData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenTriviaList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaListData = self;
        fmt::Display::fmt(data, f)
    }
}

impl borrow::Borrow<GreenTriviaListData> for GreenTriviaList {
    #[inline]
    fn borrow(&self) -> &GreenTriviaListData {
        self
    }
}

impl ops::Deref for GreenTriviaList {
    type Target = GreenTriviaListData;

    #[inline]
    fn deref(&self) -> &GreenTriviaListData {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTriviaListData>(repr)
        }
    }
}
