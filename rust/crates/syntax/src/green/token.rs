//! # Green Token - PDF Lexical Token Management
//!
//! Immutable, shareable PDF tokens with zero-cost conversions and efficient memory layout.
//!
//! ## PDF Token Significance
//!
//! PDF tokens are the fundamental building blocks of PDF content per ISO 32000-2:
//! - **Literals**: Names (`/Type`), strings (`(Hello)`), numbers (`42`, `3.14`)
//! - **Keywords**: Commands (`obj`, `endobj`, `stream`, `endstream`)
//! - **Operators**: Content stream operations (`m`, `l`, `S`, `f`)
//! - **Delimiters**: Structural boundaries (`<<`, `>>`, `[`, `]`)
//!
//! ## Memory Architecture
//!
//! ```text
//! GreenToken                     Memory Layout
//! ┌─────────────────┐           ┌─────────────┬─────────────┐
//! │ ThinArc pointer │ ────────► │ Head        │ Text Data   │
//! └─────────────────┘           │─────────────┼─────────────┤
//!         |                     │ kind, len   │ [u8; len]   │
//!         │ Deref (zero-cost)   └─────────────┴─────────────┘
//!         ▼
//! ┌─────────────────┐
//! │ GreenTokenData  │ ──► API methods: kind(), text(), etc.
//! └─────────────────┘
//! ```

use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use countme::Count;

use crate::{
    SyntaxKind,
    arc::{arc_main::Arc, header_slice::HeaderSlice, thin_arc::ThinArc},
    green::trivia::GreenTrivia,
};

type Repr = HeaderSlice<GreenTokenHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;

/// Immutable PDF token with efficient sharing and zero-cost data access.
///
/// Represents a single lexical token from PDF content, preserving exact bytes
/// for round-trip fidelity. Supports efficient cloning via reference counting.
///
/// ```text
/// PDF Example:       Token Elements:
/// /Type /Catalog     ┌─ GreenToken { kind: Name, text: "/Type" }
/// <<                 ├─ GreenToken { kind: Whitespace, text: " " }
/// /Pages 1 0 R       ├─ GreenToken { kind: Name, text: "/Catalog" }
/// >>                 ├─ GreenToken { kind: DictStart, text: "<<" }
///                    └─ GreenToken { kind: Name, text: "/Pages" }
///                    └─ ...
/// ```
///
/// ## Performance Characteristics
///
/// - **Clone**: O(1) - increments reference count
/// - **Access**: O(1) - direct memory access via deref
/// - **Memory**: Single allocation for metadata + content
/// - **Sharing**: Thread-safe reference counting
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenToken {
    /// Single allocation for metadata + text content
    ptr: ThinArc<GreenTokenHead, u8>,
}

#[derive(PartialEq, Eq, Hash)]
struct GreenTokenHead {
    kind: SyntaxKind,
    leading: GreenTrivia,
    trailing: GreenTrivia,
    _c: Count<GreenToken>,
}

#[repr(transparent)]
pub struct GreenTokenData {
    /// Underlying thin representation providing access to both header and body
    data: ReprThin,
}

impl GreenToken {
    /// Creates PDF token preserving exact bytes for round-trip fidelity.
    ///
    /// Combines token classification with raw content in a single allocation,
    /// essential for maintaining PDF parsing accuracy and enabling lossless
    /// document reconstruction.
    ///
    /// ## Memory Layout Creation
    ///
    /// ```text
    /// Input: kind=Name, text=b"/Type"
    ///        ↓
    /// ┌─────────────┬────────────┐
    /// │ Head        │ Text       │
    /// ├─────────────┼────────────┤
    /// │ kind=Name   │ "/Type"    │
    /// │ count=1     │ (5 bytes)  │
    /// └─────────────┴────────────┘
    /// ```
    ///
    /// ## PDF Context Examples
    ///
    /// ```text
    /// PDF Content:           Token Creation:
    /// "/Type"            →   new(SyntaxKind::Name, b"/Type")
    /// "<<"               →   new(SyntaxKind::DictStart, b"<<")
    /// "42"               →   new(SyntaxKind::Number, b"42")
    /// "obj"              →   new(SyntaxKind::Keyword, b"obj")
    /// "(Hello World)"    →   new(SyntaxKind::String, b"(Hello World)")
    /// ```
    ///
    /// ## Parameters
    ///
    /// * `kind` - The semantic classification of this token (Name, Number, etc.)
    /// * `text` - The exact byte sequence from the PDF file
    ///
    /// Essential for preserving PDF parsing fidelity where exact whitespace,
    /// capitalization, and byte sequences determine semantic meaning.
    #[inline]
    pub fn new(
        kind: SyntaxKind,
        text: &[u8],
        leading: GreenTrivia,
        trailing: GreenTrivia,
    ) -> GreenToken {
        let head = GreenTokenHead {
            kind,
            leading,
            trailing,
            _c: Count::new(),
        };

        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenToken { ptr }
    }

