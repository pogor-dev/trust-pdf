//! # NodeOrToken: Universal Either-Type for Tree Elements
//!
//! This module provides the `NodeOrToken<N, T>` enum, a fundamental building block
//! for representing heterogeneous tree structures where elements can be either
//! internal nodes or leaf tokens.
//!
//! ## The Either-Type Pattern
//!
//! `NodeOrToken` implements the "either-type" pattern, allowing code to work with
//! values that could be one of two types. This is essential in syntax trees where:
//! - **Nodes** represent structural elements that contain other elements
//! - **Tokens** represent atomic leaf elements containing text
//!
//! ## Design Benefits
//!
//! ### Unified Processing
//! Instead of requiring separate code paths for nodes and tokens, algorithms can
//! process mixed sequences of elements uniformly, switching behavior based on
//! the actual type at runtime.
//!
//! ### Type Safety
//! The enum provides compile-time guarantees about the possible types while
//! allowing runtime polymorphism. Pattern matching ensures all cases are handled.
//!
//! ### Memory Efficiency
//! The enum uses a discriminant tag to identify which variant is active,
//! avoiding the overhead of trait objects or boxing.
//!
//! ## PDF Syntax Context
//!
//! In PDF parsing, `NodeOrToken` handles mixed content such as:
//!
//! ### Object Content
//! PDF objects can contain both structural elements (dictionaries, arrays)
//! and primitive values (numbers, strings, names):
//! ```pdf
//! 1 0 obj
//! <<
//!   /Type /Catalog
//!   /Pages 2 0 R
//!   /Version /1.7
//! >>
//! endobj
//! ```
//! The dictionary contains both name tokens (`/Type`, `/Pages`) and
/// reference nodes (`2 0 R`).
///
/// ### Content Streams
/// Page content streams mix operators (tokens) with operands (which can be
/// nodes or tokens):
/// ```pdf
/// BT
/// /F1 12 Tf
/// 100 700 Td
/// (Hello World) Tj
/// ET
/// ```
///
/// ## Generic Parameters
///
/// - `N`: The node type (typically implementing node-like behavior)
/// - `T`: The token type (typically implementing token-like behavior)
///
/// ## Method Categories
///
/// ### Conversion Methods
/// - `into_node()`: Extract node if present, otherwise `None`
/// - `into_token()`: Extract token if present, otherwise `None`
/// - `as_node()`: Borrow node if present, otherwise `None`
/// - `as_token()`: Borrow token if present, otherwise `None`
///
/// ### Utility Methods
/// - `as_deref()`: Convert to borrowed references for both variants
/// - `fmt::Display`: Format either variant using their `Display` implementations
///
/// ## Thread Safety
///
/// `NodeOrToken<N, T>` is thread-safe when both `N` and `T` are thread-safe,
/// making it suitable for concurrent tree processing.

use std::{fmt, ops::Deref};

/// A sum type representing either a node or a token in a syntax tree.
///
/// This enum provides a unified interface for working with heterogeneous tree
/// elements, allowing algorithms to process sequences of mixed node and token
/// types without requiring separate handling for each type.
///
/// ## Variants
///
/// - `Node(N)`: Contains a node value of type `N`
/// - `Token(T)`: Contains a token value of type `T`
///
/// ## Design Rationale
///
/// Syntax trees naturally contain both structural elements (nodes) and atomic
/// content (tokens). Rather than requiring separate collection types or complex
/// trait hierarchies, this enum provides a simple, efficient way to represent
/// mixed content.
///
/// ## Performance Characteristics
///
/// - **Size**: `size_of::<NodeOrToken<N, T>>() == size_of::<N>().max(size_of::<T>()) + discriminant`
/// - **Alignment**: Aligned to the stricter requirement of `N` or `T`
/// - **Pattern matching**: Compiles to efficient jump tables or branch instructions
///
/// ## Common Usage Patterns
///
/// ```rust,no_run
/// use syntax::utility_types::node_or_token::NodeOrToken;
/// 
/// // Processing mixed content
/// fn process_elements(elements: Vec<NodeOrToken<MyNode, MyToken>>) {
///     for element in elements {
///         match element {
///             NodeOrToken::Node(node) => {
///                 // Handle structural element
///                 println!("Node: {}", node.kind());
///             }
///             NodeOrToken::Token(token) => {
///                 // Handle atomic content
///                 println!("Token: {}", token.text());
///             }
///         }
///     }
/// }
/// 
/// // Extracting specific types
/// fn extract_nodes(elements: Vec<NodeOrToken<MyNode, MyToken>>) -> Vec<MyNode> {
///     elements.into_iter()
///         .filter_map(|e| e.into_node())
///         .collect()
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeOrToken<N, T> {
    /// A structural node containing other elements.
    ///
    /// Nodes represent non-terminal elements in the syntax tree that contain
    /// child elements. In PDF context, these might be objects, dictionaries,
    /// arrays, or other compound structures.
    Node(N),
    
    /// An atomic token containing text content.
    ///
    /// Tokens represent terminal elements in the syntax tree that contain
    /// actual text data. In PDF context, these might be numbers, strings,
    /// names, keywords, or other primitive values.
    Token(T),
}

