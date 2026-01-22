use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr, slice,
};

use crate::arc::{Arc, HeaderSlice, ThinArc};
use countme::Count;

use super::diagnostic::{DiagnosticSeverity, GreenDiagnostic};

#[derive(PartialEq, Eq, Hash)]
struct GreenDiagnosticsHead {
    count: u32,
    _c: Count<GreenDiagnostics>,
}

type Repr = HeaderSlice<GreenDiagnosticsHead, [GreenDiagnostic]>;
type ReprThin = HeaderSlice<GreenDiagnosticsHead, [GreenDiagnostic; 0]>;

/// Unsized diagnostic list data storing diagnostics inline.
#[repr(transparent)]
pub struct GreenDiagnosticsData {
    data: ReprThin,
}

impl GreenDiagnosticsData {
    /// Number of diagnostics in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.header.count as usize
    }

    /// Returns true if the list contains no diagnostics.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a slice of all diagnostics in this list.
    #[inline]
    pub fn diagnostics(&self) -> &[GreenDiagnostic] {
        self.data.slice()
    }

    /// Returns the diagnostic at the given index, if it exists.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&GreenDiagnostic> {
        self.diagnostics().get(index)
    }

    /// Iterator over diagnostics in this list.
    #[inline]
    pub fn iter(&self) -> DiagnosticIter<'_> {
        DiagnosticIter {
            raw: self.diagnostics().iter(),
        }
    }

    /// Checks if any diagnostic in the list has Error severity.
    #[inline]
    pub fn has_errors(&self) -> bool {
        self.diagnostics().iter().any(|d| d.severity() == DiagnosticSeverity::Error)
    }

    /// Checks if any diagnostic in the list has Warning severity.
    #[inline]
    pub fn has_warnings(&self) -> bool {
        self.diagnostics().iter().any(|d| d.severity() == DiagnosticSeverity::Warning)
    }

    /// Count diagnostics by severity level.
    #[inline]
    pub fn count_by_severity(&self, severity: DiagnosticSeverity) -> usize {
        self.diagnostics().iter().filter(|d| d.severity() == severity).count()
    }
}

impl PartialEq for GreenDiagnosticsData {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.diagnostics() == other.diagnostics()
    }
}

impl ToOwned for GreenDiagnosticsData {
    type Owned = GreenDiagnostics;

    #[inline]
    fn to_owned(&self) -> GreenDiagnostics {
        let green = unsafe { GreenDiagnostics::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenDiagnostics::clone(&green)
    }
}

impl fmt::Debug for GreenDiagnosticsData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenDiagnostics")
            .field("count", &self.len())
            .field("diagnostics", &self.diagnostics())
            .finish()
    }
}

/// Owned diagnostic list in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenDiagnostics {
    ptr: ThinArc<GreenDiagnosticsHead, GreenDiagnostic>,
}

impl Borrow<GreenDiagnosticsData> for GreenDiagnostics {
    #[inline]
    fn borrow(&self) -> &GreenDiagnosticsData {
        self
    }
}

impl fmt::Debug for GreenDiagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenDiagnosticsData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenDiagnostics {
    /// Creates new diagnostic list from a slice of diagnostics.
    #[inline]
    pub fn new(diagnostics: &[GreenDiagnostic]) -> GreenDiagnostics {
        assert!(diagnostics.len() <= u32::MAX as usize, "diagnostic list length exceeds u32::MAX");
        let head = GreenDiagnosticsHead {
            count: diagnostics.len() as u32,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, diagnostics.iter().cloned());
        GreenDiagnostics { ptr }
    }

    /// Creates an empty diagnostic list.
    #[inline]
    pub fn empty() -> GreenDiagnostics {
        Self::new(&[])
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenDiagnostics) -> ptr::NonNull<GreenDiagnosticsData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenDiagnosticsData = &green;
        ptr::NonNull::from(green)
    }

    /// # Safety
    ///
    /// Reconstructs a `GreenDiagnostics` from a raw pointer.
    ///
    /// - The raw pointer must be valid and correctly aligned for `ReprThin`.
    /// - The lifetime of the raw pointer must outlive the created `Arc`.
    /// - The transmute operation requires memory layout compatibility between `Arc<ReprThin>` and `ThinArc<GreenDiagnosticsHead, GreenDiagnostic>`.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenDiagnosticsData>) -> GreenDiagnostics {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenDiagnosticsHead, GreenDiagnostic>>(arc)
        };
        GreenDiagnostics { ptr: arc }
    }
}

