//! # Language: Document Format Definition Trait
//!
//! This module defines the `Language` trait, which provides a complete specification
//! for a document format or programming language that can be parsed using the
//! syntax tree infrastructure. This follows the "language server" pattern where
//! multiple document formats can share the same parsing and tree manipulation code.
//!
//! ## Design Goals
//!
//! The `Language` trait enables:
//! 1. **Multi-format Support**: Same infrastructure for PDF, XML, JSON, etc.
//! 2. **Type Safety**: Each language has its own strongly-typed AST nodes
//! 3. **Extensibility**: New languages can be added without changing core code
//! 4. **IDE Integration**: Language-specific features like highlighting, completion
//!
//! ## PDF Language Implementation
//!
//! For PDF documents, a language implementation would define:
//! - **Syntax Kinds**: Object, Dictionary, Array, Stream, etc.
//! - **Root Node**: Complete PDF document with header, body, xref, trailer
//! - **Parsing Rules**: PDF-specific grammar and error recovery
//! - **Validation**: ISO 32000-2 compliance checking
//!
//! ## Usage in Compiler Pipeline
//!
//! The language trait connects several compiler phases:
//! 1. **Lexing**: Tokenize input according to language rules
//! 2. **Parsing**: Build concrete syntax tree using language grammar  
//! 3. **AST**: Convert to typed abstract syntax tree
//! 4. **Analysis**: Perform language-specific semantic analysis
//! 5. **Generation**: Output language-specific code or data
//!
//! ## Example Implementation
//!
//! ```rust,no_run
//! use syntax::{syntax::{kind::SyntaxKind, language::Language}, ast::ast_node::AstNode};
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
//! enum PdfLanguage {}
//!
//! #[derive(Debug, Clone, Copy, PartialEq)]
//! enum PdfSyntaxKind {
//!     Document, Object, Dictionary, Array, Stream, // ... more kinds
//! }
//!
//! #[derive(Debug, Clone, PartialEq)]
//! struct PdfDocument {
//!     // ... fields
//! }
//!
//! impl Language for PdfLanguage {
//!     type Kind = PdfSyntaxKind;
//!     type Root = PdfDocument;
//! }
//! ```

use std::fmt;

use crate::{ast::ast_node::AstNode, syntax::kind::SyntaxKind};

/// Trait defining a complete document format or programming language specification.
///
/// This trait connects the language-agnostic syntax tree infrastructure with
/// specific document formats. It provides the type-level information needed
/// to parse, analyze, and manipulate documents in a type-safe manner.
///
/// ## Associated Types
///
/// - `Kind`: The syntax kind enum for this language (tokens and node types)
/// - `Root`: The top-level AST node representing a complete document
///
/// ## Trait Bounds Explained
///
/// The trait requires several bounds that enable different use cases:
///
/// - `Sized`: The language type itself must have a known size at compile time
/// - `Clone + Copy`: Languages should be lightweight, copyable identifiers
/// - `fmt::Debug`: Essential for debugging and development tools
/// - `Eq + Ord`: Enables languages to be used as keys in collections
/// - `std::hash::Hash`: Allows languages to be hashed for efficient lookups
///
/// ## Thread Safety
///
/// Language implementations should be thread-safe since syntax trees and
/// parsing operations may occur across multiple threads in IDE scenarios.
///
/// ## Lifetime Considerations
///
/// Language types typically have no lifetime parameters and own all their data.
/// This makes them suitable for caching and long-term storage scenarios.
pub trait Language: Sized + Clone + Copy + fmt::Debug + Eq + Ord + std::hash::Hash {
    /// The syntax kind enumeration for this language.
    ///
    /// This defines all possible node and token types that can appear in
    /// documents of this language. Each language has its own unique set
    /// of syntax kinds tailored to its grammatical structure.
    ///
    /// # PDF Context
    /// For PDF, this would include kinds like:
    /// - **Structural**: Object, Dictionary, Array, Stream
    /// - **Primitive**: Number, String, Name, Boolean, Null
    /// - **Reference**: IndirectObject, ObjectReference
    /// - **Organizational**: XrefTable, Trailer, Header
    /// - **Content**: ContentStream, Operator, Operand
    ///
    /// # Requirements
    /// The `Kind` type must implement `SyntaxKind`, providing:
    /// - Conversion to/from raw representations
    /// - Classification of special node types (root, list)
    /// - String representation for keywords
    type Kind: SyntaxKind;

    /// The root AST node type for complete documents in this language.
    ///
    /// This represents a fully parsed document and serves as the entry point
    /// for semantic analysis, validation, and code generation. It should
    /// contain all the information needed to reconstruct the original document.
    ///
    /// # PDF Context
    /// For PDF, the root node would represent a complete PDF document containing:
    /// - File header (`%PDF-1.7`)
    /// - Object collection (indirect objects with content)
    /// - Cross-reference table (object locations)
    /// - Trailer dictionary (document metadata)
    /// - EOF marker
    ///
    /// # Requirements
    /// The `Root` type must be:
    /// - `AstNode<Language = Self>`: Properly connected to this language
    /// - `Clone`: Supports tree duplication and caching
    /// - `Eq`: Enables structural comparisons
    /// - `fmt::Debug`: Essential for debugging and development
    ///
    /// # Design Considerations
    /// The root node should:
    /// - Provide access to all major document sections
    /// - Support incremental updates and partial parsing
    /// - Enable efficient navigation to child elements
    /// - Maintain source location information for error reporting
    type Root: AstNode<Language = Self> + Clone + Eq + fmt::Debug;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test language implementation
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct TestLanguage;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum TestSyntaxKind {
        Root,
        Object,
        Dictionary,
        Array,
        Eof,
    }

