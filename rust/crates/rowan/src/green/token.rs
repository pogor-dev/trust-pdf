use std::{fmt, ptr::NonNull, slice};

use countme::Count;
use triomphe::Arc;

use crate::{
    DiagnosticInfo, SyntaxKind,
    green::{arena::GreenTree, trivia::GreenTriviaListInTree},
};

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct GreenTokenHead {
    leading_trivia: GreenTriviaListInTree,  // 8 bytes
    trailing_trivia: GreenTriviaListInTree, // 8 bytes
    full_width: u32,                        // 4 bytes
    kind: SyntaxKind,                       // 2 bytes
    _c: Count<GreenTokenInTree>,            // 0 bytes
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
pub struct GreenTokenInTree {
    /// INVARIANT: This points at a valid `GreenTokenData` followed by `text_len` bytes,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenTokenData>,
}

impl GreenTokenInTree {
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

    #[inline]
    pub fn to_green_token(self, arena: Arc<GreenTree>) -> GreenToken {
        GreenToken { token: self, arena }
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

impl PartialEq for GreenTokenInTree {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
            && self.bytes() == other.bytes()
            && self.leading_trivia() == other.leading_trivia()
            && self.trailing_trivia() == other.trailing_trivia()
    }
}

impl Eq for GreenTokenInTree {}

impl fmt::Debug for GreenTokenInTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.bytes();
        let full_bytes = self.full_bytes();
        let text_str = String::from_utf8_lossy(&bytes);
        let full_text_str = String::from_utf8_lossy(&full_bytes);
        let leading_trivia: Vec<_> = self
            .leading_trivia()
            .pieces()
            .iter()
            .map(|t| (t.kind(), String::from_utf8_lossy(t.bytes()).into_owned()))
            .collect();

        let trailing_trivia: Vec<_> = self
            .trailing_trivia()
            .pieces()
            .iter()
            .map(|t| (t.kind(), String::from_utf8_lossy(t.bytes()).into_owned()))
            .collect();

        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .field("full_text", &full_text_str)
            .field("full_width", &self.full_width())
            .field("leading_trivia", &leading_trivia)
            .field("trailing_trivia", &trailing_trivia)
            .finish()
    }
}

impl fmt::Display for GreenTokenInTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.full_bytes();
        for &byte in &bytes {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenTokenInTree {}
unsafe impl Sync for GreenTokenInTree {}

#[derive(Clone)]
pub struct GreenToken {
    pub(super) token: GreenTokenInTree,
    pub(super) arena: Arc<GreenTree>,
}

impl GreenToken {
    /// Kind of this Token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.token.kind()
    }

    /// The bytes of this Token.
    #[inline]
    pub fn bytes(&self) -> Vec<u8> {
        self.token.bytes()
    }

    #[inline]
    pub fn full_bytes(&self) -> Vec<u8> {
        self.token.full_bytes()
    }

    /// The width of this Token.
    #[inline]
    pub fn width(&self) -> u32 {
        self.token.width()
    }

    /// The full width of this Token.
    #[inline]
    pub fn full_width(&self) -> u32 {
        self.token.full_width()
    }

    /// The leading trivia of this Token.
    #[inline]
    pub fn leading_trivia(&self) -> &GreenTriviaListInTree {
        &self.token.leading_trivia()
    }

    /// The trailing trivia of this Token.
    #[inline]
    pub fn trailing_trivia(&self) -> &GreenTriviaListInTree {
        &self.token.trailing_trivia()
    }

    #[inline]
    /// Returns all diagnostics recorded for this token via the shared arena.
    pub fn diagnostics(&self) -> &[DiagnosticInfo] {
        self.arena.get_diagnostics(&self.token.into())
    }

    #[inline]
    pub(crate) fn into_raw_parts(self) -> (GreenTokenInTree, Arc<GreenTree>) {
        (self.token, self.arena)
    }
}

impl PartialEq for GreenToken {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

impl Eq for GreenToken {}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.token, f)
    }
}

