//! # AST Node Trait: Foundation for Typed Syntax Tree Access
//!
//! This module defines the core trait that enables zero-cost conversion between
//! untyped syntax nodes and typed AST nodes in the PDF compiler.
//!
//! ## The AstNode Trait
//!
//! The `AstNode` trait is the foundation of the typed AST layer. It provides:
//!
//! ### Zero-Cost Conversion
//! - Conversion from untyped `SyntaxNode` to typed AST has no runtime cost
//! - AST nodes and syntax nodes have identical memory representation
//! - Only a pointer to the tree root and a pointer to the node itself
//!
//! ### Type Safety
//! - Each AST node type corresponds to a specific PDF syntax construct
//! - Compile-time guarantees about the structure and content of nodes
//! - Language-specific operations that understand PDF semantics
//!
//! ## PDF Language Integration
//!
//! The trait is parameterized by a `Language` type that defines:
//! - Syntax kinds specific to PDF (e.g., Dictionary, Array, Stream)
//! - Trivia handling rules for PDF's whitespace-sensitive syntax
//! - Validation rules based on ISO 32000-2 specification
//!
//! ## Implementation Pattern
//!
//! Future PDF-specific AST nodes will implement this trait like:
//!
//! ```rust,no_run
//! use syntax::ast::AstNode;
//!
//! // Example of how a PDF dictionary AST node would be implemented:
//! // #[derive(Clone)]
//! // pub struct DictionaryNode {
//! //     syntax: SyntaxNode<PdfLanguage>,
//! // }
//! //
//! // impl AstNode for DictionaryNode {
//! //     type Language = PdfLanguage;
//! // }
//! ```
//!
//! ## Memory Efficiency
//!
//! The trait design ensures minimal memory overhead:
//! - No additional heap allocations for type conversion
//! - Reuses the same reference-counted syntax tree data
//! - Maintains thread safety through the underlying green tree

use crate::syntax::language::Language;

/// The main trait to go from untyped `SyntaxNode` to a typed AST node.
///
/// ## Zero-Cost Abstraction
///
/// The conversion itself has zero runtime cost: AST and syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
///
/// ## Usage Pattern
///
/// This trait is implemented by all typed AST node types in the PDF compiler.
/// Each implementation corresponds to a specific PDF syntax construct and provides
/// type-safe operations for working with that construct.
///
/// ## Generic Language Parameter
///
/// The `Language` associated type allows the trait to work with different
/// language definitions. For the PDF compiler, this will be `PdfLanguage`
/// which defines PDF-specific syntax kinds and rules.
///
/// ## Thread Safety
///
/// AST nodes are thread-safe when the underlying language implementation
/// is thread-safe, which is typically the case for immutable syntax definitions.
pub trait AstNode: Clone {
    /// The language definition that this AST node is associated with.
    ///
    /// This type parameter allows the AST system to work with different
    /// language implementations while maintaining type safety. For PDF,
    /// this will be the `PdfLanguage` type that defines PDF-specific
    /// syntax kinds, trivia rules, and validation requirements.
    type Language: Language;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::green::kind::RawSyntaxKind;
    use crate::syntax::kind::SyntaxKind;

    /// Test syntax kind for testing
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum TestSyntaxKind {
        Root,
        TestNode,
        Eof,
    }

    impl SyntaxKind for TestSyntaxKind {
        const EOF: Self = TestSyntaxKind::Eof;

        fn to_raw(&self) -> RawSyntaxKind {
            RawSyntaxKind(match self {
                TestSyntaxKind::Root => 0,
                TestSyntaxKind::TestNode => 1,
                TestSyntaxKind::Eof => 2,
            })
        }

        fn from_raw(raw: RawSyntaxKind) -> Self {
            match raw.0 {
                0 => TestSyntaxKind::Root,
                1 => TestSyntaxKind::TestNode,
                2 => TestSyntaxKind::Eof,
                _ => TestSyntaxKind::Eof,
            }
        }

        fn is_root(&self) -> bool {
            matches!(self, TestSyntaxKind::Root)
        }

        fn is_list(&self) -> bool {
            false
        }

        fn to_string(&self) -> Option<&'static str> {
            match self {
                TestSyntaxKind::Root => Some("ROOT"),
                TestSyntaxKind::TestNode => Some("TEST"),
                TestSyntaxKind::Eof => Some("EOF"),
            }
        }
    }

    /// Test language implementation for testing the AstNode trait
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct TestLanguage;

    /// Example AST root node for testing
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestRootNode;

    impl AstNode for TestRootNode {
        type Language = TestLanguage;
    }

    impl Language for TestLanguage {
        type Kind = TestSyntaxKind;
        type Root = TestRootNode;
    }

    /// Example AST node implementation for testing
    #[derive(Clone)]
    struct TestAstNode;

    impl AstNode for TestAstNode {
        type Language = TestLanguage;
    }

    #[test]
    fn test_ast_node_trait_implementation() {
        // Test that we can create an AST node
        let node = TestAstNode;
        let _cloned = node.clone();

        // Verify the language type is correct
        fn assert_language_type<T: AstNode<Language = TestLanguage>>(_: T) {}
        assert_language_type(TestAstNode);
    }

    #[test]
    fn test_ast_node_clone() {
        let node = TestAstNode;
        let cloned = node.clone();

        // Both should be valid instances
        fn assert_ast_node<T: AstNode>(_: T) {}
        assert_ast_node(node);
        assert_ast_node(cloned);
    }

    #[test]
    fn test_language_association() {
        // Test that the language type is properly associated
        type NodeLanguage = <TestAstNode as AstNode>::Language;
        let _lang: NodeLanguage = TestLanguage;
    }

    #[test]
    fn test_root_node_implementation() {
        let root = TestRootNode;
        let _cloned = root.clone();

        // Verify it implements the required traits
        assert_eq!(root, root.clone());
        println!("{:?}", root); // Test Debug implementation
    }

    #[test]
    fn test_language_kinds() {
        // Test that our test language implementation works
        assert_eq!(TestSyntaxKind::EOF, TestSyntaxKind::Eof);
        assert!(TestSyntaxKind::Root.is_root());
        assert!(!TestSyntaxKind::TestNode.is_root());
        assert!(!TestSyntaxKind::TestNode.is_list());

        // Test string representations
        assert_eq!(TestSyntaxKind::Root.to_string(), Some("ROOT"));
        assert_eq!(TestSyntaxKind::TestNode.to_string(), Some("TEST"));
        assert_eq!(TestSyntaxKind::Eof.to_string(), Some("EOF"));
    }

    #[test]
    fn test_syntax_kind_raw_conversion() {
        // Test round-trip conversion
        let kinds = [
            TestSyntaxKind::Root,
            TestSyntaxKind::TestNode,
            TestSyntaxKind::Eof,
        ];

        for kind in kinds {
            let raw = kind.to_raw();
            let converted_back = TestSyntaxKind::from_raw(raw);
            assert_eq!(kind, converted_back);
        }
    }
}
