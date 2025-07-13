//! # Green Trivia Data - PDF Trivia Access Interface
//!
//! Provides the API layer for accessing PDF trivia content and metadata.
//! This is the final view in the trivia transformation chain.

use std::{fmt, mem::ManuallyDrop, ptr};

use crate::{
    SyntaxKind,
    green::{GreenTriviaChildReprThin, trivia_child::GreenTriviaChild},
};

/// API interface for accessing PDF trivia data with zero-cost operations.
///
/// This struct provides the semantic interface for trivia elements, abstracting
/// the underlying memory representation into PDF-aware operations.
///
/// ## Memory Layout (Header/Body Separation)
///
/// ```text
/// GreenTriviaData view of memory:
///
/// ┌─────────────────────────────┬─────────────────────────────┐
/// │          HEADER             │            BODY             │
/// ├─────────────────────────────┼─────────────────────────────┤
/// │ GreenTriviaHead             │ Variable-length text        │
/// │ ┌─────────────────────────┐ │ ┌─────────────────────────┐ │
/// │ │ kind: SyntaxKind        │ │ │ [u8; len]               │ │
/// │ │ len: usize              │ │ │ actual trivia bytes     │ │
/// │ └─────────────────────────┘ │ └─────────────────────────┘ │
/// └─────────────────────────────┴─────────────────────────────┘
///
/// Header: Fixed-size metadata (kind, length)
/// Body:   Variable-size content (actual text bytes)
/// ```
///
/// ## PDF Context Examples
///
/// ```text
/// Different trivia types and their data:
///
/// Newline:     Header{kind=newline, len=1}    Body{"\n"}
/// Whitespace:  Header{kind=whitespace, len=1} Body{" "}
/// Whitespace:  Header{kind=whitespace, len=5} Body{"     "}
/// Comment:     Header{kind=comment, len=8}    Body{"%hello"}
/// ```
#[repr(transparent)]
pub(crate) struct GreenTriviaChildData {
    /// Underlying thin representation providing access to both header and body
    pub(crate) data: GreenTriviaChildReprThin,
}

impl GreenTriviaChildData {
    /// Returns the semantic kind of this trivia element.
    ///
    /// Accesses the **header** portion of the trivia to determine its PDF-specific
    /// classification (whitespace, newline, comment).
    ///
    /// ## Header Access Pattern
    ///
    /// ```text
    /// Memory Access:
    /// GreenTriviaData
    ///        ↓ .data
    /// GreenTriviaReprThin  
    ///        ↓ .header
    /// GreenTriviaHead
    ///        ↓ .kind
    /// SyntaxKind (enum value)
    /// ```
    ///
    /// ## PDF Significance
    ///
    /// The kind determines semantic meaning in PDF processing:
    /// - `Newline`: Line breaks with potential semantic significance
    /// - `Whitespace`: Spaces and tabs for formatting and separation
    /// - `Comment`: PDF comments starting with '%' character
    #[inline]
    pub(crate) fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Returns the raw byte content of this trivia element.
    ///
    /// Accesses the **body** portion containing the actual trivia text.
    /// Essential for PDF round-trip fidelity where exact bytes matter.
    ///
    /// ## Body Access Pattern
    ///
    /// ```text
    /// Memory Access:
    /// GreenTriviaData
    ///        ↓ .data
    /// GreenTriviaReprThin
    ///        ↓ .slice()
    /// Raw slice pointer + length
    ///        ↓ from_raw_parts
    /// &[u8] (safe slice view)
    /// ```
    ///
    /// ## PDF Examples
    ///
    /// ```text
    /// Newline:       text() → b"\n"
    /// Whitespace:    text() → b" "
    /// Whitespace:    text() → b"     " (5 spaces)
    /// Comment:       text() → b"%PDF-1.7"
    /// ```
    ///
    /// ## Safety
    ///
    /// Safe because the slice is created from valid memory managed by `ThinArc`.
    /// The length is stored in the header and guaranteed to match allocated space.
    #[inline]
    pub(crate) fn text(&self) -> &[u8] {
        let slice = self.data.slice();
        unsafe { std::slice::from_raw_parts(slice.as_ptr(), slice.len()) }
    }

    /// Returns the byte width (length) of this trivia element.
    ///
    /// Computed from the **body** length for consistency with actual content.
    /// Useful for PDF layout calculations and memory usage tracking.
    ///
    /// ## Usage in PDF Processing
    ///
    /// ```text
    /// Layout calculations:
    /// - Fixed-width formatting: width() for alignment verification
    /// - Trivia parsing: width() for boundary detection  
    /// - Memory management: width() for allocation size planning
    /// ```
    ///
    /// ## Implementation Note
    ///
    /// Could alternatively read from `header.len`, but using `text().len()`
    /// ensures consistency between header metadata and actual body content.
    #[inline]
    pub(crate) fn width(&self) -> u64 {
        self.text().len() as u64
    }
}

impl PartialEq for GreenTriviaChildData {
    /// Compares trivia for semantic equality (kind + content).
    ///
    /// Essential for PDF trivia deduplication, caching, and incremental updates.
    /// Two trivia elements are equal if both kind and byte content match exactly.
    ///
    /// ```text
    /// Equality Check:
    ///
    /// Step 1: Kind comparison    Step 2: Content comparison
    /// ┌─────────────────┐       ┌─────────────────────┐
    /// │ self.kind()     │  ==   │ self.text()         │
    /// │ other.kind()    │       │ other.text()        │
    /// └─────────────────┘       └─────────────────────┘
    ///         │                           │
    ///         └───────────┬───────────────┘
    ///                     ▼
    ///              true if both match
    ///
    /// Examples:
    /// Equal:     Newline("\n") == Newline("\n")         ✓
    /// Different: Newline("\n") != Whitespace("\n")     ✗ (kind differs)
    /// Different: Whitespace(" ") != Whitespace("  ")   ✗ (content differs)
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl ToOwned for GreenTriviaChildData {
    type Owned = GreenTriviaChild;

    /// Converts borrowed trivia to owned with reference counting (zero-copy).
    ///
    /// Creates a new `GreenTrivia` sharing the same memory via `ThinArc`,
    /// enabling safe ownership transfer without duplicating trivia bytes.
    ///
    /// ```text
    /// Borrowed → Owned Conversion:
    ///
    /// &GreenTriviaData           GreenTrivia
    /// ┌──────────────────┐      ┌──────────────────┐
    /// │ Borrowed view    │ ──►  │ Owned reference  │
    /// │ of shared memory │      │ with ref count   │
    /// └──────────────────┘      └──────────────────┘
    ///         │                           │
    ///         └─────── Same bytes ────────┘
    ///                 (zero copy)
    /// ```
    ///
    /// Implementation: Reconstruct from raw pointer → wrap in `ManuallyDrop` → clone reference count.
    #[inline]
    fn to_owned(&self) -> GreenTriviaChild {
        let green = unsafe { GreenTriviaChild::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTriviaChild::clone(&green)
    }
}

impl fmt::Debug for GreenTriviaChildData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTrivia")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .finish()
    }
}

impl fmt::Display for GreenTriviaChildData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.text())
    }
}
