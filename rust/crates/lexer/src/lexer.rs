use std::ops::Range;

use syntax::{GreenNodeBuilder, GreenToken, NodeOrToken, SyntaxKind};

pub struct Lexer<'source> {
    source: &'source [u8],
    position: usize,
    lexeme: Option<Range<usize>>, // start=position, end=start+width
}

#[derive(Debug, Default)]
struct TokenInfo<'a> {
    kind: SyntaxKind,
    bytes: &'a [u8],
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source [u8]) -> Self {
        Self {
            source,
            position: 0,
            lexeme: None,
        }
    }

    pub fn next_token(&mut self) -> Option<GreenToken> {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind::LexerNode.into());

        // TODO: add leading trivia handling

        let mut token_info: TokenInfo<'source> = TokenInfo::default();
        self.start_lexeme();
        self.scan_token(&mut token_info);
        self.stop_lexeme();

        // TODO: add trailing trivia handling

        builder.start_token(token_info.kind.into());
        builder.token_text(token_info.bytes);
        builder.finish_token();
        builder.finish_node();
        let node = builder.finish();

        match node.children().next() {
            Some(NodeOrToken::Token(token)) => Some(token),
            _ => None,
        }
    }

    fn scan_token(&mut self, token_info: &mut TokenInfo<'source>) {
        let first_byte = match self.peek() {
            Some(first_byte) => first_byte,
            _ => {
                token_info.kind = SyntaxKind::EndOfFileToken;
                token_info.bytes = b"";
                return;
            }
        };

        match first_byte {
            b'0'..=b'9' | b'+' | b'-' | b'.' => {
                // TODO: Architectural limits on numeric literals, I think this should be handled in Semantic analysis phase
                self.scan_numeric_literal(token_info);
            }
            _ => {}
        };
    }

    fn scan_numeric_literal(&mut self, token_info: &mut TokenInfo<'source>) {
        token_info.kind = SyntaxKind::NumericLiteralToken; // default to numeric literal
        self.advance(); // consume the first digit

        while let Some(byte) = self.peek() {
            match byte {
                b'0'..=b'9' => {
                    self.advance(); // consume the digit
                }
                b'.' => {
                    self.advance(); // consume the dot
                }
                b'+' | b'-' => {
                    // Sign not allowed after first digit (e.g., `12+34` is invalid).
                    // According to PDF Compacted Syntax Matrix, integer and/or real numbers should be separated by delimiters.
                    token_info.kind = SyntaxKind::BadToken; // mark as bad token
                    self.advance();
                }
                _ => break,
            }
        }

        token_info.bytes = self.get_lexeme_bytes();
    }

    /// Start recording a lexeme from the current position.
    fn start_lexeme(&mut self) {
        self.lexeme = Some(self.position..self.position);
    }

    /// Finalize the current lexeme by setting its end position.
    fn stop_lexeme(&mut self) {
        self.lexeme = None;
    }

    /// Get the current lexeme bytes.
    fn get_lexeme_bytes(&self) -> &'source [u8] {
        match &self.lexeme {
            Some(range) => &self.source[range.clone()],
            None => b"",
        }
    }

    /// Advance the cursor by one byte and return the byte at the new position.
    fn advance(&mut self) -> Option<u8> {
        self.advance_by(1)
    }

    /// Advance the cursor by `offset` bytes and return the byte at the new position.
    #[inline]
    fn advance_by(&mut self, offset: usize) -> Option<u8> {
        assert!(offset > 0, "Offset must be positive");
        self.position = self.position + offset;

        // Update lexeme range before retrieving byte, so it updates even at EOF
        if let Some(lexeme) = &mut self.lexeme {
            lexeme.end += offset;
        }

        let byte = self.source.get(self.position)?;
        Some(*byte)
    }

    /// Peek at the first byte without advancing the cursor.
    fn peek(&self) -> Option<u8> {
        self.peek_by(0)
    }

    /// Peek at the byte at `offset` without advancing the cursor.
    #[inline]
    fn peek_by(&self, offset: usize) -> Option<u8> {
        self.source.get(self.position + offset).copied()
    }
}

/// Check if a byte is a white-space character.
///
/// The white-space characters are:
/// - 0x00 NULL (`NUL`)
/// - 0x09 HORIZONTAL TAB (`\t`)
/// - 0x0A LINE FEED (`\n`)
/// - 0x0C FORM FEED (`\f`)
/// - 0x0D CARRIAGE RETURN (`\r`)
/// - 0x20 SPACE (` `)
///
/// See: ISO 32000-2:2020, §7.2.3 Character set, Table 1: White-space characters.
fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b'\0' | b'\t' | b'\n' | b'\x0C' | b'\r' | b' ')
}

