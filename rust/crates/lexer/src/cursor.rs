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

    /// Advance the cursor until one of the specified sequences is found or EOF is reached.
    pub(super) fn advance_until(&mut self, sequences: Vec<&[u8]>) {
        let position = self.position;
        while let Some(remaining) = self.source.get(self.position..) {
            if sequences.iter().any(|seq| remaining.starts_with(seq)) {
                break;
            }

            self.advance_by(1);
        }
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

    /// Check if the cursor has reached the end of the source.
    #[inline]
    pub(super) fn is_eof(&self) -> bool {
        self.position >= self.source.len()
    }
}
