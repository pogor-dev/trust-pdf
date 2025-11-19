use std::{fmt, ptr::NonNull, slice};

use countme::Count;

use crate::{GreenTriviaList, SyntaxKind};

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct GreenTokenHead {
    leading_trivia: GreenTriviaList,  // 8 bytes
    trailing_trivia: GreenTriviaList, // 8 bytes
    full_width: u32,                  // 4 bytes
    kind: SyntaxKind,                 // 2 bytes
    _c: Count<GreenToken>,            // 0 bytes
}

impl GreenTokenHead {
    #[inline]
    pub(super) fn new(kind: SyntaxKind, full_width: u32, leading: GreenTriviaList, trailing: GreenTriviaList) -> Self {
        Self {
            leading_trivia: leading,
            trailing_trivia: trailing,
            full_width,
            kind,
            _c: Count::new(),
        }
    }

    #[inline]
    pub(super) fn layout(text_len: u32) -> std::alloc::Layout {
        std::alloc::Layout::new::<GreenTokenHead>()
            .extend(std::alloc::Layout::array::<u8>(text_len as usize).expect("too big token"))
            .expect("too big token")
            .0
            .pad_to_align()
    }
}

/// This is used to store the token in the arena.
/// The actual text is stored inline after the head.
#[repr(C)]
pub(super) struct GreenTokenData {
    head: GreenTokenHead, // 24 bytes
    text: [u8; 0],        // 0 bytes, actual text is stored inline after this struct
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct GreenToken {
    /// INVARIANT: This points at a valid `GreenTokenData` followed by `text_len` bytes,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTokenData>,
}

impl GreenToken {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.header().kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        // SAFETY: `data`'s invariant.
        unsafe { slice::from_raw_parts(self.text_ptr_mut(), self.width() as usize) }
    }

    /// Returns the full text including leading and trailing trivia
    #[inline]
    pub fn full_text(&self) -> String {
        let mut result = String::new();
        let _ = self.write_full_text(&mut result);
        result
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.header().full_width - self.leading_trivia().full_width() - self.trailing_trivia().full_width()
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        self.header().full_width
    }

    #[inline]
    pub fn leading_trivia(&self) -> &GreenTriviaList {
        &self.header().leading_trivia
    }

    #[inline]
    pub fn trailing_trivia(&self) -> &GreenTriviaList {
        &self.header().trailing_trivia
    }

    /// Writes the full text including leading and trailing trivia to the writer
    fn write_full_text(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        // Write leading trivia
        for trivia in self.leading_trivia().pieces() {
            for &byte in trivia.text() {
                write!(writer, "{}", byte as char)?;
            }
        }

        // Write token text
        for &byte in self.text() {
            write!(writer, "{}", byte as char)?;
        }

        // Write trailing trivia
        for trivia in self.trailing_trivia().pieces() {
            for &byte in trivia.text() {
                write!(writer, "{}", byte as char)?;
            }
        }

        Ok(())
    }

    #[inline]
    fn header(&self) -> &GreenTokenHead {
        // SAFETY: The invariant on `data` ensures this is valid for reads.
        unsafe { &self.data.as_ref().head }
    }

    /// Does not require the pointer to be valid.
    #[inline]
    pub(super) fn header_ptr_mut(&self) -> *mut GreenTokenHead {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { &raw mut (*self.data.as_ptr()).head }
    }

    #[inline]
    pub(super) fn text_ptr_mut(&self) -> *mut u8 {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { (&raw mut (*self.data.as_ptr()).text).cast::<u8>() }
    }
}

impl PartialEq for GreenToken {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
            && self.text() == other.text()
            && self.leading_trivia() == other.leading_trivia()
            && self.trailing_trivia() == other.trailing_trivia()
    }
}

impl Eq for GreenToken {}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("full_text", &self.full_text())
            .field("full_width", &self.full_width())
            .finish()
    }
}

impl fmt::Display for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_full_text(f)
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenToken {}
unsafe impl Sync for GreenToken {}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenTokenHead>(), 24); // 22 bytes + 2 bytes padding
        assert_eq!(std::mem::align_of::<GreenTokenHead>(), 8); // 8 bytes alignment
    }
}

#[cfg(test)]
mod token_tests {
    use rstest::rstest;

    use crate::green::arena::GreenTree;

    use super::*;

    const INTEGER_KIND: SyntaxKind = SyntaxKind(1);
    const WHITESPACE_KIND: SyntaxKind = SyntaxKind(2);
    const COMMENT_KIND: SyntaxKind = SyntaxKind(3);

