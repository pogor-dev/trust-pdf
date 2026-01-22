use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::arc::{Arc, HeaderSlice, ThinArc};
use countme::Count;

/// Severity level of a diagnostic message.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
}

#[derive(PartialEq, Eq, Hash)]
struct GreenDiagnosticHead {
    code: u16,
    severity: DiagnosticSeverity,
    _c: Count<GreenDiagnostic>,
}

type Repr = HeaderSlice<GreenDiagnosticHead, [u8]>;
type ReprThin = HeaderSlice<GreenDiagnosticHead, [u8; 0]>;

/// Unsized diagnostic data stored inline with message bytes.
#[repr(transparent)]
pub struct GreenDiagnosticData {
    data: ReprThin,
}

impl GreenDiagnosticData {
    /// Diagnostic code identifier.
    #[inline]
    pub fn code(&self) -> u16 {
        self.data.header.code
    }

    /// Severity level of this diagnostic.
    #[inline]
    pub fn severity(&self) -> DiagnosticSeverity {
        self.data.header.severity
    }

    /// Message text as UTF-8 string.
    #[inline]
    pub fn message(&self) -> &str {
        // SAFETY: We only accept valid UTF-8 in `new()`
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
        self.code() == other.code() && self.severity() == other.severity() && self.message() == other.message()
    }
}

impl ToOwned for GreenDiagnosticData {
    type Owned = GreenDiagnostic;

    #[inline]
    fn to_owned(&self) -> GreenDiagnostic {
        let green = unsafe { GreenDiagnostic::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenDiagnostic::clone(&green)
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
            .field("code", &self.code())
            .field("severity", &self.severity())
            .field("message", &self.message())
            .finish()
    }
}

/// Diagnostic node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenDiagnostic {
    ptr: ThinArc<GreenDiagnosticHead, u8>,
}

impl Borrow<GreenDiagnosticData> for GreenDiagnostic {
    #[inline]
    fn borrow(&self) -> &GreenDiagnosticData {
        self
    }
}

impl fmt::Display for GreenDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenDiagnosticData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Debug for GreenDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenDiagnosticData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenDiagnostic {
    /// Creates new diagnostic with given code, severity, and message.
    #[inline]
    pub fn new(code: u16, severity: DiagnosticSeverity, message: &str) -> GreenDiagnostic {
        let bytes = message.as_bytes();
        assert!(bytes.len() <= u32::MAX as usize, "diagnostic message length exceeds u32::MAX");
        let head = GreenDiagnosticHead {
            code,
            severity,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, bytes.iter().copied());
        GreenDiagnostic { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenDiagnostic) -> ptr::NonNull<GreenDiagnosticData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenDiagnosticData = &green;
        ptr::NonNull::from(green)
    }

    /// # Safety
    ///
    /// Reconstructs a `GreenDiagnostic` from a raw pointer.
    ///
    /// - The raw pointer must be valid and correctly aligned for `ReprThin`.
    /// - The lifetime of the raw pointer must outlive the created `Arc`.
    /// - The transmute operation requires memory layout compatibility between `Arc<ReprThin>` and `ThinArc<GreenDiagnosticHead, u8>`.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenDiagnosticData>) -> GreenDiagnostic {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenDiagnosticHead, u8>>(arc)
        };
        GreenDiagnostic { ptr: arc }
    }
}

impl ops::Deref for GreenDiagnostic {
    type Target = GreenDiagnosticData;

    #[inline]
    fn deref(&self) -> &GreenDiagnosticData {
        unsafe {
            let repr: &Repr = &*self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenDiagnosticData>(repr)
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_green_diagnostic_head_memory_layout() {
        // GreenDiagnosticHead: code (2 bytes) + severity (1 byte) + padding + _c (0 bytes)
        assert_eq!(std::mem::size_of::<GreenDiagnosticHead>(), 4);
        assert_eq!(std::mem::align_of::<GreenDiagnosticHead>(), 2);
    }

    #[test]
    fn test_green_diagnostic_data_memory_layout() {
        // GreenDiagnosticData is transparent wrapper around ReprThin
        assert!(std::mem::size_of::<GreenDiagnosticData>() >= std::mem::size_of::<GreenDiagnosticHead>());
    }

    #[test]
    fn test_green_diagnostic_memory_layout() {
        // GreenDiagnostic wraps ThinArc pointer (8 bytes on 64-bit)
        assert_eq!(std::mem::size_of::<GreenDiagnostic>(), std::mem::size_of::<usize>());
        assert_eq!(std::mem::align_of::<GreenDiagnostic>(), std::mem::align_of::<usize>());
    }
}

#[cfg(test)]
mod green_diagnostic_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_diagnostic() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Unexpected token");
        assert_eq!(diag.code(), 1);
        assert_eq!(diag.severity(), DiagnosticSeverity::Error);
        assert_eq!(diag.message(), "Unexpected token");
    }

    #[test]
    fn test_code() {
        let diag = GreenDiagnostic::new(42, DiagnosticSeverity::Warning, "Warning message");
        assert_eq!(diag.code(), 42);
    }

