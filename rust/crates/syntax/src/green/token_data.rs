//! # Green Token Data: Immutable Leaf Elements with Text Content
//!
//! This module provides the `GreenTokenData` type, which represents immutable
//! leaf elements in the green tree. Tokens are the fundamental units that
//! contain actual text content from PDF files, including keywords, values,
//! operators, and structural elements.
//!
//! ## What is Token Data?
//!
//! Token data represents the lowest level of the syntax tree - leaf nodes
//! that contain actual text content rather than child elements. In PDF
//! processing, tokens represent:
//! - **Keywords**: `obj`, `endobj`, `stream`, `endstream`, `xref`, `trailer`
//! - **Data values**: Numbers (`42`, `3.14`), strings (`(Hello)`), names (`/Type`)
//! - **Operators**: Content stream operators (`Tf`, `BT`, `ET`, `q`, `Q`, `cm`)
//! - **Structural elements**: Brackets (`[`, `]`), delimiters (`<<`, `>>`, `(`, `)`)
//!
//! ## Memory Layout
//!
//! The token data uses a transparent representation over `GreenTokenReprThin`,
//! which provides efficient memory layout using the header-slice pattern:
//! ```text
//! [GreenTokenHead][Text bytes...]
//! ```
//!
//! This layout enables:
//! - **Cache efficiency**: Header and text data stored contiguously
//! - **Memory safety**: Compile-time guarantees about data layout
//! - **Zero-copy access**: Direct access to text bytes without allocation
//!
//! ## PDF Text Handling
//!
//! PDF text has specific characteristics that this module handles:
//! - **Binary safety**: Text is stored as `&[u8]` to handle any PDF content
//! - **Encoding neutrality**: No assumptions about text encoding at this level
//! - **Trivia preservation**: Leading and trailing whitespace/comments maintained
//! - **Exact fidelity**: Every byte from the original PDF is preserved
//!
//! ## Thread Safety
//!
//! Token data is immutable once created, making it safe to share across
//! threads without synchronization. The underlying data uses atomic reference
//! counting for memory management.

use std::fmt;

use crate::green::{GreenTokenReprThin, kind::RawSyntaxKind, trivia::GreenTrivia};

/// Immutable token data representing leaf elements in the green tree.
///
/// `GreenTokenData` contains actual text content from PDF files along with
/// metadata about the token's type and associated trivia. This is the
/// fundamental unit for all text-bearing elements in the syntax tree.
///
/// ## Design Principles
///
/// ### Transparency
/// Uses `#[repr(transparent)]` to ensure zero-cost wrapping around the
/// underlying header-slice data structure, with no runtime overhead.
///
/// ### Immutability
/// Once created, token data never changes, enabling safe sharing and
/// efficient caching across different parts of the compiler.
///
/// ### Memory Efficiency
/// The header-slice pattern stores metadata and text data contiguously,
/// improving cache locality and reducing allocation overhead.
///
/// ## PDF Context Examples
///
/// Different types of tokens this structure might represent:
/// - Object keyword: `obj` with specific trivia requirements
/// - Name token: `/Type` with PDF name syntax rules
/// - String literal: `(Hello World)` with parentheses and content
/// - Number: `123.45` with precise decimal representation
/// - Stream keyword: `stream` followed by mandatory newline
#[repr(transparent)]
pub(crate) struct GreenTokenData {
    /// The underlying header-slice data structure.
    ///
    /// Contains both the token metadata (kind, trivia) in the header
    /// and the actual text bytes in the slice portion. The transparent
    /// representation ensures this wrapper has zero runtime cost.
    pub(crate) data: GreenTokenReprThin,
}

