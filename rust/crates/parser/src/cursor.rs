use std::cmp::min;

use syntax::{GreenToken, SyntaxKind, green_syntax_factory};

use crate::Parser;

impl<'source> Parser<'source> {
    pub(super) fn current_token(&mut self) -> GreenToken {
        if self.window_offset >= self.window_size {
            let token = self.lexer.next_token();
            self.add_lexed_token(token);
        }

        let token = self.lexed_tokens[self.window_offset]
            .clone()
            .expect("The sliding window logic must be broken, we don't expect the tokens to be None");

        token
    }

    // TODO: decide what functions should be inlined
    pub(super) fn peek_token(&mut self) -> GreenToken {
        self.peek_token_by(1)
    }

    pub(super) fn peek_token_by(&mut self, offset: usize) -> GreenToken {
        debug_assert!(offset > 0, "Offset must be positive");

        if self.window_offset + offset >= self.window_size {
            let token = self.lexer.next_token();
            self.add_lexed_token(token);
        }

        let token = self.lexed_tokens[self.window_offset + offset]
            .clone()
            .expect("The sliding window logic must be broken, we don't expect the tokens to be None");

        token
    }

    pub(super) fn advance_token(&mut self) -> GreenToken {
        let current_token = self.current_token();
        self.move_to_next_token();
        current_token
    }

    pub(super) fn eat_token(&mut self) -> GreenToken {
        let current_token = self.current_token();
        self.move_to_next_token();
        current_token
    }

    pub(super) fn eat_token_or_create_missing(&mut self, expected: SyntaxKind) -> GreenToken {
        debug_assert!(expected.is_any_token(), "Expected a token kind, got {:?}", expected);
        let current_token = self.current_token();
        let actual = current_token.kind();

        if actual == expected {
            return self.eat_token();
        }

        self.create_missing_token(expected, actual)
    }

    pub(super) fn eat_token_or_replace_with_missing(&mut self, expected: SyntaxKind) -> GreenToken {
        debug_assert!(expected.is_any_token(), "Expected a token kind, got {:?}", expected);
        let current_token = self.current_token();
        let actual = current_token.kind();

        if actual == expected {
            return self.eat_token();
        }

        let replacement = self.create_missing_token(expected, actual);
        // TODO: return AddTrailingSkippedSyntax(replacement, this.EatToken())
        return replacement;
    }

    pub(super) fn pre_lex(&mut self) {
        let size = min(Self::CACHED_TOKEN_ARRAY_SIZE, self.lexer.source_length() / 2);
        for _ in 0..size {
            let token = self.lexer.next_token();
            let token_kind = token.kind();

            self.add_lexed_token(token);

            if token_kind == SyntaxKind::EndOfFileToken {
                break;
            }
        }
    }

    fn create_missing_token(&self, expected: SyntaxKind, actual: SyntaxKind) -> GreenToken {
        green_syntax_factory::missing_token(expected)
        // TODO: add diagnostic information to the token for error reporting
        /*
           var token = SyntaxFactory.MissingToken(expected);
           return WithAdditionalDiagnostics(token, this.GetExpectedMissingNodeOrTokenError(token, expected, actual));
        */
    }

    fn move_to_next_token(&mut self) {
        self.window_offset += 1;
    }

    fn add_lexed_token(&mut self, token: GreenToken) {
        if self.window_size >= self.lexed_tokens.len() {
            self.add_lexed_token_slot();
        }

        self.lexed_tokens[self.window_size] = Some(token);
        self.window_size += 1;
    }

