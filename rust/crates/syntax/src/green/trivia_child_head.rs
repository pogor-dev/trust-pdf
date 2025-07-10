//! # Green Trivia Head - PDF Trivia Metadata Header
//!
//! Fixed-size header containing trivia classification and memory management metadata.
//! Part of the header/body separation pattern for efficient trivia representation.
//!
//! ## Memory Layout Role
//!
//! ```text
//! Complete Trivia Structure:
//! ┌─────────────────────────────┬─────────────────────────────┐
//! │        THIS MODULE          │        SEPARATE MODULE      │
//! │     GreenTriviaHead         │      Variable Text Data     │
//! ├─────────────────────────────┼─────────────────────────────┤
//! │ kind: SyntaxKind            │ [u8; len]                   │
//! │ _c: Count<GreenTrivia>      │ actual trivia bytes         │
//! └─────────────────────────────┴─────────────────────────────┘
//!    ↑ Fixed size (metadata)       ↑ Variable size (content)
//! ```
//!
//! ## PDF Trivia Classification
//!
//! The header identifies PDF trivia types per ISO 32000-2:
//! - **Whitespace**: Spaces/tabs for token separation (§7.2.2)
//! - **Newlines**: Line breaks with semantic significance (§7.2.2)
//! - **Comments**: PDF comments starting with '%' (§7.2.3)

use countme::Count;

use crate::{SyntaxKind, green::trivia_child::GreenTriviaChild};

/// Header metadata for PDF trivia elements with memory management tracking.
///
/// Contains the essential classification information for trivia while enabling
/// memory usage monitoring through the `countme` crate integration.
///
/// ## Design Rationale
///
/// Separating metadata (header) from content (body) provides several benefits:
/// - **Memory efficiency**: Fixed header size regardless of content length
/// - **Cache locality**: Metadata can be accessed without loading large content
/// - **Type safety**: Header operations don't require content validation
///
/// ## PDF Context Examples
///
/// ```text
/// PDF Content:     Header Created:
/// " "          →   GreenTriviaHead { kind: Whitespace, .. }
/// "\n"         →   GreenTriviaHead { kind: Newline, .. }
/// "%comment"   →   GreenTriviaHead { kind: Comment, .. }
/// ```
#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTriviaChildHead {
    /// Semantic classification of the trivia element.
    ///
    /// Determines how this trivia should be interpreted during PDF processing:
    /// - **Whitespace**: General spacing and formatting
    /// - **Newline**: Line boundaries with potential semantic meaning
    /// - **Comment**: PDF comments that may contain metadata or instructions
    ///
    /// This field is the primary interface for trivia type discrimination
    /// throughout the compiler pipeline.
    pub(crate) kind: SyntaxKind,

    /// Memory allocation tracking for development and debugging.
    ///
    /// Enables monitoring trivia creation/destruction patterns during development.
    /// The underscore prefix indicates this is primarily for internal tooling
    /// rather than core functionality.
    ///
    /// ## Usage During Development
    ///
    /// ```text
    /// Debug builds:     Track allocation/deallocation patterns
    /// Release builds:   Zero-cost abstraction (likely optimized away)
    /// Testing:          Detect memory leaks in trivia handling
    /// ```
    _c: Count<GreenTriviaChild>,
}

impl GreenTriviaChildHead {
    /// Creates a new trivia header with the specified PDF trivia classification.
    ///
    /// Initializes both the semantic kind and memory tracking for a new trivia element.
    /// This is typically called during the lexical analysis phase when trivia
    /// elements are first identified in PDF content.
    ///
    /// ## Parameters
    ///
    /// * `kind` - The semantic classification of this trivia (whitespace, newline, comment)
    ///
    /// ## PDF Processing Context
    ///
    /// Called when the lexer identifies trivia patterns:
    /// ```text
    /// Lexer encounters:     Creates header:
    /// ' '               →   new(SyntaxKind::Whitespace)
    /// '\n'              →   new(SyntaxKind::Newline)  
    /// '%...'            →   new(SyntaxKind::Comment)
    /// ```
    ///
    /// The header is then combined with the actual trivia bytes to form
    /// a complete `GreenTrivia` instance.
    pub(crate) fn new(kind: SyntaxKind) -> Self {
        Self {
            kind,
            _c: Count::new(),
        }
    }
}
