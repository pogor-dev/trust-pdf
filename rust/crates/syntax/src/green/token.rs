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

use crate::{
    SyntaxKind,
    arc::{arc_main::Arc, thin_arc::ThinArc},
    green::{
        GreenTokenRepr, GreenTokenReprThin, token_data::GreenTokenData, token_head::GreenTokenHead,
    },
};

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
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenToken {
        let head = GreenTokenHead::new(kind);
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
    /// &GreenTokenReprThin
    ///       ↓ Convert to Arc pointer
    /// Arc<GreenTokenReprThin>
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
            let arc = Arc::from_raw(&ptr.as_ref().data as *const GreenTokenReprThin);
            mem::transmute::<Arc<GreenTokenReprThin>, ThinArc<GreenTokenHead, u8>>(arc)
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
    /// GreenTokenRepr ───────────────┐ Full representation
    ///        ↓ pointer cast         │ (with metadata)
    /// GreenTokenReprThin ───────────┤ Normalized layout  
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
            let repr: &GreenTokenRepr = &self.ptr;

            // Step 2: Normalize layout (remove ThinArc metadata)
            //   &*(ptr as *const A as *const B) pattern:
            //   - Convert to raw pointer
            //   - Reinterpret type
            //   - Dereference and re-borrow
            let repr: &GreenTokenReprThin =
                &*(repr as *const GreenTokenRepr as *const GreenTokenReprThin);

            // Step 3: Final API view (same bytes, API methods)
            mem::transmute::<&GreenTokenReprThin, &GreenTokenData>(repr)
        }
    }
}
