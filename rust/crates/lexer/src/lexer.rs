use std::ops::Range;

use syntax::{GreenCache, GreenNodeBuilder, GreenToken, GreenTriviaInTree, GreenTriviaListInTree, NodeOrToken, SyntaxKind};

// TODO: add normal & stream lexer modes
// TODO: add skip_trivia option
pub struct Lexer<'source> {
    pub(super) source: &'source [u8],
    pub(super) position: usize,
    pub(super) lexeme: Option<Range<usize>>, // start=position, end=start+width
    cache: GreenCache,
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
            cache: GreenCache::default(),
        }
    }

    /// Scans and returns the next token from the source, including its associated trivia.
    ///
    /// The token includes:
    /// - **Leading trivia**: whitespace, end-of-line sequences, and comments appearing before the token
    /// - **Token text**: the actual token bytes (e.g., numeric literal)
    /// - **Trailing trivia**: whitespace, end-of-line sequences, and comments appearing after the token
    ///
    /// Trivia is preserved for full-fidelity reconstruction of the source PDF. The token's width
    /// includes only the token text, while `full_width()` includes both trivia and text.
    ///
    /// # Returns
    ///
    /// A [`GreenToken`] representing the next lexical element. When the end of the source is reached,
    /// returns an `EndOfFileToken` with empty text and no trivia.
    ///
    /// # Example
    ///
    /// ```text
    /// Input: "  123 % comment\n"
    /// Token: kind=NumericLiteralToken, text="123"
    ///        leading="  ", trailing=" % comment\n"
    /// ```
    pub fn next_token(&mut self) -> GreenToken {
        let mut token_info: TokenInfo<'source> = TokenInfo::default();
        let leading_trivia = self.scan_trivia();
        self.scan_token(&mut token_info);
        let trailing_trivia = self.scan_trivia();

        // Build the token
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind::LexerNode.into());
        builder.token(token_info.kind.into(), token_info.bytes, leading_trivia.pieces(), trailing_trivia.pieces());
        builder.finish_node();
        let node = builder.finish();

        match node.children().next() {
            Some(NodeOrToken::Token(token)) => token,
            _ => panic!("Expected a token node"),
        }
    }

    /// Scans the main token content from the current position.
    ///
    /// This function examines the first byte at the current position and dispatches
    /// to the appropriate token-specific scanner (e.g., numeric literals). It populates
    /// the provided `token_info` with the token's kind and byte slice.
    ///
    /// Currently supports:
    /// - Numeric literals (integers and reals): `0-9`, `+`, `-`, `.`
    ///
    /// For unsupported characters, the token_info remains in its default state (kind=Unknown, bytes=empty).
    /// When EOF is reached, sets `EndOfFileToken` with empty bytes.
    fn scan_token(&mut self, token_info: &mut TokenInfo<'source>) {
        let first_byte = match self.peek() {
            Some(first_byte) => first_byte,
            _ => {
                token_info.kind = SyntaxKind::EndOfFileToken;
                token_info.bytes = b"";
                return;
            }
        };

        self.start_lexeme();

        // TODO: stop lexing when encountering delimiter characters
        match first_byte {
            b'0'..=b'9' | b'+' | b'-' | b'.' => {
                // TODO: Architectural limits on numeric literals, I think this should be handled in Semantic analysis phase
                self.scan_numeric_literal(token_info);
            }
            // TODO: Add test coverage for unknown/unsupported characters (e.g., @, #) to verify error handling behavior
            _ => {}
        };

        self.stop_lexeme();
    }

    /// Scans consecutive trivia (non-semantic elements) from the current position.
    ///
    /// Trivia includes whitespace, end-of-line sequences, and comments that don't affect
    /// the semantic meaning of the PDF but must be preserved for full-fidelity reconstruction.
    ///
    /// Recognized trivia types:
    /// - Whitespace: space, NULL, tab, form feed
    /// - End-of-line: CR, LF, or CR+LF sequences
    /// - Comments: `%` to end of line
    ///
    /// Trivia is scanned greedily until a non-trivia character is encountered.
    /// Returns a cached trivia list for efficient memory usage and deduplication.
    fn scan_trivia(&mut self) -> GreenTriviaListInTree {
        let mut trivia = Vec::new();
        loop {
            let first_byte = match self.peek() {
                Some(byte) => byte,
                _ => break,
            };

            match first_byte {
                b' ' | b'\0' | b'\t' | b'\x0C' => {
                    trivia.push(self.scan_whitespace());
                }
                b'\r' | b'\n' => {
                    trivia.push(self.scan_end_of_line());
                }
                b'%' => {
                    trivia.push(self.scan_comment());
                }
                _ => break,
            }
        }
        self.cache.trivia_list(&trivia).1
    }

    /// Scans consecutive whitespace characters and returns a cached trivia entry.
    ///
    /// Consumes space (0x20), NULL (0x00), tab (0x09), and form feed (0x0C) characters
    /// greedily until a non-whitespace character is encountered. The scanned bytes are
    /// cached for memory efficiency and deduplication.
    ///
    /// Does not consume end-of-line sequences (CR/LF) - those are handled separately.
    fn scan_whitespace(&mut self) -> GreenTriviaInTree {
        let pos = self.position;
        self.advance(); // consume the first whitespace

        while let Some(byte) = self.peek() {
            match byte {
                b' ' | b'\0' | b'\t' | b'\x0C' => {
                    self.advance(); // consume whitespace
                }
                _ => break,
            }
        }

        let spaces = &self.source[pos..self.position];
        self.cache.trivia(SyntaxKind::WhitespaceTrivia.into(), spaces).1
    }

    // TODO: Potential bug - multiple consecutive line breaks (e.g., "\n\n") are consumed as single trivia piece.
    // TODO: Add test coverage for consecutive EOL sequences (\n\n, \r\r, \r\n\r\n) to verify/document intended behavior.
    // Each EOL sequence should likely be tracked as separate EndOfLineTrivia entries for proper PDF semantics.
    /// Scans end-of-line sequences and returns a cached trivia entry.
    ///
    /// Recognizes PDF EOL formats: LF (0x0A), CR (0x0D), and CR+LF (0x0D 0x0A).
    /// Currently consumes multiple consecutive EOL sequences as a single trivia piece
    /// for efficiency, though this may need refinement for strict PDF semantics.
    ///
    /// The scanned bytes are cached for memory efficiency.
    fn scan_end_of_line(&mut self) -> GreenTriviaInTree {
        let pos = self.position;
        self.advance(); // consume the first EOL byte

        while let Some(byte) = self.peek() {
            match byte {
                b'\r' if self.peek_by(1) == Some(b'\n') => {
                    self.advance_by(2); // consume CR LF
                }
                b'\r' | b'\n' => {
                    self.advance(); // consume CARRIAGE RETURN or LINE FEED
                }
                _ => break,
            }
        }

        let eol_bytes = &self.source[pos..self.position];
        self.cache.trivia(SyntaxKind::EndOfLineTrivia.into(), eol_bytes).1
    }

    /// Scans a PDF comment and returns a cached trivia entry.
    ///
    /// Comments in PDF begin with `%` and extend to the end of the line.
    /// The comment includes the `%` character but stops before the EOL sequence.
    /// The EOL is handled separately by `scan_trivia()` as its own trivia piece.
    ///
    /// The scanned bytes (including `%`) are cached for memory efficiency.
    fn scan_comment(&mut self) -> GreenTriviaInTree {
        let pos = self.position;
        self.advance(); // consume the '%'

        while let Some(byte) = self.peek() {
            match byte {
                b'\r' | b'\n' => break, // end of comment
                _ => {
                    self.advance(); // consume comment character
                }
            }
        }

        let comment_bytes = &self.source[pos..self.position];
        self.cache.trivia(SyntaxKind::CommentTrivia.into(), comment_bytes).1
    }

    // TODO: Bug - allows multiple decimal points (e.g., "1.2.3.4" accepted as single token).
    // Should track if decimal point seen and mark as BadToken on subsequent decimal points.
    /// Scans a numeric literal (integer or real number) and populates token_info.
    ///
    /// Accepts digits (0-9), decimal points (.), and signs (+/-) at the start.
    /// Signs after the first character mark the token as `BadToken` since PDF requires
    /// numeric literals to be delimiter-separated.
    ///
    /// Updates token_info with:
    /// - `kind`: `NumericLiteralToken` for valid numbers, `BadToken` for invalid ones
    /// - `bytes`: the complete scanned byte sequence
    ///
    /// The bytes are extracted from the lexeme range and not cached directly.
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
    matches!(byte, b'\0' | b'\t' | b'\x0C' | b'\r' | b'\n' | b' ')
}

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
/// See: ISO 32000-2:2020, §7.2.3 Character set, Table 2: Delimiter characters.
///
/// ## Delimiter characters
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
/// ## Note on curly brackets
/// Delimiter characters such as `{` and `}` are only used within Type 4 PostScript calculator functions
/// (see ISO 32000-2:2020, §7.10.5).
/// These functions are included in PDF streams.
///
/// - In normal lexer mode, these characters will not be recognized as delimiters.
/// - In stream lexer mode, these characters will be recognized as delimiters.
///
/// ## Note on double character delimiters
/// In addition, double character delimiters (`<<`, `>>`) are used in dictionaries.
fn is_delimiter(byte: u8) -> bool {
    matches!(byte, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%')
}
