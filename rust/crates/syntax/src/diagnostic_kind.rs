/// Diagnostic categories emitted during PDF lexical and syntactic analysis.
///
/// Each variant represents a specific error or warning condition encountered
/// while processing PDF source code. The numeric discriminants are stable
/// and used for serialization/deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum DiagnosticKind {
    /// Unknown or unclassified diagnostic.
    Unknown = 0,
    /// Unbalanced parentheses in a literal string (ISO 32000-2:2020 §7.3.4.2).
    UnbalancedStringLiteral = 1,
    /// Invalid escape sequence in a literal string.
    InvalidEscapeInStringLiteral = 2,
    /// Invalid character encountered in hexadecimal string (ISO 32000-2:2020 §7.3.4.3).
    InvalidCharacterInHexString = 3,
    /// Missing closing angle bracket in hexadecimal string.
    UnbalancedHexString = 4,
    /// Malformed hex escape sequence in name object (ISO 32000-2:2020 §7.3.5).
    InvalidHexEscapeInName = 5,
    /// Non-regular character in name that requires hex escaping.
    InvalidNonRegularCharacterInName = 6,
    /// Missing required whitespace between tokens (SafeDocs PDF Compacted Syntax Matrix).
    MissingWhitespaceBeforeToken = 7,
}

impl DiagnosticKind {
    /// Returns a human-readable description of the diagnostic.
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticKind::Unknown => "Unknown diagnostic",
            DiagnosticKind::UnbalancedStringLiteral => "Unbalanced string literal",
            DiagnosticKind::InvalidEscapeInStringLiteral => "Invalid escape sequence in string literal",
            DiagnosticKind::InvalidCharacterInHexString => "Invalid character in hex string",
            DiagnosticKind::UnbalancedHexString => "Unbalanced hex string",
            DiagnosticKind::InvalidHexEscapeInName => "Invalid hex escape in name",
            DiagnosticKind::InvalidNonRegularCharacterInName => "Invalid character in name (needs hex escape)",
            DiagnosticKind::MissingWhitespaceBeforeToken => "Missing whitespace before token",
        }
    }
}

impl From<u16> for DiagnosticKind {
    /// Converts a numeric code to its corresponding diagnostic kind.
    #[inline]
    fn from(code: u16) -> Self {
        match code {
            1 => DiagnosticKind::UnbalancedStringLiteral,
            2 => DiagnosticKind::InvalidEscapeInStringLiteral,
            3 => DiagnosticKind::InvalidCharacterInHexString,
            4 => DiagnosticKind::UnbalancedHexString,
            5 => DiagnosticKind::InvalidHexEscapeInName,
            6 => DiagnosticKind::InvalidNonRegularCharacterInName,
            7 => DiagnosticKind::MissingWhitespaceBeforeToken,
            _ => DiagnosticKind::Unknown,
        }
    }
}

impl From<DiagnosticKind> for u16 {
    /// Converts a diagnostic kind to its numeric code.
    #[inline]
    fn from(kind: DiagnosticKind) -> u16 {
        kind as u16
    }
}
