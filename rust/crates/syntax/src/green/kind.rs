//! # Raw Syntax Kind: Type Tags for Syntax Tree Elements
//!
//! This module defines the fundamental type tagging system used throughout
//! the green tree to identify different kinds of syntax elements. It provides
//! a compact, efficient way to distinguish between various PDF syntax constructs
//! at the tree node level.
//!
//! ## What is a Syntax Kind?
//!
//! A syntax kind is a type identifier that tells us what type of syntax element
//! a particular node or token represents. For example:
//! - PDF objects, dictionaries, arrays, strings, numbers
//! - Keywords like `obj`, `endobj`, `stream`, `endstream`
//! - Structural elements like brackets, parentheses, delimiters
//! - Trivia like whitespace, comments, line breaks
//!
//! ## Design Rationale
//!
//! ### Compact Representation
//! Using a `u16` allows for 65,536 different syntax kinds, which is more than
//! sufficient for PDF syntax while maintaining a small memory footprint.
//! Each node stores this kind, so keeping it small is important for memory efficiency.
//!
//! ### Performance Characteristics
//! - **Fast Comparison**: Copy semantics with efficient equality checks
//! - **Hash-friendly**: Can be used as hash map keys for syntax analysis
//! - **Ordered**: Supports ordered operations for consistent tree traversal
//! - **Debug-friendly**: Provides readable debug output for development
//!
//! ## PDF Syntax Context
//!
//! In PDF processing, syntax kinds distinguish between elements like:
//! - **Structural**: `{`, `}`, `[`, `]`, `(`, `)`, `<`, `>`
//! - **Keywords**: `obj`, `endobj`, `stream`, `endstream`, `xref`, `trailer`
//! - **Data Types**: Numbers, strings, names, boolean values
//! - **Trivia**: Spaces, tabs, line breaks, comments
//!
//! ## Usage Pattern
//!
//! The raw syntax kind is typically wrapped by higher-level enums that provide
//! type-safe, language-specific representations of PDF syntax elements.

/// Type tag identifier for syntax tree elements.
///
/// `RawSyntaxKind` serves as a compact, universal identifier for different
/// types of syntax elements in the PDF language. Each node and token in the
/// green tree carries one of these identifiers to specify what kind of
/// syntax construct it represents.
///
/// The underlying `u16` provides sufficient space for all PDF syntax elements
/// while maintaining optimal memory usage and performance characteristics.
///
/// ## Examples in PDF Context
///
/// Different syntax kinds might represent:
/// - Object keywords: `obj`, `endobj`
/// - Data type indicators: number, string, name, dictionary
/// - Structural elements: brackets, parentheses, angle brackets
/// - Stream markers: `stream`, `endstream`
/// - Cross-reference elements: `xref`, `trailer`, `startxref`
///
/// ## Thread Safety
///
/// This type is `Copy` and contains no mutable state, making it safe to
/// share across threads without synchronization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawSyntaxKind(pub u16);
