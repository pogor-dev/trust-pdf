//! # Green Element: Unified Interface for Tree Navigation
//!
//! This module provides a unified interface for working with green tree elements,
//! abstracting over the differences between nodes (internal tree elements) and
//! tokens (leaf elements with text content). This abstraction enables generic
//! tree traversal and manipulation operations.
//!
//! ## What is a Green Element?
//!
//! A green element is either:
//! - **Node**: An internal tree element that contains child elements
//! - **Token**: A leaf element that contains actual text content
//!
//! This abstraction allows code to work with tree elements generically
//! without needing to know whether they're dealing with a node or token.
//!
//! ## Design Benefits
//!
//! ### Unified Navigation
//! Tree traversal algorithms can work with any element type:
//! ```rust,ignore
//! fn traverse(element: GreenElementRef) {
//!     // Works for both nodes and tokens
//!     println!("Element kind: {:?}", element.kind());
//!     println!("Text length: {}", element.text_len());
//! }
//! ```
//!
//! ### Type Safety
//! The `NodeOrToken` wrapper maintains type information while providing
//! common operations, preventing runtime type confusion.
//!
//! ### Performance
//! Zero-cost abstraction that compiles to the same code as direct
//! match statements, with no runtime overhead.
//!
//! ## PDF Processing Context
//!
//! In PDF syntax trees, elements represent:
//! - **Nodes**: Complex structures like dictionaries, arrays, objects
//! - **Tokens**: Simple values like numbers, strings, keywords, operators
//!
//! The unified interface enables generic operations like:
//! - Calculating total text spans for cross-reference generation
//! - Walking trees for syntax highlighting or error detection
//! - Implementing visitor patterns for semantic analysis
//!
//! ## Memory Management
//!
//! The module provides both owned and borrowed variants:
//! - **`GreenElement`**: Owned references with reference counting
//! - **`GreenElementRef`**: Borrowed references for temporary access

use crate::{
    green::{
        kind::RawSyntaxKind, node::GreenNode, node_data::GreenNodeData, token::GreenToken,
        token_data::GreenTokenData,
    },
    utility_types::node_or_token::NodeOrToken,
};

/// Owned reference to either a green node or green token.
///
/// This type alias provides a convenient way to work with green tree elements
/// when you need owned references. Both nodes and tokens use reference counting,
/// so cloning is efficient (just increments a counter).
///
/// ## Use Cases
/// - **Tree Construction**: Building new syntax trees during parsing
/// - **Long-term Storage**: Keeping references beyond the lifetime of source data
/// - **Concurrent Access**: Sharing tree elements across threads
pub(crate) type GreenElement = NodeOrToken<GreenNode, GreenToken>;

/// Borrowed reference to either green node data or green token data.
///
/// This type alias provides efficient temporary access to green tree elements
/// without affecting reference counts. Ideal for short-lived operations that
/// don't need to retain ownership.
///
/// ## Use Cases
/// - **Tree Traversal**: Walking the tree for analysis or transformation
/// - **Property Access**: Reading node/token properties without ownership
/// - **Performance-Critical Code**: Avoiding reference count operations
pub(crate) type GreenElementRef<'a> = NodeOrToken<&'a GreenNodeData, &'a GreenTokenData>;

impl GreenElementRef<'_> {
    /// Returns the syntax kind of this element.
    ///
    /// This method provides unified access to the syntax kind regardless of
    /// whether the element is a node or token. The kind identifies what type
    /// of PDF syntax construct this element represents.
    ///
    /// ## PDF Examples
    ///
    /// Different elements return different kinds:
    /// - Dictionary node: Returns dictionary syntax kind
    /// - Name token `/Type`: Returns name syntax kind  
    /// - Number token `42`: Returns number syntax kind
    /// - Keyword token `obj`: Returns keyword syntax kind
    ///
    /// ## Performance
    ///
    /// This is a zero-cost abstraction that compiles to a direct field access
    /// after the match is optimized away.
    #[allow(dead_code)] // Used by tree navigation, may not be called directly yet
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    /// Returns the total text length covered by this element.
    ///
    /// This method provides unified access to text length calculation:
    /// - **For tokens**: Returns the length of the token's text content
    /// - **For nodes**: Returns the total length including all descendant text
    ///
    /// ## PDF Context
    ///
    /// Text length is critical for PDF processing because:
    /// - **Cross-reference tables**: Require exact byte offsets to objects
    /// - **Stream processing**: Need precise length calculations for content
    /// - **Incremental updates**: Must know how much content to preserve/replace
    /// - **Memory management**: Helps estimate memory requirements for large files
    ///
    /// ## Examples
    ///
    /// - String token `(Hello)`: Returns 7 (including parentheses)
    /// - Dictionary node `<</Type/Catalog>>`: Returns total length of all content
    /// - Number token `123.45`: Returns 6 (exact character count)
    ///
    /// ## Performance
    ///
    /// For tokens, this is O(1). For nodes, the length is pre-calculated and
    /// cached during tree construction, so this is also O(1).
    #[inline]
    pub fn text_len(self) -> u64 {
        match self {
            NodeOrToken::Node(it) => it.text_len(),
            NodeOrToken::Token(it) => it.text_len(),
        }
    }
}
