use std::{
    borrow, fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use countme::Count;

use crate::{
    SyntaxKind,
    arc::{Arc, HeaderSlice, ThinArc},
    green::{byte_to_string, element::GreenElement},
};

type Repr = HeaderSlice<GreenNodeHead, [GreenElement]>;
type ReprThin = HeaderSlice<GreenNodeHead, [GreenElement; 0]>;

#[derive(PartialEq, Eq, Hash)]
struct GreenNodeHead {
    kind: SyntaxKind,
    full_text_len: u32,
    _c: Count<GreenNode>,
}

#[repr(transparent)]
pub struct GreenNodeData {
    data: ReprThin,
}

impl GreenNodeData {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        let mut combined = Vec::with_capacity(self.full_text_len() as usize);

        for element in self.data.slice() {
            combined.extend_from_slice(element.full_text());
        }

        combined
    }

    #[inline]
    pub fn full_text_len(&self) -> u32 {
        self.data.header.full_text_len
    }
}

impl ToOwned for GreenNodeData {
    type Owned = GreenNode;

    #[inline]
    fn to_owned(&self) -> GreenNode {
        let green = unsafe { GreenNode::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenNode::clone(&green)
    }
}

impl fmt::Debug for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenNode")
            .field("kind", &self.kind())
            .field("text", &self.full_text()) // TODO: replace with text?
            .finish()
    }
}

impl fmt::Display for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        byte_to_string(&self.full_text(), f) // TODO: replace with text?
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenNode {
    ptr: ThinArc<GreenNodeHead, GreenElement>,
}

impl GreenNode {
    /// Creates a new node containing the passed in pieces
    pub fn new_list<I>(kind: SyntaxKind, pieces: I) -> Self
    where
        I: IntoIterator<Item = GreenElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let pieces_vec: Vec<GreenElement> = pieces.into_iter().collect();
        let full_text_len = pieces_vec.iter().map(|p| p.full_len() as u32).sum();
        let head = GreenNodeHead {
            kind,
            full_text_len,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, pieces_vec.into_iter());
        GreenNode { ptr }
    }

    /// Creates a single piece of node from the given text.
    pub fn new_single(kind: SyntaxKind, piece: GreenElement) -> Self {
        let full_text_len = piece.full_tesxt_len();
        let head = GreenNodeHead {
            kind,
            full_text_len,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, std::iter::once(piece));
        GreenNode { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenNode) -> ptr::NonNull<GreenNodeData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenNodeData = &green;
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
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenNodeData>) -> GreenNode {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenNodeHead, GreenElement>>(arc)
        };
        GreenNode { ptr: arc }
    }
}

impl fmt::Debug for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenNodeData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenNodeData = self;
        fmt::Display::fmt(data, f)
    }
}

impl borrow::Borrow<GreenNodeData> for GreenNode {
    #[inline]
    fn borrow(&self) -> &GreenNodeData {
        self
    }
}

impl ops::Deref for GreenNode {
    type Target = GreenNodeData;

    #[inline]
    fn deref(&self) -> &GreenNodeData {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenNodeData>(repr)
        }
    }
}
