#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum DiagnosticKind {
    UnbalancedStringLiteral = 1,
}

impl DiagnosticKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticKind::UnbalancedStringLiteral => "Unbalanced string literal",
        }
    }
}

impl From<u16> for DiagnosticKind {
    #[inline]
    fn from(d: u16) -> DiagnosticKind {
        unsafe { std::mem::transmute::<u16, DiagnosticKind>(d) }
    }
}

impl From<DiagnosticKind> for u16 {
    #[inline]
    fn from(k: DiagnosticKind) -> u16 {
        k as u16
    }
}
