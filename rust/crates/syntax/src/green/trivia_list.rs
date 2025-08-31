use std::{
    borrow, fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    GreenTrivia,
    arc::{Arc, HeaderSlice, ThinArc},
};
use countme::Count;

type Repr = HeaderSlice<GreenTriviaListHead, [GreenTrivia]>;
type ReprThin = HeaderSlice<GreenTriviaListHead, [GreenTrivia; 0]>;

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTriviaListHead {
    _c: Count<GreenTriviaList>,
}

#[repr(transparent)]
pub(crate) struct GreenTriviaListData {
    data: ReprThin,
}

impl GreenTriviaListData {
    #[inline]
    pub fn header(&self) -> &GreenTriviaListHead {
        &self.data.header
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
        f.debug_struct("GreenTriviaList")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .finish()
    }
}

impl fmt::Display for GreenTriviaListData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTriviaList {
    ptr: Option<ThinArc<GreenTriviaListHead, GreenTrivia>>,
}

impl GreenTriviaList {
    /// Creates a new trivia containing the passed in pieces
    pub fn new<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = GreenTrivia>,
        I::IntoIter: ExactSizeIterator,
    {
        let data = ThinArc::from_header_and_iter(GreenTriviaListHead { _c: Count::new() }, pieces.into_iter());

        GreenTriviaList { ptr: Some(data) }
    }

    /// Creates an empty trivia
    pub fn empty() -> Self {
        GreenTriviaList { ptr: None }
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
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaListHead, u8>>(arc)
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