    /// Transfers ownership to raw pointer for FFI/custom allocators.
    ///
    /// Converts owned `GreenToken` to a raw pointer suitable for C FFI,
    /// custom memory management, or specialized container operations.
    /// The caller assumes responsibility for eventual cleanup.
    ///
    /// ## Memory Transfer Pattern
    ///
    /// ```text
    /// GreenToken (owned)
    ///       ↓ ManuallyDrop (prevent cleanup)
    /// GreenToken (wrapped)
    ///       ↓ Deref
    /// &GreenTokenData
    ///       ↓ Extract pointer
    /// NonNull<GreenTokenData>
    /// ```
    ///
    /// ## Usage Context
    ///
    /// - **FFI Integration**: Passing tokens to C/C++ PDF libraries
    /// - **Custom Allocators**: Integration with specialized memory pools
    /// - **Container Optimization**: Raw storage in specialized collections
    /// - **Serialization**: Direct memory access for high-performance I/O
    ///
    /// ## Safety Requirements
    ///
    /// Caller must ensure the returned pointer is eventually freed using
    /// `from_raw()` to prevent memory leaks and maintain proper reference counting.
    #[inline]
    pub(crate) fn into_raw(this: GreenToken) -> ptr::NonNull<GreenTokenData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTokenData = &green;
        ptr::NonNull::from(green)
    }

    /// Reconstructs owned token from raw pointer created by `into_raw()`.
    ///
    /// Restores proper ownership and reference counting from a raw pointer,
    /// typically used when receiving tokens back from FFI or custom allocators.
    /// This is the safe counterpart to `into_raw()`.
    ///
    /// ## Memory Reconstruction Process
    ///
    /// ```text
    /// Raw Pointer
    ///       ↓ Dereference safely
    /// &GreenTokenData
    ///       ↓ Access .data field
    /// &ReprThin
    ///       ↓ Convert to Arc pointer
    /// Arc<ReprThin>
    ///       ↓ Transmute layout
    /// ThinArc<GreenTokenHead, u8>
    ///       ↓ Wrap in GreenToken
    /// GreenToken (owned)
    /// ```
    ///
    /// ## Safety Requirements
    ///
    /// - Pointer must have been created by `into_raw()` or equivalent
    /// - Pointer must not have been freed already
    /// - No concurrent access to the pointed data during reconstruction
    /// - The underlying memory layout must match expected structure
    ///
    /// ## PDF Processing Context
    ///
    /// Essential for:
    /// - **Parser Integration**: Receiving tokens from C-based PDF parsers
    /// - **Memory Pool Recovery**: Retrieving tokens from custom allocators
    /// - **Deserialization**: Reconstructing tokens from serialized data
    /// - **FFI Boundaries**: Safe ownership transfer across language boundaries
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> GreenToken {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTokenHead, u8>>(arc)
        };
        GreenToken { ptr: arc }
    }
}

impl Borrow<GreenTokenData> for GreenToken {
    /// Borrows token data for collections and generic operations.
    ///
    /// Enables using `GreenToken` in hash maps/sets with `GreenTokenData` keys,
    /// supporting efficient lookups without ownership transfer. Essential for
    /// PDF token caching and deduplication during parsing.
    ///
    /// ```text
    /// Use Cases:
    /// HashMap<GreenTokenData, ParsedValue>
    ///     ↓ .get(&green_token)
    /// Uses this Borrow impl automatically
    ///
    /// Collection Operations:
    /// token_set.contains(&token)  ──► Borrow::borrow() ──► &GreenTokenData
    /// token_map.get(&token)       ──► Borrow::borrow() ──► &GreenTokenData
    /// ```
    ///
    /// ## PDF Processing Examples
    ///
    /// ```text
    /// Token Deduplication:
    /// HashSet<GreenTokenData> name_cache;
    /// name_cache.insert(token.borrow());  // Efficient deduplication
    ///
    /// Symbol Table Lookup:
    /// HashMap<GreenTokenData, Definition> symbols;
    /// symbols.get(&token);  // Fast symbol resolution
    /// ```
    ///
    /// Implementation leverages `Deref` coercion for zero-cost conversion.
    #[inline]
    fn borrow(&self) -> &GreenTokenData {
        self
    }
}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Display::fmt(data, f)
    }
}

impl ops::Deref for GreenToken {
    type Target = GreenTokenData;

