use std::iter::FusedIterator;

#[derive(Clone)]
pub struct SyntaxTriviaPiecesIterator {
    pub(crate) raw: SyntaxTrivia,
    pub(crate) next_index: usize,
    pub(crate) next_offset: TextSize,
    pub(crate) end_index: usize,
    pub(crate) end_offset: TextSize,
}

impl Iterator for SyntaxTriviaPiecesIterator {
    type Item = (TextSize, TriviaPiece);

    fn next(&mut self) -> Option<Self::Item> {
        let trivia = self.raw.get_piece(self.next_index)?;
        let piece = (self.next_offset, *trivia);

        self.next_index += 1;
        self.next_offset += trivia.text_len();

        Some(piece)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end_index.saturating_sub(self.next_index);
        (len, Some(len))
    }
}

impl FusedIterator for SyntaxTriviaPiecesIterator {}

impl DoubleEndedIterator for SyntaxTriviaPiecesIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end_index == self.next_index {
            return None;
        }

        self.end_index -= 1;

        let trivia = self.raw.get_piece(self.end_index)?;
        self.end_offset -= trivia.text_len();

        Some((self.end_offset, *trivia))
    }
}

impl ExactSizeIterator for SyntaxTriviaPiecesIterator {}
