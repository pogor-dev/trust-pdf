//! # Green Trivia Data - PDF Trivia Collection Data Access
//!
//! Provides efficient access to trivia collection contents and metadata.
//!
//! ## Data Access Pattern
//!
//! This module implements the data view for trivia collections, providing:
//! - **Zero-cost access**: Direct memory access without allocation
//! - **Children iteration**: Access to individual trivia elements
//! - **Header metadata**: Collection-level information
//! - **Display formatting**: Text reconstruction for PDF output
//!
//! ## Memory Layout Integration
//!
//! ```text
//! GreenTriviaData                Layout Access
//! ┌─────────────────┐           ┌─────────────┬─────────────────────┐
//! │ data: ThinRepr  │ ────────► │ header()    │ children()          │
//! └─────────────────┘           │─────────────┼─────────────────────┤
//!                               │ GreenHead   │ [TriviaChild; n]    │
//!                               └─────────────┴─────────────────────┘
//! ```

use std::{fmt, mem::ManuallyDrop, ptr};

use crate::green::{
    GreenTriviaReprThin, trivia::GreenTrivia, trivia_child::GreenTriviaChild,
    trivia_head::GreenTriviaHead,
};

/// Data access interface for PDF trivia collections.
///
/// Provides methods to access the header metadata and individual trivia children
/// without additional memory allocation. Used as the target of `Deref` for `GreenTrivia`.
#[repr(transparent)]
pub(crate) struct GreenTriviaData {
    /// Underlying thin representation providing access to both header and body
    pub(crate) data: GreenTriviaReprThin,
}

impl GreenTriviaData {
    /// Returns the header containing collection metadata.
    ///
    /// The header includes reference counting information and other
    /// collection-level data needed for memory management.
    #[inline]
    pub(crate) fn header(&self) -> &GreenTriviaHead {
        &self.data.header
    }

    /// Returns a slice of all trivia children in this collection.
    ///
    /// Children are stored contiguously in memory for efficient iteration.
    /// The slice provides zero-cost access to individual trivia elements.
    #[inline]
    pub fn children(&self) -> &[GreenTriviaChild] {
        self.data.slice()
    }
}

impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.children() == other.children()
    }
}

impl ToOwned for GreenTriviaData {
    type Owned = GreenTrivia;

    #[inline]
    fn to_owned(&self) -> GreenTrivia {
        let green = unsafe { GreenTrivia::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTrivia::clone(&green)
    }
}

impl fmt::Debug for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.children().iter()).finish()
    }
}

impl fmt::Display for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in self.children() {
            match std::str::from_utf8(child.text()) {
                Ok(text) => write!(f, "{}", text)?,
                Err(_) => write!(f, "{:?}", child.text())?,
            }
        }
        Ok(())
    }
}
