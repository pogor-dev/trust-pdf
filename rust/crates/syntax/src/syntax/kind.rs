//! # Syntax Kind: Language-Agnostic Node and Token Classification
//!
//! This module defines the `SyntaxKind` trait, which provides a language-agnostic
//! interface for classifying different types of syntax tree nodes and tokens.
//! This design allows the same syntax tree infrastructure to support multiple
//! language frontends (PDF, potentially others) while maintaining type safety.
//!
//! ## Design Philosophy
//!
//! The `SyntaxKind` trait follows the "typed syntax tree" pattern used by:
//! - Rust Analyzer (rust-analyzer)
//! - Roslyn (.NET compiler platform)
//! - Tree-sitter (incremental parsing)
//!
//! ## Key Benefits
//!
//! 1. **Language Independence**: The same green tree and syntax infrastructure
//!    can support multiple document formats
//! 2. **Type Safety**: Each language can define its own strongly-typed kind enum
//! 3. **Extensibility**: New syntax kinds can be added without changing core infrastructure
//! 4. **Performance**: Using enums allows fast pattern matching and dispatching
//!
//! ## PDF-Specific Context
//!
//! For PDF parsing, syntax kinds might include:
//! - **Structural**: Objects, dictionaries, arrays, streams
//! - **Primitive**: Numbers, strings, names, booleans  
//! - **Reference**: Object references, indirect objects
//! - **Organizational**: Cross-reference tables, trailers
//! - **Meta**: Comments, whitespace, keywords
//!
//! ## Usage Pattern
//!
//! ```rust,no_run
//! use syntax::syntax::kind::SyntaxKind;
//! use syntax::green::kind::RawSyntaxKind;
//!
//! // Example PDF syntax kind implementation
//! #[derive(Debug, Clone, Copy, PartialEq)]
//! enum PdfSyntaxKind {
//!     Root,
//!     Object,
//!     Dictionary,
//!     Array,
//!     // ... other PDF-specific kinds
//! }
//!
//! impl SyntaxKind for PdfSyntaxKind {
//!     const EOF: Self = PdfSyntaxKind::Eof;
//!     
//!     fn to_raw(&self) -> RawSyntaxKind {
//!         RawSyntaxKind(*self as u16)
//!     }
//!     
//!     // ... other trait methods
//! }
//! ```

use std::fmt;

use crate::green::kind::RawSyntaxKind;

/// Language-agnostic trait for classifying syntax tree nodes and tokens.
///
/// This trait provides a common interface that allows the syntax tree infrastructure
/// to work with different document formats while maintaining type safety. Each
/// language or document format implements this trait with its own specific kinds.
///
/// ## Implementation Requirements
///
/// Implementers must provide:
/// - An `EOF` constant representing end-of-file
/// - Conversion to/from raw syntax kinds for storage
/// - Classification methods for special node types
/// - Optional string representation for keywords/punctuation
///
/// ## Design Constraints
///
/// The trait requires:
/// - `Debug`: Essential for debugging parser issues and tree inspection
/// - `PartialEq`: Needed for comparing kinds and pattern matching
/// - `Copy`: Kinds should be small, copyable values for performance
///
/// ## Thread Safety
///
/// While not explicitly required, implementations should be thread-safe
/// since syntax trees may be shared across threads for incremental parsing.
pub trait SyntaxKind: fmt::Debug + PartialEq + Copy {
    /// The end-of-file marker for this syntax kind.
    ///
    /// This constant represents the end of input during parsing and is used
    /// to signal when no more tokens are available. It should be a unique
    /// value that doesn't conflict with any actual syntax elements.
    ///
    /// # PDF Context
    /// In PDF parsing, EOF might occur:
    /// - At the actual end of the file
    /// - At the end of a stream's content
    /// - When parsing encounters an unrecoverable error
    const EOF: Self;

    /// Converts this syntax kind to a raw (untyped) representation.
    ///
    /// Raw syntax kinds are used internally for storage and manipulation
    /// in the green tree layer, where type information is erased for
    /// memory efficiency and language independence.
    ///
    /// # Implementation Notes
    /// - Should be a bijection with `from_raw`
    /// - Typically implemented as a simple cast to integer
    /// - Must be consistent across program runs for caching
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use syntax::syntax::kind::SyntaxKind;
    /// # use syntax::green::kind::RawSyntaxKind;
    /// # #[derive(Debug, Clone, Copy, PartialEq)]
    /// # enum MyKind { Object, Dictionary }
    /// # impl SyntaxKind for MyKind {
    /// #     const EOF: Self = MyKind::Object;
    /// fn to_raw(&self) -> RawSyntaxKind {
    ///     RawSyntaxKind(*self as u16)
    /// }
    /// #     fn from_raw(raw: RawSyntaxKind) -> Self { MyKind::Object }
    /// #     fn is_root(&self) -> bool { false }
    /// #     fn is_list(&self) -> bool { false }
    /// #     fn to_string(&self) -> Option<&'static str> { None }
    /// # }
    /// ```
    fn to_raw(&self) -> RawSyntaxKind;