impl<N, T> NodeOrToken<N, T> {
    /// Converts the enum into a node if it contains one, otherwise returns `None`.
    ///
    /// This method consumes the enum and returns the contained node value if
    /// the enum is the `Node` variant, or `None` if it's the `Token` variant.
    ///
    /// ## Usage
    ///
    /// This is useful for extracting nodes from mixed collections:
    /// ```rust,no_run
    /// # use syntax::utility_types::node_or_token::NodeOrToken;
    /// # type MyNode = i32; type MyToken = String;
    /// let elements: Vec<NodeOrToken<MyNode, MyToken>> = vec![
    ///     NodeOrToken::Node(42),
    ///     NodeOrToken::Token("hello".to_string()),
    /// ];
    /// 
    /// let nodes: Vec<MyNode> = elements.into_iter()
    ///     .filter_map(|e| e.into_node())
    ///     .collect();
    /// assert_eq!(nodes, vec![42]);
    /// ```
    pub fn into_node(self) -> Option<N> {
        match self {
            NodeOrToken::Node(node) => Some(node),
            NodeOrToken::Token(_) => None,
        }
    }

    /// Converts the enum into a token if it contains one, otherwise returns `None`.
    ///
    /// This method consumes the enum and returns the contained token value if
    /// the enum is the `Token` variant, or `None` if it's the `Node` variant.
    ///
    /// ## Usage
    ///
    /// This is useful for extracting tokens from mixed collections:
    /// ```rust,no_run
    /// # use syntax::utility_types::node_or_token::NodeOrToken;
    /// # type MyNode = i32; type MyToken = String;
    /// let elements: Vec<NodeOrToken<MyNode, MyToken>> = vec![
    ///     NodeOrToken::Node(42),
    ///     NodeOrToken::Token("hello".to_string()),
    /// ];
    /// 
    /// let tokens: Vec<MyToken> = elements.into_iter()
    ///     .filter_map(|e| e.into_token())
    ///     .collect();
    /// assert_eq!(tokens, vec!["hello".to_string()]);
    /// ```
    pub fn into_token(self) -> Option<T> {
        match self {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(token) => Some(token),
        }
    }

    /// Returns a reference to the contained node if present, otherwise returns `None`.
    ///
    /// This method borrows the enum and returns a reference to the contained
    /// node value if the enum is the `Node` variant, or `None` if it's the `Token` variant.
    ///
    /// ## Usage
    ///
    /// This is useful for inspecting nodes without consuming the enum:
    /// ```rust,no_run
    /// # use syntax::utility_types::node_or_token::NodeOrToken;
    /// # type MyNode = i32; type MyToken = String;
    /// let element = NodeOrToken::Node(42);
    /// 
    /// if let Some(node_ref) = element.as_node() {
    ///     println!("Node value: {}", node_ref);
    /// }
    /// // element is still available for use
    /// ```
    pub fn as_node(&self) -> Option<&N> {
        match self {
            NodeOrToken::Node(node) => Some(node),
            NodeOrToken::Token(_) => None,
        }
    }

