//! # Green Token Header: Metadata for Immutable Syntax Tokens
//!
//! This module defines the header structure for green tree tokens, containing
//! the essential metadata for leaf elements in the syntax tree. Tokens represent
//! the actual text content in PDF files, along with their associated trivia
//! (whitespace, comments, formatting).
//!
//! ## What is a Token Header?
//!
//! A token header contains the metadata that describes a syntax token:
//! - **Kind identification**: What type of PDF syntax element this represents
//! - **Trivia management**: Leading and trailing whitespace/comments
//! - **Memory tracking**: Allocation counting for debugging and profiling
//!
//! ## PDF Token Examples
//!
//! Different types of tokens this header might describe:
//! - **Keywords**: `obj`, `endobj`, `stream`, `endstream`, `xref`, `trailer`
//! - **Data values**: Numbers (`42`, `3.14`), strings (`(Hello)`), names (`/Type`)
//! - **Operators**: Content stream operators (`Tf`, `BT`, `ET`, `q`, `Q`)
//! - **Structural**: Brackets (`[`, `]`), delimiters (`<<`, `>>`), parentheses
//!
//! ## Trivia Handling
//!
//! PDF syntax has specific requirements for whitespace and formatting:
//! - **Object boundaries**: Specific newline requirements around `obj`/`endobj`
//! - **Stream keywords**: Mandatory newlines after `stream` (ISO 32000-2 §7.3.8)
//! - **Cross-reference formatting**: Fixed-width space formatting (ISO 32000-2 §7.5.4)
//! - **Content streams**: Space-separated tokens only (ISO 32000-2 §8.1.1)
//!
//! ## Memory Efficiency
//!
//! The header is designed for optimal memory usage:
//! - Compact representation for frequently allocated tokens
//! - Efficient trivia storage that avoids duplication
//! - Memory tracking for large PDF file processing

use countme::Count;

use crate::green::{kind::RawSyntaxKind, token::GreenToken, trivia::GreenTrivia};

/// Header structure for green tree syntax tokens.
///
/// `GreenTokenHead` contains the essential metadata for each token
/// (leaf element) in the immutable green tree. It describes what type
/// of syntax element the token represents and manages its associated
/// trivia (whitespace, comments, formatting).
///
/// ## Design Rationale
///
/// Tokens are the leaf nodes of the syntax tree that contain actual
/// text content. The header stores metadata separately from the text
/// to enable efficient memory layout and fast property access.
///
/// ## Field Descriptions
///
/// - **`kind`**: Identifies the type of PDF syntax element
/// - **`leading`**: Whitespace/comments that appear before the token text
/// - **`trailing`**: Whitespace/comments that appear after the token text  
/// - **`_c`**: Memory allocation counter for debugging
///
/// ## PDF Processing Context
///
/// In PDF files, tokens with their trivia represent:
/// - Keywords with required formatting: `stream\n` (newline required)
/// - Values with flexible spacing: `/Type /Catalog` (spaces as needed)
/// - Structural elements: `<<` `>>` with surrounding whitespace
/// - Content operators: `BT ET` with space separation
///
/// ## Thread Safety
///
/// The header is immutable once created and safe to share across threads.
/// The memory counter uses atomic operations internally.
#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTokenHead {
    /// The syntax kind that identifies what type of PDF construct this token represents.
    ///
    /// Examples include keywords (`obj`, `stream`), data types (number, string, name),
    /// or structural elements (brackets, delimiters). This determines how the token
    /// should be interpreted during semantic analysis.
    pub(crate) kind: RawSyntaxKind,
    
    /// Trivia (whitespace, comments) that appears before the token text.
    ///
    /// Leading trivia is crucial for PDF compliance as some constructs have
    /// specific formatting requirements. For example, the `obj` keyword might
    /// need specific newline handling to separate object headers from content.
    pub(crate) leading: GreenTrivia,
    
    /// Trivia (whitespace, comments) that appears after the token text.
    ///
    /// Trailing trivia ensures proper spacing between tokens and handles
    /// mandatory formatting requirements like the newline after `stream`
    /// keywords required by the PDF specification.
    pub(crate) trailing: GreenTrivia,
    
    /// Memory allocation counter for debugging and profiling.
    ///
    /// Tracks the number of live `GreenToken` instances to help monitor
    /// memory usage patterns during PDF parsing, especially important
    /// for large documents with many tokens.
    _c: Count<GreenToken>,
}
