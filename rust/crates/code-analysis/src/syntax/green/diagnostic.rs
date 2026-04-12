use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::arc::{Arc, HeaderSlice, ThinArc};
use countme::Count;

use crate::DiagnosticKind;

type Repr = HeaderSlice<GreenDiagnosticHead, [u8]>;
type ReprThin = HeaderSlice<GreenDiagnosticHead, [u8; 0]>;

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

impl ToOwned for GreenDiagnosticData {
    type Owned = GreenDiagnostic;

    #[inline]
    fn to_owned(&self) -> GreenDiagnostic {
        let green = unsafe { GreenDiagnostic::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenDiagnostic::clone(&green)
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
    /// Consumes the handle and returns a raw non-null pointer to the data.
    #[inline]
    pub(crate) fn into_raw(this: GreenDiagnostic) -> ptr::NonNull<GreenDiagnosticData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenDiagnosticData = &green;
        ptr::NonNull::from(green)
    }

    /// Reconstructs an owned handle from a raw pointer.
    ///
    /// # Safety
    ///
    /// The raw pointer must have been produced by `into_raw` and not yet
    /// consumed. The underlying `Arc` allocation must still be live.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenDiagnosticData>) -> GreenDiagnostic {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenDiagnosticHead, u8>>(arc)
        };
        GreenDiagnostic { ptr: arc }
    }

    #[inline]
    pub(crate) fn diagnostics(&self) -> Option<Vec<crate::GreenDiagnostic>> {
        use crate::syntax::green::diagnostics;

        diagnostics::get_diagnostics(self.diagnostics_key())
    }

    #[inline]
    fn clear_diagnostics(&self) {
        use crate::syntax::green::diagnostics;

        diagnostics::remove_diagnostics(self.diagnostics_key());
    }

    #[inline]
    fn diagnostics_key(&self) -> usize {
        let data: &GreenDiagnosticData = self;
        data as *const GreenDiagnosticData as usize
    }
}

impl Drop for GreenDiagnostic {
    #[inline]
    fn drop(&mut self) {
        // Clear side-table diagnostics only for the final owner.
        // This avoids duplicate removals while cloned green handles are
        // still alive and keeps diagnostics lifetime tied to green data.
        let should_clear = self.ptr.with_arc(|arc| arc.is_unique());
        if should_clear {
            self.clear_diagnostics();
        }
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
