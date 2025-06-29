//! # Green Token: Immutable Leaf Nodes in the Syntax Tree
//!
//! This module implements `GreenToken`, which represents immutable leaf nodes in the
//! concrete syntax tree. Green tokens are the basic building blocks that contain
//! actual text content like numbers, strings, keywords, and operators.
//!
//! ## Green Tree Architecture
//!
//! The "green" tree is the immutable layer of the syntax tree where:
//! - All nodes and tokens are immutable once created
//! - Memory is efficiently shared through atomic reference counting
//! - Text content is stored directly in tokens
//! - No parent pointers (bottom-up construction)
//!
//! ## Memory Layout
//!
//! `GreenToken` uses a memory-optimized layout:
//! ```
//! ┌─────────────────┐
//! │ ThinArc pointer │ → ┌─────────────────┐
//! └─────────────────┘   │ GreenTokenHead  │
//!                       │ - kind          │
//!                       │ - trivia info   │
//!                       ├─────────────────┤
//!                       │ Text bytes...   │
//!                       │ (UTF-8 content) │
//!                       └─────────────────┘
//! ```
//!
//! ## PDF Token Examples
//!
//! Different types of tokens found in PDF files:
//! - **Keywords**: `obj`, `endobj`, `stream`, `endstream`, `true`, `false`, `null`
//! - **Numbers**: `42`, `3.14159`, `-17`, `+0.5`
//! - **Strings**: `(Hello World)`, `<48656C6C6F>`
//! - **Names**: `/Type`, `/Font`, `/Contents`
//! - **Operators**: `l`, `m`, `BT`, `ET`, `Tj`
//! - **Delimiters**: `[`, `]`, `<<`, `>>`, `(`, `)`
//!
//! ## Thread Safety
//!
//! `GreenToken` is thread-safe due to:
//! - Immutable content (no mutation after creation)
//! - Atomic reference counting in the underlying `ThinArc`
//! - No shared mutable state
//!
//! ## Memory Efficiency
//!
//! The implementation optimizes for:
//! - Minimal memory overhead per token
//! - Efficient sharing of identical tokens
//! - Cache-friendly memory layout
//! - Reduced pointer indirection

use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    arc::{arc::Arc, thin_arc::ThinArc},
    green::{
        GreenTokenRepr, GreenTokenReprThin, kind::RawSyntaxKind, token_data::GreenTokenData,
        token_head::GreenTokenHead,
    },
};

/// An immutable leaf node in the green syntax tree containing text content.
///
/// `GreenToken` represents terminal symbols in the concrete syntax tree - the actual
/// text content like keywords, identifiers, numbers, and operators. Each token has:
/// - A syntax kind identifying its grammatical role
/// - Text content (stored as UTF-8 bytes)
/// - Optional leading/trailing trivia (whitespace, comments)
///
/// # Memory Representation
///
/// The token uses `ThinArc` for memory efficiency, storing the text content inline
/// with the header information. This reduces memory fragmentation and improves
/// cache locality when traversing the syntax tree.
///
/// # Immutability Guarantee
///
/// Once created, tokens cannot be modified. This enables:
/// - Safe sharing across threads
/// - Structural sharing for identical content
/// - Caching and memoization optimizations
/// - Incremental parsing and updates
///
/// # PDF Context
///
/// In PDF parsing, tokens represent all the atomic elements:
/// - Object and stream boundaries (`obj`, `endobj`, `stream`, `endstream`)
/// - Data values (numbers, strings, booleans, null)
/// - References and names (object references, dictionary keys)
/// - Content stream operators and operands
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenToken {
    ptr: ThinArc<GreenTokenHead, u8>,
}

