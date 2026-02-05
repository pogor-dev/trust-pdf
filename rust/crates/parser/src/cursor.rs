use syntax::{GreenToken, SyntaxKind};

use crate::Parser;

impl<'source> Parser<'source> {
    pub(super) fn current_token(&mut self) -> GreenToken {
        if self.token_offset >= self.token_count {
            let token = self.lexer.next_token();
            self.add_lexed_token(token);
        }

        let token = self.lexer_tokens[self.token_offset].clone();
        token
    }

    pub(super) fn peek_token(&mut self) -> GreenToken {
        self.peek_token_by(1)
    }

    pub(super) fn peek_token_by(&mut self, offset: usize) -> GreenToken {
        debug_assert!(offset > 0, "Offset must be positive");

        if self.token_offset + offset >= self.token_count {
            let token = self.lexer.next_token();
            self.add_lexed_token(token);
        }

        let token = self.lexer_tokens[self.token_offset + offset].clone();
        token
    }

    pub(super) fn advance_token(&mut self) -> GreenToken {
        let current_token = self.current_token();
        self.token_offset += 1;
        current_token
    }

    pub(super) fn pre_lex(&mut self) {
        for _ in 0..self.lexer_tokens.capacity() - 1 {
            let token = self.lexer.next_token();
            let token_kind = token.kind();

            self.add_lexed_token(token);

            if token_kind == SyntaxKind::EndOfFileToken {
                break;
            }
        }
    }

    fn add_lexed_token(&mut self, token: GreenToken) {
        if self.token_count >= self.lexer_tokens.len() {
            self.add_token_slot();
        }

        self.lexer_tokens[self.token_count] = token;
        self.token_count += 1;
    }