impl GreenTokenData {
    /// Returns the syntax kind that identifies what type of PDF element this token represents.
    ///
    /// The kind determines how this token should be interpreted during semantic analysis
    /// and what role it plays in the overall PDF structure. This is essential for
    /// distinguishing between different types of content that might look similar
    /// in raw text form.
    ///
    /// ## PDF Examples
    ///
    /// - Keywords: `obj` returns a keyword kind, `stream` returns a stream marker kind
    /// - Data types: `42` returns a number kind, `(text)` returns a string kind
    /// - Names: `/Type` returns a name kind with the leading slash
    /// - Operators: `Tf` returns a text operator kind in content streams
    ///
    /// ## Performance
    ///
    /// This is a zero-cost operation that directly accesses a field in the header,
    /// with no computation or allocation required.
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.data.header.kind
    }

    /// Returns the complete text content of this token, including all trivia.
    ///
    /// This method provides access to the raw bytes that represent this token
    /// in the original PDF file. The text includes not just the core token
    /// content, but also any leading and trailing trivia (whitespace, comments)
    /// that was associated with this token during parsing.
    ///
    /// ## PDF Context
    ///
    /// The returned text preserves exact fidelity to the original PDF:
    /// - **Encoding preservation**: Bytes are returned as-is without interpretation
    /// - **Trivia inclusion**: All whitespace and formatting is maintained
    /// - **Binary safety**: Can handle any PDF content including binary data
    ///
    /// ## Examples
    ///
    /// - Simple keyword: `obj` returns `b"obj"`
    /// - With trivia: `  /Type  ` returns the spaces included
    /// - String content: `(Hello)` returns `b"(Hello)"` with parentheses
    /// - Numbers: `123.45` returns `b"123.45"` with exact formatting
    ///
    /// ## Memory Safety
    ///
    /// Uses unsafe pointer operations internally, but the safety is guaranteed
    /// by the header-slice invariants maintained during construction.
    ///
    /// ## Performance
    ///
    /// Zero-copy operation that directly accesses the underlying byte storage
    /// without allocation or copying.
    #[inline]
    pub fn text(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.slice().as_ptr(), self.data.slice().len()) }
    }

    /// Returns the total byte length of this token's text content.
    ///
    /// This method calculates the length of the complete token text,
    /// including all trivia. The length represents the exact number
    /// of bytes this token occupies in the original PDF file.
    ///
    /// ## PDF Processing Importance
    ///
    /// Accurate length calculation is critical for PDF operations:
    /// - **Cross-reference tables**: Require exact byte offsets (ISO 32000-2 §7.5.4)
    /// - **Stream processing**: Content length must be precisely calculated
    /// - **Incremental updates**: Need to know how much content to preserve/replace
    /// - **Memory planning**: Helps estimate memory requirements for large files
    ///
    /// ## Performance
    ///
    /// This is an O(1) operation that just returns the pre-calculated length
    /// from the underlying slice, with no traversal or computation required.
    ///
    /// ## Return Value
    ///
    /// Returns `u64` to handle extremely large PDF files (multi-gigabyte
    /// documents are not uncommon in enterprise environments).
    #[inline]
    pub fn text_len(&self) -> u64 {
        self.text().len() as u64
    }

    /// Returns the leading trivia associated with this token.
    ///
    /// Leading trivia consists of whitespace, comments, and other non-semantic
    /// content that appears before the token's main content. This trivia is
    /// crucial for maintaining PDF formatting requirements and enabling
    /// exact reconstruction of the original document.
    ///
    /// ## PDF Trivia Significance
    ///
    /// In PDF syntax, leading trivia can be semantically important:
    /// - **Object headers**: Newlines may separate `obj` declarations from content
    /// - **Content streams**: Proper spacing between operators is required
    /// - **Cross-reference entries**: Fixed-width formatting with significant spaces
    /// - **Comments**: Developer annotations that should be preserved
    ///
    /// ## Use Cases
    ///
    /// - **Syntax highlighting**: Render whitespace and comments appropriately
    /// - **Code formatting**: Maintain or improve PDF structure presentation
    /// - **Error reporting**: Include context around problematic tokens
    /// - **Round-trip fidelity**: Preserve exact original formatting
    ///
    /// ## Performance
    ///
    /// Direct field access with no computation or allocation overhead.
    #[inline]
    pub fn leading_trivia(&self) -> &GreenTrivia {
        &self.data.header.leading
    }

    /// Returns the trailing trivia associated with this token.
    ///
    /// Trailing trivia consists of whitespace, comments, and other non-semantic
    /// content that appears after the token's main content. This trivia is
    /// essential for proper PDF formatting and compliance with specification
    /// requirements.
    ///
    /// ## PDF Specification Requirements
    ///
    /// Trailing trivia often has specific requirements in PDF:
    /// - **Stream keywords**: Must be followed by a newline (ISO 32000-2 §7.3.8)
    /// - **Object endings**: Proper spacing around `endobj` keywords
    /// - **Array/dictionary separators**: Appropriate spacing between elements
    /// - **Line endings**: Platform-specific line break handling
    ///
    /// ## Examples
    ///
    /// - Stream marker: `stream\n` - the newline is mandatory trailing trivia
    /// - Dictionary separator: `,  ` - comma followed by spaces for readability
    /// - Object ending: `endobj   ` - keyword with trailing spaces before next element
    ///
    /// ## Implementation Note
    ///
    /// Like leading trivia, this is stored in the token header for efficient
    /// access and is preserved with perfect fidelity from the original PDF.
    #[inline]
    pub fn trailing_trivia(&self) -> &GreenTrivia {
        &self.data.header.trailing
    }
}

/// Debug formatting for development and diagnostic purposes.
///
/// Provides a structured view of the token's components for debugging,
/// testing, and development tools. The debug output includes all the
/// essential information about the token in a readable format.
///
/// ## Output Format
///
/// The debug representation includes:
/// - **Kind**: The syntax kind identifier for this token type
/// - **Text**: The raw byte content as a debug-formatted byte slice
/// - **Leading**: Leading trivia with its own debug representation
/// - **Trailing**: Trailing trivia with its own debug representation
///
/// ## Use Cases
///
/// - **Unit testing**: Verify token parsing results in test assertions
/// - **Development debugging**: Understand token structure during development
/// - **Error diagnostics**: Include token details in error messages
/// - **IDE integration**: Display token information in development tools
impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .field("leading", &self.leading_trivia())
            .field("trailing", &self.trailing_trivia())
            .finish()
    }
}

/// Display formatting for human-readable token representation.
///
/// Provides a clean, human-readable representation of the token's text
/// content for display purposes. This is useful for showing PDF content
/// to users, generating readable output, and creating formatted representations.
///
/// ## Encoding Handling
///
/// The display implementation uses `String::from_utf8_lossy` to handle
/// potentially non-UTF-8 content gracefully:
/// - **Valid UTF-8**: Displays as normal text
/// - **Invalid sequences**: Replaced with Unicode replacement characters (�)
/// - **Binary content**: Safely displayed without panicking
///
/// ## PDF Context
///
/// PDF files can contain various text encodings and binary content,
/// so the lossy conversion ensures display never fails while providing
/// reasonable output for debugging and user interfaces.
///
/// ## Use Cases
///
/// - **Syntax highlighting**: Display token content in editors
/// - **Error messages**: Show problematic tokens to users
/// - **PDF viewers**: Render text content appropriately
/// - **Development tools**: Show readable token content in debugging interfaces
impl fmt::Display for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.text()))
    }
}
