use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr, slice,
};

use crate::arc::{Arc, HeaderSlice, ThinArc};
use countme::Count;

use super::diagnostic::{DiagnosticSeverity, GreenDiagnostic};
#[cfg(test)]
use crate::DiagnosticKind;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenDiagnosticsHead {
    count: u32,                  // 4 bytes (`u32`)
    _c: Count<GreenDiagnostics>, // 0 bytes
}

/// Unsized diagnostic list data storing diagnostics inline.
#[repr(transparent)]
pub(crate) struct GreenDiagnosticsData {
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
        self.diagnostics().iter().any(|diagnostic| diagnostic.severity() == DiagnosticSeverity::Error)
    }

    /// Checks if any diagnostic in the list has Warning severity.
    #[inline]
    pub fn has_warnings(&self) -> bool {
        self.diagnostics().iter().any(|diagnostic| diagnostic.severity() == DiagnosticSeverity::Warning)
    }

    /// Count diagnostics by severity level.
    #[inline]
    pub fn count_by_severity(&self, severity: DiagnosticSeverity) -> usize {
        self.diagnostics().iter().filter(|diagnostic| diagnostic.severity() == severity).count()
    }
}

impl PartialEq for GreenDiagnosticsData {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.diagnostics() == other.diagnostics()
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

impl fmt::Display for GreenDiagnosticsData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.diagnostics().iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{diagnostic}")?;
        }
        Ok(())
    }
}

/// Owned diagnostic list in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenDiagnostics {
    ptr: ThinArc<GreenDiagnosticsHead, GreenDiagnostic>,
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
}

impl_green_boilerplate!(GreenDiagnosticsHead, GreenDiagnosticsData, GreenDiagnostics, GreenDiagnostic);

/// Iterator over diagnostics in a diagnostic list.
#[derive(Debug, Clone)]
pub(crate) struct DiagnosticIter<'a> {
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
    fn test_green_diagnostics_head_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenDiagnosticsHead>(), 4);
        assert_eq!(std::mem::align_of::<GreenDiagnosticsHead>(), 4);
    }

    #[test]
    fn test_green_diagnostics_data_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenDiagnosticsData>(), 16);
            assert_eq!(std::mem::align_of::<GreenDiagnosticsData>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenDiagnosticsData>(), 8);
            assert_eq!(std::mem::align_of::<GreenDiagnosticsData>(), 4);
        }
    }

    #[test]
    fn test_green_diagnostics_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenDiagnostics>(), 8);
            assert_eq!(std::mem::align_of::<GreenDiagnostics>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenDiagnostics>(), 4);
            assert_eq!(std::mem::align_of::<GreenDiagnostics>(), 4);
        }
    }
}