    #[rstest]
    fn test_kind() {
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia);
        assert_eq!(token.kind(), INTEGER_KIND);
    }

    #[rstest]
    #[case(b"", b"123", b"", b"123")]
    #[case::with_leading(b"  ", b"obj", b"", b"obj")]
    #[case::with_trailing(b"", b"null", b"\n", b"null")]
    #[case::with_both(b"\t", b"true", b" ", b"true")]
    #[case::with_comment(b"% comment\n", b"42", b"\r\n", b"42")]
    fn test_text(#[case] leading: &[u8], #[case] text: &[u8], #[case] trailing: &[u8], #[case] expected: &[u8]) {
        let mut arena = GreenTree::new();

        let leading_trivia = if leading.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading);
            arena.alloc_trivia_list(&[trivia])
        };

        let trailing_trivia = if trailing.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(COMMENT_KIND, trailing);
            arena.alloc_trivia_list(&[trivia])
        };

        let token = arena.alloc_token(INTEGER_KIND, text, leading_trivia, trailing_trivia);
        assert_eq!(token.text(), expected);
    }

    #[rstest]
    #[case(b"", b"123", b"", 3)]
    #[case::with_leading(b"  ", b"obj", b"", 3)]
    #[case::with_trailing(b"", b"null", b"\n", 4)]
    #[case::with_both(b"\t", b"true", b" ", 4)]
    #[case::with_comment(b"% comment\n", b"42", b"\r\n", 2)]
    fn test_width(#[case] leading: &[u8], #[case] text: &[u8], #[case] trailing: &[u8], #[case] expected: u32) {
        let mut arena = GreenTree::new();

        let leading_trivia = if leading.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading);
            arena.alloc_trivia_list(&[trivia])
        };

        let trailing_trivia = if trailing.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(COMMENT_KIND, trailing);
            arena.alloc_trivia_list(&[trivia])
        };

        let token = arena.alloc_token(INTEGER_KIND, text, leading_trivia, trailing_trivia);
        assert_eq!(token.width(), expected);
    }

    #[rstest]
    #[case(b"", b"123", b"", 3)]
    #[case::with_leading(b"  ", b"obj", b"", 5)]
    #[case::with_trailing(b"", b"null", b"\n", 5)]
    #[case::with_both(b"\t", b"true", b" ", 6)]
    #[case::with_comment(b"% comment\n", b"42", b"\r\n", 14)]
    fn test_full_width(#[case] leading: &[u8], #[case] text: &[u8], #[case] trailing: &[u8], #[case] expected: u32) {
        let mut arena = GreenTree::new();

        let leading_trivia = if leading.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading);
            arena.alloc_trivia_list(&[trivia])
        };

        let trailing_trivia = if trailing.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(COMMENT_KIND, trailing);
            arena.alloc_trivia_list(&[trivia])
        };

        let token = arena.alloc_token(INTEGER_KIND, text, leading_trivia, trailing_trivia);
        assert_eq!(token.full_width(), expected);
    }

    #[rstest]
    #[case(b"", b"obj", b"", "obj")]
    #[case::with_leading(b"  ", b"endobj", b"", "  endobj")]
    #[case::with_trailing(b"", b"stream", b"\n", "stream\n")]
    #[case::with_both(b"\t", b"true", b" ", "\ttrue ")]
    #[case::with_comment(b"% comment\n", b"null", b"\r\n", "% comment\nnull\r\n")]
    #[case::with_name(b" \t", b"/Name", b" \n", " \t/Name \n")]
    fn test_full_text(#[case] leading: &[u8], #[case] text: &[u8], #[case] trailing: &[u8], #[case] expected: &str) {
        let mut arena = GreenTree::new();

        let leading_trivia = if leading.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading);
            arena.alloc_trivia_list(&[trivia])
        };

        let trailing_trivia = if trailing.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(COMMENT_KIND, trailing);
            arena.alloc_trivia_list(&[trivia])
        };

        let token = arena.alloc_token(INTEGER_KIND, text, leading_trivia, trailing_trivia);
        assert_eq!(token.full_text(), expected);
    }

    #[rstest]
    #[case(b"", 0)]
    #[case::with_space(b" ", 1)]
    #[case::with_tab(b"\t", 1)]
    #[case::with_newline(b"\n", 1)]
    #[case::with_multiple(b"  \t\n", 4)]
    fn test_leading_trivia(#[case] leading: &[u8], #[case] expected_width: u32) {
        let mut arena = GreenTree::new();

        let leading_trivia = if leading.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading);
            arena.alloc_trivia_list(&[trivia])
        };

        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token = arena.alloc_token(INTEGER_KIND, b"42", leading_trivia, empty_trivia);

        assert_eq!(token.leading_trivia().full_width(), expected_width);

        // Verify the actual trivia text matches
        let trivia_pieces = token.leading_trivia().pieces();
        if !leading.is_empty() {
            assert_eq!(trivia_pieces.len(), 1);
            assert_eq!(trivia_pieces[0].text(), leading);
        } else {
            assert_eq!(trivia_pieces.len(), 0);
        }
    }

    #[rstest]
    #[case(b"", 0)]
    #[case::with_space(b" ", 1)]
    #[case::with_newline(b"\n", 1)]
    #[case::with_comment(b"% comment\n", 10)]
    #[case::with_multiple(b"\r\n", 2)]
    fn test_trailing_trivia(#[case] trailing: &[u8], #[case] expected_width: u32) {
        let mut arena = GreenTree::new();

        let trailing_trivia = if trailing.is_empty() {
            arena.alloc_trivia_list(&[])
        } else {
            let trivia = arena.alloc_trivia(COMMENT_KIND, trailing);
            arena.alloc_trivia_list(&[trivia])
        };

        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, trailing_trivia);

        assert_eq!(token.trailing_trivia().full_width(), expected_width);

        // Verify the actual trivia text matches
        let trivia_pieces = token.trailing_trivia().pieces();
        if !trailing.is_empty() {
            assert_eq!(trivia_pieces.len(), 1);
            assert_eq!(trivia_pieces[0].text(), trailing);
        } else {
            assert_eq!(trivia_pieces.len(), 0);
        }
    }

    #[rstest]
    fn test_eq() {
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);

        let token1 = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia);
        let token2 = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia);
        assert_eq!(token1, token2);

        // Different kind
        let token3 = arena.alloc_token(WHITESPACE_KIND, b"42", empty_trivia, empty_trivia);
        assert_ne!(token1, token3);

        // Different text
        let token4 = arena.alloc_token(INTEGER_KIND, b"123", empty_trivia, empty_trivia);
        assert_ne!(token1, token4);

        // Different leading trivia
        let leading = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let leading_list = arena.alloc_trivia_list(&[leading]);
        let token5 = arena.alloc_token(INTEGER_KIND, b"42", leading_list, empty_trivia);
        assert_ne!(token1, token5);

        // Different trailing trivia
        let trailing = arena.alloc_trivia(COMMENT_KIND, b"\n");
        let trailing_list = arena.alloc_trivia_list(&[trailing]);
        let token6 = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, trailing_list);
        assert_ne!(token1, token6);
    }

    #[rstest]
    fn test_display() {
        let mut arena = GreenTree::new();

        // Token without trivia
        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia);
        assert_eq!(format!("{}", token), "42");

        // Token with leading trivia
        let leading = arena.alloc_trivia(WHITESPACE_KIND, b"  ");
        let leading_list = arena.alloc_trivia_list(&[leading]);
        let token = arena.alloc_token(INTEGER_KIND, b"obj", leading_list, empty_trivia);
        assert_eq!(format!("{}", token), "  obj");

        // Token with trailing trivia
        let trailing = arena.alloc_trivia(COMMENT_KIND, b"\n");
        let trailing_list = arena.alloc_trivia_list(&[trailing]);
        let token = arena.alloc_token(INTEGER_KIND, b"null", empty_trivia, trailing_list);
        assert_eq!(format!("{}", token), "null\n");

        // Token with both trivia
        let token = arena.alloc_token(INTEGER_KIND, b"true", leading_list, trailing_list);
        assert_eq!(format!("{}", token), "  true\n");
    }

    #[rstest]
    fn test_debug() {
        let mut arena = GreenTree::new();

        // Token with leading and trailing trivia
        let leading = arena.alloc_trivia(WHITESPACE_KIND, b"  ");
        let leading_list = arena.alloc_trivia_list(&[leading]);
        let trailing = arena.alloc_trivia(COMMENT_KIND, b"\n");
        let trailing_list = arena.alloc_trivia_list(&[trailing]);
        let token = arena.alloc_token(INTEGER_KIND, b"42", leading_list, trailing_list);

        let debug_output = format!("{:?}", token);
        assert_eq!(debug_output, "GreenToken { kind: SyntaxKind(1), full_text: \"  42\\n\", full_width: 5 }");
    }
}
