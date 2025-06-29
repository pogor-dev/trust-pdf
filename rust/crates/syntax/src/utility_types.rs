//! # Utility Types: Common Data Structures for Syntax Trees
//!
//! This module provides fundamental utility types that are used throughout
//! the syntax tree implementation. These types enable flexible and type-safe
//! handling of the heterogeneous data structures that appear in syntax trees.
//!
//! ## Design Philosophy
//!
//! The utility types follow several key principles:
//!
//! ### Generic and Reusable
//! Types are parameterized to work with different node and token types,
//! making them suitable for various language frontends while maintaining
//! type safety.
//!
//! ### Zero-Cost Abstractions
//! All utility types are designed to have minimal runtime overhead,
//! compiling down to efficient machine code without boxing or indirection.
//!
//! ### Composable Design
//! Types can be combined and nested to create more complex data structures
//! while preserving the individual properties of each component.
//!
//! ## Core Utility Types
//!
//! ### NodeOrToken<N, T>
//! A fundamental enum that can hold either a node or a token, enabling
//! uniform handling of heterogeneous tree elements. This is crucial for:
//! - Cursor navigation that moves through mixed content
//! - Parsers that need to handle both structural and leaf elements
//! - APIs that work with any tree element regardless of type
//!
//! ## PDF Parsing Context
//!
//! In PDF syntax trees, utility types handle various scenarios:
//!
//! ### Mixed Content Navigation
//! PDF documents contain both structural elements (objects, dictionaries)
//! and leaf content (numbers, strings, names). Utility types allow uniform
//! processing of these mixed element types.
//!
//! ### Flexible Element Access
//! Different parts of the compiler need different views of the same data:
//! - Parsers need raw element access for building trees
//! - Semantic analyzers need typed access to specific element kinds
//! - Code generators need to iterate over elements uniformly
//!
//! ### Error Recovery
//! When parsing malformed PDF files, utility types enable the parser to
//! continue processing by treating unexpected elements uniformly rather
//! than failing on type mismatches.
//!
//! ## Module Components
//!
//! - [`node_or_token`]: The fundamental either-type for nodes and tokens
//!
//! ## Thread Safety
//!
//! All utility types are thread-safe when their generic parameters are
//! thread-safe, making them suitable for use in concurrent parsing and
//! analysis scenarios.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use syntax::utility_types::node_or_token::NodeOrToken;
//!
//! // Process mixed content uniformly
//! fn process_element(element: NodeOrToken<MyNode, MyToken>) {
//!     match element {
//!         NodeOrToken::Node(node) => {
//!             // Handle structural element
//!             println!("Processing node with {} children", node.child_count());
//!         },
//!         NodeOrToken::Token(token) => {
//!             // Handle leaf content
//!             println!("Processing token: {}", token.text());
//!         }
//!     }
//! }
//! ```

pub(crate) mod node_or_token;
