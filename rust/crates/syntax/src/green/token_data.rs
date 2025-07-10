//! # Green Token Data - PDF Token Access Interface
//!
//! Provides the API layer for accessing PDF token content and metadata.
//! This is the final view in the token transformation chain, offering
//! zero-cost operations for token inspection and manipulation.
//!
//! ## Memory Layout (Header/Body Separation)
//!
//! ```text
//! GreenTokenData view of memory:
//!
//! ┌─────────────────────────────┬─────────────────────────────┐
//! │          HEADER             │            BODY             │
//! ├─────────────────────────────┼─────────────────────────────┤
//! │ GreenTokenHead              │ Variable-length text        │
//! │ ┌─────────────────────────┐ │ ┌─────────────────────────┐ │
//! │ │ kind: SyntaxKind        │ │ │ [u8; len]               │ │
//! │ │ _c: Count<GreenToken>   │ │ │ actual token bytes      │ │
//! │ └─────────────────────────┘ │ └─────────────────────────┘ │
//! └─────────────────────────────┴─────────────────────────────┘
//!
//! Header: Fixed-size metadata (kind, memory tracking)
//! Body:   Variable-size content (actual token text bytes)
//! ```
//!
//! ## PDF Context Examples
//!
//! ```text
//! Different token types and their data:
//!
//! Name:        Header{kind=Name, ..}      Body{"/Type"}
//! Number:      Header{kind=Number, ..}    Body{"42"}
//! String:      Header{kind=String, ..}    Body{"(Hello)"}
//! Keyword:     Header{kind=Keyword, ..}   Body{"obj"}
//! Delimiter:   Header{kind=DictStart, ..} Body{"<<"}
//! ```

use std::{fmt, mem::ManuallyDrop, ptr};

use crate::{
    SyntaxKind,
    green::{GreenTokenReprThin, token::GreenToken, trivia_child::GreenTriviaChild},
};

/// API interface for accessing PDF token data with zero-cost operations.
///
/// This struct provides the semantic interface for token elements, abstracting
/// the underlying memory representation into PDF-aware operations. Essential
/// for token inspection during parsing, validation, and document reconstruction.
///
/// ## Performance Characteristics
///
/// All operations are zero-cost abstractions:
/// - `kind()`: Direct field access from header
/// - `text()`: Pointer arithmetic to body section  
/// - `width()`: Length calculation from body size
/// - No allocations or copies during normal usage
///
/// ## PDF Token Processing
///
/// Used throughout the compiler pipeline:
/// - **Lexical Analysis**: Token classification and content extraction
/// - **Parsing**: Syntax tree construction based on token properties
/// - **Validation**: Token format verification against PDF specification
/// - **Serialization**: Round-trip reconstruction preserving exact bytes
#[repr(transparent)]
pub struct GreenTokenData {
    /// Underlying thin representation providing access to both header and body
    pub(crate) data: GreenTokenReprThin,
}

impl GreenTokenData {
    /// Returns the semantic kind of this token element.
    ///
    /// Accesses the **header** portion of the token to determine its PDF-specific
    /// classification (Name, Number, String, Keyword, etc.). Essential for
    /// syntax analysis and determining parsing behavior.
    ///
    /// ## Header Access Pattern
    ///
    /// ```text
    /// Memory Access:
    /// GreenTokenData
    ///        ↓ .data
    /// GreenTokenReprThin  
    ///        ↓ .header
    /// GreenTokenHead
    ///        ↓ .kind
    /// SyntaxKind (enum value)
    /// ```
    ///
    /// ## PDF Significance
    ///
    /// The kind determines semantic meaning in PDF processing:
    /// - `Name`: PDF names like `/Type`, `/Pages` (§7.3.5)
    /// - `Number`: Integer and real numbers `42`, `3.14` (§7.3.3)
    /// - `String`: Literal strings `(Hello)`, `<48656C6C6F>` (§7.3.4)
    /// - `Keyword`: PDF keywords `obj`, `endobj`, `stream` (§7.3.6)
    /// - `Delimiter`: Structural delimiters `<<`, `>>`, `[`, `]` (§7.3.6)
    /// - `Operator`: Content stream operators `m`, `l`, `S`, `f` (§8.1.1)
    ///
    /// ## Usage in Parsing
    ///
    /// ```text
    /// Parser Decision Tree:
    /// match token.kind() {
    ///     SyntaxKind::Name => parse_name_object(),
    ///     SyntaxKind::Number => parse_numeric_value(),
    ///     SyntaxKind::DictStart => parse_dictionary(),
    ///     ...
    /// }
    /// ```
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Returns the raw byte content of this token element.
    ///
    /// Accesses the **body** portion containing the actual token text.
    /// Critical for PDF round-trip fidelity where exact bytes, capitalization,
    /// and formatting must be preserved for document integrity.
    ///
    /// ## Body Access Pattern
    ///
    /// ```text
    /// Memory Access:
    /// GreenTokenData
    ///        ↓ .data
    /// GreenTokenReprThin
    ///        ↓ .slice()
    /// Raw slice pointer + length
    ///        ↓ from_raw_parts
    /// &[u8] (safe slice view)
    /// ```
    ///
    /// ## PDF Examples
    ///
    /// ```text
    /// Token Content:         text() Result:
    /// PDF name "/Type"   →   b"/Type"
    /// Number "42"        →   b"42"
    /// String "(Hello)"   →   b"(Hello)"
    /// Hex string "<48>"  →   b"<48656C6C6F>"
    /// Keyword "obj"      →   b"obj"
    /// Dict start "<<"    →   b"<<"
    /// ```
    ///
    /// ## Critical for PDF Compliance
    ///
    /// - **Case sensitivity**: PDF names are case-sensitive
    /// - **Whitespace preservation**: Some contexts require exact spacing
    /// - **Encoding accuracy**: String literals must preserve exact bytes
    /// - **Round-trip integrity**: Output must match input byte-for-byte
    ///
    /// ## Safety
    ///
    /// Safe because the slice is created from valid memory managed by `ThinArc`.
    /// The length is guaranteed to match allocated space in the body section.
    #[inline]
    pub fn text(&self) -> &[u8] {
        let slice = self.data.slice();
        unsafe { std::slice::from_raw_parts(slice.as_ptr(), slice.len()) }
    }

