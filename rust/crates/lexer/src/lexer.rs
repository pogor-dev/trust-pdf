use std::ops::Range;

use syntax::{GreenCache, GreenNodeBuilder, GreenToken, GreenTriviaInTree, GreenTriviaListInTree, NodeOrToken, SyntaxKind};

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

    /// Returns the next token from the source.
    /// If the end of the source is reached, returns an EOF token.
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
            b'0'..=b'9' | b'+' | b'-' | b'.' => {
                // TODO: Architectural limits on numeric literals, I think this should be handled in Semantic analysis phase
                self.scan_numeric_literal(token_info);
            }
            _ => {}
        };

        self.stop_lexeme();
    }

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
                _ => break,
            }
        }
        self.cache.trivia_list(&trivia).1
    }

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

        let spaces = &self.source[pos..self.position];
        self.cache.trivia(SyntaxKind::EndOfLineTrivia.into(), spaces).1
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
    use pretty_assertions::assert_eq;
    use syntax::{GreenNode, GreenNodeBuilder, GreenToken, NodeOrToken, SyntaxKind, tree};

    #[test]
    fn test_numeric_literal_123() {
        let mut lexer = Lexer::new(b"123");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"123");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_43445() {
        let mut lexer = Lexer::new(b"43445");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"43445");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_plus_17() {
        let mut lexer = Lexer::new(b"+17");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"+17");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_minus_98() {
        let mut lexer = Lexer::new(b"-98");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"-98");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_0() {
        let mut lexer = Lexer::new(b"0");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"0");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_00987() {
        let mut lexer = Lexer::new(b"00987");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"00987");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_34_5() {
        let mut lexer = Lexer::new(b"34.5");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"34.5");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_minus_3_62() {
        let mut lexer = Lexer::new(b"-3.62");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"-3.62");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_plus_123_6() {
        let mut lexer = Lexer::new(b"+123.6");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"+123.6");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_4_() {
        let mut lexer = Lexer::new(b"4.");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"4.");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_numeric_literal_minus__002() {
        let mut lexer = Lexer::new(b"-.002");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"-.002");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_literal_009_87() {
        let mut lexer = Lexer::new(b"009.87");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"009.87");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_numeric_literal__3_4() {
        let mut lexer = Lexer::new(b".34");
        let token = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b".34");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_plus_plus_invalid() {
        let mut lexer = Lexer::new(b"++");
        let token: GreenToken = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::BadToken, b"++");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_numeric_plus_minus_345_minus_36_invalid() {
        let mut lexer = Lexer::new(b"+345-36");
        let token: GreenToken = lexer.next_token();
        assert_numeric_literal_token(&token, SyntaxKind::BadToken, b"+345-36");
        assert_eof_token(&lexer.next_token());
    }

    #[test]
    fn test_trivia_single_space() {
        let mut lexer = Lexer::new(b"009 345");
        let actual_node = generate_node_from_lexer(&mut lexer);

        let expected_node = tree! {
            SyntaxKind::LexerNode.into() => {
                (SyntaxKind::NumericLiteralToken.into()) => {
                    text(b"009"),
                    trivia(SyntaxKind::WhitespaceTrivia.into(), b" "),
                },
                (SyntaxKind::NumericLiteralToken.into(), b"345")
            }
        };

        assert_nodes_equal(&expected_node, &actual_node);
    }

    #[test]
    fn test_trivia_multiple_spaces() {
        let mut lexer = Lexer::new(b"009       345");
        let actual_node = generate_node_from_lexer(&mut lexer);

        let expected_node = tree! {
            SyntaxKind::LexerNode.into() => {
                (SyntaxKind::NumericLiteralToken.into()) => {
                    text(b"009"),
                    trivia(SyntaxKind::WhitespaceTrivia.into(), b"       "),
                },
                (SyntaxKind::NumericLiteralToken.into(), b"345")
            }
        };

        assert_nodes_equal(&expected_node, &actual_node);
    }

    #[test]
    fn test_trivia_different_whitespaces() {
        let mut lexer = Lexer::new(b"\r\0009 \t \x0C\r\n345\0\t\x0C \n");
        let actual_node = generate_node_from_lexer(&mut lexer);

        let expected_node = tree! {
            SyntaxKind::LexerNode.into() => {
                (SyntaxKind::NumericLiteralToken.into()) => {
                    trivia(SyntaxKind::EndOfLineTrivia.into(), b"\r"),
                    trivia(SyntaxKind::WhitespaceTrivia.into(), b"\0"),
                    text(b"009"),
                    trivia(SyntaxKind::WhitespaceTrivia.into(), b" \t \x0C"),
                    trivia(SyntaxKind::EndOfLineTrivia.into(), b"\r\n"),
                },
                (SyntaxKind::NumericLiteralToken.into()) => {
                    text(b"345"),
                    trivia(SyntaxKind::WhitespaceTrivia.into(), b"\0\t\x0C "),
                    trivia(SyntaxKind::EndOfLineTrivia.into(), b"\n"),
                }
            }
        };

        assert_nodes_equal(&expected_node, &actual_node);
    }

    fn assert_nodes_equal(expected: &GreenNode, actual: &GreenNode) {
        let actual_children: Vec<GreenToken> = actual
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Token(token) => Some(token),
                _ => None,
            })
            .collect();

        let expected_children: Vec<GreenToken> = expected
            .children()
            .filter_map(|child| match child {
                NodeOrToken::Token(token) => Some(token),
                _ => None,
            })
            .collect();

        assert_eq!(actual_children, expected_children);
    }

    fn generate_node_from_lexer(lexer: &mut Lexer) -> GreenNode {
        let tokens: Vec<_> = std::iter::from_fn(|| Some(lexer.next_token()))
            .take_while(|t| t.kind() != SyntaxKind::EndOfFileToken.into())
            .collect();

        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind::LexerNode.into());
        tokens.iter().for_each(|token| {
            builder.token(token.kind(), &token.bytes(), token.leading_trivia().pieces(), token.trailing_trivia().pieces());
        });
        builder.finish_node();
        builder.finish()
    }

    fn assert_numeric_literal_token(token: &GreenToken, expected_kind: SyntaxKind, expected_bytes: &[u8]) {
        let actual_node = generate_lexer_node_tree(token);
        let expected_node = tree! {
            SyntaxKind::LexerNode.into() => {
                (expected_kind.into(), expected_bytes)
            }
        };

        let actual_token = actual_node.children().next().unwrap();
        let expected_token = expected_node.children().next().unwrap();
        assert_eq!(format!("{:?}", actual_token), format!("{:?}", expected_token));
        assert_eq!(actual_node, expected_node);
    }

    fn assert_eof_token(token: &GreenToken) {
        let actual_node = generate_lexer_node_tree(token);
        let expected_node = tree! {
            SyntaxKind::LexerNode.into() => {
                (SyntaxKind::EndOfFileToken.into(), b"")
            }
        };

        let actual_token = actual_node.children().next().unwrap();
        let expected_token = expected_node.children().next().unwrap();
        assert_eq!(format!("{:?}", actual_token), format!("{:?}", expected_token));
        assert_eq!(actual_node, expected_node);
    }

    fn generate_lexer_node_tree(token: &GreenToken) -> GreenNode {
        tree! {
            SyntaxKind::LexerNode.into() => {
                (token.kind(), &token.bytes())
            }
        }
    }
}
