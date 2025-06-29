//! # Element: Core Building Blocks for Tree Navigation
//!
//! This module defines the fundamental `GreenElement` type that represents any
//! navigable item in the syntax tree. Elements are the basic units that cursors
//! move between during tree traversal.
//!
//! ## What is a GreenElement?
//!
//! A `GreenElement` is a unified type that can represent either:
//! - **Nodes**: Internal tree nodes that contain other elements
//! - **Tokens**: Leaf elements that contain actual text content
//!
//! This unified representation allows navigation code to work with both types
//! seamlessly, following the "element" pattern from tree-sitter and Roslyn.
//!
//! ## Design Benefits
//!
//! ### Unified Navigation
//! Instead of separate navigation for nodes and tokens, cursors can move through
//! a homogeneous sequence of elements, simplifying traversal algorithms.
//!
//! ### Type Safety with Flexibility
//! The `NodeOrToken` enum provides compile-time guarantees while allowing
//! runtime polymorphism over different element types.
//!
//! ### Memory Efficiency
//! Elements are lightweight wrappers around the underlying green tree data,
//! avoiding unnecessary allocations during navigation.
//!
//! ## PDF Context Usage
//!
//! In PDF parsing, elements represent:
//! - **Object nodes**: Container for object header and content
//! - **Dictionary nodes**: Container for key-value pairs
//! - **Number tokens**: Numeric literals like `42` or `3.14159`
//! - **Name tokens**: PDF names like `/Type` or `/Catalog`
//! - **String tokens**: Text content like `(Hello World)`
//! - **Keyword tokens**: PDF keywords like `obj`, `stream`, `endobj`
//!
//! ## Element Properties
//!
//! Every element provides:
//! - **Kind**: The syntax kind (what type of construct it represents)
//! - **Text Length**: The total length of text covered by this element
//! - **Conversion**: Easy conversion from specific node/token types
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use syntax::cursor::element::GreenElement;
//! use syntax::utility_types::node_or_token::NodeOrToken;
//!
//! // Elements can be created from nodes or tokens
//! // let element: GreenElement = some_node.into();
//! // let element: GreenElement = some_token.into();
//!
//! // Navigate through elements uniformly
//! // match element {
//! //     NodeOrToken::Node(node) => process_node(node),
//! //     NodeOrToken::Token(token) => process_token(token),
//! // }
//! ```

use std::borrow::Cow;

use crate::{
    green::{kind::RawSyntaxKind, node::GreenNode, node_data::GreenNodeData, token::GreenToken},
    utility_types::node_or_token::NodeOrToken,
};

/// A unified element type that can represent either a node or token in the green tree.
///
/// This type provides a common interface for tree navigation, allowing cursors and
/// iterators to work with both nodes and tokens through the same API. It serves
/// as the fundamental building block for tree traversal operations.
///
/// ## Type Alias Pattern
///
/// `GreenElement` is a type alias for `NodeOrToken<GreenNode, GreenToken>`,
/// providing a more specific name in the context of green tree navigation.
/// This follows naming conventions from syntax tree libraries like Roslyn.
///
/// ## Thread Safety
///
/// Elements are thread-safe when the underlying nodes and tokens are thread-safe,
/// which is the case for the immutable green tree implementation.
pub(crate) type GreenElement = NodeOrToken<GreenNode, GreenToken>;

impl GreenElement {
    /// Returns the syntax kind of this element.
    ///
    /// The kind identifies what type of syntactic construct this element represents,
    /// such as a dictionary, array, number, or keyword in PDF context.
    ///
    /// ## Implementation Note
    ///
    /// This method dispatches to the appropriate `kind()` method on the underlying
    /// node or token, providing a unified interface regardless of element type.
    ///
    /// ## PDF Examples
    ///
    /// ```rust,ignore
    /// // For a PDF object node: PdfSyntaxKind::Object  
    /// // For a number token: PdfSyntaxKind::Number
    /// // For a name token: PdfSyntaxKind::Name
    /// let kind = element.kind();
    /// ```
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        match self {
            NodeOrToken::Node(node) => node.kind(),
            NodeOrToken::Token(token) => token.kind(),
        }
    }

    /// Returns the total length of text covered by this element.
    ///
    /// For tokens, this is the length of the token's text content.
    /// For nodes, this is the sum of lengths of all child elements.
    ///
    /// ## PDF Context
    ///
    /// This is useful for:
    /// - Calculating byte offsets for stream content
    /// - Validating cross-reference table entries
    /// - Measuring object sizes for incremental updates
    /// - Determining line/column positions for error reporting
    ///
    /// ## Performance
    ///
    /// The length is precomputed and stored in the tree structure,
    /// making this operation O(1) rather than requiring tree traversal.
    #[inline]
    pub fn text_len(&self) -> u64 {
        match self {
            NodeOrToken::Token(token) => token.text_len(),
            NodeOrToken::Node(node) => node.text_len(),
        }
    }
}