impl GreenToken {
    /// Returns the syntax kind of this token.
    ///
    /// The kind identifies the grammatical role of the token in the syntax tree,
    /// such as whether it's a keyword, number, string, operator, etc.
    ///
    /// # PDF Examples
    /// - Keywords: `obj`, `endobj`, `stream`, `null`, `true`
    /// - Numbers: Any numeric literal kind
    /// - Strings: Text or hexadecimal string literals
    /// - Names: Dictionary keys and PDF name objects
    /// - Operators: Content stream operators like `BT`, `Tj`, `l`, `m`
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use syntax::green::{token::GreenToken, kind::RawSyntaxKind};
    /// # let token = todo!("Create a token");
    /// let kind = token.kind();
    /// // Use kind to determine how to handle this token
    /// ```
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.data.header.kind
    }

    /// Returns the length of the text content of this token in bytes.
    ///
    /// This represents the number of UTF-8 bytes in the token's text content,
    /// not including any trivia (whitespace, comments) that might be attached.
    ///
    /// # PDF Context
    /// Useful for:
    /// - Calculating file positions and offsets
    /// - Validating fixed-width formats (cross-reference entries)
    /// - Memory allocation and buffer sizing
    /// - Progress reporting during parsing
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use syntax::green::token::GreenToken;
    /// # let token = todo!("Create a token");
    /// let len = token.text_len();
    /// println!("Token occupies {} bytes", len);
    /// ```
    #[inline]
    pub fn text_len(&self) -> u64 {
        self.text().len() as u64
    }

    /// Creates a `GreenToken` from a raw pointer.
    ///
    /// # Safety
    /// 
    /// This function is unsafe because:
    /// - The caller must ensure `ptr` points to valid, initialized `GreenTokenData`
    /// - The pointer must remain valid for the lifetime of the returned token
    /// - The reference count must be properly managed
    /// - Memory layout must match the expected `ThinArc` representation
    ///
    /// # Usage
    /// This is primarily used internally by the parser and tree construction code
    /// to convert from raw allocated memory to typed tokens.
    ///
    /// # Implementation Notes
    /// The function performs unsafe transmutations between different Arc
    /// representations to enable the thin pointer optimization used by `ThinArc`.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> GreenToken {
        let arc = unsafe { Arc::from_raw(&ptr.as_ref().data as *const GreenTokenReprThin) };
        let arc =
            unsafe { mem::transmute::<Arc<GreenTokenReprThin>, ThinArc<GreenTokenHead, u8>>(arc) };
        GreenToken { ptr: arc }
    }
}

impl ops::Deref for GreenToken {
    type Target = GreenTokenData;

    /// Provides transparent access to the underlying token data.
    ///
    /// This allows `GreenToken` to act as if it directly contains the methods
    /// and fields of `GreenTokenData`, while maintaining the efficient memory
    /// layout and reference counting of `ThinArc`.
    ///
    /// # Implementation Details
    /// The deref implementation performs unsafe pointer manipulation to convert
    /// between the fat pointer representation (with slice length) and the thin
    /// pointer representation used for storage efficiency.
    #[inline]
    fn deref(&self) -> &GreenTokenData {
        unsafe {
            let repr: &GreenTokenRepr = &self.ptr;
            let repr: &GreenTokenReprThin =
                &*(repr as *const GreenTokenRepr as *const GreenTokenReprThin);
            mem::transmute::<&GreenTokenReprThin, &GreenTokenData>(repr)
        }
    }
}

impl ToOwned for GreenTokenData {
    type Owned = GreenToken;

    /// Converts borrowed token data to an owned token.
    ///
    /// This is used when you have a reference to token data and need to create
    /// an owned token that manages its own reference count. It's essential for
    /// operations that need to return tokens from borrowed contexts.
    ///
    /// # Memory Safety
    /// The implementation carefully manages the reference count to ensure the
    /// data remains valid while avoiding double-free or use-after-free bugs.
    #[inline]
    fn to_owned(&self) -> GreenToken {
        unsafe {
            let green = GreenToken::from_raw(ptr::NonNull::from(self));
            let green = ManuallyDrop::new(green);
            GreenToken::clone(&green)
        }
    }
}

impl Borrow<GreenTokenData> for GreenToken {
    /// Allows borrowing the token data from an owned token.
    ///
    /// This enables `GreenToken` to be used in contexts that expect borrowed
    /// `GreenTokenData`, facilitating APIs that work with both owned and
    /// borrowed token representations.
    #[inline]
    fn borrow(&self) -> &GreenTokenData {
        self
    }
}

