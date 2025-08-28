use std::{fmt, io};

use crate::{GreenNode, SyntaxKind, TokenText};

pub struct SyntaxTrivia {
    kind: SyntaxKind,
    full_width: usize,
    text: TokenText,
}

impl SyntaxTrivia {
    #[inline]
    pub fn new_with_text(kind: SyntaxKind, text: Vec<u8>) -> Self {
        let full_width = text.len();
        Self {
            kind,
            full_width,
            text: TokenText::Owned(text),
        }
    }
}

impl GreenNode for SyntaxTrivia {
    type GreenNodeType = dyn GreenNode;

    #[inline]
    fn kind(&self) -> SyntaxKind {
        self.kind
    }

    #[inline]
    fn to_string(&self) -> Vec<u8> {
        self.text.to_vec()
    }

    #[inline]
    fn to_full_string(&self) -> Vec<u8> {
        self.text.to_vec()
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
    fn leading_trivia_width(&self) -> usize {
        0
    }

    #[inline]
    fn trailing_trivia_width(&self) -> usize {
        0
    }

    fn write_trivia_to<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_full_string())
    }

    fn width(&self) -> usize {
        self.full_width() - self.leading_trivia_width() - self.trailing_trivia_width()
    }

    fn slot(&self, _index: usize) -> Option<&Self::GreenNodeType> {
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

    fn leading_trivia(&self) -> Option<&Self::GreenNodeType> {
        None
    }

    fn trailing_trivia(&self) -> Option<&Self::GreenNodeType> {
        None
    }

    fn write_token_to<W: io::Write>(&self, _writer: &mut W, _leading: bool, _trailing: bool) -> io::Result<()> {
        Ok(())
    }

    fn write_to<W: io::Write>(&self, writer: &mut W, leading: bool, trailing: bool) -> io::Result<()>
    where
        Self: Sized,
        Self: GreenNode<GreenNodeType = Self>,
    {
        // Use explicit stack to avoid stack overflow on deeply nested structures
        let mut stack: Vec<(&Self, bool, bool)> = Vec::new();
        stack.push((self, leading, trailing));

        while let Some((current_node, current_leading, current_trailing)) = stack.pop() {
            if current_node.is_token() {
                current_node.write_token_to(writer, current_leading, current_trailing)?;
                continue;
            }

            if current_node.is_trivia() {
                current_node.write_trivia_to(writer)?;
                continue;
            }

            let first_index = Self::get_first_non_null_child_index(current_node);
            let last_index = Self::get_last_non_null_child_index(current_node);

            // Push children in reverse order (since stack is LIFO)
            for i in (first_index..=last_index).rev() {
                if let Some(child) = current_node.slot(i) {
                    let first = i == first_index;
                    let last = i == last_index;

                    let child_leading = current_leading || !first;
                    let child_trailing = current_trailing || !last;

                    stack.push((child, child_leading, child_trailing));
                }
            }
        }

        Ok(())
    }

    fn get_first_non_null_child_index(node: &Self) -> usize
    where
        Self: Sized,
    {
        for i in 0..node.slot_count() {
            if node.slot(i).is_some() {
                return i;
            }
        }
        0 // If no children found
    }

    fn get_last_non_null_child_index(node: &Self) -> usize
    where
        Self: Sized,
    {
        for i in (0..node.slot_count()).rev() {
            if node.slot(i).is_some() {
                return i;
            }
        }
        0 // If no children found
    }

    fn get_first_terminal(&self) -> Option<&Self::GreenNodeType>
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        let mut node: Option<&Self::GreenNodeType> = Some(self);

        loop {
            let current = node?;

            // Find first non-null child
            let mut first_child = None;
            let slot_count = current.slot_count();

            for i in 0..slot_count {
                if let Some(child) = current.slot(i) {
                    first_child = Some(child);
                    break;
                }
            }

            node = first_child;

            // Optimization: if no children or reached terminal, stop
            if node.map(|n| n.slot_count()).unwrap_or(0) == 0 {
                break;
            }
        }

        node
    }

    fn get_last_terminal(&self) -> Option<&Self::GreenNodeType>
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        let mut node: Option<&Self::GreenNodeType> = Some(self);

        loop {
            let current = node?;

            // Find last non-null child
            let mut last_child = None;
            let slot_count = current.slot_count();

            for i in (0..slot_count).rev() {
                if let Some(child) = current.slot(i) {
                    last_child = Some(child);
                    break;
                }
            }

            node = last_child;

            // Optimization: if no children or reached terminal, stop
            if node.map(|n| n.slot_count()).unwrap_or(0) == 0 {
                break;
            }
        }

        node
    }
}