    /// Zero-cost conversion via memory layout reinterpretation.
    ///
    /// Provides direct access to token data through safe type reinterpretation,
    /// enabling efficient access to token properties without allocation or copying.
    /// This is the core mechanism for the zero-cost abstraction pattern.
    ///
    /// ```text
    /// Memory Transformation Chain:
    ///
    /// ThinArc<Head,u8>
    ///        ↓ &self.ptr
    /// Repr ───────────────┐ Full representation
    ///        ↓ pointer cast         │ (with metadata)
    /// ReprThin ───────────┤ Normalized layout  
    ///        ↓ transmute            │ (clean structure)
    /// GreenTokenData ────────────┘ API interface
    ///
    /// Same bytes, different type views
    /// ```
    ///
    /// ## PDF Processing Benefits
    ///
    /// - **Zero-cost access**: No allocation for token.kind() or token.text()
    /// - **Type safety**: Compile-time guarantees about memory layout
    /// - **Performance**: Direct memory access without indirection
    /// - **Ergonomics**: Natural method call syntax (token.text() vs token.deref().text())
    ///
    /// ## Memory Safety
    ///
    /// Safe because:
    /// 1. `ThinArc` guarantees valid memory layout
    /// 2. `transmute` preserves underlying bytes
    /// 3. Type representations are layout-compatible
    /// 4. Reference lifetime tied to `GreenToken` lifetime
    ///
    /// The pointer cast pattern `&*(ptr as *const A as *const B)`:
    /// - Converts to raw pointer (removes borrow checker restrictions)
    /// - Reinterprets type without changing bytes
    /// - Re-borrows with appropriate lifetime
    #[inline]
    fn deref(&self) -> &GreenTokenData {
        unsafe {
            // Step 1: Get full memory representation
            let repr: &Repr = &self.ptr;

            // Step 2: Normalize layout (remove ThinArc metadata)
            //   &*(ptr as *const A as *const B) pattern:
            //   - Convert to raw pointer
            //   - Reinterpret type
            //   - Dereference and re-borrow
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);

            // Step 3: Final API view (same bytes, API methods)
            mem::transmute::<&ReprThin, &GreenTokenData>(repr)
        }
    }
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
    /// ReprThin  
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
    /// ReprThin
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
    pub fn width(&self) -> u32 {
        self.text().len() as u32
    }

    /// Returns the total byte width including leading and trailing trivia.
    ///
    /// Calculates the complete width by summing the token's content width with
    /// the widths of its associated leading and trailing trivia. Essential for
    /// accurate layout calculations and position tracking during PDF processing.
    ///
    /// ## Width Calculation Formula
    ///
    /// ```text
    /// full_width = token_width + leading_trivia_width + trailing_trivia_width
    ///
    /// Example:
    /// Leading trivia: "  %comment\n"     (11 bytes)
    /// Token content:  "/Type"            (6 bytes)  
    /// Trailing trivia: " "               (1 byte)
    /// Total width:    11 + 6 + 1 = 18 bytes
    /// ```
    #[inline]
    pub fn full_width(&self) -> u32 {
        let leading = self.leading_trivia().width();
        let trailing = self.trailing_trivia().width();
        (self.width() + leading + trailing) as u32
    }

    /// Returns the leading trivia associated with this token.
    ///
    /// Provides access to trivia elements (whitespace, comments, newlines) that
    /// appear before this token in the source text. Leading trivia is semantically
    /// attached to the token for PDF layout preservation and round-trip fidelity.
    ///
    /// ## Trivia Attachment Model
    ///
    /// ```text
    /// PDF Source:    "  %comment\n  /Type  %trailing\n"
    /// Tokenization:  [leading="  %comment\n  "][token="/Type"][trailing="  %trailing\n"]
    ///                          ↑                      ↑                    ↑
    ///                    leading_trivia()         token content      trailing_trivia()
    /// ```
    ///
    /// ## Usage Examples
    ///
    /// ```text
    /// Dictionary entry:
    /// Leading: "  "           (indentation)
    /// Token:   "/Pages"       (dictionary key)
    ///
    /// Object header:
    /// Leading: "\n"           (line break)
    /// Token:   "7"            (object number)
    ///
    /// Stream boundary:
    /// Leading: "%comment\n  " (comment + spacing)
    /// Token:   "stream"       (stream keyword)
    /// ```
    #[inline]
    pub fn leading_trivia(&self) -> &GreenTrivia {
        &self.data.header.leading
    }

    /// Returns the trailing trivia associated with this token.
    ///
    /// Provides access to trivia elements (whitespace, comments, newlines) that
    /// appear after this token in the source text. Trailing trivia is semantically
    /// attached to the token for PDF layout preservation and round-trip fidelity.
    ///
    /// ## Trivia Attachment Model
    ///
    /// ```text
    /// PDF Source:    "/Type  %comment\n  /Pages"
    /// Tokenization:  [token="/Type"][trailing="  %comment\n  "][token="/Pages"]
    ///                        ↑                   ↑                      ↑
    ///                  token content      trailing_trivia()       next token
    /// ```
    ///
    /// ## Usage Examples
    ///
    /// ```text
    /// Dictionary key-value:
    /// Token:    "/Type"      (dictionary key)
    /// Trailing: " "          (separator space)
    ///
    /// Object number:
    /// Token:    "7"          (object number)
    /// Trailing: " "          (space before generation)
    ///
    /// Array element:
    /// Token:    "42"         (array element)
    /// Trailing: "  %note\n"  (spacing + comment)
    /// ```
    #[inline]
    pub fn trailing_trivia(&self) -> &GreenTrivia {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(24, std::mem::size_of::<GreenTokenHead>());
        assert_eq!(8, std::mem::size_of::<GreenToken>());
    }
}
