//! # Node Kind: Hierarchical Relationships in the Red-Green Tree
//!
//! This module defines the `NodeKind` enum that distinguishes between root
//! and child nodes in the red-green tree architecture. This distinction is
//! crucial for proper memory management and tree navigation while maintaining
//! the performance benefits of the red-green design.
//!
//! ## Red-Green Tree Hierarchy
//!
//! The red-green tree architecture uses two types of nodes to optimize
//! memory usage and enable incremental parsing:
//!
//! ### Root Nodes
//! - **Strong references**: Hold strong references to green tree elements
//! - **No parent**: Represent top-level syntax elements in a file
//! - **Ownership**: Own their portion of the green tree
//! - **Lifetime**: Live as long as someone holds a reference to them
//!
//! ### Child Nodes  
//! - **Weak references**: Hold weak references to green tree elements
//! - **Parent links**: Maintain strong references to their parent red nodes
//! - **Memory efficiency**: Don't create reference cycles with the green tree
//! - **Navigation**: Enable upward tree traversal through parent links
//!
//! ## Memory Management Benefits
//!
//! This design prevents memory leaks while enabling efficient navigation:
//!
//! ### Reference Cycle Prevention
//! ```text
//! Root Node ←─ strong ──→ Green Tree
//!     ↑                       ↓
//!   strong                  weak
//!     │                      ↓  
//! Child Node ←─ weak ────→ Green Node
//! ```
//! Child nodes can't create cycles because they use weak references to green data.
//!
//! ### Efficient Navigation
//! - **Downward**: Follow green tree structure efficiently
//! - **Upward**: Use parent links in red tree for context
//! - **Sibling**: Navigate through parent to find siblings
//!
//! ## PDF Processing Context
//!
//! In PDF syntax trees:
//! - **Root nodes**: Represent complete PDF objects, top-level dictionaries, or file structures  
//! - **Child nodes**: Represent nested elements like dictionary entries, array items, or sub-structures
//!
//! This hierarchy enables efficient processing of large PDF documents where
//! we might only need to work with specific sections at a time.

use std::rc::Rc;

use crate::cursor::{
    green_element::GreenElement, node_data::NodeData, weak_green_element::WeakGreenElement,
};

/// Represents the hierarchical relationship of a node in the red-green tree.
///
/// `NodeKind` distinguishes between root nodes (which own green tree data)
/// and child nodes (which reference their parents and use weak references
/// to avoid memory cycles). This design is essential for the red-green tree
/// architecture's memory efficiency and incremental parsing capabilities.
///
/// ## Variant Purposes
///
/// ### Root Nodes
/// Used for top-level syntax elements that serve as entry points into
/// the syntax tree. These typically represent:
/// - Complete PDF objects (from `obj` to `endobj`)
/// - Top-level file structures (header, cross-reference table, trailer)
/// - Independent syntactic units that can be parsed separately
///
/// ### Child Nodes
/// Used for nested syntax elements that exist within other structures:
/// - Dictionary key-value pairs within a parent dictionary
/// - Array elements within a parent array  
/// - Sub-components of complex objects (stream dictionaries, content)
/// - Any syntax element that has a structural parent
///
/// ## Memory Safety
///
/// The weak reference design ensures that child nodes don't prevent
/// garbage collection of green tree data while still enabling efficient
/// access to syntax information.
#[derive(Debug)]
pub(crate) enum NodeKind {
    /// A root node that owns green tree data and has no parent.
    ///
    /// Root nodes serve as entry points into syntax trees and hold strong
    /// references to green tree elements. They represent top-level constructs
    /// that can exist independently in a PDF document.
    ///
    /// ## Characteristics
    /// - **Strong ownership**: Holds a strong reference to green tree data
    /// - **No parent**: Represents a top-level syntactic element
    /// - **Memory responsibility**: Keeps the green tree alive while referenced
    /// - **Entry point**: Serves as the starting point for tree traversal
    ///
    /// ## PDF Examples
    /// - Complete PDF objects: `1 0 obj ... endobj`
    /// - File trailer: `trailer << ... >>`
    /// - Cross-reference tables: `xref ... `
    /// - Document header: `%PDF-1.7`
    ///
    /// ## Lifetime
    /// The green tree data remains alive as long as any root node
    /// referencing it exists. When the last root node is dropped,
    /// the associated green tree can be garbage collected.
    #[allow(dead_code)] // Variants used by tree construction, may not be accessed directly yet
    Root {
        /// Strong reference to the green tree element this root owns.
        green: GreenElement,
    },
    
    /// A child node that references its parent and weakly references green data.
    ///
    /// Child nodes represent nested syntax elements within larger structures.
    /// They maintain parent relationships for upward navigation while using
    /// weak references to prevent memory cycles with the green tree.
    ///
    /// ## Characteristics
    /// - **Parent relationship**: Strong reference to parent red node for navigation
    /// - **Weak green reference**: Prevents reference cycles while allowing access
    /// - **Context preservation**: Maintains hierarchical relationships for analysis
    /// - **Memory efficiency**: Doesn't prevent garbage collection of unused green data
    ///
    /// ## PDF Examples
    /// - Dictionary entries: `/Type` or `/Catalog` within `<</Type/Catalog>>`
    /// - Array elements: Individual items within `[item1 item2 item3]`
    /// - Object components: The `obj` keyword within `1 0 obj ... endobj`
    /// - Stream parts: Content within a stream object
    ///
    /// ## Navigation Benefits
    /// Child nodes enable efficient tree navigation:
    /// - **Upward**: Follow parent links to find containing structures
    /// - **Sibling**: Navigate through parent to find related elements
    /// - **Context**: Access parent information for semantic analysis
    #[allow(dead_code)] // Variants used by tree construction, may not be accessed directly yet
    Child {
        /// Weak reference to the corresponding green tree element.
        /// Prevents reference cycles while allowing access to syntax data.
        green: WeakGreenElement,
        
        /// Strong reference to the parent red node.
        /// Enables upward navigation and context preservation.
        parent: Rc<NodeData>,
    },
}
