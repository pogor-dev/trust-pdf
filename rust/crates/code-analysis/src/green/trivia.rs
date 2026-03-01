//! Green trivia representation with inline PDF trivia bytes.
//!
//! This variant stores per-instance trivia text bytes inline in the green node
//! tail. Trivia text is read from the inline byte slice provided by callers.

use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    arc::{Arc, HeaderSlice, ThinArc},
    green::flags::GreenFlags,
};
use countme::Count;

use crate::SyntaxKind;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTriviaHead {
    kind: SyntaxKind,       // 2 bytes
    flags: GreenFlags,      // 1 byte
    _c: Count<GreenTrivia>, // 0 bytes
}

/// Borrowed trivia view with inline trivia text.
#[repr(transparent)]
pub(crate) struct GreenTriviaData {
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
    pub fn width(&self) -> u8 {
        self.data.slice().len() as u8
    }

    /// Returns the flags of this trivia.
    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }
}

impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
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
        let text = self.text();
        let text_str = String::from_utf8_lossy(text);

        f.debug_struct("GreenTrivia")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .finish()
    }
}

/// Leaf node in the immutable tree.
///
/// Represents trivia with caller-provided text bytes stored inline.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, u8>,
}

impl GreenTrivia {
    /// Creates new trivia.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        let flags = GreenFlags::IS_NOT_MISSING; // Trivia created via `new` is always not-missing
        let head = GreenTriviaHead { kind, flags, _c: Count::new() };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenTrivia { ptr }
    }
}

impl_green_boilerplate!(GreenTriviaHead, GreenTriviaData, GreenTrivia, u8);

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_trivia_head_memory_layout() {
        // GreenTriviaHead: kind (2 bytes) + flags (1 byte) + _c (0 bytes)
        // Expected: 2 + 1 + 1 padding for alignment = 4 bytes
        assert_eq!(std::mem::size_of::<GreenTriviaHead>(), 4);
        assert_eq!(std::mem::align_of::<GreenTriviaHead>(), 2);
    }

    #[test]
    fn test_green_trivia_data_memory_layout() {
        // GreenTriviaData on 64-bit targets:
        // header (4 bytes) + padding (4 bytes) + length (8 bytes) = 16 bytes
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTriviaData>(), 16);
            assert_eq!(std::mem::align_of::<GreenTriviaData>(), 8);
        }

        // GreenTriviaData on 32-bit targets:
        // header (4 bytes) + length (4 bytes) = 8 bytes
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTriviaData>(), 8);
            assert_eq!(std::mem::align_of::<GreenTriviaData>(), 4);
        }
    }

    #[test]
    fn test_green_trivia_memory_layout() {
        // GreenTrivia wraps a ThinArc pointer.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTrivia>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTrivia>(), 4);
        }
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
    fn test_new_when_created_expect_is_not_missing_flag_set() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        assert!(trivia.flags().contains(GreenFlags::IS_NOT_MISSING));
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
        let expected = "GreenTrivia { kind: WhitespaceTrivia, text: \" \", width: 1 }";
        assert_eq!(debug_str, expected);
    }

    #[test]
    fn test_empty_text() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"");
        assert_eq!(trivia.text(), b"");
        assert_eq!(trivia.width(), 0);
    }

    #[test]
    fn test_multiline_comment_text() {
        let text = b"% line1\n% line2\n% line3";
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, text);
        assert_eq!(trivia.text(), text);
        assert_eq!(trivia.width(), text.len() as u8);
    }

    #[test]
    fn test_unicode_comment_text() {
        let text = b"% \xE4\xBD\xA0\xE5\xA5\xBD\xE4\xB8\x96\xE7\x95\x8C";
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, text);
        assert_eq!(trivia.text(), text);
        assert_eq!(trivia.width(), text.len() as u8);
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
