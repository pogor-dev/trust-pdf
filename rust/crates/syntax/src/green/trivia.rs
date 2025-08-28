use std::{fmt, io};

use crate::{GreenNode, SyntaxKind};

pub struct SyntaxTrivia<'a> {
    kind: SyntaxKind,
    full_width: usize,
    text: &'a [u8],
}

impl<'a> SyntaxTrivia<'a> {
    #[inline]
    pub fn new_with_text(kind: SyntaxKind, text: &'a [u8]) -> Self {
        let full_width = text.len();
        Self { kind, full_width, text }
    }
}

impl GreenNode for SyntaxTrivia<'_> {
    #[inline]
    fn kind(&self) -> SyntaxKind {
        self.kind
    }

    #[inline]
    fn to_string(&self) -> &[u8] {
        &self.text
    }

    #[inline]
    fn to_full_string(&self) -> &[u8] {
        &self.text
    }

    #[inline]
    fn full_width(&self) -> usize {
        self.full_width
    }

    #[inline]
    fn is_trivia(&self) -> bool {
        true
    }

    #[inline]
    fn leading_trivia<GreenNode>(&self) -> Option<&GreenNode> {
        None
    }

    #[inline]
    fn trailing_trivia<GreenNode>(&self) -> Option<&GreenNode> {
        None
    }

    #[inline]
    fn leading_trivia_width(&self) -> usize {
        0
    }

    #[inline]
    fn trailing_trivia_width(&self) -> usize {
        0
    }

    fn write_trivia_to<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self.to_full_string())
    }

    fn width(&self) -> usize {
        self.full_width() - self.leading_trivia_width() - self.trailing_trivia_width()
    }

    fn slot<T: GreenNode>(&self, _index: usize) -> Option<&T> {
        None
    }

    fn slot_count(&self) -> usize {
        0
    }

    fn is_token(&self) -> bool {
        false
    }

    fn is_list(&self) -> bool {
        self.kind() == SyntaxKind::List
    }

    fn has_leading_trivia(&self) -> bool {
        self.leading_trivia_width() != 0
    }

    fn has_trailing_trivia(&self) -> bool {
        self.trailing_trivia_width() != 0
    }
}

impl Clone for SyntaxTrivia<'_> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            full_width: self.full_width,
            text: self.text,
        }
    }
}

impl PartialEq for SyntaxTrivia<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.full_width == other.full_width && self.text == other.text
    }
}

impl Eq for SyntaxTrivia<'_> {}

unsafe impl Send for SyntaxTrivia<'_> {}
unsafe impl Sync for SyntaxTrivia<'_> {}

impl fmt::Debug for SyntaxTrivia<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxTrivia")
            .field("kind", &self.kind())
            .field("full_width", &self.full_width())
            .field("text", &String::from_utf8_lossy(self.to_full_string()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::whitespace(SyntaxKind::WhitespaceTrivia, b" ")]
    #[case::comment(SyntaxKind::CommentTrivia, b"% Comment 1")]
    #[case::end_of_line(SyntaxKind::EndOfLineTrivia, b"\r\n")]
    fn test_to_string(#[case] kind: SyntaxKind, #[case] text: &[u8]) {
        let token = SyntaxTrivia::new_with_text(kind, text);
        assert_eq!(token.to_string(), text);
        assert_eq!(token.to_full_string(), text);
    }

    #[rstest]
    #[case::whitespace(SyntaxKind::WhitespaceTrivia, b" ")]
    #[case::comment(SyntaxKind::CommentTrivia, b"% Comment 1")]
    #[case::end_of_line(SyntaxKind::EndOfLineTrivia, b"\r\n")]
    fn test_width(#[case] kind: SyntaxKind, #[case] text: &[u8]) {
        let token = SyntaxTrivia::new_with_text(kind, text);
        assert_eq!(token.width(), text.len());
        assert_eq!(token.full_width(), text.len());
    }

    #[rstest]
    fn test_is_trivia() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ");
        assert!(token.is_trivia());
    }

    #[rstest]
    fn test_is_not_token() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ");
        assert!(!token.is_token());
    }

    #[rstest]
    fn test_is_not_list() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ");
        assert!(!token.is_list());
    }

    #[rstest]
    fn test_no_nested_trivia() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ");
        assert!(!token.has_leading_trivia());
        assert!(!token.has_trailing_trivia());
        assert_eq!(token.leading_trivia_width(), 0);
        assert_eq!(token.trailing_trivia_width(), 0);
        assert_eq!(token.leading_trivia::<SyntaxTrivia>(), None);
        assert_eq!(token.trailing_trivia::<SyntaxTrivia>(), None);
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(2)]
    fn test_slot_with_any_index_expect_none(#[case] index: usize) {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ");
        assert_eq!(token.slot::<SyntaxTrivia>(index), None);
    }

    #[rstest]
    fn test_slot_count_expect_zero() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ");
        assert_eq!(token.slot_count(), 0);
    }

    #[rstest]
    fn test_clone() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ");
        let cloned = token.clone();
        assert_eq!(token, cloned);
    }

    #[rstest]
    #[case::same_kind_same_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::WhitespaceTrivia, b" ", b" ", true)]
    #[case::same_kind_different_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::WhitespaceTrivia, b" ", b"  ", false)]
    #[case::different_kind_same_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::CommentTrivia, b" ", b" ", false)]
    #[case::different_kind_different_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::CommentTrivia, b" ", b"  ", false)]
    fn test_eq(#[case] kind: SyntaxKind, #[case] expected_kind: SyntaxKind, #[case] text: &[u8], #[case] expected_text: &[u8], #[case] expected: bool) {
        let token = SyntaxTrivia::new_with_text(kind, text);
        let other = SyntaxTrivia::new_with_text(expected_kind, expected_text);
        assert_eq!(token == other, expected);
    }

    #[rstest]
    #[case::whitespace(SyntaxKind::WhitespaceTrivia, b" ", "SyntaxTrivia { kind: WhitespaceTrivia, full_width: 1, text: \" \" }")]
    #[case::comment(
        SyntaxKind::CommentTrivia,
        b"% Comment 1",
        "SyntaxTrivia { kind: CommentTrivia, full_width: 11, text: \"% Comment 1\" }"
    )]
    #[case::end_of_line(SyntaxKind::EndOfLineTrivia, b"\r\n", "SyntaxTrivia { kind: EndOfLineTrivia, full_width: 2, text: \"\\r\\n\" }")]
    fn test_debug(#[case] kind: SyntaxKind, #[case] text: &[u8], #[case] expected: &str) {
        let token = SyntaxTrivia::new_with_text(kind, text);
        assert_eq!(format!("{:?}", token), expected);
    }
}
