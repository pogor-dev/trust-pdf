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

    fn get_slot<T: GreenNode>(&self, _index: usize) -> Option<&T> {
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

    fn write_token_to<T: GreenNode, W: io::Write>(&self, _writer: &mut W, _leading: bool, _trailing: bool) -> io::Result<()> {
        Ok(())
    }

    fn write_to<T: GreenNode, W: io::Write>(&self, writer: &mut W, leading: bool, trailing: bool) -> io::Result<()>
    where
        Self: Sized,
    {
        // Use explicit stack to avoid stack overflow on deeply nested structures
        let mut stack: Vec<(&Self, bool, bool)> = Vec::new();
        stack.push((self, leading, trailing));

        while let Some((current_node, current_leading, current_trailing)) = stack.pop() {
            if current_node.is_token() {
                current_node.write_token_to::<T, W>(writer, current_leading, current_trailing)?;
                continue;
            }

            if current_node.is_trivia() {
                current_node.write_trivia_to::<W>(writer)?;
                continue;
            }

            let first_index = Self::get_first_non_null_child_index(current_node);
            let last_index = Self::get_last_non_null_child_index(current_node);

            // Push children in reverse order (since stack is LIFO)
            for i in (first_index..=last_index).rev() {
                if let Some(child) = current_node.get_slot::<Self>(i) {
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
            if node.get_slot::<Self>(i).is_some() {
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
            if node.get_slot::<Self>(i).is_some() {
                return i;
            }
        }
        0 // If no children found
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
}