impl Clone for SyntaxTrivia {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            full_width: self.full_width,
            text: self.text.clone(),
        }
    }
}

impl PartialEq for SyntaxTrivia {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.full_width == other.full_width && self.text == other.text
    }
}

impl Eq for SyntaxTrivia {}

unsafe impl Send for SyntaxTrivia {}
unsafe impl Sync for SyntaxTrivia {}

impl fmt::Debug for SyntaxTrivia {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SyntaxTrivia")
            .field("kind", &self.kind())
            .field("full_width", &self.full_width())
            .field("text", &String::from_utf8_lossy(&self.to_full_string()))
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
        let token = SyntaxTrivia::new_with_text(kind, text.to_vec());
        assert_eq!(token.to_string(), text);
        assert_eq!(token.to_full_string(), text);
    }

    #[rstest]
    #[case::whitespace(SyntaxKind::WhitespaceTrivia, b" ")]
    #[case::comment(SyntaxKind::CommentTrivia, b"% Comment 1")]
    #[case::end_of_line(SyntaxKind::EndOfLineTrivia, b"\r\n")]
    fn test_width(#[case] kind: SyntaxKind, #[case] text: &[u8]) {
        let token = SyntaxTrivia::new_with_text(kind, text.to_vec());
        assert_eq!(token.width(), text.len());
        assert_eq!(token.full_width(), text.len());
    }

    #[rstest]
    fn test_is_trivia() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".to_vec());
        assert!(token.is_trivia());
    }

    #[rstest]
    fn test_is_not_token() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".to_vec());
        assert!(!token.is_token());
    }

    #[rstest]
    fn test_is_not_list() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".to_vec());
        assert!(!token.is_list());
    }

    #[rstest]
    fn test_no_nested_trivia() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".to_vec());
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
    fn test_slot_with_any_index_expect_none(#[case] index: usize) {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".to_vec());
        assert_eq!(token.slot(index), None);
    }

    #[rstest]
    fn test_slot_count_expect_zero() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".to_vec());
        assert_eq!(token.slot_count(), 0);
    }

    #[rstest]
    fn test_clone() {
        let token = SyntaxTrivia::new_with_text(SyntaxKind::WhitespaceTrivia, b" ".to_vec());
        let cloned = token.clone();
        assert_eq!(token, cloned);
    }

    #[rstest]
    #[case::same_kind_same_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::WhitespaceTrivia, b" ", b" ", true)]
    #[case::same_kind_different_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::WhitespaceTrivia, b" ", b"  ", false)]
    #[case::different_kind_same_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::CommentTrivia, b" ", b" ", false)]
    #[case::different_kind_different_text(SyntaxKind::WhitespaceTrivia, SyntaxKind::CommentTrivia, b" ", b"  ", false)]
    fn test_eq(#[case] kind: SyntaxKind, #[case] expected_kind: SyntaxKind, #[case] text: &[u8], #[case] expected_text: &[u8], #[case] expected: bool) {
        let token = SyntaxTrivia::new_with_text(kind, text.to_vec());
        let other = SyntaxTrivia::new_with_text(expected_kind, expected_text.to_vec());
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
        let token = SyntaxTrivia::new_with_text(kind, text.to_vec());
        assert_eq!(format!("{:?}", token), expected);
    }
}
