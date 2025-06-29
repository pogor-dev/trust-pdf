//! # AST Module: Abstract Syntax Tree Implementation
//!
//! This module provides the Abstract Syntax Tree (AST) layer on top of the green tree,
//! offering a typed, language-specific API for working with PDF syntax structures.
//!
//! ## What is an Abstract Syntax Tree?
//!
//! An Abstract Syntax Tree (AST) is a tree representation of the abstract syntactic
//! structure of source code written in a programming language. In our case, it represents
//! the structure of PDF documents. Each node of the tree denotes a construct occurring
//! in the PDF language.
//!
//! ## Relationship to Green Tree
//!
//! The AST provides a typed interface over the untyped green tree:
//! - **Green Tree**: Raw, untyped syntax nodes and tokens with perfect fidelity
//! - **AST**: Typed wrapper that provides PDF-specific operations and guarantees
//!
//! ## Zero-Cost Abstractions
//!
//! The AST is designed as a zero-cost abstraction over the green tree:
//! - AST nodes are just typed wrappers around syntax nodes
//! - Conversion between AST and syntax nodes has no runtime cost
//! - Memory layout is identical between AST and syntax representations
//!
//! ## PDF-Specific AST Nodes
//!
//! The AST will provide typed nodes for PDF constructs such as:
//! - **Dictionary objects**: Key-value pairs with type validation
//! - **Array objects**: Ordered collections with element type checking
//! - **Stream objects**: Data with associated dictionary headers
//! - **Cross-reference tables**: File structure navigation information
//! - **Content streams**: Page content and graphics instructions
//!
//! Each AST node understands the PDF specification requirements for its syntax.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use syntax::ast::AstNode;
//! // Example showing how AST nodes will be used:
//! // let dict_node: DictionaryNode = syntax_node.cast()?;
//! // let keys = dict_node.keys();  // Typed iteration over dictionary keys
//! // let value = dict_node.get("/Type")?;  // Type-safe key lookup
//! ```

pub(crate) mod ast_node;
