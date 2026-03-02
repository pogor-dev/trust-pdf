use crate::lexer::Lexer;

impl<'source> Lexer<'source> {
    /// Start recording a lexeme from the current position.
    pub(super) fn start_lexeme(&mut self) {
        self.lexeme = Some(self.position..self.position);
    }

    /// Finalize the current lexeme by setting its end position.
    pub(super) fn stop_lexeme(&mut self) {
        self.lexeme = None;
    }

    /// Get the current lexeme bytes.
    pub(super) fn get_lexeme_bytes(&self) -> &'source [u8] {
        match &self.lexeme {
            Some(range) => &self.source[range.clone()],
            None => b"",
        }
    }

    /// Advance the cursor by one byte and return the byte at the new position.
    pub(super) fn advance(&mut self) -> Option<u8> {
        self.advance_by(1)
    }

    /// Advance the cursor by `offset` bytes and return the byte at the new position.
    #[inline]
    pub(super) fn advance_by(&mut self, offset: usize) -> Option<u8> {
        assert!(offset > 0, "Offset must be positive");
        self.position = self.position + offset;

        // Update lexeme range before retrieving byte, so it updates even at EOF
        if let Some(lexeme) = &mut self.lexeme {
            lexeme.end += offset;
        }

        let byte = self.source.get(self.position)?;
        Some(*byte)
    }

    /// Advance until any provided byte sequence is at the current position, or consume to EOF.
    ///
    /// Behavior on ties: if multiple sequences match at the same position, the first
    /// element in `sequences` wins because iteration short-circuits on the first match.
    /// If no sequence is found, the cursor advances to EOF and stops.
    pub(super) fn advance_until(&mut self, sequences: &[&[u8]]) {
        while let Some(remaining) = self.source.get(self.position..) {
            if remaining.is_empty() {
                break; // reached EOF without a match
            }

            if sequences.iter().any(|seq| remaining.starts_with(seq)) {
                break;
            }

            self.advance_by(1);
        }
    }

    /// Check if the bytes at the current position match the given sequence.
    ///
    /// Returns `true` if all bytes in `sequence` match starting from the current position,
    /// `false` otherwise (including when there aren't enough bytes remaining).
    #[inline]
    pub(super) fn matches_sequence(&self, sequence: &[u8]) -> bool {
        self.source.get(self.position..).map_or(false, |remaining| remaining.starts_with(sequence))
    }

    /// Peek at the first byte without advancing the cursor.
    pub(super) fn peek(&self) -> Option<u8> {
        self.peek_by(0)
    }

    /// Peek at the byte at `offset` without advancing the cursor.
    #[inline]
    pub(super) fn peek_by(&self, offset: usize) -> Option<u8> {
        self.source.get(self.position + offset).copied()
    }
}
