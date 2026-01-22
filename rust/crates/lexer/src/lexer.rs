use std::ops::Range;

use syntax::{DiagnosticKind, DiagnosticSeverity, GreenElement, GreenNode, GreenToken, GreenTrivia, NodeCache, SyntaxKind};

// TODO: add normal & stream lexer modes
// TODO: add skip_trivia option
/// Tokenizes PDF source code into a stream of tokens with full trivia preservation.
///
/// Scans byte sequences and emits tokens following ISO 32000-2:2020 lexical rules.
/// Preserves all whitespace and comments as trivia for full-fidelity reconstruction.
pub struct Lexer<'source> {
    pub(super) source: &'source [u8],
    pub(super) position: usize,
    pub(super) lexeme: Option<Range<usize>>, // start=position, end=start+width
    is_raw_stream: bool,
    cache: NodeCache,
}

#[derive(Debug)]
struct TokenInfo<'a> {
    kind: SyntaxKind,
    bytes: &'a [u8],
    diagnostics: Vec<(DiagnosticSeverity, u16, &'static str)>,
}

impl<'a> Default for TokenInfo<'a> {
    fn default() -> Self {
        Self {
            kind: SyntaxKind::None,
            bytes: b"",
            diagnostics: Vec::new(),
        }
    }
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source [u8]) -> Self {
        Self {
            source,
            position: 0,
            lexeme: None,
            is_raw_stream: false,
            cache: NodeCache::default(),
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
    /// returns a [`SyntaxKind::EndOfFileToken`] with empty text and no trivia.
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
        let leading_trivia = self.scan_trivia(&token_info);
        self.scan_token(&mut token_info);
        let trailing_trivia = self.scan_trivia(&token_info);

        // Build trivia lists
        let leading = if leading_trivia.is_empty() {
            None
        } else {
            Some(GreenNode::new(
                SyntaxKind::List,
                leading_trivia.into_iter().map(GreenElement::Trivia).collect::<Vec<_>>(),
                None,
            ))
        };

        let trailing = if trailing_trivia.is_empty() {
            None
        } else {
            Some(GreenNode::new(
                SyntaxKind::List,
                trailing_trivia.into_iter().map(GreenElement::Trivia).collect::<Vec<_>>(),
                None,
            ))
        };

        // Build diagnostics if any
        let diagnostics = if token_info.diagnostics.is_empty() {
            None
        } else {
            let diag_list = token_info
                .diagnostics
                .iter()
                .map(|(severity, code, message)| self.cache.diagnostic(*code, *severity, message).1)
                .collect::<Vec<_>>();
            Some(syntax::GreenDiagnostics::new(&diag_list))
        };

        // Create the token using cache for deduplication
        self.cache.token(token_info.kind, token_info.bytes, leading, trailing, diagnostics).1
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
    /// Unknown/unsupported characters are scanned as [`SyntaxKind::BadToken`] and continue until
    /// a delimiter, whitespace, or EOF is encountered.
    /// When EOF is reached, sets [`SyntaxKind::EndOfFileToken`] with empty bytes.
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

        match first_byte {
            _ if self.is_raw_stream => {
                // In raw stream mode, everything until 'endstream' is treated as a raw data token
                self.scan_raw_stream_data(token_info);
            }
            b'0'..=b'9' | b'+' | b'-' | b'.' => {
                self.scan_numeric_literal(token_info);
            }
            b'(' => {
                self.scan_literal_string(token_info);
            }
            b'<' if self.peek_by(1) == Some(b'<') => {
                self.scan_dict_open(token_info); // Dictionary opening: `<<`
            }
            b'<' => {
                self.scan_hex_string(token_info); // Hex string: `<`
            }
            b'>' if self.peek_by(1) == Some(b'>') => {
                self.scan_dict_close(token_info); // Dictionary closing: `>>`
            }
            b'>' => {
                self.scan_bad_token(token_info); // Single `>` is invalid
            }
            b'[' => {
                self.scan_array_open(token_info);
            }
            b']' => {
                self.scan_array_close(token_info);
            }
            b'/' => {
                self.scan_name(token_info);
            }
            b'a'..=b'z' | b'A'..=b'Z' => {
                self.scan_keyword(token_info);
            }
            _ => {
                self.scan_bad_token(token_info);
            }
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
    /// Returns a vector of GreenTrivia elements.
    fn scan_trivia(&mut self, token_info: &TokenInfo<'source>) -> Vec<GreenTrivia> {
        let mut trivia = Vec::new();
        loop {
            let first_byte = match self.peek() {
                Some(byte) => byte,
                _ => break,
            };

            match first_byte {
                _ if token_info.kind == SyntaxKind::RawStreamDataToken => {
                    break; // In raw stream mode, do not scan trivia within the raw data token
                }
                b'\r' | b'\n' if self.is_raw_stream => {
                    trivia.push(self.scan_end_of_line());
                    break; // In raw stream mode, stop trivia scanning at EOL
                }
                _ if self.is_raw_stream => {
                    break; // In raw stream mode, do not scan trivia within the raw data token
                }
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
        trivia
    }

    /// Scans consecutive whitespace characters and returns a trivia element.
    ///
    /// Consumes space (0x20), NULL (0x00), tab (0x09), and form feed (0x0C) characters
    /// greedily until a non-whitespace character is encountered.
    ///
    /// Does not consume end-of-line sequences (CR/LF) - those are handled separately.
    fn scan_whitespace(&mut self) -> GreenTrivia {
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
        self.cache.trivia(SyntaxKind::WhitespaceTrivia, spaces).1
    }

    /// Scans a single end-of-line sequence and returns a trivia element.
    ///
    /// Recognizes PDF EOL formats as [`SyntaxKind::EndOfLineTrivia`]: LF (0x0A), CR (0x0D), or CR+LF (0x0D 0x0A).
    /// Consumes exactly one EOL sequence per call. Multiple consecutive EOLs (e.g., "\n\n") are handled
    /// by the caller invoking this method repeatedly via `scan_trivia()`, creating separate trivia entries
    /// for each EOL sequence for proper PDF semantics.
    ///
    /// See: ISO 32000-2:2020, §7.2.3 Character set.
    fn scan_end_of_line(&mut self) -> GreenTrivia {
        let pos = self.position;

        if let Some(byte) = self.peek() {
            match byte {
                b'\r' if self.peek_by(1) == Some(b'\n') => {
                    self.advance_by(2); // consume CR LF
                }
                b'\r' | b'\n' => {
                    self.advance(); // consume CARRIAGE RETURN or LINE FEED
                }
                _ => {}
            }
        }

        let eol_bytes = &self.source[pos..self.position];
        self.cache.trivia(SyntaxKind::EndOfLineTrivia, eol_bytes).1
    }

    /// Scans a PDF comment and returns a trivia element.
    ///
    /// Comments in PDF begin with `%` and extend to the end of the line.
    /// The comment includes the `%` character but stops before the EOL sequence.
    /// The EOL is handled separately by `scan_trivia()` as its own trivia piece.
    ///
    /// See: ISO 32000-2:2020, §7.2.4 Comments.
    fn scan_comment(&mut self) -> GreenTrivia {
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
        self.cache.trivia(SyntaxKind::CommentTrivia, comment_bytes).1
    }

    /// Scans a numeric literal (integer or real number) and populates token_info.
    ///
    /// Accepts digits (0-9), decimal points (.), and signs (+/-) at the start.
    /// Marks the token as [`SyntaxKind::BadToken`] when:
    /// - Multiple decimal points are encountered (e.g., `12.34.56`, `.1.2.3`)
    /// - Signs appear after the first character (e.g., `12+34`, `12-34`)
    ///
    /// According to the SafeDocs PDF Compacted Syntax Matrix and ISO 32000-2:2020 §7.2.3,
    /// numeric literals immediately followed by letters require whitespace (Integer → Boolean/Name/Null).
    /// A diagnostic is emitted when a numeric is directly followed by a letter.
    ///
    /// Updates token_info with:
    /// - `kind`: [`SyntaxKind::NumericLiteralToken`] for valid numbers, [`SyntaxKind::BadToken`] for invalid ones
    /// - `bytes`: the complete scanned byte sequence
    ///
    /// See: ISO 32000-2:2020, §7.3.3 Numbers (integers and reals).
    fn scan_numeric_literal(&mut self, token_info: &mut TokenInfo<'source>) {
        // TODO: Architectural limits on numeric literals, I think this should be handled in semantic analysis phase
        token_info.kind = SyntaxKind::NumericLiteralToken; // default to numeric literal
        let mut seen_dot = false;
        self.advance(); // consume the first digit

        while let Some(byte) = self.peek() {
            match byte {
                b'0'..=b'9' => {
                    self.advance(); // consume the digit
                }
                b'.' => {
                    if seen_dot {
                        // ISO 32000-2:2020 clause 7.3.3: Numbers (integers and reals) must be separated
                        // by token delimiters or whitespace. Multiple decimal points are invalid.
                        // So if we encounter numbers as `12.34.56` or `.1.2.3`, we should mark it as invalid token
                        token_info.kind = SyntaxKind::BadToken;
                    }
                    seen_dot = true;
                    self.advance(); // consume the dot
                }
                b'+' | b'-' => {
                    // Sign not allowed after first digit (e.g., `12+34` is invalid).
                    // ISO 32000-2:2020 clause 7.3.3: Integer and real numbers must be separated by delimiters.
                    token_info.kind = SyntaxKind::BadToken; // mark as bad token
                    self.advance();
                }
                _ => break,
            }
        }

        token_info.bytes = self.get_lexeme_bytes();

        // Check if this numeric is immediately followed by a letter.
        // SafeDocs PDF Compacted Syntax Matrix: Integer → Boolean/Name/Null requires whitespace.
        // Emit diagnostic if letter follows without whitespace.
        if matches!(self.peek(), Some(b'a'..=b'z' | b'A'..=b'Z')) {
            // TODO: move in parser phase
            let kind = DiagnosticKind::MissingWhitespaceBeforeToken;
            token_info.diagnostics.push((DiagnosticSeverity::Error, kind.into(), kind.as_str()));
        }
    }

    /// Scans a literal string token and populates token_info.
    ///
    /// A literal string in PDF is enclosed in parentheses: `(...)`.
    /// Scans from the opening `(` through the closing `)` and marks it as [`SyntaxKind::StringLiteralToken`].
    ///
    /// Supports both balanced unescaped parentheses (tracked via nesting) and escaped parentheses.
    /// Escaped parentheses (`\(`, `\)`) should not affect the nesting count, though full escape
    /// sequence handling is deferred to semantic analysis. The string closes when nesting returns to zero.
    ///
    /// Updates token_info with:
    /// - `kind`: [`SyntaxKind::StringLiteralToken`]
    /// - `bytes`: the complete scanned byte sequence including parentheses
    ///
    /// See: ISO 32000-2:2020, §7.3.4.2 Literal Strings.
    fn scan_literal_string(&mut self, token_info: &mut TokenInfo<'source>) {
        // TODO: Handle escape sequences within literal strings (e.g., `\(`, `\)`, `\\`, octal sequences) in semantic analysis phase
        token_info.kind = SyntaxKind::StringLiteralToken;
        self.advance(); // consume the opening '('
        let mut nesting = 1; // nesting starts at 1 for the initial consumed '('

        while let Some(byte) = self.peek() {
            match byte {
                b'\\'
                    if matches!(
                        self.peek_by(1),
                        Some(b'(') | Some(b')') | Some(b'\\') | Some(b'n') | Some(b'r') | Some(b't') | Some(b'b') | Some(b'f')
                    ) =>
                {
                    // Handle recognized one-char escape sequences: \n, \r, \t, \b, \f, \(, \), \\
                    self.advance_by(2); // consume both backslash and escaped character
                }
                b'\\' if matches!(self.peek_by(1), Some(b'0'..=b'7')) => {
                    // Octal escape: \ddd (up to 3 octal digits). Consume backslash + up to 3 octal digits.
                    self.advance(); // consume backslash
                    let mut count = 0;
                    // Consume up to three octal digits
                    while count < 3 {
                        if let Some(next) = self.peek() {
                            if (b'0'..=b'7').contains(&next) {
                                self.advance();
                                count += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                b'\\' if matches!(self.peek_by(1), Some(b'\r' | b'\n')) => {
                    // Line continuation: backslash followed by EOL is ignored (ISO 32000-2:2020 §7.3.4.2)
                    self.advance(); // consume backslash
                    let next = self.advance(); // consume CR/LF
                    // If CR, consume following LF if present
                    if next == Some(b'\r') && self.peek() == Some(b'\n') {
                        self.advance();
                    }
                }
                b'\\' if matches!(self.peek_by(1), Some(_)) => {
                    // Unknown escape: emit warning, consume backslash only; next char handled normally
                    let kind = DiagnosticKind::InvalidEscapeInStringLiteral;
                    token_info.diagnostics.push((DiagnosticSeverity::Warning, kind.into(), kind.as_str()));
                    self.advance();
                }
                b'\\' if matches!(self.peek_by(1), None) => {
                    // Backslash at EOF: consume backslash only and exit loop; string will be unbalanced
                    self.advance();
                    break;
                }
                b'(' => {
                    nesting += 1;
                    self.advance();
                }
                b')' => {
                    self.advance(); // consume the ')'
                    nesting -= 1;

                    if nesting == 0 {
                        break; // exit when string is fully closed
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }
        token_info.bytes = self.get_lexeme_bytes();

        // If nesting is not zero, the string is unbalanced
        if nesting != 0 {
            let kind = DiagnosticKind::UnbalancedStringLiteral;
            token_info.diagnostics.push((DiagnosticSeverity::Error, kind.into(), kind.as_str()));
        }
    }

    /// Scans a hexadecimal string token and populates token_info.
    ///
    /// A hexadecimal string in PDF is enclosed in angle brackets: `<...>`.
    /// Contains hexadecimal digits (0-9, A-F, a-f) with optional whitespace (ignored).
    /// Each pair of hex digits defines one byte. If odd number of digits, final digit assumes trailing 0.
    ///
    /// Updates token_info with:
    /// - `kind`: [`SyntaxKind::HexStringLiteralToken`]
    /// - `bytes`: the complete scanned byte sequence including angle brackets
    ///
    /// See: ISO 32000-2:2020, §7.3.4.3 Hexadecimal strings.
    fn scan_hex_string(&mut self, token_info: &mut TokenInfo<'source>) {
        token_info.kind = SyntaxKind::HexStringLiteralToken;
        self.advance(); // consume the opening '<'
        let mut has_invalid_character = false;
        let mut closed = false;

        while let Some(byte) = self.peek() {
            match byte {
                b if is_hexcode(b) => {
                    self.advance(); // consume hex digit
                }
                _ if is_whitespace(byte, true) => {
                    // Whitespace is ignored in hex strings per §7.3.4.3
                    self.advance();
                }
                b'>' => {
                    self.advance(); // consume closing '>'
                    closed = true;
                    break;
                }
                _ => {
                    // Invalid character in hex string: mark and consume
                    has_invalid_character = true;
                    self.advance();
                }
            }
        }

        token_info.bytes = self.get_lexeme_bytes();

        // Emit diagnostics after scanning
        if has_invalid_character {
            let kind = DiagnosticKind::InvalidCharacterInHexString;
            token_info.diagnostics.push((DiagnosticSeverity::Error, kind.into(), kind.as_str()));
        }

        if !closed {
            let kind = DiagnosticKind::UnbalancedHexString;
            token_info.diagnostics.push((DiagnosticSeverity::Error, kind.into(), kind.as_str()));
        }
    }

    /// Scans a name object beginning with `/` as defined in §7.3.5.
    ///
    /// Stops at delimiter characters or whitespace and accepts `#xx` hex escapes.
    /// Emits error diagnostics for invalid hex escapes or non-regular characters that should be hex-escaped.
    fn scan_name(&mut self, token_info: &mut TokenInfo<'source>) {
        // TODO: Architectural limits on name length, I think this should be handled in semantic analysis phase
        token_info.kind = SyntaxKind::NameLiteralToken;
        self.advance(); // consume '/'

        let mut has_invalid_hex_escape = false;
        let mut has_non_regular_character = false;

        while let Some(byte) = self.peek() {
            if is_whitespace(byte, true) || is_delimiter(byte, false) {
                break;
            }

            match byte {
                b'#' if matches!(self.peek_by(1), Some(b) if is_hexcode(b)) && matches!(self.peek_by(2), Some(b) if is_hexcode(b)) => {
                    // Valid hex escape: consume '#xx'
                    self.advance_by(3);
                }
                b'#' if matches!(self.peek_by(1), Some(b) if is_hexcode(b)) => {
                    // Single hex digit or malformed second: consume '#' and first digit, emit diagnostic
                    has_invalid_hex_escape = true;
                    self.advance_by(2);
                }
                b'#' => {
                    // '#' not followed by hex digits: consume '#' only, emit diagnostic
                    has_invalid_hex_escape = true;
                    self.advance();
                }
                b if is_regular_name_char(b) => {
                    self.advance();
                }
                _ => {
                    has_non_regular_character = true;
                    self.advance();
                }
            }
        }

        token_info.bytes = self.get_lexeme_bytes();

        if has_invalid_hex_escape {
            let kind = DiagnosticKind::InvalidHexEscapeInName;
            token_info.diagnostics.push((DiagnosticSeverity::Error, kind.into(), kind.as_str()));
        }

        if has_non_regular_character {
            let kind = DiagnosticKind::InvalidNonRegularCharacterInName;
            token_info.diagnostics.push((DiagnosticSeverity::Error, kind.into(), kind.as_str()));
        }
    }

    /// Scans keywords and boolean/null literals beginning with ASCII letters.
    ///
    /// Scans all consecutive ASCII letters to form a complete keyword, then matches against
    /// known keywords (`true`, `false`, `null`). Unrecognized keywords are scanned as
    /// [`SyntaxKind::BadToken`].
    ///
    /// According to the SafeDocs PDF Compacted Syntax Matrix and ISO 32000-2:2020 §7.2.3,
    /// boolean literals immediately followed by digits require whitespace (Boolean → Integer/Real).
    /// A diagnostic is emitted when a keyword is directly followed by a digit, dot, or sign.
    ///
    /// This approach is efficient—it scans the entire word once, then matches, avoiding
    /// excessive character-by-character lookahead.
    ///
    /// See: ISO 32000-2:2020, §7.3.2 Boolean objects, §7.3.9 Null object.
    fn scan_keyword(&mut self, token_info: &mut TokenInfo<'source>) {
        self.advance(); // consume the first letter

        // Scan all consecutive ASCII letters
        while let Some(byte) = self.peek() {
            match byte {
                b'a'..=b'z' | b'A'..=b'Z' => {
                    self.advance();
                }
                _ => break,
            }
        }

        let keyword_bytes = self.get_lexeme_bytes();

        // Match against known keywords
        token_info.kind = match keyword_bytes {
            b"true" => SyntaxKind::TrueKeyword,
            b"false" => SyntaxKind::FalseKeyword,
            b"null" => SyntaxKind::NullKeyword,
            b"obj" => SyntaxKind::IndirectObjectKeyword,
            b"endobj" => SyntaxKind::IndirectEndObjectKeyword,
            b"R" => SyntaxKind::IndirectReferenceKeyword,
            b"stream" => {
                self.is_raw_stream = true; // enter raw stream mode
                SyntaxKind::StreamKeyword
            }
            b"endstream" => SyntaxKind::EndStreamKeyword,
            b"xref" => SyntaxKind::XRefKeyword,
            b"f" => SyntaxKind::XRefFreeEntryKeyword,
            b"n" => SyntaxKind::XRefInUseEntryKeyword,
            b"trailer" => SyntaxKind::FileTrailerKeyword,
            b"startxref" => SyntaxKind::StartXRefKeyword,
            _ => SyntaxKind::BadToken,
        };

        token_info.bytes = keyword_bytes;

        // Check if this is a keyword immediately followed by a digit, dot, or sign.
        // SafeDocs PDF Compacted Syntax Matrix: Boolean → Integer/Real requires whitespace.
        // Emit diagnostic if numeric start follows without whitespace.
        if token_info.kind != SyntaxKind::BadToken && matches!(self.peek(), Some(b'0'..=b'9' | b'.' | b'+' | b'-')) {
            // TODO: move in parser phase
            let kind = DiagnosticKind::MissingWhitespaceBeforeToken;
            token_info.diagnostics.push((DiagnosticSeverity::Error, kind.into(), kind.as_str()));
        }
    }

    /// Scans the array opening bracket `[` as [`SyntaxKind::OpenBracketToken`].
    ///
    /// See: ISO 32000-2:2020, §7.3.6 Array objects.
    fn scan_array_open(&mut self, token_info: &mut TokenInfo<'source>) {
        token_info.kind = SyntaxKind::OpenBracketToken;
        self.advance(); // consume '['
        token_info.bytes = self.get_lexeme_bytes();
    }

    /// Scans the array closing bracket `]` as [`SyntaxKind::CloseBracketToken`].
    ///
    /// See: ISO 32000-2:2020, §7.3.6 Array objects.
    fn scan_array_close(&mut self, token_info: &mut TokenInfo<'source>) {
        token_info.kind = SyntaxKind::CloseBracketToken;
        self.advance(); // consume ']'
        token_info.bytes = self.get_lexeme_bytes();
    }

    /// Scans the dictionary opening bracket `<<` as [`SyntaxKind::OpenDictToken`].
    ///
    /// See: ISO 32000-2:2020, §7.3.7 Dictionary objects.
    fn scan_dict_open(&mut self, token_info: &mut TokenInfo<'source>) {
        token_info.kind = SyntaxKind::OpenDictToken;
        self.advance_by(2); // consume '<<'
        token_info.bytes = self.get_lexeme_bytes();
    }

    /// Scans the dictionary closing bracket `>>` as [`SyntaxKind::CloseDictToken`].
    ///
    /// See: ISO 32000-2:2020, §7.3.7 Dictionary objects.
    fn scan_dict_close(&mut self, token_info: &mut TokenInfo<'source>) {
        token_info.kind = SyntaxKind::CloseDictToken;
        self.advance_by(2); // consume '>>'
        token_info.bytes = self.get_lexeme_bytes();
    }

    /// Scans raw stream data until the `endstream` keyword is encountered.
    ///
    /// Consumes all bytes as raw stream data until it finds the `endstream` keyword.
    /// The `endstream` keyword itself is not consumed - it will be scanned as a separate token.
    ///
    /// Stream data can contain any bytes and is not interpreted as PDF objects during lexing.
    /// The actual decoding and filtering of stream data is handled in semantic analysis.
    ///
    /// See: ISO 32000-2:2020, §7.3.8 Stream objects.
    fn scan_raw_stream_data(&mut self, token_info: &mut TokenInfo<'source>) {
        token_info.kind = SyntaxKind::RawStreamDataToken;
        // There should be an end-of-line marker after the data and before endstream
        // This marker shall not be included in the stream length.
        // See: https://github.com/pdf-association/pdf-issues/issues/572
        self.advance_until(&[b"\nendstream", b"\r\nendstream", b"endstream"]);
        token_info.bytes = self.get_lexeme_bytes();
        self.is_raw_stream = false; // exit raw stream mode after scanning
    }

    /// Scans unknown/unsupported characters as a [`SyntaxKind::BadToken`].
    ///
    /// Consumes characters greedily until a delimiter, whitespace, or EOF is encountered.
    /// This ensures that sequences like `@#$` are captured as a single bad token for better
    /// error reporting and recovery.
    fn scan_bad_token(&mut self, token_info: &mut TokenInfo<'source>) {
        token_info.kind = SyntaxKind::BadToken;
        self.advance(); // consume the first bad character

        while let Some(byte) = self.peek() {
            // Stop at whitespace or delimiters
            if is_whitespace(byte, true) || is_delimiter(byte, false) {
                break;
            }
            self.advance(); // consume the bad character
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
fn is_whitespace(byte: u8, include_eol: bool) -> bool {
    match byte {
        b'\0' | b'\t' | b'\x0C' | b' ' => true,
        b'\r' | b'\n' if include_eol => true,
        _ => false,
    }
}

/// Returns true when the byte is a hexadecimal digit (`0-9`, `A-F`, `a-f`).
#[inline]
fn is_hexcode(byte: u8) -> bool {
    matches!(byte, b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f')
}

/// Returns true for regular name characters according to ISO 32000-2:2020 §7.3.5 Name objects.
///
/// Regular characters are bytes in the range `!` to `~` (33–126) **excluding**:
/// - the number sign (`#`, 0x23), which marks hexadecimal escapes in names
/// - PDF delimiter characters (see [`is_delimiter`]), which always terminate a name
///
/// See: ISO 32000-2:2020, §7.3.5 Name objects.
#[inline]
fn is_regular_name_char(byte: u8) -> bool {
    matches!(byte, b'!'..=b'~') && byte != b'#' && !is_delimiter(byte, false)
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
fn is_delimiter(byte: u8, include_postscript_delimiters: bool) -> bool {
    match byte {
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'/' | b'%' => true,
        b'{' | b'}' if include_postscript_delimiters => true,
        _ => false,
    }
}