impl fmt::Display for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.token, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenTokenHead>(), 24); // 22 bytes + 2 bytes padding
        assert_eq!(std::mem::align_of::<GreenTokenHead>(), 8); // 8 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTokenData>(), 24); // 24 bytes
        assert_eq!(std::mem::align_of::<GreenTokenData>(), 8); // 8 bytes alignment

        assert_eq!(std::mem::size_of::<GreenTokenInTree>(), 8); // 8 bytes
        assert_eq!(std::mem::align_of::<GreenTokenInTree>(), 8); // 8 bytes alignment

        assert_eq!(std::mem::size_of::<GreenToken>(), 16); // 16 bytes
        assert_eq!(std::mem::align_of::<GreenToken>(), 8); // 8 bytes alignment
    }
}

#[cfg(test)]
mod token_tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::diagnostics::DiagnosticSeverity::{Error, Info, Warning};
    use crate::green::arena::GreenTree;

    const INTEGER_KIND: SyntaxKind = SyntaxKind(1);
    const WHITESPACE_KIND: SyntaxKind = SyntaxKind(2);
    const COMMENT_KIND: SyntaxKind = SyntaxKind(3);

    #[test]
    fn test_kind() {
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token = arena
            .alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia)
            .to_green_token(arena.shareable());

        assert_eq!(token.kind(), INTEGER_KIND);
    }

    #[test]
    fn test_bytes() {
        let cases = [
            (b"".to_vec(), b"123".to_vec(), b"".to_vec(), b"123".to_vec()),
            (b"  ".to_vec(), b"obj".to_vec(), b"".to_vec(), b"obj".to_vec()),
            (b"".to_vec(), b"null".to_vec(), b"\n".to_vec(), b"null".to_vec()),
            (b"\t".to_vec(), b"true".to_vec(), b" ".to_vec(), b"true".to_vec()),
            (b"% comment\n".to_vec(), b"42".to_vec(), b"\r\n".to_vec(), b"42".to_vec()),
        ];

        for (leading, text, trailing, expected) in cases {
            let mut arena = GreenTree::new();

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

            let token = arena
                .alloc_token(INTEGER_KIND, text.as_slice(), leading_trivia, trailing_trivia)
                .to_green_token(arena.shareable());
            assert_eq!(token.bytes().as_slice(), expected.as_slice());
        }
    }

    #[test]
    fn test_width() {
        let cases = [
            (b"".to_vec(), b"123".to_vec(), b"".to_vec(), 3),
            (b"  ".to_vec(), b"obj".to_vec(), b"".to_vec(), 3),
            (b"".to_vec(), b"null".to_vec(), b"\n".to_vec(), 4),
            (b"\t".to_vec(), b"true".to_vec(), b" ".to_vec(), 4),
            (b"% comment\n".to_vec(), b"42".to_vec(), b"\r\n".to_vec(), 2),
        ];

        for (leading, text, trailing, expected) in cases {
            let mut arena = GreenTree::new();

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

            let token = arena
                .alloc_token(INTEGER_KIND, text.as_slice(), leading_trivia, trailing_trivia)
                .to_green_token(arena.shareable());
            assert_eq!(token.width(), expected);
        }
    }

    #[test]
    fn test_full_width() {
        let cases = [
            (b"".to_vec(), b"123".to_vec(), b"".to_vec(), 3),
            (b"  ".to_vec(), b"obj".to_vec(), b"".to_vec(), 5),
            (b"".to_vec(), b"null".to_vec(), b"\n".to_vec(), 5),
            (b"\t".to_vec(), b"true".to_vec(), b" ".to_vec(), 6),
            (b"% comment\n".to_vec(), b"42".to_vec(), b"\r\n".to_vec(), 14),
        ];

        for (leading, text, trailing, expected) in cases {
            let mut arena = GreenTree::new();

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

            let token = arena
                .alloc_token(INTEGER_KIND, text.as_slice(), leading_trivia, trailing_trivia)
                .to_green_token(arena.shareable());
            assert_eq!(token.full_width(), expected);
        }
    }

    #[test]
    fn test_full_bytes() {
        let cases = [
            (b"".to_vec(), b"obj".to_vec(), b"".to_vec(), b"obj".to_vec()),
            (b"  ".to_vec(), b"endobj".to_vec(), b"".to_vec(), b"  endobj".to_vec()),
            (b"".to_vec(), b"stream".to_vec(), b"\n".to_vec(), b"stream\n".to_vec()),
            (b"\t".to_vec(), b"true".to_vec(), b" ".to_vec(), b"\ttrue ".to_vec()),
            (b"% comment\n".to_vec(), b"null".to_vec(), b"\r\n".to_vec(), b"% comment\nnull\r\n".to_vec()),
            (b" \t".to_vec(), b"/Name".to_vec(), b" \n".to_vec(), b" \t/Name \n".to_vec()),
        ];

        for (leading, text, trailing, expected) in cases {
            let mut arena = GreenTree::new();

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

            let token = arena
                .alloc_token(INTEGER_KIND, text.as_slice(), leading_trivia, trailing_trivia)
                .to_green_token(arena.shareable());
            assert_eq!(token.full_bytes().as_slice(), expected.as_slice());
        }
    }

    #[test]
    fn test_leading_trivia() {
        let cases = [
            (b"".to_vec(), 0),
            (b" ".to_vec(), 1),
            (b"\t".to_vec(), 1),
            (b"\n".to_vec(), 1),
            (b"  \t\n".to_vec(), 4),
        ];

        for (leading, expected_width) in cases {
            let mut arena = GreenTree::new();

            let leading_trivia = if leading.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(WHITESPACE_KIND, leading.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let empty_trivia = arena.alloc_trivia_list(&[]);
            let token = arena
                .alloc_token(INTEGER_KIND, b"42", leading_trivia, empty_trivia)
                .to_green_token(arena.shareable());
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
        let cases = [
            (b"".to_vec(), 0),
            (b" ".to_vec(), 1),
            (b"\n".to_vec(), 1),
            (b"% comment\n".to_vec(), 10),
            (b"\r\n".to_vec(), 2),
        ];

        for (trailing, expected_width) in cases {
            let mut arena = GreenTree::new();

            let trailing_trivia = if trailing.is_empty() {
                arena.alloc_trivia_list(&[])
            } else {
                let trivia = arena.alloc_trivia(COMMENT_KIND, trailing.as_slice());
                arena.alloc_trivia_list(&[trivia])
            };

            let empty_trivia = arena.alloc_trivia_list(&[]);
            let token = arena
                .alloc_token(INTEGER_KIND, b"42", empty_trivia, trailing_trivia)
                .to_green_token(arena.shareable());
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
        let shareable = arena.shareable();
        let token1 = token1.to_green_token(shareable.clone());
        let token2 = token2.to_green_token(shareable.clone());
        assert_eq!(token1, token2);

        // Different kind
        let mut arena = GreenTree::new();
        let token3 = arena.alloc_token(WHITESPACE_KIND, b"42", empty_trivia, empty_trivia);
        let shareable = arena.shareable();
        let token3 = token3.to_green_token(shareable.clone());
        assert_ne!(token1, token3);

        // Different text
        let mut arena = GreenTree::new();
        let token4 = arena
            .alloc_token(INTEGER_KIND, b"123", empty_trivia, empty_trivia)
            .to_green_token(arena.shareable());

        assert_ne!(token1, token4);

        // Different leading trivia
        let mut arena = GreenTree::new();
        let leading = arena.alloc_trivia(WHITESPACE_KIND, b" ");
        let leading_list = arena.alloc_trivia_list(&[leading]);
        let token5 = arena
            .alloc_token(INTEGER_KIND, b"42", leading_list, empty_trivia)
            .to_green_token(arena.shareable());

        assert_ne!(token1, token5);

        // Different trailing trivia
        let mut arena = GreenTree::new();
        let trailing = arena.alloc_trivia(COMMENT_KIND, b"\n");
        let trailing_list = arena.alloc_trivia_list(&[trailing]);
        let token6 = arena
            .alloc_token(INTEGER_KIND, b"42", empty_trivia, trailing_list)
            .to_green_token(arena.shareable());

        assert_ne!(token1, token6);
    }

    #[test]
    fn test_display() {
        // Token without trivia
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia);
        let shareable = arena.shareable();
        let token = token.to_green_token(shareable);
        assert_eq!(format!("{}", token), "42");

        // Token with leading trivia
        let mut arena = GreenTree::new();
        let leading = arena.alloc_trivia(WHITESPACE_KIND, b"  ");
        let leading_list = arena.alloc_trivia_list(&[leading]);
        let token = arena.alloc_token(INTEGER_KIND, b"obj", leading_list, empty_trivia);
        let token = token.to_green_token(arena.shareable());
        assert_eq!(format!("{}", token), "  obj");

        // Token with trailing trivia
        let mut arena = GreenTree::new();
        let trailing = arena.alloc_trivia(COMMENT_KIND, b"\n");
        let trailing_list = arena.alloc_trivia_list(&[trailing]);
        let token = arena
            .alloc_token(INTEGER_KIND, b"null", empty_trivia, trailing_list)
            .to_green_token(arena.shareable());

        assert_eq!(format!("{}", token), "null\n");

        // Token with both trivia
        let mut arena = GreenTree::new();
        let token = arena
            .alloc_token(INTEGER_KIND, b"true", leading_list, trailing_list)
            .to_green_token(arena.shareable());

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
        let token = arena
            .alloc_token(INTEGER_KIND, b"42", leading_list, trailing_list)
            .to_green_token(arena.shareable());

        let debug_output = format!("{:?}", token);
        assert_eq!(
            debug_output,
            "GreenToken { kind: SyntaxKind(1), text: \"42\", width: 2, full_text: \"  42\\n\", full_width: 5, leading_trivia: [(SyntaxKind(2), \"  \")], trailing_trivia: [(SyntaxKind(3), \"\\n\")] }"
        );
    }

    #[test]
    fn test_into_raw_parts() {
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);

        let token = arena
            .alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia)
            .to_green_token(arena.shareable());

        let (raw_token, raw_arena) = token.clone().into_raw_parts();

        assert_eq!(raw_token, token.token);
        assert_eq!(Arc::as_ptr(&raw_arena), Arc::as_ptr(&token.arena));
    }

    #[test]
    fn test_diagnostics_when_no_diagnostics_expect_empty() {
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token = arena
            .alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia)
            .to_green_token(arena.shareable());

        assert_eq!(token.diagnostics().len(), 0);
    }

    #[test]
    fn test_diagnostics_when_single_diagnostic_expect_returned() {
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token_in_tree = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia);

        let diagnostic = DiagnosticInfo::new(1, "test error", Error);
        arena.alloc_diagnostic(&token_in_tree.into(), diagnostic.clone());

        let token = token_in_tree.to_green_token(arena.shareable());
        let diagnostics = token.diagnostics();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0], diagnostic);
    }

    #[test]
    fn test_diagnostics_when_multiple_diagnostics_expect_all_returned() {
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token_in_tree = arena.alloc_token(INTEGER_KIND, b"42", empty_trivia, empty_trivia);

        let diag1 = DiagnosticInfo::new(1, "error 1", Error);
        let diag2 = DiagnosticInfo::new(2, "warning 1", Warning);
        let diag3 = DiagnosticInfo::new(3, "info 1", Info);

        arena.alloc_diagnostic(&token_in_tree.into(), diag1.clone());
        arena.alloc_diagnostic(&token_in_tree.into(), diag2.clone());
        arena.alloc_diagnostic(&token_in_tree.into(), diag3.clone());

        let token = token_in_tree.to_green_token(arena.shareable());
        let diagnostics = token.diagnostics();

        assert_eq!(diagnostics.len(), 3);
        assert_eq!(diagnostics[0], diag1);
        assert_eq!(diagnostics[1], diag2);
        assert_eq!(diagnostics[2], diag3);
    }
}
