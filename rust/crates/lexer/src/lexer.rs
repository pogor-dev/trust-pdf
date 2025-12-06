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
            _ => return, // TODO: add a test for this case
        };

        match first_byte {
            b'0'..=b'9' => {
                self.scan_numeric_literal(token_info);
            }
            _ => {}
        };
    }

    fn scan_numeric_literal(&mut self, token_info: &mut TokenInfo<'source>) {
        self.advance(); // consume the first digit

        while let Some(byte) = self.peek() {
            match byte {
                b'0'..=b'9' => {
                    self.advance();
                }
                _ => break,
            }
        }

        token_info.kind = SyntaxKind::IntegerLiteralToken;
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
    use syntax::SyntaxKind;

    use super::Lexer;

    #[test]
    fn test_numeric_literal() {
        let source = b"12345";
        let mut lexer = Lexer::new(source);

        let token = lexer.next_token().unwrap(); // unwrap is not recommended in production code, but acceptable in tests
        assert_eq!(token.kind(), SyntaxKind::IntegerLiteralToken.into());
        assert_eq!(token.bytes(), b"12345");
    }
}
