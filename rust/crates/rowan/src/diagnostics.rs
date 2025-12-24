use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticInfo {
    pub code: u16,
    pub message: &'static str,
    pub severity: DiagnosticSeverity,
}

impl DiagnosticInfo {
    pub fn new(code: u16, message: &'static str, severity: DiagnosticSeverity) -> Self {
        Self { code, message, severity }
    }
}

impl fmt::Display for DiagnosticInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Example: "PDF0001: Unexpected token"
        write!(f, "PDF{:04}: {}", self.code as u32, self.message)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}
