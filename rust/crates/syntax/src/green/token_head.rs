//! # Green Token Head - PDF Token Metadata Header
//!
//! Fixed-size header containing token classification and memory management metadata.
//! Part of the header/body separation pattern for efficient token representation.
//!
//! ## Memory Layout Role
//!
//! ```text
//! Complete Token Structure:
//! ┌─────────────────────────────┬─────────────────────────────┐
//! │        THIS MODULE          │        SEPARATE MODULE      │
//! │     GreenTokenHead          │      Variable Text Data     │
//! ├─────────────────────────────┼─────────────────────────────┤
//! │ kind: SyntaxKind            │ [u8; len]                   │
//! │ _c: Count<GreenToken>       │ actual token bytes          │
//! └─────────────────────────────┴─────────────────────────────┘
//!    ↑ Fixed size (metadata)       ↑ Variable size (content)
//! ```
//!
//! ## PDF Token Classification
//!
//! The header identifies PDF token types per ISO 32000-2:
//! - **Names**: PDF name objects starting with '/' (§7.3.5)
//! - **Numbers**: Integer and real numeric values (§7.3.3)
//! - **Strings**: Literal and hexadecimal strings (§7.3.4)
//! - **Keywords**: PDF reserved words like 'obj', 'stream' (§7.3.6)
//! - **Delimiters**: Structural boundaries '<<', '>>', '[', ']' (§7.3.6)
//! - **Operators**: Content stream operators 'm', 'l', 'S' (§8.1.1)

use countme::Count;

use crate::{
    SyntaxKind,
    green::{token::GreenToken, trivia::GreenTrivia},
};

/// Header metadata for PDF token elements with memory management tracking.
///
/// Contains the essential classification information for tokens while enabling
/// memory usage monitoring through the `countme` crate integration. This
/// separation allows efficient metadata access without loading token content.
///
/// ## Design Rationale
///
/// Separating metadata (header) from content (body) provides several benefits:
/// - **Memory efficiency**: Fixed header size regardless of token length
/// - **Cache locality**: Metadata can be accessed without loading large tokens
/// - **Type safety**: Header operations don't require content validation
/// - **Performance**: Kind-based dispatch without string processing
///
/// ## PDF Context Examples
///
/// ```text
/// PDF Content:     Header Created:
/// "/Type"      →   GreenTokenHead { kind: Name, .. }
/// "42"         →   GreenTokenHead { kind: Number, .. }
/// "(Hello)"    →   GreenTokenHead { kind: String, .. }
/// "obj"        →   GreenTokenHead { kind: Keyword, .. }
/// "<<"         →   GreenTokenHead { kind: DictStart, .. }
/// "m"          →   GreenTokenHead { kind: Operator, .. }
/// ```
#[derive(PartialEq, Eq, Hash)]
pub(super) struct GreenTokenHead {
    /// Semantic classification of the token element.
    ///
    /// Determines how this token should be interpreted during PDF processing:
    /// - **Names**: Object identifiers and dictionary keys (`/Type`, `/Pages`)
    /// - **Numbers**: Numeric values for coordinates, sizes, indices (`42`, `3.14`)
    /// - **Strings**: Text content and binary data (`(Hello)`, `<48656C6C6F>`)
    /// - **Keywords**: PDF structural commands (`obj`, `endobj`, `stream`)
    /// - **Delimiters**: Container boundaries (`<<`, `>>`, `[`, `]`)
    /// - **Operators**: Graphics state and content operators (`m`, `l`, `S`, `f`)
    ///
    /// This field drives parsing decisions, semantic analysis, and serialization
    /// behavior throughout the compiler pipeline.
    pub(super) kind: SyntaxKind,

    pub(super) leading: GreenTrivia,
    pub(super) trailing: GreenTrivia,

    /// Memory allocation tracking for development and debugging.
    ///
    /// Enables monitoring token creation/destruction patterns during development
    /// and testing. The underscore prefix indicates this is primarily for
    /// internal tooling rather than core PDF processing functionality.
    ///
    /// ## Usage During Development
    ///
    /// ```text
    /// Debug builds:     Track allocation/deallocation patterns
    /// Release builds:   Zero-cost abstraction (likely optimized away)
    /// Testing:          Detect memory leaks in token handling
    /// Profiling:         Analyze token usage patterns
    /// ```
    ///
    /// Particularly useful for:
    /// - Detecting token memory leaks during parsing
    /// - Profiling token allocation patterns in large documents
    /// - Validating token cleanup in error handling paths
    /// - Benchmarking memory efficiency improvements
    _c: Count<GreenToken>,
}

impl GreenTokenHead {
    /// Creates a new token header with the specified PDF token classification.
    ///
    /// Initializes both the semantic kind and memory tracking for a new token element.
    /// This is typically called during the lexical analysis phase when token
    /// types are first identified from PDF content.
    ///
    /// ## Parameters
    ///
    /// * `kind` - The semantic classification of this token (Name, Number, String, etc.)
    ///
    /// ## PDF Processing Context
    ///
    /// Called when the lexer identifies token patterns:
    /// ```text
    /// Lexer encounters:     Creates header:
    /// '/Type'           →   new(SyntaxKind::Name)
    /// '42'              →   new(SyntaxKind::Number)  
    /// '(Hello World)'   →   new(SyntaxKind::String)
    /// 'obj'             →   new(SyntaxKind::Keyword)
    /// '<<'              →   new(SyntaxKind::DictStart)
    /// 'm'               →   new(SyntaxKind::Operator)
    /// ```
    ///
    /// The header is then combined with the actual token bytes to form
    /// a complete `GreenToken` instance through `ThinArc::from_header_and_iter()`.
    ///
    /// ## Memory Management
    ///
    /// The `Count<GreenToken>` field is automatically initialized to track
    /// this token's lifecycle, enabling development-time memory analysis
    /// and leak detection without runtime overhead in release builds.
    ///
    /// ## Compiler Pipeline Usage
    ///
    /// ```text
    /// Lexical Analysis:
    /// Raw PDF bytes → Token classification → GreenTokenHead::new(kind)
    ///                                                    ↓
    /// Token Construction:                                
    /// Header + content bytes → ThinArc allocation → GreenToken
    /// ```
    pub(super) fn new(kind: SyntaxKind, leading: GreenTrivia, trailing: GreenTrivia) -> Self {
        Self {
            kind,
            leading,
            trailing,
            _c: Count::new(),
        }
    }
}
