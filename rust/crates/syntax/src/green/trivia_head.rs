//! # Green Trivia Head - PDF Trivia Collection Header
//!
//! Memory management header for trivia collections with reference counting integration.
//!
//! ## Header Responsibilities
//!
//! The trivia head manages collection-level metadata:
//! - **Reference counting**: Integration with `countme` for memory tracking
//! - **Collection identity**: Ensures proper equality and hashing behavior
//! - **Memory alignment**: Provides proper header layout for `ThinArc`
//!
//! ## Memory Integration
//!
//! ```text
//! ThinArc Layout                 Header Role
//! ┌─────────────┬─────────────┐ ┌─────────────┐
//! │ Header      │ Body        │ │ TriviaHead  │
//! │─────────────┼─────────────┤ │─────────────┤
//! │ TriviaHead  │ [Children]  │ │ count info  │
//! └─────────────┴─────────────┘ └─────────────┘
//! ```

use countme::Count;

use crate::green::trivia::GreenTrivia;

/// Header metadata for PDF trivia collections.
///
/// Contains reference counting information and other collection-level
/// metadata needed for proper memory management and identity.
#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTriviaHead {
    /// Reference counting integration for memory usage tracking.
    ///
    /// This field enables monitoring of trivia collection instances
    /// for debugging and performance analysis purposes.
    _c: Count<GreenTrivia>,
}

impl GreenTriviaHead {
    /// Creates a new trivia collection header.
    ///
    /// Initializes the reference counting system and prepares
    /// the header for use in a `ThinArc` allocation.
    pub(crate) fn new() -> Self {
        Self { _c: Count::new() }
    }
}
