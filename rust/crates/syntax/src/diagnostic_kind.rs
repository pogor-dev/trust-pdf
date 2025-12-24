#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
/// Enumerates diagnostic categories emitted by the lexer/parser.
pub enum DiagnosticKind {
    UnbalancedStringLiteral = 1,
}

impl DiagnosticKind {
    /// Human-readable label for this diagnostic kind, used in emitted messages.
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticKind::UnbalancedStringLiteral => "Unbalanced string literal",
        }
    }
}

impl From<u16> for DiagnosticKind {
    #[inline]
    /// Converts a serialized discriminant into a diagnostic kind; panics on unknown values.
    fn from(d: u16) -> DiagnosticKind {
        match d {
            1 => DiagnosticKind::UnbalancedStringLiteral,
            _ => panic!("invalid DiagnosticKind discriminant: {}", d),
        }
    }
}

impl From<DiagnosticKind> for u16 {
    #[inline]
    /// Serializes a diagnostic kind to its numeric discriminant for storage or transport.
    fn from(k: DiagnosticKind) -> u16 {
        k as u16
    }
}
