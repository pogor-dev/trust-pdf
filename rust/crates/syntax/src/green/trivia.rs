use std::{borrow::Cow, fmt, hash};

use crate::{
    GreenNode, SyntaxKind,
    green::{NodeOrToken, Trivia},
};

pub struct GreenTrivia<'a> {
    kind: SyntaxKind,
    /// Full width of the trivia in the document,
    /// Using `u64` to match the default Size type parameter of GreenNode trait
    width: u64,
    text: Cow<'a, [u8]>,
}

impl<'a> GreenTrivia<'a> {
    #[inline]
    pub fn new_with_text(kind: SyntaxKind, text: Cow<'a, [u8]>) -> Self {
        let full_width = text.len() as u64;
        Self { kind, width: full_width, text }
    }
}

impl<'a> GreenNode<'a, u64> for GreenTrivia<'a> {
    #[inline]
    fn kind(&self) -> SyntaxKind {
        self.kind
    }

    #[inline]
    fn to_string(&self) -> Cow<'a, [u8]> {
        self.text.clone()
    }

    #[inline]
    fn to_full_string(&self) -> Cow<'a, [u8]> {
        self.text.clone()
    }

    #[inline]
    fn is_trivia(&self) -> bool {
        true
    }

    #[inline]
    fn slot_count(&self) -> u8 {
        0
    }

    #[inline]
    fn full_width(&self) -> u64 {
        self.width
    }

    #[inline]
    fn leading_trivia_width(&self) -> u64 {
        0
    }

    #[inline]
    fn trailing_trivia_width(&self) -> u64 {
        0
    }

    #[inline]
    fn slot(&self, _index: u8) -> Option<NodeOrToken<'a>> {
        None
    }

    #[inline]
    fn leading_trivia(&self) -> Option<Trivia<'a>> {
        None
    }

    #[inline]
    fn trailing_trivia(&self) -> Option<Trivia<'a>> {
        None
    }
}

impl Clone for GreenTrivia<'_> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            width: self.width,
            text: self.text.clone(),
        }
    }
}

impl hash::Hash for GreenTrivia<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.width.hash(state);
        self.text.hash(state);
    }
}

impl PartialEq for GreenTrivia<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.width == other.width && self.text == other.text
    }
}

impl Eq for GreenTrivia<'_> {}

unsafe impl Send for GreenTrivia<'_> {}
unsafe impl Sync for GreenTrivia<'_> {}

impl fmt::Debug for GreenTrivia<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GreenTrivia")
            .field("kind", &self.kind())
            .field("full_width", &self.full_width())
            .field("text", &String::from_utf8_lossy(&self.text))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::whitespace(SyntaxKind::WhitespaceTrivia)]
    #[case::comment(SyntaxKind::CommentTrivia)]
    #[case::end_of_line(SyntaxKind::EndOfLineTrivia)]
    fn test_kind(#[case] kind: SyntaxKind) {
        let token = GreenTrivia::new_with_text(kind, b"some text".into());
        assert_eq!(token.kind(), kind);
    }

    #[rstest]
    #[case::whitespace(SyntaxKind::WhitespaceTrivia, b" ")]
    #[case::comment(SyntaxKind::CommentTrivia, b"% Comment 1")]
    #[case::end_of_line(SyntaxKind::EndOfLineTrivia, b"\r\n")]
    fn test_to_string(#[case] kind: SyntaxKind, #[case] text: &[u8]) {
        let token = GreenTrivia::new_with_text(kind, text.into());
        assert_eq!(token.to_string(), text);
        assert_eq!(token.to_full_string(), text);
    }

    #[rstest]
    #[case::whitespace(SyntaxKind::WhitespaceTrivia, b" ")]
    #[case::comment(SyntaxKind::CommentTrivia, b"% Comment 1")]
    #[case::end_of_line(SyntaxKind::EndOfLineTrivia, b"\r\n")]
    fn test_width(#[case] kind: SyntaxKind, #[case] text: &[u8]) {
        let token = GreenTrivia::new_with_text(kind, text.into());
        assert_eq!(token.width(), text.len() as u64);
        assert_eq!(token.full_width(), text.len() as u64);
    }

    #[rstest]
    fn test_is_trivia() {
        let token = GreenTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".into());
        assert!(token.is_trivia());
    }

    #[rstest]
    fn test_is_not_token() {
        let token = GreenTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".into());
        assert!(!token.is_token());
    }

    #[rstest]
    fn test_is_not_list() {
        let token = GreenTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".into());
        assert!(!token.is_list());
    }

    #[rstest]
    fn test_no_nested_trivia() {
        let token = GreenTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".into());
        assert!(!token.has_leading_trivia());
        assert!(!token.has_trailing_trivia());
        assert_eq!(token.leading_trivia_width(), 0);
        assert_eq!(token.trailing_trivia_width(), 0);
        assert_eq!(token.leading_trivia(), None);
        assert_eq!(token.trailing_trivia(), None);
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(2)]
    fn test_slot_with_any_index_expect_none(#[case] index: u8) {
        let token = GreenTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".into());
        assert_eq!(token.slot(index), None);
    }

    #[rstest]
    fn test_slot_count_expect_zero() {
        let token = GreenTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".into());
        assert_eq!(token.slot_count(), 0);
    }

    #[rstest]
    fn test_clone() {
        let token = GreenTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".into());
        let cloned = token.clone();
        assert_eq!(token, cloned);
    }

    #[rstest]
    #[case::same_kind_same_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::WhitespaceTrivia, b" ", b" ", true)]
    #[case::same_kind_different_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::WhitespaceTrivia, b" ", b"  ", false)]
    #[case::different_kind_same_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::CommentTrivia, b" ", b" ", false)]
    #[case::different_kind_different_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::CommentTrivia, b" ", b"  ", false)]
    fn test_eq(#[case] kind: SyntaxKind, #[case] expected_kind: SyntaxKind, #[case] text: &[u8], #[case] expected_text: &[u8], #[case] expected: bool) {
        let token = GreenTrivia::new_with_text(kind, text.into());
        let other = GreenTrivia::new_with_text(expected_kind, expected_text.into());
        assert_eq!(token == other, expected);
    }

    #[rstest]
    #[case::whitespace(SyntaxKind::WhitespaceTrivia, b" ", "GreenTrivia { kind: WhitespaceTrivia, full_width: 1, text: \" \" }")]
    #[case::comment(
        SyntaxKind::CommentTrivia,
        b"% Comment 1",
        "GreenTrivia { kind: CommentTrivia, full_width: 11, text: \"% Comment 1\" }"
    )]
    #[case::end_of_line(SyntaxKind::EndOfLineTrivia, b"\r\n", "GreenTrivia { kind: EndOfLineTrivia, full_width: 2, text: \"\\r\\n\" }")]
    fn test_debug(#[case] kind: SyntaxKind, #[case] text: &[u8], #[case] expected: &str) {
        let token = GreenTrivia::new_with_text(kind, text.into());
        assert_eq!(format!("{:?}", token), expected);
    }
}
