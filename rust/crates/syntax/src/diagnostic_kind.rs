/// Enumerates diagnostic categories emitted by the lexer/parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum DiagnosticKind {
    Unknown = 0,
    UnbalancedStringLiteral = 1,
    InvalidEscapeInStringLiteral = 2,
    InvalidCharacterInHexString = 3,
    UnbalancedHexString = 4,
    InvalidHexEscapeInName = 5,
    InvalidNonRegularCharacterInName = 6,
    MissingWhitespaceBeforeToken = 7,
}

impl DiagnosticKind {
    /// Human-readable label for this diagnostic kind, used in emitted messages.
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticKind::UnbalancedStringLiteral => "Unbalanced string literal",
            DiagnosticKind::InvalidEscapeInStringLiteral => "Invalid escape sequence in string literal",
            DiagnosticKind::InvalidCharacterInHexString => "Invalid character in hex string",
            DiagnosticKind::UnbalancedHexString => "Unbalanced hex string",
            DiagnosticKind::InvalidHexEscapeInName => "Invalid hex escape in name",
            DiagnosticKind::InvalidNonRegularCharacterInName => "Invalid character in name. Non-regular characters must be hex-escaped using #xx notation",
            DiagnosticKind::MissingWhitespaceBeforeToken => "Whitespace required before this token (SafeDocs PDF Compacted Syntax Matrix violation)",
            DiagnosticKind::Unknown => "Unknown diagnostic",
        }
    }
}

impl From<u16> for DiagnosticKind {
    /// Converts a serialized discriminant into a diagnostic kind, or returns error for unknown values.
    #[inline]
    fn from(d: u16) -> DiagnosticKind {
        match d {
            1 => DiagnosticKind::UnbalancedStringLiteral,
            2 => DiagnosticKind::InvalidEscapeInStringLiteral,
            3 => DiagnosticKind::InvalidCharacterInHexString,
            4 => DiagnosticKind::UnbalancedHexString,
            5 => DiagnosticKind::InvalidHexEscapeInName,
            6 => DiagnosticKind::InvalidNonRegularCharacterInName,
            7 => DiagnosticKind::MissingWhitespaceBeforeToken,
            _ => DiagnosticKind::Unknown, // Default to unknown diagnostic type
        }
    }
}

impl From<DiagnosticKind> for u16 {
    /// Serializes a diagnostic kind to its numeric discriminant for storage or transport.
    #[inline]
    fn from(k: DiagnosticKind) -> u16 {
        k as u16
    }
}