    #[test]
    fn test_severity() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Info, "Info message");
        assert_eq!(diag.severity(), DiagnosticSeverity::Info);
    }

    #[test]
    fn test_message() {
        let msg = "Expected closing bracket";
        let diag = GreenDiagnostic::new(10, DiagnosticSeverity::Error, msg);
        assert_eq!(diag.message(), msg);
    }

    #[test]
    fn test_message_len() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Hello");
        assert_eq!(diag.message_len(), 5);
    }

    #[test]
    fn test_eq_when_all_fields_match_expect_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        assert_eq!(diag1, diag2);
    }

    #[test]
    fn test_eq_when_different_code_expect_not_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Error, "Error");
        assert_ne!(diag1, diag2);
    }

    #[test]
    fn test_eq_when_different_severity_expect_not_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Message");
        let diag2 = GreenDiagnostic::new(1, DiagnosticSeverity::Warning, "Message");
        assert_ne!(diag1, diag2);
    }

    #[test]
    fn test_eq_when_different_message_expect_not_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error 1");
        let diag2 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error 2");
        assert_ne!(diag1, diag2);
    }

    #[test]
    fn test_clone() {
        let diag1 = GreenDiagnostic::new(5, DiagnosticSeverity::Warning, "Clone test");
        let diag2 = diag1.clone();
        assert_eq!(diag1, diag2);
        assert_eq!(diag2.code(), 5);
        assert_eq!(diag2.severity(), DiagnosticSeverity::Warning);
        assert_eq!(diag2.message(), "Clone test");
    }

    #[test]
    fn test_display() {
        let diag = GreenDiagnostic::new(42, DiagnosticSeverity::Error, "Test error");
        assert_eq!(diag.to_string(), "PDF0042: Test error");
    }

    #[test]
    fn test_debug() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Debug test");
        let debug_str = format!("{:?}", diag);
        assert_eq!(debug_str, "GreenDiagnostic { code: 1, severity: Error, message: \"Debug test\" }");
    }

    #[test]
    fn test_empty_message() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "");
        assert_eq!(diag.message(), "");
        assert_eq!(diag.message_len(), 0);
    }

    #[test]
    fn test_long_message() {
        let msg = "This is a very long diagnostic message that spans multiple lines and contains a lot of text to test the handling of longer messages in the diagnostic system.";
        let diag = GreenDiagnostic::new(100, DiagnosticSeverity::Warning, msg);
        assert_eq!(diag.message(), msg);
        assert_eq!(diag.message_len(), msg.len() as u32);
    }

    #[test]
    fn test_unicode_message() {
        let msg = "Invalid character: 不合法";
        let diag = GreenDiagnostic::new(50, DiagnosticSeverity::Error, msg);
        assert_eq!(diag.message(), msg);
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let diag = GreenDiagnostic::new(10, DiagnosticSeverity::Warning, "Raw test");
        let ptr = GreenDiagnostic::into_raw(diag.clone());
        let reconstructed = unsafe { GreenDiagnostic::from_raw(ptr) };
        assert_eq!(diag, reconstructed);
    }

    #[test]
    fn test_borrow() {
        let diag = GreenDiagnostic::new(20, DiagnosticSeverity::Info, "Borrow test");
        let borrowed: &GreenDiagnosticData = diag.borrow();
        assert_eq!(borrowed.code(), 20);
        assert_eq!(borrowed.severity(), DiagnosticSeverity::Info);
        assert_eq!(borrowed.message(), "Borrow test");
    }

    #[test]
    fn test_all_severity_levels() {
        let info = GreenDiagnostic::new(1, DiagnosticSeverity::Info, "Info");
        let warning = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning");
        let error = GreenDiagnostic::new(3, DiagnosticSeverity::Error, "Error");

        assert_eq!(info.severity(), DiagnosticSeverity::Info);
        assert_eq!(warning.severity(), DiagnosticSeverity::Warning);
        assert_eq!(error.severity(), DiagnosticSeverity::Error);
    }
}

#[cfg(test)]
mod green_diagnostic_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned() {
        let diag = GreenDiagnostic::new(15, DiagnosticSeverity::Error, "Owned test");
        let data: &GreenDiagnosticData = &*diag;
        let owned = data.to_owned();
        assert_eq!(diag, owned);
    }

    #[test]
    fn test_eq_when_all_fields_match_expect_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Test");
        let diag2 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Test");
        let data1: &GreenDiagnosticData = &*diag1;
        let data2: &GreenDiagnosticData = &*diag2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_code_expect_not_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Test");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Error, "Test");
        let data1: &GreenDiagnosticData = &*diag1;
        let data2: &GreenDiagnosticData = &*diag2;
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_severity_expect_not_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Test");
        let diag2 = GreenDiagnostic::new(1, DiagnosticSeverity::Warning, "Test");
        let data1: &GreenDiagnosticData = &*diag1;
        let data2: &GreenDiagnosticData = &*diag2;
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_message_expect_not_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Test 1");
        let diag2 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Test 2");
        let data1: &GreenDiagnosticData = &*diag1;
        let data2: &GreenDiagnosticData = &*diag2;
        assert_ne!(data1, data2);
    }
}
