use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::arc::{Arc, HeaderSlice, ThinArc};
use countme::Count;

use crate::DiagnosticKind;

/// Severity level of a diagnostic message.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(crate) enum DiagnosticSeverity {
    Info = 1,
    Warning = 2,
    Error = 3,
}

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenDiagnosticHead {
    kind: DiagnosticKind,         // 2 bytes (`repr(u16)`)
    severity: DiagnosticSeverity, // 1 byte (`repr(u8)`)
    _c: Count<GreenDiagnostic>,   // 0 bytes
}

/// Unsized diagnostic data stored inline with message bytes.
#[repr(transparent)]
pub(crate) struct GreenDiagnosticData {
    data: ReprThin,
}

impl GreenDiagnosticData {
    /// Diagnostic kind identifier.
    #[inline]
    pub fn kind(&self) -> DiagnosticKind {
        self.data.header.kind
    }

    /// Numeric diagnostic code.
    #[inline]
    pub fn code(&self) -> u16 {
        self.kind().into()
    }

    /// Severity level of this diagnostic.
    #[inline]
    pub fn severity(&self) -> DiagnosticSeverity {
        self.data.header.severity
    }

    /// Message text as UTF-8 string.
    #[inline]
    pub fn message(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.data.slice()) }
    }

    /// Returns the length of the message.
    #[inline]
    pub fn message_len(&self) -> u32 {
        self.message().len() as u32
    }
}

impl PartialEq for GreenDiagnosticData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.severity() == other.severity() && self.message() == other.message()
    }
}

impl fmt::Display for GreenDiagnosticData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PDF{:04}: {}", self.code(), self.message())
    }
}

impl fmt::Debug for GreenDiagnosticData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenDiagnostic")
            .field("kind", &self.kind())
            .field("code", &self.code())
            .field("severity", &self.severity())
            .field("message", &self.message())
            .finish()
    }
}

/// Diagnostic node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenDiagnostic {
    ptr: ThinArc<GreenDiagnosticHead, u8>,
}

impl GreenDiagnostic {
    /// Creates new diagnostic with given kind, severity, and message.
    #[inline]
    pub fn new(kind: DiagnosticKind, severity: DiagnosticSeverity, message: &str) -> GreenDiagnostic {
        let bytes = message.as_bytes();
        assert!(bytes.len() <= u32::MAX as usize, "diagnostic message length exceeds u32::MAX");

        let head = GreenDiagnosticHead {
            kind,
            severity,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, bytes.iter().copied());
        GreenDiagnostic { ptr }
    }
}

impl_green_boilerplate!(GreenDiagnosticHead, GreenDiagnosticData, GreenDiagnostic, u8);

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_green_diagnostic_head_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenDiagnosticHead>(), 4);
        assert_eq!(std::mem::align_of::<GreenDiagnosticHead>(), 2);
    }

    #[test]
    fn test_green_diagnostic_data_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenDiagnosticData>(), 16);
            assert_eq!(std::mem::align_of::<GreenDiagnosticData>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenDiagnosticData>(), 8);
            assert_eq!(std::mem::align_of::<GreenDiagnosticData>(), 4);
        }
    }

    #[test]
    fn test_green_diagnostic_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenDiagnostic>(), 8);
            assert_eq!(std::mem::align_of::<GreenDiagnostic>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenDiagnostic>(), 4);
            assert_eq!(std::mem::align_of::<GreenDiagnostic>(), 4);
        }
    }
}

#[cfg(test)]
mod green_diagnostic_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_diagnostic() {
        let diag = GreenDiagnostic::new(DiagnosticKind::UnbalancedStringLiteral, DiagnosticSeverity::Error, "Unexpected token");
        assert_eq!(diag.kind(), DiagnosticKind::UnbalancedStringLiteral);
        assert_eq!(diag.code(), 1);
        assert_eq!(diag.severity(), DiagnosticSeverity::Error);
        assert_eq!(diag.message(), "Unexpected token");
    }

    #[test]
    fn test_message_len() {
        let diag = GreenDiagnostic::new(DiagnosticKind::UnbalancedStringLiteral, DiagnosticSeverity::Error, "Hello");
        assert_eq!(diag.message_len(), 5);
    }

    #[test]
    fn test_display() {
        let diag = GreenDiagnostic::new(DiagnosticKind::MissingWhitespaceBeforeToken, DiagnosticSeverity::Error, "Test error");
        assert_eq!(diag.to_string(), "PDF0007: Test error");
    }

    #[test]
    fn test_into_raw_and_from_raw_when_roundtrip_expect_equal() {
        let diag = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "Raw test");
        let ptr = GreenDiagnostic::into_raw(diag.clone());
        let reconstructed = unsafe { GreenDiagnostic::from_raw(ptr) };
        assert_eq!(diag, reconstructed);
    }

    #[test]
    fn test_borrow_when_called_expect_data_access() {
        let diag = GreenDiagnostic::new(DiagnosticKind::InvalidHexEscapeInName, DiagnosticSeverity::Info, "Borrow test");
        let borrowed: &GreenDiagnosticData = diag.borrow();
        assert_eq!(borrowed.kind(), DiagnosticKind::InvalidHexEscapeInName);
        assert_eq!(borrowed.code(), 5);
        assert_eq!(borrowed.severity(), DiagnosticSeverity::Info);
        assert_eq!(borrowed.message(), "Borrow test");
    }
}
