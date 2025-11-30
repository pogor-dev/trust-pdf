use std::{fmt, ptr::NonNull, slice};

use countme::Count;

use crate::{SyntaxKind, green::trivia::GreenTriviaListInTree};

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct GreenTokenHead {
    leading_trivia: GreenTriviaListInTree,  // 8 bytes
    trailing_trivia: GreenTriviaListInTree, // 8 bytes
    full_width: u32,                        // 4 bytes
    kind: SyntaxKind,                       // 2 bytes
    _c: Count<GreenToken>,                  // 0 bytes
}

impl GreenTokenHead {
    #[inline]
    pub(super) fn new(kind: SyntaxKind, full_width: u32, leading: GreenTriviaListInTree, trailing: GreenTriviaListInTree) -> Self {
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
    pub fn bytes(&self) -> Vec<u8> {
        self.write_to(false, false)
    }

    /// Returns the full bytes including leading and trailing trivia
    #[inline]
    pub fn full_bytes(&self) -> Vec<u8> {
        self.write_to(true, true)
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
    pub fn leading_trivia(&self) -> &GreenTriviaListInTree {
        &self.header().leading_trivia
    }

    #[inline]
    pub fn trailing_trivia(&self) -> &GreenTriviaListInTree {
        &self.header().trailing_trivia
    }

    /// Writes the token to a byte vector with conditional trivia inclusion
    ///
    /// # Parameters
    /// * `leading` - If true, include the leading trivia
    /// * `trailing` - If true, include the trailing trivia
    pub(crate) fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        let mut output = Vec::new();

        if leading {
            output.extend_from_slice(&self.leading_trivia().full_bytes());
        }

        // SAFETY: `data`'s invariant.
        let bytes = unsafe { slice::from_raw_parts(self.bytes_ptr_mut(), self.width() as usize) };
        output.extend_from_slice(bytes);

        if trailing {
            output.extend_from_slice(&self.trailing_trivia().full_bytes());
        }

        output
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
    pub(super) fn bytes_ptr_mut(&self) -> *mut u8 {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { (&raw mut (*self.data.as_ptr()).text).cast::<u8>() }
    }
}

impl PartialEq for GreenToken {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
            && self.bytes() == other.bytes()
            && self.leading_trivia() == other.leading_trivia()
            && self.trailing_trivia() == other.trailing_trivia()
    }
}

impl Eq for GreenToken {}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let full_bytes = self.full_bytes();
        let full_text_str = String::from_utf8_lossy(&full_bytes);
        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("full_text", &full_text_str)
            .field("full_width", &self.full_width())
            .finish()
    }
}

