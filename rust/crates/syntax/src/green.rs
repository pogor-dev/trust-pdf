//! # Green Tree: Immutable Syntax Tree Implementation
//!
//! This module contains the "green" layer of our syntax tree architecture,
//! providing immutable, memory-efficient syntax tree nodes and tokens.
//! The green tree is the foundation that enables incremental parsing,
//! memory sharing, and thread-safe operations.
//!
//! ## What is the Green Tree?
//!
//! The green tree is an immutable representation of the parsed syntax where:
//! - **All data is immutable** once created, enabling safe sharing across threads
//! - **Memory is efficiently shared** through atomic reference counting
//! - **Full fidelity is preserved** including all whitespace and trivia
//! - **Compact representation** uses specialized memory layouts for performance
//!
//! ## Architecture Components
//!
//! The green tree consists of several core components:
//!
//! ### Nodes and Tokens
//! - **`GreenNode`**: Internal tree nodes that contain child elements
//! - **`GreenToken`**: Leaf nodes that contain actual text content
//! - **`GreenTrivia`**: Whitespace, comments, and other non-semantic content
//!
//! ### Memory Management
//! - **`Arc`-based storage**: Thread-safe reference counting without weak references
//! - **Header-slice patterns**: Optimized memory layout for variable-length data
//! - **Thin representations**: Zero-sized array types for compile-time optimization
//!
//! ## PDF-Specific Design
//!
//! The green tree handles PDF syntax requirements:
//! - **Byte-level precision**: Exact positioning for cross-reference tables
//! - **Large file support**: Memory-efficient handling of multi-megabyte documents
//! - **Incremental updates**: Enables efficient re-parsing of modified sections
//! - **Trivia preservation**: Maintains PDF formatting requirements exactly
//!
//! ## Memory Layout Types
//!
//! This module defines several type aliases for different memory representations:
//! - **Thin types**: Zero-sized arrays for compile-time type safety
//! - **Full types**: Variable-length slices for runtime data storage
//! - **Header-slice patterns**: Combine metadata with variable-length data

pub(crate) mod kind;
pub(crate) mod node;
pub(crate) mod node_data;
pub(crate) mod node_head;
pub(crate) mod node_slot;
pub(crate) mod node_slots;
pub(crate) mod token;
pub(crate) mod token_data;
pub(crate) mod token_head;
pub(crate) mod trivia;
pub(crate) mod trivia_data;

use crate::{
    arc::header_slice::HeaderSlice,
    green::{
        node_head::GreenNodeHead, node_slot::Slot, token_head::GreenTokenHead,
        trivia_data::GreenTriviaHead,
    },
    syntax::trivia_piece::TriviaPiece,
};

type GreenTriviaReprThin = HeaderSlice<GreenTriviaHead, [TriviaPiece; 0]>;

type GreenTokenRepr = HeaderSlice<GreenTokenHead, [u8]>;
type GreenTokenReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;

type GreenNodeRepr = HeaderSlice<GreenNodeHead, [Slot]>;
type GreenNodeReprThin = HeaderSlice<GreenNodeHead, [Slot; 0]>;