    /// Creates a syntax kind from a raw representation.
    ///
    /// This is the inverse of `to_raw` and is used when reconstructing
    /// typed syntax kinds from the untyped green tree representation.
    ///
    /// # Implementation Notes
    /// - Should be a bijection with `to_raw`
    /// - Must handle all possible raw values gracefully
    /// - Consider using a default/error kind for invalid values
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use syntax::syntax::kind::SyntaxKind;
    /// # use syntax::green::kind::RawSyntaxKind;
    /// # #[derive(Debug, Clone, Copy, PartialEq)]
    /// # enum MyKind { Object, Dictionary }
    /// # impl SyntaxKind for MyKind {
    /// #     const EOF: Self = MyKind::Object;
    /// #     fn to_raw(&self) -> RawSyntaxKind { RawSyntaxKind(0) }
    /// fn from_raw(raw: RawSyntaxKind) -> Self {
    ///     match raw.0 {
    ///         0 => MyKind::Object,
    ///         1 => MyKind::Dictionary,
    ///         _ => MyKind::Object, // fallback
    ///     }
    /// }
    /// #     fn is_root(&self) -> bool { false }
    /// #     fn is_list(&self) -> bool { false }
    /// #     fn to_string(&self) -> Option<&'static str> { None }
    /// # }
    /// ```
    fn from_raw(raw: RawSyntaxKind) -> Self;

    /// Returns `true` if this kind represents a root node of the syntax tree.
    ///
    /// Root nodes are special nodes that serve as the top-level container
    /// for an entire document or major structural unit. They typically have
    /// special parsing and validation rules.
    ///
    /// # PDF Context
    /// In PDF syntax, root nodes might include:
    /// - Document root (containing all objects)
    /// - Object root (containing object content)
    /// - Stream content root
    /// - Cross-reference section root
    ///
    /// # Usage
    /// The parser and tree manipulation code uses this to:
    /// - Apply special validation rules
    /// - Handle top-level recovery scenarios
    /// - Determine tree traversal boundaries
    fn is_root(&self) -> bool;

    /// Returns `true` if this kind represents a list node.
    ///
    /// List nodes are container nodes that hold sequences of similar elements.
    /// They often have special parsing rules for handling separators, optional
    /// elements, and error recovery.
    ///
    /// # PDF Context
    /// PDF list nodes might include:
    /// - Array elements
    /// - Dictionary entries
    /// - Object sequences
    /// - Cross-reference entry lists
    /// - Content stream operation sequences
    ///
    /// # Usage
    /// This information helps:
    /// - Apply appropriate formatting during pretty-printing
    /// - Handle incremental updates correctly
    /// - Implement list-specific navigation patterns
    fn is_list(&self) -> bool;

    /// Returns a string representation for keywords and punctuation, or `None` for other kinds.
    ///
    /// This method provides the exact text representation for syntax kinds that
    /// have fixed string representations (keywords, operators, punctuation).
    /// Returns `None` for kinds that have variable content (identifiers, literals).
    ///
    /// # PDF Context
    /// PDF keywords that would return strings:
    /// - `"obj"`, `"endobj"` - object boundaries
    /// - `"stream"`, `"endstream"` - stream boundaries  
    /// - `"xref"` - cross-reference table marker
    /// - `"trailer"` - trailer dictionary marker
    /// - `"startxref"` - file position marker
    /// - `"true"`, `"false"`, `"null"` - boolean/null literals
    ///
    /// # Usage
    /// This is used for:
    /// - Syntax highlighting (keywords vs identifiers)
    /// - Pretty-printing and code generation
    /// - Parser error messages ("expected 'obj', found 'foo'")
    /// - Completion and IntelliSense features
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use syntax::syntax::kind::SyntaxKind;
    /// # use syntax::green::kind::RawSyntaxKind;
    /// # #[derive(Debug, Clone, Copy, PartialEq)]
    /// # enum PdfKind { ObjKeyword, Identifier }
    /// # impl SyntaxKind for PdfKind {
    /// #     const EOF: Self = PdfKind::ObjKeyword;
    /// #     fn to_raw(&self) -> RawSyntaxKind { RawSyntaxKind(0) }
    /// #     fn from_raw(raw: RawSyntaxKind) -> Self { PdfKind::ObjKeyword }
    /// #     fn is_root(&self) -> bool { false }
    /// #     fn is_list(&self) -> bool { false }
    /// fn to_string(&self) -> Option<&'static str> {
    ///     match self {
    ///         PdfKind::ObjKeyword => Some("obj"),
    ///         PdfKind::Identifier => None, // Variable content
    ///     }
    /// }
    /// # }
    /// ```
    fn to_string(&self) -> Option<&'static str>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test syntax kind implementation for testing
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum TestSyntaxKind {
        Root,
        List,
        ObjKeyword,
        StreamKeyword,
        Identifier,
        Number,
        Eof,
    }

