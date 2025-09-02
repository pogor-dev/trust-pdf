use std::{
    borrow, fmt, iter,
    mem::{self, ManuallyDrop},
    ops, ptr, slice,
};

use countme::Count;

use crate::{
    GreenToken, NodeOrToken, SyntaxKind,
    arc::{Arc, HeaderSlice, ThinArc},
    green::{byte_to_string, element::GreenElement},
};

type Repr = HeaderSlice<GreenNodeHead, [Slot]>;
type ReprThin = HeaderSlice<GreenNodeHead, [Slot; 0]>;

#[derive(PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    kind: SyntaxKind,
    full_text_len: u32, // TODO: limitation 4GB?
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
    pub(crate) fn slice(&self) -> &[Slot] {
        self.data.slice()
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        &[] // TODO: fix
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        let mut combined = Vec::with_capacity(self.full_text_len() as usize);

        for element in self.data.slice() {
            // combined.extend_from_slice(element.full_text());
        }

        combined
    }

    #[inline]
    pub fn text_len(&self) -> u32 {
        0 // TODO: fix
    }

    #[inline]
    pub fn full_text_len(&self) -> u32 {
        self.data.header.full_text_len
    }

    /// Returns the slots of this node. Every node of a specific kind has the same number of slots
    /// to allow using fixed offsets to retrieve a specific child even if some other child is missing.
    #[inline]
    pub(crate) fn slots(&self) -> Slots<'_> {
        Slots { raw: self.slice().iter() }
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
    ptr: ThinArc<GreenNodeHead, Slot>,
}

impl GreenNode {
    /// Creates a new node containing the passed in pieces
    pub fn new_list<I>(kind: SyntaxKind, slots: I) -> Self
    where
        I: IntoIterator<Item = GreenElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut full_text_len = 0;
        let children = slots.into_iter().map(|el| {
            let rel_offset = full_text_len;
            full_text_len += el.full_text_len();
            match el {
                NodeOrToken::Node(node) => Slot::Node { rel_offset, node },
                NodeOrToken::Token(token) => Slot::Token { rel_offset, token },
            }
        });

        let data = ThinArc::from_header_and_iter(
            GreenNodeHead {
                kind,
                full_text_len: 0,
                _c: Count::new(),
            },
            children,
        );

        // XXX: fixup `full_text_len` after construction, because we can't iterate
        // `children` twice.
        let data = {
            let mut data = Arc::from_thin(data);
            Arc::get_mut(&mut data).unwrap().header.full_text_len = full_text_len;
            Arc::into_thin(data)
        };

        GreenNode { ptr: data }
    }

    /// Creates a single piece of node from the given text.
    pub fn new_single(kind: SyntaxKind, slot: GreenElement) -> Self {
        GreenNode::new_list(kind, std::iter::once(slot))
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
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenNodeHead, Slot>>(arc)
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Slot {
    Node { rel_offset: u32, node: GreenNode },
    Token { rel_offset: u32, token: GreenToken },
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Slot::Node { node, .. } => fmt::Display::fmt(node, f),
            Slot::Token { token, .. } => fmt::Display::fmt(token, f),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Slots<'a> {
    pub(crate) raw: slice::Iter<'a, Slot>,
}

// NB: forward everything stable that iter::Slice specializes as of Rust 1.39.0
impl ExactSizeIterator for Slots<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.raw.len()
    }
}

impl<'a> Iterator for Slots<'a> {
    type Item = &'a Slot;

    #[inline]
    fn next(&mut self) -> Option<&'a Slot> {
        self.raw.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.raw.count()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth(n)
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let mut accum = init;
        for x in self {
            accum = f(accum, x);
        }
        accum
    }
}

impl<'a> DoubleEndedIterator for Slots<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth_back(n)
    }

    #[inline]
    fn rfold<Acc, Fold>(mut self, init: Acc, mut f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let mut accum = init;
        while let Some(x) = self.next_back() {
            accum = f(accum, x);
        }
        accum
    }
}

impl iter::FusedIterator for Slots<'_> {}
