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
    pub fn width(&self) -> u32 {
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
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text_str = String::from_utf8_lossy(self.text());
        f.debug_struct("GreenTrivia").field("kind", &self.kind()).field("text", &text_str).finish()
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
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        assert!(text.len() <= u32::MAX as usize, "trivia text length exceeds u32::MAX");
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
            let repr: &Repr = &*self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTriviaData>(repr)
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_green_trivia_head_memory_layout() {
        // GreenTriviaHead: kind (2 bytes) + _c (0 bytes)
        assert_eq!(std::mem::size_of::<GreenTriviaHead>(), 2);
        assert_eq!(std::mem::align_of::<GreenTriviaHead>(), 2);
    }

    #[test]
    fn test_green_trivia_data_memory_layout() {
        // GreenTriviaData is transparent wrapper around HeaderSlice<GreenTriviaHead, [u8; 0]>
        // HeaderSlice with repr(C): header(2 bytes) + padding(6 bytes) + length(8 bytes) + slice(0 bytes) = 16 bytes
        assert_eq!(std::mem::size_of::<GreenTriviaData>(), 16);
        assert_eq!(std::mem::align_of::<GreenTriviaData>(), std::mem::align_of::<usize>());
    }

    #[test]
    fn test_green_trivia_memory_layout() {
        // GreenTrivia wraps ThinArc pointer (8 bytes on 64-bit)
        assert_eq!(std::mem::size_of::<GreenTrivia>(), std::mem::size_of::<usize>());
        assert_eq!(std::mem::align_of::<GreenTrivia>(), std::mem::align_of::<usize>());
    }
}

#[cfg(test)]
mod green_trivia_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_trivia() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert_eq!(trivia.kind(), SyntaxKind::WhitespaceTrivia);
        assert_eq!(trivia.text(), b" ");
    }

    #[test]
    fn test_kind() {
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, b"% comment");
        assert_eq!(trivia.kind(), SyntaxKind::CommentTrivia);
    }

    #[test]
    fn test_text() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"   ");
        assert_eq!(trivia.text(), b"   ");
    }

    #[test]
    fn test_width() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n\t");
        assert_eq!(trivia.width(), 2);
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert_eq!(trivia1, trivia2);
    }

    #[test]
    fn test_eq_when_different_text_expect_not_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n");
        assert_ne!(trivia1, trivia2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::CommentTrivia, b" ");
        assert_ne!(trivia1, trivia2);
    }

    #[test]
    fn test_clone() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" \n\t");
        let trivia2 = trivia1.clone();
        assert_eq!(trivia1, trivia2);
        assert_eq!(trivia2.kind(), SyntaxKind::WhitespaceTrivia);
        assert_eq!(trivia2.text(), b" \n\t");
    }

    #[test]
    fn test_display() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" \n\t");
        assert_eq!(trivia.to_string(), " \n\t");
    }

    #[test]
    fn test_debug() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let debug_str = format!("{:?}", trivia);
        assert_eq!(debug_str, "GreenTrivia { kind: WhitespaceTrivia, text: \" \" }");
    }

    #[test]
    fn test_empty_text() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"");
        assert_eq!(trivia.text(), b"");
        assert_eq!(trivia.width(), 0);
    }

    #[test]
    fn test_multiline_text() {
        let text = b"line1\nline2\nline3";
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, text);
        assert_eq!(trivia.text(), text);
        assert_eq!(trivia.width(), text.len() as u32);
    }

    #[test]
    fn test_unicode_text() {
        let text = b"% \xE4\xBD\xA0\xE5\xA5\xBD\xE4\xB8\x96\xE7\x95\x8C";
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, text);
        assert_eq!(trivia.text(), text);
        assert_eq!(trivia.width(), text.len() as u32);
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let ptr = GreenTrivia::into_raw(trivia.clone());
        let reconstructed = unsafe { GreenTrivia::from_raw(ptr) };
        assert_eq!(trivia, reconstructed);
    }

    #[test]
    fn test_borrow() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let borrowed: &GreenTriviaData = trivia.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::WhitespaceTrivia);
        assert_eq!(borrowed.text(), b" ");
    }
}

#[cfg(test)]
mod green_trivia_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let data: &GreenTriviaData = &*trivia;
        let owned = data.to_owned();
        assert_eq!(trivia, owned);
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let data1: &GreenTriviaData = &*trivia1;
        let data2: &GreenTriviaData = &*trivia2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_text_expect_not_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n");
        let data1: &GreenTriviaData = &*trivia1;
        let data2: &GreenTriviaData = &*trivia2;
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_kind_expect_not_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::CommentTrivia, b" ");
        let data1: &GreenTriviaData = &*trivia1;
        let data2: &GreenTriviaData = &*trivia2;
        assert_ne!(data1, data2);
    }
}
