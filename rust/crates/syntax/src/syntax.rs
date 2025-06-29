//! # Syntax Module: Typed API for PDF Syntax Trees
//!
//! This module provides the typed, language-specific layer that sits on top
//! of the raw green tree. It offers a PDF-aware API for working with syntax
//! trees, including proper handling of PDF-specific trivia and formatting rules.
//!
//! ## What is the Syntax Layer?
//!
//! The syntax layer bridges the gap between:
//! - **Raw Green Tree**: Untyped, immutable tree nodes with perfect fidelity
//! - **AST Layer**: High-level, language-specific abstractions for PDF constructs
//!
//! This layer provides:
//! - **Type-safe node access** with PDF-specific node kinds
//! - **Trivia handling** that respects PDF formatting requirements
//! - **Language integration** for PDF syntax analysis tools
//! - **Iterator patterns** for efficient tree traversal
//!
//! ## PDF Syntax Considerations
//!
//! PDF syntax has unique characteristics that this layer addresses:
//!
//! ### Semantic Whitespace
//! Unlike many programming languages, PDF has contexts where whitespace is semantically significant:
//! - **Object headers**: Newlines separate `obj` keyword from content (ISO 32000-2 §7.3.10)
//! - **Stream boundaries**: Newlines are required after `stream` keyword (ISO 32000-2 §7.3.8)
//! - **Cross-reference entries**: Fixed-width formatting with significant spaces (ISO 32000-2 §7.5.4)
//!
//! ### Trivia Preservation
//! The syntax layer ensures that all formatting is preserved exactly:
//! - Whitespace positioning for PDF compliance
//! - Comment preservation for debugging and tooling
//! - Line ending handling across platforms
//!
//! ## Module Components
//!
//! - **`kind`**: PDF-specific syntax node type definitions
//! - **`language`**: Language integration traits for PDF syntax
//! - **`trivia`**: Whitespace and comment handling with PDF semantics
//! - **`trivia_piece`**: Individual trivia elements (spaces, newlines, comments)
//! - **`trivia_pieces_iterator`**: Efficient iteration over trivia collections

pub(crate) mod kind;
pub(crate) mod language;
pub(crate) mod trivia;
pub(crate) mod trivia_piece;
pub(crate) mod trivia_piece_kind;
pub(crate) mod trivia_pieces_iterator;