    /// Returns a reference to the contained token if present, otherwise returns `None`.
    ///
    /// This method borrows the enum and returns a reference to the contained
    /// token value if the enum is the `Token` variant, or `None` if it's the `Node` variant.
    ///
    /// ## Usage
    ///
    /// This is useful for inspecting tokens without consuming the enum:
    /// ```rust,no_run
    /// # use syntax::utility_types::node_or_token::NodeOrToken;
    /// # type MyNode = i32; type MyToken = String;
    /// let element = NodeOrToken::Token("hello".to_string());
    /// 
    /// if let Some(token_ref) = element.as_token() {
    ///     println!("Token value: {}", token_ref);
    /// }
    /// // element is still available for use
    /// ```
    pub fn as_token(&self) -> Option<&T> {
        match self {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(token) => Some(token),
        }
    }
}

impl<N: Deref, T: Deref> NodeOrToken<N, T> {
    /// Converts the enum to use borrowed references of the dereferenced values.
    ///
    /// This method is useful when working with smart pointer types like `Arc<T>` or `Rc<T>`,
    /// allowing you to convert `NodeOrToken<Arc<Node>, Arc<Token>>` to 
    /// `NodeOrToken<&Node, &Token>` for more efficient processing.
    ///
    /// ## Generic Constraints
    ///
    /// Both `N` and `T` must implement `Deref`, typically because they are
    /// smart pointers, references, or other types that can be dereferenced.
    ///
    /// ## Usage
    ///
    /// ```rust,no_run
    /// # use syntax::utility_types::node_or_token::NodeOrToken;
    /// # use std::sync::Arc;
    /// # type MyNode = i32; type MyToken = String;
    /// let element: NodeOrToken<Arc<MyNode>, Arc<MyToken>> = 
    ///     NodeOrToken::Node(Arc::new(42));
    /// 
    /// let borrowed: NodeOrToken<&MyNode, &MyToken> = element.as_deref();
    /// // Now we can work with borrowed references instead of owned values
    /// ```
    pub(crate) fn as_deref(&self) -> NodeOrToken<&N::Target, &T::Target> {
        match self {
            NodeOrToken::Node(node) => NodeOrToken::Node(&**node),
            NodeOrToken::Token(token) => NodeOrToken::Token(&**token),
        }
    }
}