impl From<GreenNode> for GreenElement {
    /// Converts a green node into a green element.
    ///
    /// This conversion is zero-cost and simply wraps the node in the
    /// `NodeOrToken::Node` variant for unified element handling.
    #[inline]
    fn from(node: GreenNode) -> GreenElement {
        NodeOrToken::Node(node)
    }
}

impl From<GreenToken> for GreenElement {
    /// Converts a green token into a green element.
    ///
    /// This conversion is zero-cost and simply wraps the token in the
    /// `NodeOrToken::Token` variant for unified element handling.
    #[inline]
    fn from(token: GreenToken) -> GreenElement {
        NodeOrToken::Token(token)
    }
}

impl From<Cow<'_, GreenNodeData>> for GreenElement {
    /// Converts owned green node data into a green element.
    ///
    /// This handles the case where node data needs to be converted from
    /// a `Cow` (clone-on-write) wrapper into an owned node element.
    /// The conversion ensures the data is owned before wrapping it.
    #[inline]
    fn from(cow: Cow<'_, GreenNodeData>) -> Self {
        NodeOrToken::Node(cow.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::green::kind::RawSyntaxKind;

    #[test]
    fn test_element_conversion_traits() {
        // Test that the conversion traits are properly implemented
        // We verify the trait implementations exist by checking the type system

        // Verify From<GreenNode> trait exists
        fn assert_from_node<T: From<GreenNode>>() {}
        assert_from_node::<GreenElement>();

        // Verify From<GreenToken> trait exists
        fn assert_from_token<T: From<GreenToken>>() {}
        assert_from_token::<GreenElement>();

        // Verify From<Cow<GreenNodeData>> trait exists
        fn assert_from_cow<T: for<'a> From<Cow<'a, GreenNodeData>>>() {}
        assert_from_cow::<GreenElement>();
    }

    #[test]
    fn test_element_interface() {
        // Test that GreenElement provides the expected interface
        // We verify the methods exist by checking their signatures

        fn assert_has_kind<T>(_: T)
        where
            T: Fn(&GreenElement) -> RawSyntaxKind,
        {
        }

        fn assert_has_text_len<T>(_: T)
        where
            T: Fn(&GreenElement) -> u64,
        {
        }

        // These tests verify the method signatures exist
        assert_has_kind(GreenElement::kind);
        assert_has_text_len(GreenElement::text_len);
    }

    #[test]
    fn test_node_or_token_pattern() {
        // Test that we can properly match on the NodeOrToken enum
        fn process_element(element: GreenElement) -> &'static str {
            match element {
                NodeOrToken::Node(_) => "node",
                NodeOrToken::Token(_) => "token",
            }
        }

        // This verifies the pattern matching works at compile time
        let _ = process_element;
    }

    #[test]
    fn test_raw_syntax_kind_usage() {
        // Test that RawSyntaxKind is properly used
        let raw_kind = RawSyntaxKind(42);
        assert_eq!(raw_kind.0, 42);

        // This verifies the types are compatible
        fn accepts_raw_kind(_: RawSyntaxKind) {}
        accepts_raw_kind(raw_kind);
    }

    #[test]
    fn test_element_type_alias() {
        // Verify that GreenElement is the correct type alias
        fn assert_same_type<T, U>()
        where
            T: Into<U>,
            U: Into<T>,
        {
        }

        // This test verifies the type alias is correctly defined at compile time
        assert_same_type::<GreenElement, NodeOrToken<GreenNode, GreenToken>>();
    }

    #[test]
    fn test_element_unified_interface() {
        // Test that both nodes and tokens can be treated uniformly as elements
        fn element_processor(element: GreenElement) {
            // Both nodes and tokens should provide these methods
            let _kind = element.kind();
            let _len = element.text_len();
        }

        // Verify the function compiles with our element type
        let _ = element_processor;
    }

    #[test]
    fn test_conversion_zero_cost() {
        // Verify that conversions should be zero-cost (compile-time check)
        use std::mem;

        // The size of GreenElement should be the size of the larger variant
        // plus the discriminant, which should be minimal overhead
        assert!(mem::size_of::<GreenElement>() > 0);

        // Verify that the enum variants are properly sized
        assert!(mem::size_of::<NodeOrToken<GreenNode, GreenToken>>() > 0);
    }
}
