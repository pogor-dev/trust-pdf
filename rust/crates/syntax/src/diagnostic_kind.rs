/// Enumerates diagnostic categories emitted by the lexer/parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum DiagnosticKind {
    Unknown = 0,
    UnbalancedStringLiteral = 1,
}

impl DiagnosticKind {
    /// Human-readable label for this diagnostic kind, used in emitted messages.
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticKind::UnbalancedStringLiteral => "Unbalanced string literal",
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
