use crate::SyntaxKind;

pub trait GreenNode {
    fn kind(&self) -> SyntaxKind;

    fn text(&self) -> &[u8];

    fn full_text(&self) -> &[u8];

    #[inline]
    fn width(&self) -> usize {
        self.full_width() - self.get_leading_trivia_width() - self.get_trailing_trivia_width()
    }

    fn full_width(&self) -> usize;

    #[inline]
    fn get_slot(&self, _index: usize) -> Option<&dyn GreenNode> {
        None
    }

    #[inline]
    fn slot_count(&self) -> usize {
        0
    }

    #[inline]
    fn is_token(&self) -> bool {
        false
    }

    #[inline]
    fn is_trivia(&self) -> bool {
        false
    }

    #[inline]
    fn is_list(&self) -> bool {
        self.kind() == SyntaxKind::List
    }

    fn get_leading_trivia(&self) -> Option<&dyn GreenNode>;

    fn get_trailing_trivia(&self) -> Option<&dyn GreenNode>;

    fn get_leading_trivia_width(&self) -> usize;

    fn get_trailing_trivia_width(&self) -> usize;

    #[inline]
    fn has_leading_trivia(&self) -> bool {
        self.get_leading_trivia_width() != 0
    }

    #[inline]
    fn has_trailing_trivia(&self) -> bool {
        self.get_trailing_trivia_width() != 0
    }
}
