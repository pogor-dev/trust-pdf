//! # Green Trivia - PDF Syntactic Trivia Management
//!
//! Immutable, shareable PDF trivia (whitespace, newlines, comments) with zero-cost conversions.
//!
//! ## PDF Trivia Significance
//!
//! PDF whitespace often has semantic meaning per ISO 32000-2:
//! - **Stream separators**: Required `\n` after `stream` (§7.3.8)
//! - **Xref tables**: Fixed-width spacing (§7.5.4)
//! - **Content streams**: Space-separated tokens (§8.1.1)
//!
//! ## Memory Architecture
//!
//! ```text
//! GreenTrivia                    Memory Layout
//! ┌─────────────────┐           ┌─────────────┬─────────────┐
//! │ ThinArc pointer │ ────────► │ Head        │ Text Data   │
//! └─────────────────┘           │─────────────┼─────────────┤
//!         |                     │ kind, len   │ [u8; len]   │
//!         │ Deref (zero-cost)   └─────────────┴─────────────┘
//!         ▼
//! ┌─────────────────┐
//! │ GreenTriviaData │ ──► API methods: kind(), text(), etc.
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
};

type ReprThin = HeaderSlice<GreenTriviaChildHead, [u8; 0]>;
type Repr = HeaderSlice<GreenTriviaChildHead, [u8]>;

/// Immutable PDF trivia with efficient sharing and zero-cost data access.
///
/// ```text
/// PDF Example:   Trivia Elements:
/// 7 0 obj %abc    ┌─ Whitespace(" ")
///                 ├─ Comment("abc")
///                 └─ Newline("\n")
/// ```
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTriviaChild {
    /// Single allocation for metadata + text content
    ptr: ThinArc<GreenTriviaChildHead, u8>,
}

#[derive(PartialEq, Eq, Hash)]
struct GreenTriviaChildHead {
    kind: SyntaxKind,
    _c: Count<GreenTriviaChild>,
}

#[repr(transparent)]
pub struct GreenTriviaChildData {
    /// Underlying thin representation providing access to both header and body
    data: ReprThin,
}

impl GreenTriviaChild {
    /// Creates PDF trivia preserving exact bytes for round-trip fidelity.
    ///
    /// ```text
    /// Input: kind=Newline, text=b"\n"
    ///        ↓
    /// ┌─────────────┬────────┐
    /// │ Head        │ Text   │
    /// ├─────────────┼────────┤
    /// │ kind=newline│ "\n"   │
    /// └─────────────┴────────┘
    /// ```
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenTriviaChild {
        let head = GreenTriviaChildHead {
            kind,
            _c: Count::new(),
        };

        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenTriviaChild { ptr }
    }

    /// Transfers ownership to raw pointer for FFI/custom allocators.
    ///
    /// ```text
    /// GreenTrivia (owned)
    ///       ↓ ManuallyDrop (prevent cleanup)
    /// GreenTrivia (wrapped)
    ///       ↓ Deref
    /// &GreenTriviaData
    ///       ↓ Extract pointer
    /// NonNull<GreenTriviaData>
    /// ```
    ///
    /// Caller must eventually free the returned pointer.
    #[inline]
    pub(crate) fn into_raw(this: GreenTriviaChild) -> ptr::NonNull<GreenTriviaChildData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaChildData = &green;
        ptr::NonNull::from(green)
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaChildData>) -> GreenTriviaChild {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaChildHead, u8>>(arc)
        };
        GreenTriviaChild { ptr: arc }
    }
}

impl Borrow<GreenTriviaChildData> for GreenTriviaChild {
    /// Borrows trivia data for collections and generic operations.
    ///
    /// Enables using `GreenTrivia` in hash maps/sets with `GreenTriviaData` keys,
    /// supporting efficient lookups without ownership transfer.
    ///
    /// ```text
    /// Use Cases:
    /// HashMap<GreenTriviaData, T>
    ///     ↓ .get(&green_trivia)
    /// Uses this Borrow impl automatically
    ///
    /// Collection Operations:
    /// set.contains(&trivia)  ──► Borrow::borrow() ──► &GreenTriviaData
    /// map.get(&trivia)       ──► Borrow::borrow() ──► &GreenTriviaData
    /// ```
    ///
    /// Implementation leverages `Deref` coercion for zero-cost conversion.
    #[inline]
    fn borrow(&self) -> &GreenTriviaChildData {
        self
    }
}

impl ops::Deref for GreenTriviaChild {
    type Target = GreenTriviaChildData;

    /// Zero-cost conversion via memory layout reinterpretation.
    ///
    /// ```text
    /// Memory Transformation Chain:
    ///
    /// ThinArc<Head,u8>
    ///        ↓ &self.ptr
    /// GreenTriviaRepr ──────────────┐ Full representation
    ///        ↓ pointer cast         │ (with metadata)
    /// GreenTriviaReprThin ──────────┤ Normalized layout  
    ///        ↓ transmute            │ (clean structure)
    /// GreenTriviaData ──────────────┘ API interface
    ///
    /// Same bytes, different type views
    /// ```
    #[inline]
    fn deref(&self) -> &GreenTriviaChildData {
        unsafe {
            // Step 1: Get full memory representation
            let repr: &Repr = &self.ptr;

            // Step 2: Normalize layout (remove metadata)
            //   &*(ptr as *const A as *const B) pattern:
            //   - Convert to raw pointer
            //   - Reinterpret type
            //   - Dereference and re-borrow
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);

            // Step 3: Final API view (same bytes, API methods)
            mem::transmute::<&ReprThin, &GreenTriviaChildData>(repr)
        }
    }
}

