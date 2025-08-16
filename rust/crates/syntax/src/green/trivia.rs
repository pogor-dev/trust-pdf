//! Immutable, shareable collections of PDF trivia elements with efficient memory layout.

use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use countme::Count;

use crate::{
    SyntaxKind,
    arc::{arc_main::Arc, header_slice::HeaderSlice, thin_arc::ThinArc},
};

type ReprThin = HeaderSlice<GreenTriviaHead, [GreenTriviaChild; 0]>;
type Repr = HeaderSlice<GreenTriviaHead, [GreenTriviaChild]>;

type ChildReprThin = HeaderSlice<GreenTriviaChildHead, [u8; 0]>;
type ChildRepr = HeaderSlice<GreenTriviaChildHead, [u8]>;

/// Immutable PDF trivia collection with efficient sharing.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, GreenTriviaChild>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct GreenTriviaHead {
    _c: Count<GreenTrivia>,
}

#[repr(transparent)]
pub struct GreenTriviaData {
    /// Underlying thin representation providing access to both header and body
    data: ReprThin,
}

impl GreenTrivia {
    /// Creates a new trivia collection from an iterator of trivia children.
    #[inline]
    pub fn new<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = GreenTriviaChild>,
        I::IntoIter: ExactSizeIterator,
    {
        let data =
            ThinArc::from_header_and_iter(GreenTriviaHead { _c: Count::new() }, pieces.into_iter());

        GreenTrivia { ptr: data }
    }

    /// Converts the trivia collection to a raw pointer for FFI operations.
    ///
    /// # Safety
    /// The returned pointer must be converted back using `from_raw` to prevent memory leaks.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn into_raw(this: GreenTrivia) -> ptr::NonNull<GreenTriviaData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaData = &green;
        ptr::NonNull::from(green)
    }

    /// Creates a trivia collection from a raw pointer.
    ///
    /// # Safety
    /// The pointer must have been created by `into_raw` and not yet reclaimed.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaData>) -> GreenTrivia {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaHead, GreenTriviaChild>>(arc)
        };
        GreenTrivia { ptr: arc }
    }
}

impl Borrow<GreenTriviaData> for GreenTrivia {
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
            // Zero-cost memory layout reinterpretation for efficient access
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTriviaData>(repr)
        }
    }
}

impl std::fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the Deref trait to access GreenTriviaData and its Debug impl
        (**self).fmt(f)
    }
}

impl GreenTriviaData {
    /// Returns a slice of all trivia children in this collection.
    #[inline]
    pub fn children(&self) -> &[GreenTriviaChild] {
        self.data.slice()
    }

    /// Returns the total byte width of all trivia children in this collection.
    #[inline]
    pub fn width(&self) -> u32 {
        self.children().iter().map(|c| c.width()).sum()
    }

    /// Returns the concatenated text content of all trivia children as a Vec<u8>.
    #[inline]
    pub fn text(&self) -> Vec<u8> {
        let total_width = self.width() as usize;
        let mut result = Vec::with_capacity(total_width);

        for child in self.children() {
            result.extend_from_slice(child.text());
        }
        result
    }
}

impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.children() == other.children()
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
        f.debug_list().entries(self.children().iter()).finish()
    }
}

impl fmt::Display for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in self.children() {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTriviaChild {
    /// Single allocation for metadata + text content
    ptr: ThinArc<GreenTriviaChildHead, u8>,
}

#[derive(PartialEq, Eq, Hash)]
struct GreenTriviaChildHead {
    kind: SyntaxKind,
    _c: Count<GreenTriviaChild>,
}

#[repr(transparent)]
pub struct GreenTriviaChildData {
    /// Underlying thin Childrepresentation providing access to both header and body
    data: ChildReprThin,
}

impl GreenTriviaChild {
    /// Creates PDF trivia preserving exact bytes for round-trip fidelity.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTriviaChild {
        let head = GreenTriviaChildHead {
            kind,
            _c: Count::new(),
        };

        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenTriviaChild { ptr }
    }

    /// Transfers ownership to raw pointer for FFI/custom allocators.
    ///
    /// Caller must eventually free the returned pointer.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn into_raw(this: GreenTriviaChild) -> ptr::NonNull<GreenTriviaChildData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaChildData = &green;
        ptr::NonNull::from(green)
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaChildData>) -> GreenTriviaChild {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ChildReprThin);
            mem::transmute::<Arc<ChildReprThin>, ThinArc<GreenTriviaChildHead, u8>>(arc)
        };
        GreenTriviaChild { ptr: arc }
    }
}

impl Borrow<GreenTriviaChildData> for GreenTriviaChild {
    /// Borrows trivia data for collections and generic operations.
    #[inline]
    fn borrow(&self) -> &GreenTriviaChildData {
        self
    }
}

impl ops::Deref for GreenTriviaChild {
    type Target = GreenTriviaChildData;

    /// Zero-cost conversion via memory layout reinterpretation.
    #[inline]
    fn deref(&self) -> &GreenTriviaChildData {
        unsafe {
            // Zero-cost memory layout reinterpretation for efficient access
            let repr: &ChildRepr = &self.ptr;
            let repr: &ChildReprThin = &*(repr as *const ChildRepr as *const ChildReprThin);
            mem::transmute::<&ChildReprThin, &GreenTriviaChildData>(repr)
        }
    }
}

impl fmt::Display for GreenTriviaChild {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTriviaChildData = self;
        fmt::Display::fmt(data, f)
    }
}

impl std::fmt::Debug for GreenTriviaChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the Deref trait to access GreenTriviaData and its Debug impl
        (**self).fmt(f)
    }
}

impl GreenTriviaChildData {
    /// Returns the semantic kind of this trivia element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Returns the raw byte content of this trivia element.
    #[inline]
    pub fn text(&self) -> &[u8] {
        let slice = self.data.slice();
        unsafe { std::slice::from_raw_parts(slice.as_ptr(), slice.len()) }
    }

    /// Returns the byte width (length) of this trivia element.
    #[inline]
    pub fn width(&self) -> u32 {
        self.text().len() as u32
    }
}

impl PartialEq for GreenTriviaChildData {
    /// Compares trivia for semantic equality (kind + content).
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl ToOwned for GreenTriviaChildData {
    type Owned = GreenTriviaChild;

    /// Converts borrowed trivia to owned with reference counting (zero-copy).
    #[inline]
    fn to_owned(&self) -> GreenTriviaChild {
        let green = unsafe { GreenTriviaChild::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTriviaChild::clone(&green)
    }
}

impl fmt::Debug for GreenTriviaChildData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTrivia")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .finish()
    }
}

impl fmt::Display for GreenTriviaChildData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(self.text()) {
            Ok(text) => write!(f, "{}", text),
            Err(_) => write!(f, "{:?}", self.text()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivia_sizes() {
        assert_eq!(2, std::mem::size_of::<GreenTriviaChildHead>());
        assert_eq!(8, std::mem::size_of::<GreenTriviaChild>());
    }

    #[test]
    fn trivia_child_sizes() {
        assert_eq!(0, std::mem::size_of::<GreenTriviaHead>());
        assert_eq!(8, std::mem::size_of::<GreenTrivia>());
    }
}