    impl SyntaxKind for TestSyntaxKind {
        const EOF: Self = TestSyntaxKind::Eof;

        fn to_raw(&self) -> crate::green::kind::RawSyntaxKind {
            crate::green::kind::RawSyntaxKind(*self as u16)
        }

        fn from_raw(raw: crate::green::kind::RawSyntaxKind) -> Self {
            match raw.0 {
                0 => TestSyntaxKind::Root,
                1 => TestSyntaxKind::Object,
                2 => TestSyntaxKind::Dictionary,
                3 => TestSyntaxKind::Array,
                4 => TestSyntaxKind::Eof,
                _ => TestSyntaxKind::Eof,
            }
        }

        fn is_root(&self) -> bool {
            matches!(self, TestSyntaxKind::Root)
        }

        fn is_list(&self) -> bool {
            matches!(self, TestSyntaxKind::Array)
        }

        fn to_string(&self) -> Option<&'static str> {
            match self {
                TestSyntaxKind::Object => Some("obj"),
                _ => None,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestDocument {
        content: String,
    }

    impl AstNode for TestDocument {
        type Language = TestLanguage;
    }

    impl Language for TestLanguage {
        type Kind = TestSyntaxKind;
        type Root = TestDocument;
    }

    #[test]
    fn test_language_trait_implementation() {
        // Test that our language implementation compiles and works
        let _language = TestLanguage;
        
        // Test that associated types work correctly
        let kind = TestSyntaxKind::Object;
        assert_eq!(kind.to_string(), Some("obj"));
        assert!(!kind.is_root());
        
        let root_kind = TestSyntaxKind::Root;
        assert!(root_kind.is_root());
        
        let array_kind = TestSyntaxKind::Array;
        assert!(array_kind.is_list());
    }

    #[test]
    fn test_language_trait_bounds() {
        // Test that language can be cloned and copied
        let lang1 = TestLanguage;
        let lang2 = lang1;
        let lang3 = lang1.clone();
        
        // All should be equal
        assert_eq!(lang1, lang2);
        assert_eq!(lang1, lang3);
    }

    #[test]
    fn test_language_in_collections() {
        use std::collections::{HashMap, HashSet};
        
        // Test that language can be used as a key
        let mut set = HashSet::new();
        set.insert(TestLanguage);
        assert!(set.contains(&TestLanguage));
        
        let mut map = HashMap::new();
        map.insert(TestLanguage, "test language");
        assert_eq!(map.get(&TestLanguage), Some(&"test language"));
    }

    #[test]
    fn test_language_ordering() {
        // Test that languages can be ordered (useful for consistent iteration)
        let lang1 = TestLanguage;
        let lang2 = TestLanguage;
        
        assert_eq!(lang1.cmp(&lang2), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_language_debug() {
        let language = TestLanguage;
        let debug_str = format!("{:?}", language);
        assert!(debug_str.contains("TestLanguage"));
    }

    #[test]
    fn test_root_node_requirements() {
        // Test that root node meets the trait requirements
        let root = TestDocument {
            content: "test content".to_string(),
        };
        
        // Should be cloneable
        let root2 = root.clone();
        assert_eq!(root, root2);
        
        // Should be debuggable
        let debug_str = format!("{:?}", root);
        assert!(debug_str.contains("TestDocument"));
    }

    #[test]
    fn test_pdf_like_syntax_kinds() {
        // Test PDF-like syntax kind behaviors
        let object_kind = TestSyntaxKind::Object;
        let dict_kind = TestSyntaxKind::Dictionary;
        let array_kind = TestSyntaxKind::Array;
        let root_kind = TestSyntaxKind::Root;
        
        // Objects have string representation
        assert_eq!(object_kind.to_string(), Some("obj"));
        assert_eq!(dict_kind.to_string(), None);
        
        // Arrays are lists (like PDF arrays)
        assert!(!object_kind.is_list());
        assert!(!dict_kind.is_list());
        assert!(array_kind.is_list());
        
        // Only root is root
        assert!(!object_kind.is_root());
        assert!(!dict_kind.is_root());
        assert!(!array_kind.is_root());
        assert!(root_kind.is_root());
    }

    // Test that language can be used as a const generic parameter
    fn language_generic_function<L: Language>() -> L {
        todo!("Function that works with any language")
    }

    #[test]
    fn test_language_as_generic_parameter() {
        // This test ensures the trait is properly defined for generic usage
        let _result: fn() -> TestLanguage = language_generic_function::<TestLanguage>;
    }
}