impl ops::Deref for GreenDiagnostics {
    type Target = GreenDiagnosticsData;

    #[inline]
    fn deref(&self) -> &GreenDiagnosticsData {
        unsafe {
            let repr: &Repr = &*self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenDiagnosticsData>(repr)
        }
    }
}

/// Iterator over diagnostics in a diagnostic list.
#[derive(Debug, Clone)]
pub struct DiagnosticIter<'a> {
    raw: slice::Iter<'a, GreenDiagnostic>,
}

impl<'a> Iterator for DiagnosticIter<'a> {
    type Item = &'a GreenDiagnostic;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.raw.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth(n)
    }
}

impl<'a> DoubleEndedIterator for DiagnosticIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth_back(n)
    }
}

impl<'a> ExactSizeIterator for DiagnosticIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.raw.len()
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_green_diagnostic_list_head_memory_layout() {
        // GreenDiagnosticsHead: count (4 bytes) + _c (0 bytes)
        assert_eq!(std::mem::size_of::<GreenDiagnosticsHead>(), 4);
        assert_eq!(std::mem::align_of::<GreenDiagnosticsHead>(), 4);
    }

    #[test]
    fn test_green_diagnostic_list_memory_layout() {
        // GreenDiagnostics wraps ThinArc pointer (8 bytes on 64-bit)
        assert_eq!(std::mem::size_of::<GreenDiagnostics>(), std::mem::size_of::<usize>());
        assert_eq!(std::mem::align_of::<GreenDiagnostics>(), std::mem::align_of::<usize>());
    }
}