    /// Returns the byte width (length) of this token element.
    ///
    /// Computed from the **body** length for consistency with actual content.
    /// Essential for PDF layout calculations, memory usage tracking, and
    /// position management during parsing and serialization.
    ///
    /// ## Usage in PDF Processing
    ///
    /// ```text
    /// Parsing Applications:
    /// - Position tracking: current_pos += token.width()
    /// - Buffer sizing: allocate_buffer(total_width)
    /// - Offset calculations: xref_offset = base + token.width()
    /// - Memory planning: estimate_memory_usage(token_count * avg_width)
    /// ```
    ///
    /// ## Performance Note
    ///
    /// Width is computed from `text().len()` rather than storing separately
    /// to ensure consistency between header metadata and actual body content.
    /// The computation is O(1) as it only reads the slice length field.
    ///
    /// ## Examples
    ///
    /// ```text
    /// Token:           Width:
    /// "/Type"      →   6 bytes
    /// "42"         →   2 bytes  
    /// "(Hello)"    →   7 bytes
    /// "<<"         →   2 bytes
    /// "3.14159"    →   7 bytes
    /// ```
    #[inline]
    pub(crate) fn width(&self) -> u64 {
        self.text().len() as u64
    }

    #[inline]
    pub fn leading_trivia(&self) -> &GreenTriviaChild {
        &self.data.header.leading
    }

    #[inline]
    pub fn trailing_trivia(&self) -> &GreenTriviaChild {
        &self.data.header.trailing
    }
}

impl PartialEq for GreenTokenData {
    /// Compares tokens for semantic equality (kind + content).
    ///
    /// Two tokens are considered equal if they have the same classification
    /// and identical byte content. Essential for token deduplication,
    /// caching, and semantic analysis during PDF processing.
    ///
    /// ## Comparison Strategy
    ///
    /// ```text
    /// Equality Check:
    /// 1. Kind comparison (fast, single enum check)
    /// 2. Text comparison (byte-by-byte if kinds match)
    ///
    /// Short-circuit: Different kinds → immediately false
    /// ```
    ///
    /// ## PDF Semantic Examples
    ///
    /// ```text
    /// Equal Tokens:
    /// Name("/Type") == Name("/Type")     ✓
    /// Number("42") == Number("42")       ✓
    ///
    /// Different Tokens:
    /// Name("/Type") != Name("/Pages")    ✗ (different text)
    /// Name("/Type") != String("/Type")   ✗ (different kind)
    /// Number("42") != Number("042")      ✗ (different representation)
    /// ```
    ///
    /// ## Performance Notes
    ///
    /// - Kind comparison: O(1) enum equality
    /// - Text comparison: O(n) where n = min(len1, len2)
    /// - Early termination on kind mismatch for efficiency
    ///
    /// ## Usage in Collections
    ///
    /// Enables efficient use in hash-based collections:
    /// ```text
    /// HashSet<GreenTokenData>     // Token deduplication
    /// HashMap<GreenTokenData, T>  // Token-based lookup tables
    /// ```
    fn eq(&self, other: &Self) -> bool {
        // TODO: trivia equality?
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl ToOwned for GreenTokenData {
    type Owned = GreenToken;

    /// Creates an owned token from borrowed token data.
    ///
    /// Converts `&GreenTokenData` to `GreenToken` by incrementing the reference
    /// count of the underlying shared data. This is a zero-copy operation that
    /// creates a new owned handle to the same memory.
    ///
    /// ## Memory Management
    ///
    /// ```text
    /// Ownership Transfer:
    /// &GreenTokenData (borrowed)
    ///        ↓ to_owned()
    /// GreenToken (owned, ref_count++)
    ///        ↓ Same underlying memory
    /// Shared data unchanged
    /// ```
    ///
    /// ## Safety Pattern
    ///
    /// Uses `ManuallyDrop` to safely convert the reference to owned form:
    /// 1. Create `GreenToken` from raw pointer
    /// 2. Wrap in `ManuallyDrop` to prevent double-free
    /// 3. Clone to increment reference count
    /// 4. Return cloned owned version
    ///
    /// ## PDF Processing Context
    ///
    /// Commonly used when:
    /// - Converting borrowed tokens from parsing to owned tokens for storage
    /// - Creating owned copies for background processing
    /// - Building token collections that outlive the parsing context
    /// - Transferring tokens between different processing stages
    #[inline]
    fn to_owned(&self) -> GreenToken {
        let green = unsafe { GreenToken::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenToken::clone(&green)
    }
}

impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTokenData")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .finish()
    }
}

impl fmt::Display for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = String::from_utf8_lossy(self.text());
        write!(f, "{}", text)
    }
}
