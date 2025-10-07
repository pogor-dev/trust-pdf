use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticInfo {
    pub code: ErrorCode,
    pub message: &'static str,
    pub severity: DiagnosticSeverity,
    pub offset: usize,
    pub length: usize,
}

impl DiagnosticInfo {
    pub fn new_with_offset_and_length(code: ErrorCode, offset: usize, length: usize) -> Self {
        Self {
            code: code.clone(),
            message: get_error_message(&code),
            severity: get_severity(&code),
            offset,
            length,
        }
    }
}

impl fmt::Display for DiagnosticInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Example: "PDF0001: Unexpected token"
        write!(f, "PDF{:04}: {}", self.code as u32, self.message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    UnexpectedToken = 1,
}

pub fn get_error_message(code: &ErrorCode) -> &'static str {
    match code {
        ErrorCode::UnexpectedToken => "Unexpected token",
    }
}

pub fn get_severity(code: &ErrorCode) -> DiagnosticSeverity {
    match code {
        ErrorCode::UnexpectedToken => DiagnosticSeverity::Error,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}