#[cfg(test)]
mod green_diagnostic_list_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_empty_list() {
        let list = GreenDiagnostics::new(&[]);
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn test_empty() {
        let list = GreenDiagnostics::empty();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn test_new_single_diagnostic() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diag.clone()]);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
        assert_eq!(list.get(0), Some(&diag));
    }

    #[test]
    fn test_new_multiple_diagnostics() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error 1");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning 1");
        let diag3 = GreenDiagnostic::new(3, DiagnosticSeverity::Info, "Info 1");
        let list = GreenDiagnostics::new(&[diag1.clone(), diag2.clone(), diag3.clone()]);

        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(&diag1));
        assert_eq!(list.get(1), Some(&diag2));
        assert_eq!(list.get(2), Some(&diag3));
    }

    #[test]
    fn test_get_out_of_bounds() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diag]);
        assert_eq!(list.get(1), None);
        assert_eq!(list.get(100), None);
    }

    #[test]
    fn test_diagnostics_slice() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diag1.clone(), diag2.clone()]);

        let diagnostics = list.diagnostics();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0], diag1);
        assert_eq!(diagnostics[1], diag2);
    }

    #[test]
    fn test_iter() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diag1.clone(), diag2.clone()]);

        let collected: Vec<_> = list.iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], &diag1);
        assert_eq!(collected[1], &diag2);
    }

    #[test]
    fn test_iter_double_ended() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diag1.clone(), diag2.clone()]);

        let mut iter = list.iter();
        assert_eq!(iter.next_back(), Some(&diag2));
        assert_eq!(iter.next(), Some(&diag1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_exact_size() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diag1, diag2]);

        let iter = list.iter();
        assert_eq!(iter.len(), 2);
    }

    #[test]
    fn test_has_errors_when_no_errors_expect_false() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Warning, "Warning");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Info, "Info");
        let list = GreenDiagnostics::new(&[diag1, diag2]);
        assert!(!list.has_errors());
    }

    #[test]
    fn test_has_errors_when_has_errors_expect_true() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diag1, diag2]);
        assert!(list.has_errors());
    }

    #[test]
    fn test_has_warnings_when_no_warnings_expect_false() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Info, "Info");
        let list = GreenDiagnostics::new(&[diag1, diag2]);
        assert!(!list.has_warnings());
    }

    #[test]
    fn test_has_warnings_when_has_warnings_expect_true() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Warning, "Warning");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diag1, diag2]);
        assert!(list.has_warnings());
    }

    #[test]
    fn test_count_by_severity() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error 1");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Error, "Error 2");
        let diag3 = GreenDiagnostic::new(3, DiagnosticSeverity::Warning, "Warning");
        let diag4 = GreenDiagnostic::new(4, DiagnosticSeverity::Info, "Info");
        let list = GreenDiagnostics::new(&[diag1, diag2, diag3, diag4]);

        assert_eq!(list.count_by_severity(DiagnosticSeverity::Error), 2);
        assert_eq!(list.count_by_severity(DiagnosticSeverity::Warning), 1);
        assert_eq!(list.count_by_severity(DiagnosticSeverity::Info), 1);
    }

    #[test]
    fn test_eq_when_same_diagnostics_expect_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning");
        let list1 = GreenDiagnostics::new(&[diag1.clone(), diag2.clone()]);
        let list2 = GreenDiagnostics::new(&[diag1, diag2]);
        assert_eq!(list1, list2);
    }

    #[test]
    fn test_eq_when_different_length_expect_not_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning");
        let list1 = GreenDiagnostics::new(&[diag1.clone()]);
        let list2 = GreenDiagnostics::new(&[diag1, diag2]);
        assert_ne!(list1, list2);
    }

    #[test]
    fn test_eq_when_different_order_expect_not_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Warning, "Warning");
        let list1 = GreenDiagnostics::new(&[diag1.clone(), diag2.clone()]);
        let list2 = GreenDiagnostics::new(&[diag2, diag1]);
        assert_ne!(list1, list2);
    }

    #[test]
    fn test_clone() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let list1 = GreenDiagnostics::new(&[diag.clone()]);
        let list2 = list1.clone();
        assert_eq!(list1, list2);
        assert_eq!(list2.len(), 1);
        assert_eq!(list2.get(0), Some(&diag));
    }

    #[test]
    fn test_debug() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diag]);
        let debug_str = format!("{:?}", list);
        assert!(debug_str.contains("GreenDiagnostics"));
        assert!(debug_str.contains("count"));
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diag.clone()]);
        let ptr = GreenDiagnostics::into_raw(list.clone());
        let reconstructed = unsafe { GreenDiagnostics::from_raw(ptr) };
        assert_eq!(list, reconstructed);
    }

    #[test]
    fn test_borrow() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diag.clone()]);
        let borrowed: &GreenDiagnosticsData = list.borrow();
        assert_eq!(borrowed.len(), 1);
        assert_eq!(borrowed.get(0), Some(&diag));
    }
}

#[cfg(test)]
mod green_diagnostic_list_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diag.clone()]);
        let data: &GreenDiagnosticsData = &*list;
        let owned = data.to_owned();
        assert_eq!(list, owned);
    }

    #[test]
    fn test_eq_when_same_diagnostics_expect_equal() {
        let diag = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error");
        let list1 = GreenDiagnostics::new(&[diag.clone()]);
        let list2 = GreenDiagnostics::new(&[diag]);
        let data1: &GreenDiagnosticsData = &*list1;
        let data2: &GreenDiagnosticsData = &*list2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_diagnostics_expect_not_equal() {
        let diag1 = GreenDiagnostic::new(1, DiagnosticSeverity::Error, "Error 1");
        let diag2 = GreenDiagnostic::new(2, DiagnosticSeverity::Error, "Error 2");
        let list1 = GreenDiagnostics::new(&[diag1]);
        let list2 = GreenDiagnostics::new(&[diag2]);
        let data1: &GreenDiagnosticsData = &*list1;
        let data2: &GreenDiagnosticsData = &*list2;
        assert_ne!(data1, data2);
    }
}