impl fmt::Display for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.full_bytes();
        for &byte in &bytes {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenToken {}
unsafe impl Sync for GreenToken {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenTokenHead>(), 24); // 22 bytes + 2 bytes padding
        assert_eq!(std::mem::align_of::<GreenTokenHead>(), 8); // 8 bytes alignment
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;
    use crate::green::arena::GreenTree;

    const INTEGER_KIND: SyntaxKind = SyntaxKind(1);
    const WHITESPACE_KIND: SyntaxKind = SyntaxKind(2);
    const COMMENT_KIND: SyntaxKind = SyntaxKind(3);

    #[test]
    fn test_kind() {
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia);
        assert_eq!(token.kind(), INTEGER_KIND);
    }

    #[test]
    fn test_bytes() {
        let mut arena = GreenTree::new();
        let cases = [
            (b"".to_vec(), b"123".to_vec(), b"".to_vec(), b"123".to_vec()),
            (b"  ".to_vec(), b"obj".to_vec(), b"".to_vec(), b"obj".to_vec()),
            (b"".to_vec(), b"null".to_vec(), b"\n".to_vec(), b"null".to_vec()),
            (b"\t".to_vec(), b"true".to_vec(), b" ".to_vec(), b"true".to_vec()),
            (b"% comment\n".to_vec(), b"42".to_vec(), b"\r\n".to_vec(), b"42".to_vec()),
        ];

        for (leading, text, trailing, expected) in cases {
            let leading_trivia = if leading.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let trailing_trivia = if trailing.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(COMMENT_KIND, trailing.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let token = arena.alloc_token(INTEGER_KIND, text.as_slice(), leading_trivia, trailing_trivia);
            assert_eq!(token.bytes().as_slice(), expected.as_slice());
        }
    }

    #[test]
    fn test_width() {
        let mut arena = GreenTree::new();
        let cases = [
            (b"".to_vec(), b"123".to_vec(), b"".to_vec(), 3),
            (b"  ".to_vec(), b"obj".to_vec(), b"".to_vec(), 3),
            (b"".to_vec(), b"null".to_vec(), b"\n".to_vec(), 4),
            (b"\t".to_vec(), b"true".to_vec(), b" ".to_vec(), 4),
            (b"% comment\n".to_vec(), b"42".to_vec(), b"\r\n".to_vec(), 2),
        ];

        for (leading, text, trailing, expected) in cases {
            let leading_trivia = if leading.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let trailing_trivia = if trailing.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(COMMENT_KIND, trailing.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let token = arena.alloc_token(INTEGER_KIND, text.as_slice(), leading_trivia, trailing_trivia);
            assert_eq!(token.width(), expected);
        }
    }

    #[test]
    fn test_full_width() {
        let mut arena = GreenTree::new();
        let cases = [
            (b"".to_vec(), b"123".to_vec(), b"".to_vec(), 3),
            (b"  ".to_vec(), b"obj".to_vec(), b"".to_vec(), 5),
            (b"".to_vec(), b"null".to_vec(), b"\n".to_vec(), 5),
            (b"\t".to_vec(), b"true".to_vec(), b" ".to_vec(), 6),
            (b"% comment\n".to_vec(), b"42".to_vec(), b"\r\n".to_vec(), 14),
        ];

        for (leading, text, trailing, expected) in cases {
            let leading_trivia = if leading.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let trailing_trivia = if trailing.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(COMMENT_KIND, trailing.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let token = arena.alloc_token(INTEGER_KIND, text.as_slice(), leading_trivia, trailing_trivia);
            assert_eq!(token.full_width(), expected);
        }
    }

    #[test]
    fn test_full_bytes() {
        let mut arena = GreenTree::new();
        let cases = [
            (b"".to_vec(), b"obj".to_vec(), b"".to_vec(), b"obj".to_vec()),
            (b"  ".to_vec(), b"endobj".to_vec(), b"".to_vec(), b"  endobj".to_vec()),
            (b"".to_vec(), b"stream".to_vec(), b"\n".to_vec(), b"stream\n".to_vec()),
            (b"\t".to_vec(), b"true".to_vec(), b" ".to_vec(), b"\ttrue ".to_vec()),
            (b"% comment\n".to_vec(), b"null".to_vec(), b"\r\n".to_vec(), b"% comment\nnull\r\n".to_vec()),
            (b" \t".to_vec(), b"/Name".to_vec(), b" \n".to_vec(), b" \t/Name \n".to_vec()),
        ];

        for (leading, text, trailing, expected) in cases {
            let leading_trivia = if leading.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let trailing_trivia = if trailing.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(COMMENT_KIND, trailing.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let token = arena.alloc_token(INTEGER_KIND, text.as_slice(), leading_trivia, trailing_trivia);
            assert_eq!(token.full_bytes().as_slice(), expected.as_slice());
        }
    }

    #[test]
    fn test_leading_trivia() {
        let mut arena = GreenTree::new();
        let cases = [
            (b"".to_vec(), 0),
            (b" ".to_vec(), 1),
            (b"\t".to_vec(), 1),
            (b"\n".to_vec(), 1),
            (b"  \t\n".to_vec(), 4),
        ];

        for (leading, expected_width) in cases {
            let leading_trivia = if leading.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let empty_trivia = arena.alloc_trivia_list(&[]);
            let token = arena.alloc_token(INTEGER_KIND, b"42", leading_trivia, empty_trivia);
            assert_eq!(token.leading_trivia().full_width(), expected_width);

            let trivia_pieces = token.leading_trivia().pieces();
            if !leading.is_empty() {
                assert_eq!(trivia_pieces.len(), 1);
                assert_eq!(trivia_pieces[0].bytes(), leading.as_slice());
            } else {
                assert_eq!(trivia_pieces.len(), 0);
            }
        }
    }

    #[test]
    fn test_trailing_trivia() {
        let mut arena = GreenTree::new();
        let cases = [
            (b"".to_vec(), 0),
            (b" ".to_vec(), 1),
            (b"\n".to_vec(), 1),
            (b"% comment\n".to_vec(), 10),
            (b"\r\n".to_vec(), 2),
        ];

        for (trailing, expected_width) in cases {
            let trailing_trivia = if trailing.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(COMMENT_KIND, trailing.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let empty_trivia = arena.alloc_trivia_list(&[]);
            let token = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, trailing_trivia);
            assert_eq!(token.trailing_trivia().full_width(), expected_width);

            let trivia_pieces = token.trailing_trivia().pieces();
            if !trailing.is_empty() {
                assert_eq!(trivia_pieces.len(), 1);
                assert_eq!(trivia_pieces[0].bytes(), trailing.as_slice());
            } else {
                assert_eq!(trivia_pieces.len(), 0);
            }
        }
    }

    #[test]
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

    #[test]
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

    #[test]
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
