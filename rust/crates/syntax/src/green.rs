//! # Green Tree - Immutable PDF Syntax Tree Layer
//!
//! Foundational layer providing immutable, shareable tree structures for PDF syntax analysis.
//!
//! ## PDF Syntax Representation
//!
//! ```text
//! PDF Content:           Green Tree Structure:
//! 1 0 obj                ┌─ GreenNode(ObjectDecl)
//! <<                     │  ├─ GreenToken(Number, "1")
//! /Type /Catalog         │  ├─ GreenToken(Number, "0")
//! >>                     │  ├─ GreenToken(Keyword, "obj")
//! endobj                 │  └─ GreenNode(Dictionary)
//! ```

mod element;
mod node;
mod token;
mod trivia;

#[cfg(test)]
mod tests;

/// Semantic classification for PDF syntax elements.
///
/// Determines parsing behavior and semantic analysis for each token and node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);