#[cfg(test)]
mod green_diagnostics_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_when_empty_expect_empty_list() {
        let list = GreenDiagnostics::new(&[]);
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn test_empty_when_called_expect_empty_list() {
        let list = GreenDiagnostics::empty();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn test_new_when_single_diagnostic_expect_single_item() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::UnbalancedStringLiteral, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diagnostic.clone()]);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
        assert_eq!(list.get(0), Some(&diagnostic));
    }

    #[test]
    fn test_new_when_multiple_diagnostics_expect_all_items() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::UnbalancedStringLiteral, DiagnosticSeverity::Error, "Error 1");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::MissingWhitespaceBeforeToken, DiagnosticSeverity::Warning, "Warning 1");
        let diagnostic3 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Info, "Info 1");
        let list = GreenDiagnostics::new(&[diagnostic1.clone(), diagnostic2.clone(), diagnostic3.clone()]);

        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(&diagnostic1));
        assert_eq!(list.get(1), Some(&diagnostic2));
        assert_eq!(list.get(2), Some(&diagnostic3));
    }

    #[test]
    fn test_get_when_index_out_of_bounds_expect_none() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diagnostic]);
        assert_eq!(list.get(1), None);
        assert_eq!(list.get(100), None);
    }

    #[test]
    fn test_diagnostics_when_called_expect_slice_contents() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::MissingWhitespaceBeforeToken, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diagnostic1.clone(), diagnostic2.clone()]);

        let diagnostics = list.diagnostics();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0], diagnostic1);
        assert_eq!(diagnostics[1], diagnostic2);
    }

    #[test]
    fn test_iter_when_called_expect_forward_iteration() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::MissingWhitespaceBeforeToken, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diagnostic1.clone(), diagnostic2.clone()]);

        let collected: Vec<_> = list.iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], &diagnostic1);
        assert_eq!(collected[1], &diagnostic2);
    }

    #[test]
    fn test_iter_when_double_ended_expect_reverse_iteration() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::MissingWhitespaceBeforeToken, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diagnostic1.clone(), diagnostic2.clone()]);

        let mut iter = list.iter();
        assert_eq!(iter.next_back(), Some(&diagnostic2));
        assert_eq!(iter.next(), Some(&diagnostic1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_when_exact_size_expect_len() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::MissingWhitespaceBeforeToken, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diagnostic1, diagnostic2]);

        let iter = list.iter();
        assert_eq!(iter.len(), 2);
    }

    #[test]
    fn test_has_errors_when_no_errors_expect_false() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "Warning");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Info, "Info");
        let list = GreenDiagnostics::new(&[diagnostic1, diagnostic2]);
        assert!(!list.has_errors());
    }

    #[test]
    fn test_has_errors_when_has_errors_expect_true() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "Warning");
        let list = GreenDiagnostics::new(&[diagnostic1, diagnostic2]);
        assert!(list.has_errors());
    }

    #[test]
    fn test_has_warnings_when_no_warnings_expect_false() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Info, "Info");
        let list = GreenDiagnostics::new(&[diagnostic1, diagnostic2]);
        assert!(!list.has_warnings());
    }

    #[test]
    fn test_has_warnings_when_has_warnings_expect_true() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "Warning");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diagnostic1, diagnostic2]);
        assert!(list.has_warnings());
    }

    #[test]
    fn test_count_by_severity_when_mixed_expect_counts() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error 1");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error 2");
        let diagnostic3 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "Warning");
        let diagnostic4 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Info, "Info");
        let list = GreenDiagnostics::new(&[diagnostic1, diagnostic2, diagnostic3, diagnostic4]);

        assert_eq!(list.count_by_severity(DiagnosticSeverity::Error), 2);
        assert_eq!(list.count_by_severity(DiagnosticSeverity::Warning), 1);
        assert_eq!(list.count_by_severity(DiagnosticSeverity::Info), 1);
    }

    #[test]
    fn test_eq_when_same_diagnostics_expect_equal() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "Warning");
        let list1 = GreenDiagnostics::new(&[diagnostic1.clone(), diagnostic2.clone()]);
        let list2 = GreenDiagnostics::new(&[diagnostic1, diagnostic2]);
        assert_eq!(list1, list2);
    }

    #[test]
    fn test_eq_when_different_length_expect_not_equal() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "Warning");
        let list1 = GreenDiagnostics::new(&[diagnostic1.clone()]);
        let list2 = GreenDiagnostics::new(&[diagnostic1, diagnostic2]);
        assert_ne!(list1, list2);
    }

    #[test]
    fn test_eq_when_different_order_expect_not_equal() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "Warning");
        let list1 = GreenDiagnostics::new(&[diagnostic1.clone(), diagnostic2.clone()]);
        let list2 = GreenDiagnostics::new(&[diagnostic2, diagnostic1]);
        assert_ne!(list1, list2);
    }

    #[test]
    fn test_clone_when_called_expect_equal_copy() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let list1 = GreenDiagnostics::new(&[diagnostic.clone()]);
        let list2 = list1.clone();
        assert_eq!(list1, list2);
        assert_eq!(list2.len(), 1);
        assert_eq!(list2.get(0), Some(&diagnostic));
    }

    #[test]
    fn test_debug_when_formatted_expect_diagnostics_struct() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diagnostic]);
        let debug_str = format!("{:?}", list);
        assert!(debug_str.contains("GreenDiagnostics"));
        assert!(debug_str.contains("count"));
    }

    #[test]
    fn test_into_raw_and_from_raw_when_roundtrip_expect_equal() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diagnostic.clone()]);
        let ptr = GreenDiagnostics::into_raw(list.clone());
        let reconstructed = unsafe { GreenDiagnostics::from_raw(ptr) };
        assert_eq!(list, reconstructed);
    }

    #[test]
    fn test_borrow_when_called_expect_data_access() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diagnostic.clone()]);
        let borrowed: &GreenDiagnosticsData = list.borrow();
        assert_eq!(borrowed.len(), 1);
        assert_eq!(borrowed.get(0), Some(&diagnostic));
    }
}

#[cfg(test)]
mod green_diagnostics_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_owned_when_called_expect_equivalent_owned_value() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let list = GreenDiagnostics::new(&[diagnostic.clone()]);
        let data: &GreenDiagnosticsData = &*list;
        let owned = data.to_owned();
        assert_eq!(list, owned);
    }

    #[test]
    fn test_eq_when_same_diagnostics_expect_equal() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error");
        let list1 = GreenDiagnostics::new(&[diagnostic.clone()]);
        let list2 = GreenDiagnostics::new(&[diagnostic]);
        let data1: &GreenDiagnosticsData = &*list1;
        let data2: &GreenDiagnosticsData = &*list2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_eq_when_different_diagnostics_expect_not_equal() {
        let diagnostic1 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error 1");
        let diagnostic2 = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Error, "Error 2");
        let list1 = GreenDiagnostics::new(&[diagnostic1]);
        let list2 = GreenDiagnostics::new(&[diagnostic2]);
        let data1: &GreenDiagnosticsData = &*list1;
        let data2: &GreenDiagnosticsData = &*list2;
        assert_ne!(data1, data2);
    }
}