    fn add_token_slot(&mut self) {
        // Shift tokens to the left if we've consumed more than half the buffer.
        // This prevents unbounded growth when incrementally lexing.
        if self.token_offset > self.lexer_tokens.len() >> 1 {
            let shift_offset = self.token_offset;
            let shift_count = self.token_count - shift_offset;
            debug_assert!(shift_offset > 0, "shift_offset must be greater than 0");

            // Shift remaining unconsumed tokens to the beginning of the buffer.
            // SAFETY: We're shifting left (non-overlapping for reads) and the pointers are valid.
            if shift_count > 0 {
                unsafe {
                    std::ptr::copy(self.lexer_tokens.as_ptr().add(shift_offset), self.lexer_tokens.as_mut_ptr(), shift_count);
                }
            }

            self.first_token += shift_offset;
            self.token_count -= shift_offset;
            self.token_offset -= shift_offset;
        } else {
            // Increase capacity and logical length without initializing new slots.
            self.lexer_tokens.reserve(self.lexer_tokens.len() * 2);
            // SAFETY: We've just reserved space, so new capacity exists.
            // The uninitialized slots will be written to immediately in add_lexed_token.
            unsafe {
                let new_len = self.lexer_tokens.capacity();
                self.lexer_tokens.set_len(new_len);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lexer::Lexer;
    use syntax::SyntaxKind;

    use crate::Parser;

    /// Helper to create a parser from PDF source
    fn create_parser(source: &[u8]) -> Parser<'_> {
        let lexer = Lexer::new(source);
        Parser::new(lexer)
    }

    #[test]
    fn test_cursor_current_token_returns_first_token() {
        let mut parser = create_parser(b"[");

        let current = parser.current_token();

        assert_eq!(current.kind(), SyntaxKind::OpenBracketToken);
        assert_eq!(current.text(), b"[");
    }

    #[test]
    fn test_cursor_current_token_without_advancing_stays_same() {
        let mut parser = create_parser(b"[");

        let first = parser.current_token();
        let second = parser.current_token();

        assert_eq!(first.kind(), second.kind());
        assert_eq!(first.text(), second.text());
    }

    #[test]
    fn test_cursor_peek_token_returns_next_without_advancing() {
        let mut parser = create_parser(b"[]");

        let current = parser.current_token();
        let peeked = parser.peek_token();

        assert_eq!(current.kind(), SyntaxKind::OpenBracketToken);
        // Peeked should be the closing ]
        assert_eq!(peeked.kind(), SyntaxKind::CloseBracketToken);

        // Current should still be first [
        let current_again = parser.current_token();
        assert_eq!(current_again.kind(), SyntaxKind::OpenBracketToken);
    }

    #[test]
    fn test_cursor_peek_token_by_offset() {
        let mut parser = create_parser(b"[1]");

        let current = parser.current_token(); // [
        let peek_1 = parser.peek_token_by(1); // 1
        let peek_2 = parser.peek_token_by(2); // ]

        assert_eq!(current.kind(), SyntaxKind::OpenBracketToken);
        assert_eq!(peek_1.kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(peek_1.text(), b"1");
        assert_eq!(peek_2.kind(), SyntaxKind::CloseBracketToken);
    }

    #[test]
    fn test_cursor_advance_token_moves_forward() {
        let mut parser = create_parser(b"[]");

        let first = parser.current_token();
        assert_eq!(first.kind(), SyntaxKind::OpenBracketToken);

        parser.advance_token();

        let second = parser.current_token();
        assert_eq!(second.kind(), SyntaxKind::CloseBracketToken);
    }

    #[test]
    fn test_cursor_sequence_of_advances_through_simple_array() {
        let mut parser = create_parser(b"[1 2]");

        // [ 1 2 ]
        let expected_sequence: &[(SyntaxKind, &[u8])] = &[
            (SyntaxKind::OpenBracketToken, b"["),
            (SyntaxKind::NumericLiteralToken, b"1"),
            (SyntaxKind::NumericLiteralToken, b"2"),
            (SyntaxKind::CloseBracketToken, b"]"),
        ];

        for (expected_kind, expected_text) in expected_sequence {
            let token = parser.current_token();
            assert_eq!(token.kind(), *expected_kind, "Expected {:?}, got {:?}", expected_kind, token.kind());
            assert_eq!(token.text(), *expected_text);
            parser.advance_token();
        }

        // After all tokens, should hit EOF
        let eof = parser.current_token();
        assert_eq!(eof.kind(), SyntaxKind::EndOfFileToken);
    }

    #[test]
    fn test_cursor_pre_lex_fills_token_buffer() {
        let mut parser = create_parser(b"[1 2]");

        // Note: pre_lex loops based on initial capacity, which may be small
        // So this mainly tests that pre_lex doesn't panic
        parser.pre_lex();

        // We should be able to traverse without errors
        let current = parser.current_token();
        assert_eq!(current.kind(), SyntaxKind::OpenBracketToken);
    }

    #[test]
    fn test_cursor_handles_dictionary() {
        let mut parser = create_parser(b"<</Items/Value>>");

        let expected_sequence: &[(SyntaxKind, &[u8])] = &[
            (SyntaxKind::OpenDictToken, b"<<"),
            (SyntaxKind::NameLiteralToken, b"/Items"),
            (SyntaxKind::NameLiteralToken, b"/Value"),
            (SyntaxKind::CloseDictToken, b">>"),
        ];

        for (expected_kind, expected_text) in expected_sequence {
            let token = parser.current_token();
            assert_eq!(token.kind(), *expected_kind);
            assert_eq!(token.text(), *expected_text);
            parser.advance_token();
        }
    }

    #[test]
    fn test_cursor_with_large_token_stream() {
        // This test should trigger buffer shifting if implemented
        // Create a large token stream: [ 1 1 1 1 ... 1 ]
        let mut source = b"[".to_vec();
        for _ in 0..100 {
            source.extend_from_slice(b" 1");
        }
        source.push(b']');

        let mut parser = create_parser(&source);

        // Advance through many tokens to potentially trigger shifting
        for _ in 0..50 {
            parser.advance_token();
        }

        // Should still be able to get current token without panicking
        let token = parser.current_token();
        assert_eq!(token.kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(token.text(), b"1");
    }
}