impl std::fmt::Debug for GreenTriviaChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the Deref trait to access GreenTriviaData and its Debug impl
        (**self).fmt(f)
    }
}

impl GreenTriviaChildData {
    /// Returns the semantic kind of this trivia element.
    ///
    /// Accesses the **header** portion of the trivia to determine its PDF-specific
    /// classification (whitespace, newline, comment).
    ///
    /// ## Header Access Pattern
    ///
    /// ```text
    /// Memory Access:
    /// GreenTriviaData
    ///        ↓ .data
    /// GreenTriviaReprThin  
    ///        ↓ .header
    /// GreenTriviaHead
    ///        ↓ .kind
    /// SyntaxKind (enum value)
    /// ```
    ///
    /// ## PDF Significance
    ///
    /// The kind determines semantic meaning in PDF processing:
    /// - `Newline`: Line breaks with potential semantic significance
    /// - `Whitespace`: Spaces and tabs for formatting and separation
    /// - `Comment`: PDF comments starting with '%' character
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Returns the raw byte content of this trivia element.
    ///
    /// Accesses the **body** portion containing the actual trivia text.
    /// Essential for PDF round-trip fidelity where exact bytes matter.
    ///
    /// ## Body Access Pattern
    ///
    /// ```text
    /// Memory Access:
    /// GreenTriviaData
    ///        ↓ .data
    /// GreenTriviaReprThin
    ///        ↓ .slice()
    /// Raw slice pointer + length
    ///        ↓ from_raw_parts
    /// &[u8] (safe slice view)
    /// ```
    ///
    /// ## PDF Examples
    ///
    /// ```text
    /// Newline:       text() → b"\n"
    /// Whitespace:    text() → b" "
    /// Whitespace:    text() → b"     " (5 spaces)
    /// Comment:       text() → b"%PDF-1.7"
    /// ```
    ///
    /// ## Safety
    ///
    /// Safe because the slice is created from valid memory managed by `ThinArc`.
    /// The length is stored in the header and guaranteed to match allocated space.
    #[inline]
    pub fn text(&self) -> &[u8] {
        let slice = self.data.slice();
        unsafe { std::slice::from_raw_parts(slice.as_ptr(), slice.len()) }
    }

    /// Returns the byte width (length) of this trivia element.
    ///
    /// Computed from the **body** length for consistency with actual content.
    /// Useful for PDF layout calculations and memory usage tracking.
    ///
    /// ## Usage in PDF Processing
    ///
    /// ```text
    /// Layout calculations:
    /// - Fixed-width formatting: width() for alignment verification
    /// - Trivia parsing: width() for boundary detection  
    /// - Memory management: width() for allocation size planning
    /// ```
    ///
    /// ## Implementation Note
    ///
    /// Could alternatively read from `header.len`, but using `text().len()`
    /// ensures consistency between header metadata and actual body content.
    #[inline]
    pub fn width(&self) -> u32 {
        self.text().len() as u32
    }
}

impl PartialEq for GreenTriviaChildData {
    /// Compares trivia for semantic equality (kind + content).
    ///
    /// Essential for PDF trivia deduplication, caching, and incremental updates.
    /// Two trivia elements are equal if both kind and byte content match exactly.
    ///
    /// ```text
    /// Equality Check:
    ///
    /// Step 1: Kind comparison    Step 2: Content comparison
    /// ┌─────────────────┐       ┌─────────────────────┐
    /// │ self.kind()     │  ==   │ self.text()         │
    /// │ other.kind()    │       │ other.text()        │
    /// └─────────────────┘       └─────────────────────┘
    ///         │                           │
    ///         └───────────┬───────────────┘
    ///                     ▼
    ///              true if both match
    ///
    /// Examples:
    /// Equal:     Newline("\n") == Newline("\n")         ✓
    /// Different: Newline("\n") != Whitespace("\n")     ✗ (kind differs)
    /// Different: Whitespace(" ") != Whitespace("  ")   ✗ (content differs)
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl ToOwned for GreenTriviaChildData {
    type Owned = GreenTriviaChild;

    /// Converts borrowed trivia to owned with reference counting (zero-copy).
    ///
    /// Creates a new `GreenTrivia` sharing the same memory via `ThinArc`,
    /// enabling safe ownership transfer without duplicating trivia bytes.
    ///
    /// ```text
    /// Borrowed → Owned Conversion:
    ///
    /// &GreenTriviaData           GreenTrivia
    /// ┌──────────────────┐      ┌──────────────────┐
    /// │ Borrowed view    │ ──►  │ Owned reference  │
    /// │ of shared memory │      │ with ref count   │
    /// └──────────────────┘      └──────────────────┘
    ///         │                           │
    ///         └─────── Same bytes ────────┘
    ///                 (zero copy)
    /// ```
    ///
    /// Implementation: Reconstruct from raw pointer → wrap in `ManuallyDrop` → clone reference count.
    #[inline]
    fn to_owned(&self) -> GreenTriviaChild {
        let green = unsafe { GreenTriviaChild::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTriviaChild::clone(&green)
    }
}

impl fmt::Debug for GreenTriviaChildData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenTrivia")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .finish()
    }
}

impl fmt::Display for GreenTriviaChildData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(2, std::mem::size_of::<GreenTriviaChildHead>());
        assert_eq!(8, std::mem::size_of::<GreenTriviaChild>());
    }
}