/// Check if a sequence of bytes make an EOL (end of line).
///
/// An EOL is defined as either:
/// - A single LINE FEED (`\n`, 0x0A)
/// - A single CARRIAGE RETURN (`\r`, 0x0D)
/// - A CARRIAGE RETURN followed by a LINE FEED (`\r\n`, 0x0D 0x0A)
///
/// See: ISO 32000-2:2020, §7.2.3 Character set.
fn is_eol(bytes: &[u8]) -> bool {
    match bytes {
        [b'\n'] => true,
        [b'\r'] => true,
        [b'\r', b'\n'] => true,
        _ => false,
    }
}

/// Check if a byte is a delimiter character.
///
/// The delimiter characters are:
/// - 0x28 LEFT PARENTHESIS (`(`)
/// - 0x29 RIGHT PARENTHESIS (`)`)
/// - 0x3C LESS-THAN SIGN (`<`)
/// - 0x3E GREATER-THAN SIGN (`>`)
/// - 0x5B LEFT SQUARE BRACKET (`[`)
/// - 0x5D RIGHT SQUARE BRACKET (`]`)
/// - 0x7B LEFT CURLY BRACKET (`{`)
/// - 0x7D RIGHT CURLY BRACKET (`}`)
/// - 0x2F SOLIDUS (`/`)
/// - 0x25 PERCENT SIGN (`%`)
///
/// See: ISO 32000-2:2020, §7.2.3 Character set, Table 2: Delimiter characters.
fn is_delimiter(byte: u8) -> bool {
    matches!(byte, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%')
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use syntax::{GreenToken, SyntaxKind};

    #[test]
    fn test_numeric_literal_123() {
        let mut lexer = Lexer::new(b"123");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"123");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_43445() {
        let mut lexer = Lexer::new(b"43445");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"43445");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_plus_17() {
        let mut lexer = Lexer::new(b"+17");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"+17");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_minus_98() {
        let mut lexer = Lexer::new(b"-98");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"-98");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_0() {
        let mut lexer = Lexer::new(b"0");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"0");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_00987() {
        let mut lexer = Lexer::new(b"00987");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"00987");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_34_5() {
        let mut lexer = Lexer::new(b"34.5");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"34.5");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_minus_3_62() {
        let mut lexer = Lexer::new(b"-3.62");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"-3.62");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_plus_123_6() {
        let mut lexer = Lexer::new(b"+123.6");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"+123.6");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_4_() {
        let mut lexer = Lexer::new(b"4.");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"4.");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_numeric_literal_minus__002() {
        let mut lexer = Lexer::new(b"-.002");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"-.002");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    fn test_numeric_literal_009_87() {
        let mut lexer = Lexer::new(b"009.87");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"009.87");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_numeric_literal__3_4() {
        let mut lexer = Lexer::new(b".34");
        let token = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b".34");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_numeric_plus_plus_invalid() {
        let mut lexer = Lexer::new(b"++");
        let token: GreenToken = lexer.next_token().unwrap();
        assert_numeric_literal_token(&token, SyntaxKind::BadToken, b"++");
        assert_eof_token(&lexer.next_token().unwrap());
    }

    fn assert_numeric_literal_token(token: &GreenToken, expected_kind: SyntaxKind, expected_bytes: &[u8]) {
        assert_eq!(Into::<SyntaxKind>::into(token.kind()), expected_kind);
        assert_eq!(token.bytes(), expected_bytes);
        assert_eq!(token.width(), expected_bytes.len() as u32);
        assert_eq!(token.full_width(), expected_bytes.len() as u32);
        assert_eq!(token.trailing_trivia().pieces().len(), 0);
        assert_eq!(token.leading_trivia().pieces().len(), 0);
    }

    fn assert_eof_token(token: &GreenToken) {
        assert_eq!(Into::<SyntaxKind>::into(token.kind()), SyntaxKind::EndOfFileToken);
        assert_eq!(token.bytes(), b"");
        assert_eq!(token.width(), 0);
        assert_eq!(token.full_width(), 0);
        assert_eq!(token.trailing_trivia().pieces().len(), 0);
        assert_eq!(token.leading_trivia().pieces().len(), 0);
    }
}
