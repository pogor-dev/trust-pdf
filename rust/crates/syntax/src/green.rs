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

pub(crate) mod builder;
pub(crate) mod element;
pub(crate) mod node;
pub(crate) mod node_cache;
pub(crate) mod token;
pub(crate) mod trivia;

pub(crate) use self::{element::GreenElementRef, node::GreenChild};

pub use self::{
    builder::{Checkpoint, GreenNodeBuilder},
    node::{GreenNode, GreenNodeData, NodeChildren},
    node_cache::NodeCache,
    token::{GreenToken, GreenTokenData},
    trivia::{GreenTrivia, GreenTriviaData},
};

#[cfg(test)]
#[path = "green/tests/lib.rs"]
mod tests;

/// Semantic classification for PDF syntax elements.
///
/// Determines parsing behavior and semantic analysis for each token and node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);
