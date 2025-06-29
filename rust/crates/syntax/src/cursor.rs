//! # Cursor Module: Navigation and Iteration for Syntax Trees
//!
//! This module provides efficient navigation and iteration utilities for traversing
//! syntax trees. It enables moving through the tree structure while maintaining
//! performance and memory efficiency, following patterns used in Rust Analyzer.
//!
//! ## Design Goals
//!
//! The cursor system provides:
//! 1. **Efficient Navigation**: Move through tree nodes without allocating new objects
//! 2. **Type Safety**: Strongly typed access to different node and token types
//! 3. **Trivia Handling**: Navigate through whitespace and comments correctly
//! 4. **Incremental Updates**: Support for efficiently updating tree positions
//!
//! ## Navigation Patterns
//!
//! The cursor supports several navigation patterns common in compiler design:
//!
//! ### Tree Traversal
//! - **Depth-first**: Visit all descendants before siblings
//! - **Breadth-first**: Visit all siblings before descendants  
//! - **Parent navigation**: Move up the tree hierarchy
//! - **Sibling navigation**: Move horizontally through child lists
//!
//! ### Content-Aware Navigation
//! - **Skip trivia**: Move only through significant content
//! - **Include trivia**: Access whitespace and formatting information
//! - **Token-only**: Navigate through tokens without entering nodes
//! - **Node-only**: Navigate through structural nodes without tokens
//!
//! ## PDF-Specific Considerations
//!
//! PDF syntax has specific navigation requirements due to its structure:
//!
//! ### Whitespace Significance
//! - Some contexts require exact whitespace preservation (streams, xref)
//! - Other contexts allow flexible whitespace handling (dictionaries)
//! - Line endings have semantic meaning in specific constructs
//!
//! ### Object Relationships
//! - Navigate between object definitions and references
//! - Follow cross-reference table entries to object locations
//! - Traverse stream content while maintaining dictionary associations
//!
//! ### Error Recovery
//! - Skip malformed objects while maintaining document structure
//! - Navigate around corrupted sections to salvage readable content
//! - Preserve navigation state during incremental parsing updates
//!
//! ## Module Components
//!
//! - [`element`]: Core element types that can be navigated (nodes, tokens)
//! - [`green_element`]: Immutable tree elements optimized for sharing
//! - [`node_data`]: Data structures for representing node information
//! - [`node_kind`]: Type classification for different node types
//! - [`token`]: Token-specific navigation and access utilities
//! - [`trivia`]: Handling of whitespace, comments, and formatting
//! - [`trivia_pieces_iterator`]: Efficient iteration over trivia sequences
//! - [`weak_green_element`]: Weak references for cyclic tree structures
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! // Example showing how cursor navigation will be used:
//! // let mut cursor = TreeCursor::new(root_node);
//! // cursor.goto_first_child();  // Enter first child
//! // while cursor.goto_next_sibling() {  // Visit all siblings
//! //     if cursor.kind() == PdfSyntaxKind::Object {
//! //         // Process PDF object
//! //         process_object(cursor.current());
//! //     }
//! // }
//! ```

pub(crate) mod element;
pub(crate) mod green_element;
pub(crate) mod node_data;
pub(crate) mod node_kind;
pub(crate) mod token;
pub(crate) mod trivia;
pub(crate) mod trivia_pieces_iterator;
pub(crate) mod weak_green_element;