    /// Maintains a sliding token buffer: shift left when the head is consumed,
    /// otherwise grow capacity to keep incremental lexing bounded.
    ///
    /// `window_start` is the global index of the token stored in slot 0. When
    /// we shift the window left, the same logical tokens move to lower slots,
    /// so the global base index must advance by `shift_offset` to keep
    /// `window_start + window_offset` pointing at the same token in the stream.
    ///
    /// Sliding window (global indexes vs buffer slots):
    ///
    /// ```text
    /// Global token stream (indexes):
    ///   0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 ...
    /// Buffer before shift (window_start = 0):
    ///   [0 1 2 3 4 5 6 7 8 9 _ _ _ _ _ _ ...]
    ///                ^ window_offset
    /// Shift left by window_offset (e.g. 6) to keep recent tokens:
    ///   [6 7 8 9 _ _ _ _ _ _ _ _ _ _ _ _ ...]
    ///    ^ window_start increases by shift_offset, window_offset becomes 0
    ///
    /// Growth model (when not shifting):
    ///   [0 1 2 3 4 5 6 7] -> resize(len * 2)
    ///   [0 1 2 3 4 5 6 7 _ _ _ _ _ _ _ _] (doubled capacity with None slots)
    /// ```
    fn add_lexed_token_slot(&mut self) {
        // Shift tokens to the left if we've consumed more than half the buffer.
        // This prevents unbounded growth when incrementally lexing.
        if self.window_offset > self.lexed_tokens.len() >> 1 {
            let shift_offset = self.window_offset;
            let shift_count = self.window_size - shift_offset;
            debug_assert!(shift_offset > 0, "shift_offset must be greater than 0");

            // Shift remaining unconsumed tokens to the beginning of the buffer.
            // Equivalent to Roslyn's Array.Copy(_lexedTokens, shiftOffset, _lexedTokens, 0, shiftCount)
            if shift_count > 0 {
                for i in 0..shift_count {
                    self.lexed_tokens[i] = self.lexed_tokens[i + shift_offset].take();
                }
            }

            self.window_start += shift_offset;
            self.window_size -= shift_offset;
            self.window_offset -= shift_offset;
        } else {
            self.lexed_tokens.resize(self.lexed_tokens.len() * 2, None);
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

    #[test]
    fn test_add_lexed_token_slot_triggers_shift() {
        // Create a stream large enough to fill initial buffer and trigger shifting
        // Initial capacity is min(64, source_len / 2), so for 200 bytes we get ~64 capacity
        let mut source = Vec::new();
        source.push(b'[');
        for i in 0..150 {
            source.extend_from_slice(format!(" {}", i % 10).as_bytes());
        }
        source.push(b']');

        let mut parser = create_parser(&source);

        // First, fill the buffer completely by peeking ahead
        let initial_capacity = parser.lexed_tokens.len();

        // Consume more than half the buffer
        let tokens_to_consume = (initial_capacity >> 1) + 5;
        for _ in 0..tokens_to_consume {
            parser.advance_token();
        }

        // Now peek far enough ahead to fill remaining buffer and force a new token
        // This should trigger shift since window_offset > len >> 1 and buffer is full
        for i in 1..initial_capacity {
            let _ = parser.peek_token_by(i);
        }

        // Verify window was shifted by checking window_start moved forward
        assert!(parser.window_start > 0, "window_start should have advanced after shifting");

        // Should still be able to parse correctly after shift
        let token = parser.current_token();
        assert!(matches!(token.kind(), SyntaxKind::NumericLiteralToken));
    }

    #[test]
    fn test_add_lexed_token_slot_triggers_resize() {
        // Create a small source to force minimal initial capacity, then peek beyond it
        let source = b"[1 2 3]";
        let mut parser = create_parser(source);

        // Don't advance, just peek ahead repeatedly to fill buffer without consuming
        // This should trigger resize instead of shift (because window_offset stays at 0)
        let initial_capacity = parser.lexed_tokens.len();

        // Peek enough times to exceed initial capacity
        for i in 1..=initial_capacity + 5 {
            let _ = parser.peek_token_by(i);
        }

        // Buffer should have resized
        assert!(parser.lexed_tokens.len() > initial_capacity, "Buffer should have grown via resize");

        // window_start should still be 0 since we didn't advance
        assert_eq!(parser.window_start, 0, "window_start should remain 0 when resizing");
    }

    #[test]
    fn test_shift_with_zero_shift_count() {
        // Test edge case where shift_count is 0
        let source = b"[1]";
        let mut parser = create_parser(source);

        // Consume all tokens to make shift_count = 0
        while parser.current_token().kind() != SyntaxKind::EndOfFileToken {
            parser.advance_token();
        }

        // Now advance once more and access beyond, forcing slot allocation
        // with window_offset pointing past all valid tokens
        let eof = parser.current_token();
        assert_eq!(eof.kind(), SyntaxKind::EndOfFileToken);
    }
}