    impl SyntaxKind for TestSyntaxKind {
        const EOF: Self = TestSyntaxKind::Eof;

        fn to_raw(&self) -> RawSyntaxKind {
            RawSyntaxKind(*self as u16)
        }

        fn from_raw(raw: RawSyntaxKind) -> Self {
            match raw.0 {
                0 => TestSyntaxKind::Root,
                1 => TestSyntaxKind::List,
                2 => TestSyntaxKind::ObjKeyword,
                3 => TestSyntaxKind::StreamKeyword,
                4 => TestSyntaxKind::Identifier,
                5 => TestSyntaxKind::Number,
                6 => TestSyntaxKind::Eof,
                _ => TestSyntaxKind::Eof, // fallback
            }
        }

        fn is_root(&self) -> bool {
            matches!(self, TestSyntaxKind::Root)
        }

        fn is_list(&self) -> bool {
            matches!(self, TestSyntaxKind::List)
        }

        fn to_string(&self) -> Option<&'static str> {
            match self {
                TestSyntaxKind::ObjKeyword => Some("obj"),
                TestSyntaxKind::StreamKeyword => Some("stream"),
                _ => None,
            }
        }
    }

    #[test]
    fn test_syntax_kind_raw_conversion() {
        let kind = TestSyntaxKind::ObjKeyword;
        let raw = kind.to_raw();
        let recovered = TestSyntaxKind::from_raw(raw);
        
        assert_eq!(kind, recovered);
    }

    #[test]
    fn test_eof_constant() {
        assert_eq!(TestSyntaxKind::EOF, TestSyntaxKind::Eof);
    }

    #[test]
    fn test_root_identification() {
        assert!(TestSyntaxKind::Root.is_root());
        assert!(!TestSyntaxKind::List.is_root());
        assert!(!TestSyntaxKind::ObjKeyword.is_root());
    }

    #[test]
    fn test_list_identification() {
        assert!(!TestSyntaxKind::Root.is_list());
        assert!(TestSyntaxKind::List.is_list());
        assert!(!TestSyntaxKind::ObjKeyword.is_list());
    }

    #[test]
    fn test_keyword_string_representation() {
        assert_eq!(TestSyntaxKind::ObjKeyword.to_string(), Some("obj"));
        assert_eq!(TestSyntaxKind::StreamKeyword.to_string(), Some("stream"));
        assert_eq!(TestSyntaxKind::Identifier.to_string(), None);
        assert_eq!(TestSyntaxKind::Number.to_string(), None);
    }

    #[test]
    fn test_all_raw_values_roundtrip() {
        for i in 0..=6u16 {
            let raw = RawSyntaxKind(i);
            let kind = TestSyntaxKind::from_raw(raw);
            let raw2 = kind.to_raw();
            
            // Should roundtrip correctly for valid values
            if i <= 6 {
                assert_eq!(raw.0, raw2.0);
            }
        }
    }

    #[test]
    fn test_invalid_raw_values_handled() {
        // Test that invalid raw values don't panic
        let invalid = RawSyntaxKind(999);
        let kind = TestSyntaxKind::from_raw(invalid);
        
        // Should fallback to EOF
        assert_eq!(kind, TestSyntaxKind::Eof);
    }

    #[test]
    fn test_pdf_like_syntax_scenario() {
        // Simulate PDF-like syntax kinds
        assert!(TestSyntaxKind::Root.is_root()); // Document root
        assert!(TestSyntaxKind::List.is_list()); // Array or dict entries
        assert_eq!(TestSyntaxKind::ObjKeyword.to_string(), Some("obj")); // PDF keyword
        assert_eq!(TestSyntaxKind::StreamKeyword.to_string(), Some("stream")); // PDF keyword
        assert_eq!(TestSyntaxKind::Identifier.to_string(), None); // Variable content
    }

    #[test]
    fn test_syntax_kind_copy_semantics() {
        let kind1 = TestSyntaxKind::ObjKeyword;
        let kind2 = kind1; // Should copy, not move
        
        assert_eq!(kind1, kind2);
        
        // Both should still be usable
        assert_eq!(kind1.to_string(), Some("obj"));
        assert_eq!(kind2.to_string(), Some("obj"));
    }

    #[test]
    fn test_syntax_kind_debug() {
        let kind = TestSyntaxKind::ObjKeyword;
        let debug_str = format!("{:?}", kind);
        
        assert!(debug_str.contains("ObjKeyword"));
    }
}