impl<N: fmt::Display, T: fmt::Display> fmt::Display for NodeOrToken<N, T> {
    /// Formats the contained value using its `Display` implementation.
    ///
    /// This allows `NodeOrToken` to be formatted for output, delegating to
    /// the appropriate `Display` implementation of the contained node or token.
    ///
    /// ## Behavior
    ///
    /// - For `Node(n)`: Calls `fmt::Display::fmt(&n, f)`
    /// - For `Token(t)`: Calls `fmt::Display::fmt(&t, f)`
    ///
    /// ## Usage
    ///
    /// ```rust,no_run
    /// # use syntax::utility_types::node_or_token::NodeOrToken;
    /// # type MyNode = i32; type MyToken = String;
    /// let node_element = NodeOrToken::Node(42);
    /// let token_element = NodeOrToken::Token("hello".to_string());
    /// 
    /// println!("Node: {}", node_element);   // Prints: Node: 42
    /// println!("Token: {}", token_element); // Prints: Token: hello
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeOrToken::Node(node) => fmt::Display::fmt(node, f),
            NodeOrToken::Token(token) => fmt::Display::fmt(token, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_node_creation_and_extraction() {
        let element: NodeOrToken<i32, String> = NodeOrToken::Node(42);
        
        // Test into_node
        assert_eq!(element.into_node(), Some(42));
        
        // Test as_node
        let element: NodeOrToken<i32, String> = NodeOrToken::Node(42);
        assert_eq!(element.as_node(), Some(&42));
        
        // Test into_token returns None for node
        let element: NodeOrToken<i32, String> = NodeOrToken::Node(42);
        assert_eq!(element.into_token(), None::<String>);
        
        // Test as_token returns None for node
        let element: NodeOrToken<i32, String> = NodeOrToken::Node(42);
        assert_eq!(element.as_token(), None::<&String>);
    }

    #[test]
    fn test_token_creation_and_extraction() {
        let element: NodeOrToken<i32, String> = NodeOrToken::Token("hello".to_string());
        
        // Test into_token
        assert_eq!(element.into_token(), Some("hello".to_string()));
        
        // Test as_token
        let element: NodeOrToken<i32, String> = NodeOrToken::Token("hello".to_string());
        assert_eq!(element.as_token(), Some(&"hello".to_string()));
        
        // Test into_node returns None for token
        let element: NodeOrToken<i32, String> = NodeOrToken::Token("hello".to_string());
        assert_eq!(element.into_node(), None::<i32>);
        
        // Test as_node returns None for token
        let element: NodeOrToken<i32, String> = NodeOrToken::Token("hello".to_string());
        assert_eq!(element.as_node(), None::<&i32>);
    }

    #[test]
    fn test_pattern_matching() {
        let node_element: NodeOrToken<i32, String> = NodeOrToken::Node(42);
        let token_element: NodeOrToken<i32, String> = NodeOrToken::Token("hello".to_string());
        
        // Test matching on node
        match node_element {
            NodeOrToken::Node(n) => assert_eq!(n, 42),
            NodeOrToken::Token(_) => panic!("Expected node"),
        }
        
        // Test matching on token
        match token_element {
            NodeOrToken::Node(_) => panic!("Expected token"),
            NodeOrToken::Token(t) => assert_eq!(t, "hello"),
        }
    }

    #[test]
    fn test_derive_traits() {
        let node1: NodeOrToken<i32, String> = NodeOrToken::Node(42);
        let node2: NodeOrToken<i32, String> = NodeOrToken::Node(42);
        let node3: NodeOrToken<i32, String> = NodeOrToken::Node(43);
        let token1: NodeOrToken<i32, String> = NodeOrToken::Token("hello".to_string());
        
        // Test PartialEq
        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
        assert_ne!(node1, token1);
        
        // Test Clone
        let cloned = node1.clone();
        assert_eq!(node1, cloned);
        
        // Test Debug
        let debug_str = format!("{:?}", node1);
        assert!(debug_str.contains("Node"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_display_implementation() {
        let node_element: NodeOrToken<i32, String> = NodeOrToken::Node(42);
        let token_element: NodeOrToken<i32, String> = NodeOrToken::Token("hello".to_string());
        
        assert_eq!(format!("{}", node_element), "42");
        assert_eq!(format!("{}", token_element), "hello");
    }

    #[test]  
    fn test_as_deref() {
        let node_element: NodeOrToken<Arc<i32>, Arc<String>> = NodeOrToken::Node(Arc::new(42));
        let token_element: NodeOrToken<Arc<i32>, Arc<String>> = NodeOrToken::Token(Arc::new("hello".to_string()));
        
        // Test as_deref for node
        match node_element.as_deref() {
            NodeOrToken::Node(n) => assert_eq!(*n, 42),
            NodeOrToken::Token(_) => panic!("Expected node"),
        }
        
        // Test as_deref for token
        match token_element.as_deref() {
            NodeOrToken::Node(_) => panic!("Expected token"),
            NodeOrToken::Token(t) => assert_eq!(*t, "hello"),
        }
    }

    #[test]
    fn test_filtering_collections() {
        let elements = vec![
            NodeOrToken::Node(1),
            NodeOrToken::Token("a".to_string()),
            NodeOrToken::Node(2),
            NodeOrToken::Token("b".to_string()),
            NodeOrToken::Node(3),
        ];
        
        // Filter nodes
        let nodes: Vec<i32> = elements.iter()
            .filter_map(|e| e.as_node())
            .cloned()
            .collect();
        assert_eq!(nodes, vec![1, 2, 3]);
        
        // Filter tokens
        let tokens: Vec<String> = elements.iter()
            .filter_map(|e| e.as_token())
            .cloned()
            .collect();
        assert_eq!(tokens, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_memory_efficiency() {
        use std::mem;
        
        // Verify enum size is reasonable
        assert!(mem::size_of::<NodeOrToken<i32, String>>() > 0);
        assert!(mem::size_of::<NodeOrToken<i32, String>>() >= mem::size_of::<String>());
        
        // Verify alignment
        assert!(mem::align_of::<NodeOrToken<i32, String>>() > 0);
    }

    #[test]
    fn test_conversion_chains() {
        let elements = vec![
            NodeOrToken::Node(1),
            NodeOrToken::Token("hello".to_string()),
            NodeOrToken::Node(2),
        ];
        
        // Test chaining conversions
        let result: Vec<i32> = elements
            .into_iter()
            .filter_map(|e| e.into_node())
            .collect();
        
        assert_eq!(result, vec![1, 2]);
    }
}