impl fmt::Debug for GreenToken {
    /// Formats the token for debugging output.
    ///
    /// This delegates to the `Debug` implementation of `GreenTokenData`,
    /// providing detailed information about the token's kind, text content,
    /// and any attached trivia.
    ///
    /// # Output Format
    /// The debug output typically includes:
    /// - Token kind (e.g., `KEYWORD`, `NUMBER`, `STRING`)
    /// - Text content (the actual token text)
    /// - Trivia information (leading/trailing whitespace, comments)
    /// - Length and position information
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenToken {
    /// Formats the token for user-friendly display.
    ///
    /// This typically shows just the text content of the token without
    /// debug metadata, making it suitable for pretty-printing syntax trees
    /// or displaying tokens in error messages.
    ///
    /// # PDF Context
    /// For PDF tokens, this would display:
    /// - Keywords: `obj`, `stream`, `true`
    /// - Numbers: `42`, `3.14159`
    /// - Strings: `(Hello)`, `<48656C6C6F>`
    /// - Names: `/Type`, `/Font`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Display::fmt(data, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests are conceptual since we can't easily create GreenTokens
    // without the full parser infrastructure. In a real implementation, these
    // would use builder functions or factory methods.

    #[test]
    fn test_token_properties() {
        // This test demonstrates the expected interface, though it can't run
        // without proper token construction infrastructure
        
        // let token = create_test_token("obj", SyntaxKind::ObjKeyword);
        // assert_eq!(token.text(), "obj");
        // assert_eq!(token.text_len(), 3);
        // assert_eq!(token.kind(), SyntaxKind::ObjKeyword.to_raw());
    }

    #[test]
    fn test_pdf_token_scenarios() {
        // Test scenarios for different types of PDF tokens
        
        // PDF object keyword
        // let obj_token = create_test_token("obj", PdfSyntaxKind::ObjKeyword);
        // assert_eq!(obj_token.text(), "obj");
        
        // PDF number
        // let num_token = create_test_token("42", PdfSyntaxKind::Number);
        // assert_eq!(num_token.text(), "42");
        
        // PDF string
        // let str_token = create_test_token("(Hello World)", PdfSyntaxKind::String);
        // assert_eq!(str_token.text(), "(Hello World)");
        
        // PDF name
        // let name_token = create_test_token("/Type", PdfSyntaxKind::Name);
        // assert_eq!(name_token.text(), "/Type");
    }

    #[test]
    fn test_token_equality_and_hashing() {
        // Test that tokens with the same content are equal and hash the same
        
        // let token1 = create_test_token("stream", PdfSyntaxKind::StreamKeyword);
        // let token2 = create_test_token("stream", PdfSyntaxKind::StreamKeyword);
        // assert_eq!(token1, token2);
        
        // let mut set = std::collections::HashSet::new();
        // set.insert(token1.clone());
        // assert!(set.contains(&token2));
    }

    #[test]
    fn test_token_cloning() {
        // Test that tokens can be cloned efficiently (should be cheap due to Arc)
        
        // let original = create_test_token("endobj", PdfSyntaxKind::EndObjKeyword);
        // let cloned = original.clone();
        // assert_eq!(original, cloned);
        // assert_eq!(original.text(), cloned.text());
        // assert_eq!(original.kind(), cloned.kind());
    }

    #[test]
    fn test_token_display_formatting() {
        // Test that tokens format correctly for debugging and display
        
        // let token = create_test_token("null", PdfSyntaxKind::NullKeyword);
        // let display_str = format!("{}", token);
        // assert_eq!(display_str, "null");
        
        // let debug_str = format!("{:?}", token);
        // assert!(debug_str.contains("null"));
        // assert!(debug_str.contains("NullKeyword") || debug_str.contains("kind"));
    }

    #[test]
    fn test_pdf_content_stream_operators() {
        // Test tokens for PDF content stream operators
        
        // BT (begin text)
        // let bt_token = create_test_token("BT", PdfSyntaxKind::Operator);
        // assert_eq!(bt_token.text(), "BT");
        
        // Tj (show text)
        // let tj_token = create_test_token("Tj", PdfSyntaxKind::Operator);
        // assert_eq!(tj_token.text(), "Tj");
        
        // Numbers used as operands
        // let coord_token = create_test_token("100.5", PdfSyntaxKind::Number);
        // assert_eq!(coord_token.text(), "100.5");
    }

    #[test]
    fn test_pdf_cross_reference_tokens() {
        // Test tokens found in cross-reference tables
        
        // Offset values
        // let offset_token = create_test_token("0000000009", PdfSyntaxKind::Number);
        // assert_eq!(offset_token.text(), "0000000009");
        // assert_eq!(offset_token.text_len(), 10); // Fixed width
        
        // Generation numbers
        // let gen_token = create_test_token("00000", PdfSyntaxKind::Number);
        // assert_eq!(gen_token.text(), "00000");
        // assert_eq!(gen_token.text_len(), 5); // Fixed width
        
        // Entry flags
        // let flag_token = create_test_token("n", PdfSyntaxKind::Identifier);
        // assert_eq!(flag_token.text(), "n");
    }

    #[test]
    fn test_token_borrow_semantics() {
        // Test that borrowing works correctly
        
        // let token = create_test_token("trailer", PdfSyntaxKind::TrailerKeyword);
        // let borrowed: &GreenTokenData = token.borrow();
        // assert_eq!(borrowed.text(), "trailer");
    }

    #[test]
    fn test_token_memory_efficiency() {
        // Test memory characteristics (conceptual - actual measurements would 
        // require runtime profiling)
        
        // Tokens should be small and efficient
        // assert!(std::mem::size_of::<GreenToken>() <= 16); // Pointer-sized
        
        // Cloning should be cheap (just bumping reference count)
        // let token = create_test_token("xref", PdfSyntaxKind::XrefKeyword);
        // let start = std::time::Instant::now();
        // for _ in 0..1000 {
        //     let _cloned = token.clone();
        // }
        // let duration = start.elapsed();
        // assert!(duration < std::time::Duration::from_millis(1));
    }
}
