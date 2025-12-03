use syntax::GreenToken;

pub struct Lexer<'source> {
    source: &'source [u8],
    len_remaining: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source [u8]) -> Self {
        Self {
            source,
            len_remaining: source.len(),
        }
    }

    pub fn next_token(&mut self) -> Option<GreenToken> {
        let Some(first_byte) = self.advance() else {
            return GreenToken::new(SyntaxKind::Eof, 0);
        };

        loop {
            if self.position >= self.source.len() {
                return None;
            }

            let byte = self.source[self.position];

            match byte {
                b @ b'0'..=b'9' => {
                    return None; // Placeholder for number tokenization
                }
                _ => {
                    panic!("Unrecognized token starting with byte: {}", byte);
                }
            }

            self.position += 1;
        }
    }

    /// Advance the cursor by one byte and return the byte at the new position.
    fn advance(&mut self) -> Option<&u8> {
        self.source.iter().next()
    }

    /// Peek at the first byte without advancing the cursor.
    fn peek_first(&self) -> Option<&u8> {
        // `.next()` optimizes better than `.nth(0)`
        self.source.iter().next()
    }

    /// Peek at the second byte without advancing the cursor.
    fn peek_second(&self) -> Option<&u8> {
        // `.next()` optimizes better than `.nth(1)`
        let mut iter = self.source.iter();
        iter.next();
        iter.next()
    }

    /// Peek at the third byte without advancing the cursor.
    fn peek_third(&self) -> Option<&u8> {
        // `.next()` optimizes better than `.nth(2)`
        let mut iter = self.source.iter();
        iter.next();
        iter.next();
        iter.next()
    }

    /// Checks if there is nothing more to consume.
    fn is_eof(&self) -> bool {
        self.source.is_empty()
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
}
