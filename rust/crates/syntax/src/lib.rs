//! # Syntax Crate: Concrete Syntax Tree for PDF Compiler
//!
//! This crate implements the syntax layer of the PDF compiler, providing a complete
//! Concrete Syntax Tree (CST) implementation with full fidelity preservation of all
//! syntactic trivia. It follows the architecture patterns from Rust Analyzer and Roslyn.
//!
//! ## Architecture Overview
//!
//! The syntax crate is built around several key concepts:
//!
//! ### Green Tree (Immutable Layer)
//! The "green" tree is an immutable, compact representation of the syntax tree where:
//! - All nodes and tokens are immutable once created
//! - Memory is efficiently shared through atomic reference counting
//! - Trivia (whitespace, comments) is preserved with exact fidelity
//! - Suitable for caching and incremental parsing
//!
//! ### Syntax Layer (Typed API)
//! Provides a typed, language-specific API over the green tree:
//! - Language-agnostic traits for different PDF syntax kinds
//! - Type-safe node and token access
//! - Maintains references to parent/child relationships
//!
//! ### Arc-based Memory Management
//! Uses a custom Arc implementation optimized for syntax trees:
//! - Thread-safe reference counting without weak references
//! - Optimized memory layout for header+slice patterns
//! - Better performance than `std::sync::Arc` for this use case
//!
//! ## PDF-Specific Considerations
//!
//! This syntax tree is designed specifically for PDF parsing, where trivia handling
//! is critical due to PDF's syntax requirements:
//!
//! - **Object declarations**: Newlines separate headers from object bodies (ISO 32000-2 §7.3.10)
//! - **Stream keywords**: Newlines are required after 'stream' (ISO 32000-2 §7.3.8)
//! - **Cross-reference entries**: Fixed-width, space-sensitive formatting (ISO 32000-2 §7.5.4)
//! - **Content streams**: Space-separated tokens only (ISO 32000-2 §8.1.1)
//!
//! ## Module Organization
//!
//! - [`arc`]: Custom atomic reference counting for efficient memory sharing
//! - [`green`]: Immutable syntax tree nodes and tokens with trivia preservation
//! - [`syntax`]: Language-agnostic traits and utilities for syntax kinds
//! - [`ast`]: Abstract syntax tree nodes providing typed access to the green tree
//! - [`cursor`]: Navigation and iteration utilities for traversing syntax trees
//! - [`utility_types`]: Common types used throughout the syntax system
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use syntax::{
//!     green::{node::GreenNode, token::GreenToken},
//!     syntax::{trivia_piece::TriviaPiece, trivia_piece_kind::TriviaPieceKind}
//! };
//!
//! // Create trivia for PDF-specific whitespace requirements
//! let whitespace = TriviaPiece::whitespace(4);
//! let newline = TriviaPiece::line_feed(1);
//!
//! // These would be used during parsing to preserve the exact formatting
//! // required by PDF specification sections
//! ```

mod arc;
mod ast;
mod cursor;
mod green;
mod syntax;
mod utility_types;

// Re-export key types for public API
pub use syntax::{
    kind::SyntaxKind, language::Language, trivia_piece::TriviaPiece,
    trivia_piece_kind::TriviaPieceKind,
};
