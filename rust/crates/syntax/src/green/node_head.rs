//! # Green Node Header: Metadata for Immutable Syntax Nodes
//!
//! This module defines the header structure for green tree nodes, containing
//! the essential metadata that describes each syntax node's characteristics.
//! The header is designed for memory efficiency and fast access to critical
//! node properties.
//!
//! ## What is a Node Header?
//!
//! A node header contains the metadata that describes a syntax node without
//! including the actual child data. This separation allows for:
//! - **Efficient memory layout**: Headers and children can be stored together
//! - **Fast property access**: Common properties are immediately available
//! - **Cache efficiency**: Headers are accessed frequently and kept compact
//! - **Memory accounting**: Track allocation patterns for debugging
//!
//! ## Design Rationale
//!
//! ### Header-Slice Pattern
//! The header is designed to work with the header-slice memory layout pattern:
//! ```text
//! [GreenNodeHead][Child1][Child2][Child3]...
//! ```
//! This layout provides better cache locality than separate allocations.
//!
//! ### PDF Processing Requirements
//! PDF processing requires efficient access to:
//! - **Node type identification**: Quickly determine syntax construct type
//! - **Length calculations**: Essential for cross-reference table generation
//! - **Memory tracking**: Important for large PDF file processing
//!
//! ## Memory Efficiency
//!
//! The header is kept minimal to reduce memory overhead:
//! - Fixed-size structure for predictable layout
//! - No padding between fields for optimal packing
//! - Essential data only, with derived properties computed on demand

use countme::Count;

use crate::green::{kind::RawSyntaxKind, node::GreenNode};

/// Header structure for green tree syntax nodes.
///
/// `GreenNodeHead` contains the essential metadata for each syntax node
/// in the immutable green tree. It provides fast access to the node's
/// type and size information without requiring access to child elements.
///
/// ## Memory Layout
///
/// This header is designed to be stored alongside child elements in a
/// single allocation using the header-slice pattern, providing excellent
/// cache locality and memory efficiency.
///
/// ## Field Description
///
/// - **`kind`**: Identifies what type of PDF syntax this node represents
/// - **`text_len`**: Total byte length of text covered by this node and all descendants
/// - **`_c`**: Memory allocation counter for debugging and profiling
///
/// ## PDF Context
///
/// In PDF processing, this header might describe nodes such as:
/// - Dictionary objects with their total content length
/// - Array structures spanning multiple child elements  
/// - Stream objects including headers and content
/// - Complete PDF objects from `obj` to `endobj`
///
/// ## Thread Safety
///
/// The header is immutable once created and can be safely shared across
/// threads. The memory counter uses atomic operations internally.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    /// The syntax kind that identifies what type of PDF construct this node represents.
    ///
    /// This could be a dictionary, array, object definition, stream, or any other
    /// structural element in PDF syntax. The kind determines how the node's
    /// children should be interpreted and processed.
    pub(super) kind: RawSyntaxKind,

    /// Total length in bytes of all text covered by this node.
    ///
    /// This includes the text of this node and all of its descendants,
    /// providing the total span of the PDF content that this syntax
    /// subtree represents. Critical for PDF operations that require
    /// precise byte positioning, such as cross-reference table generation.
    pub(super) text_len: u64,

    /// Memory allocation counter for debugging and profiling.
    ///
    /// Tracks the number of live `GreenNode` instances to help identify
    /// memory usage patterns and potential leaks during PDF processing.
    _c: Count<GreenNode>,
}
