//! # Green Trivia Collection - PDF Trivia Sequence Management
//!
//! Immutable, shareable collections of PDF trivia with zero-cost conversions and efficient memory layout.
//!
//! ## PDF Trivia Collections
//!
//! PDF syntax often requires sequences of trivia elements per ISO 32000-2:
//! - **Leading trivia**: Comments and whitespace before tokens
//! - **Trailing trivia**: Whitespace and comments after tokens
//! - **Xref spacing**: Fixed-width whitespace sequences (§7.5.4)
//! - **Stream boundaries**: Precise newline requirements (§7.3.8)
//!
//! ## Memory Architecture
//!
//! ```text
//! GreenTrivia                    Memory Layout
//! ┌─────────────────┐           ┌─────────────┬─────────────────────┐
//! │ ThinArc pointer │ ────────► │ Head        │ [TriviaChild; n]    │
//! └─────────────────┘           │─────────────┼─────────────────────┤
//!         |                     │ count info  │ child1, child2, ... │
//!         │ Deref (zero-cost)   └─────────────┴─────────────────────┘
//!         ▼
//! ┌─────────────────┐
//! │ GreenTriviaData │ ──► API methods: children(), header(), etc.
//! └─────────────────┘
//! ```
//!
//! ## Usage Examples
//!
//! ```text
//! PDF Fragment:      Trivia Collection:
//! %comment           ┌─ Comment("%comment")
//!                    ├─ Newline("\n")
//!   /Type            ├─ Whitespace("  ")
//!                    └─ (token: /Type)
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

type ReprThin = HeaderSlice<GreenTriviaHead, [GreenTriviaChild; 0]>;
type Repr = HeaderSlice<GreenTriviaHead, [GreenTriviaChild]>;

type ChildReprThin = HeaderSlice<GreenTriviaChildHead, [u8; 0]>;
type ChildRepr = HeaderSlice<GreenTriviaChildHead, [u8]>;

/// Immutable PDF trivia collection with efficient sharing and zero-cost data access.
///
/// Represents a sequence of trivia elements (whitespace, comments, newlines) that
/// appear together in PDF content. Supports efficient cloning via reference counting.
///
/// ```text
/// PDF Example:        Trivia Collection Elements:
/// %header comment     ┌─ Comment("%header comment")
///                     ├─ Newline("\n")
/// 1 0 obj             ├─ Whitespace("")
///                     └─ (continues to next token)
/// ```
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, GreenTriviaChild>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct GreenTriviaHead {
    _c: Count<GreenTrivia>,
}

#[repr(transparent)]
pub struct GreenTriviaData {
    /// Underlying thin representation providing access to both header and body
    data: ReprThin,
}

impl GreenTrivia {
    /// Creates a new trivia collection from an iterator of trivia children.
    ///
    /// The iterator must provide an exact size hint for efficient memory allocation.
    /// All trivia children are stored contiguously in memory for cache efficiency.
    #[inline]
    pub fn new<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = GreenTriviaChild>,
        I::IntoIter: ExactSizeIterator,
    {
        let data =
            ThinArc::from_header_and_iter(GreenTriviaHead { _c: Count::new() }, pieces.into_iter());

        GreenTrivia { ptr: data }
    }

    /// Converts the trivia collection to a raw pointer for FFI operations.
    ///
    /// # Safety
    /// The returned pointer must be converted back using `from_raw` to prevent memory leaks.
    /// The pointer remains valid as long as there are other references to the data.
    #[inline]
    pub(crate) fn into_raw(this: GreenTrivia) -> ptr::NonNull<GreenTriviaData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaData = &green;
        ptr::NonNull::from(green)
    }

    /// Creates a trivia collection from a raw pointer.
    ///
    /// # Safety
    /// The pointer must have been created by `into_raw` and not yet reclaimed.
    /// This operation assumes ownership of the reference count.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaData>) -> GreenTrivia {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaHead, GreenTriviaChild>>(arc)
        };
        GreenTrivia { ptr: arc }
    }
}

impl Borrow<GreenTriviaData> for GreenTrivia {
    #[inline]
    fn borrow(&self) -> &GreenTriviaData {
        self
    }
}

impl ops::Deref for GreenTrivia {
    type Target = GreenTriviaData;

    #[inline]
    fn deref(&self) -> &GreenTriviaData {
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
            mem::transmute::<&ReprThin, &GreenTriviaData>(repr)
        }
    }
}

impl std::fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the Deref trait to access GreenTriviaData and its Debug impl
        (**self).fmt(f)
    }
}

impl GreenTriviaData {
    /// Returns a slice of all trivia children in this collection.
    ///
    /// Children are stored contiguously in memory for efficient iteration.
    /// The slice provides zero-cost access to individual trivia elements.
    #[inline]
    pub fn children(&self) -> &[GreenTriviaChild] {
        self.data.slice()
    }

    /// Returns the total byte width of all trivia children in this collection.
    ///
    /// Calculates the cumulative width by summing the individual widths of all
    /// child trivia elements. Essential for PDF layout calculations and memory
    /// allocation planning.
    ///
    /// ## Example Usage
    ///
    /// ```text
    /// PDF trivia: "%comment\n  "
    /// Children:   [Comment(8), Newline(1), Whitespace(2)]
    /// Total width: 8 + 1 + 2 = 11 bytes
    /// ```
    #[inline]
    pub fn width(&self) -> u32 {
        self.children().iter().map(|c| c.width()).sum()
    }

    /// Returns the concatenated text content of all trivia children as a String.
    ///
    /// Efficiently combines all child trivia text into a single String using
    /// pre-calculated capacity to avoid reallocations. Critical for PDF round-trip
    /// fidelity where exact trivia preservation is required.
    ///
    /// ## Example
    ///
    /// ```text
    /// Input children: [Comment("%PDF-1.7"), Newline("\n"), Whitespace("  ")]
    /// Output string: "%PDF-1.7\n  "
    /// ```
    #[inline]
    pub fn text(&self) -> String {
        let total_width = self.width() as usize;
        let mut result = String::with_capacity(total_width);

        for child in self.children() {
            // SAFETY: We know the total width, so this won't reallocate
            unsafe {
                result.as_mut_vec().extend_from_slice(child.text());
            }
        }
        result
    }
}

impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.children() == other.children()
    }
}

impl ToOwned for GreenTriviaData {
    type Owned = GreenTrivia;

    #[inline]
    fn to_owned(&self) -> GreenTrivia {
        let green = unsafe { GreenTrivia::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenTrivia::clone(&green)
    }
}

impl fmt::Debug for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.children().iter()).finish()
    }
}

impl fmt::Display for GreenTriviaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in self.children() {
            match std::str::from_utf8(child.text()) {
                Ok(text) => write!(f, "{}", text)?,
                Err(_) => write!(f, "{:?}", child.text())?,
            }
        }
        Ok(())
    }
}

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
    /// Underlying thin Childrepresentation providing access to both header and body
    data: ChildReprThin,
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
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ChildReprThin);
            mem::transmute::<Arc<ChildReprThin>, ThinArc<GreenTriviaChildHead, u8>>(arc)
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
    /// GreenTriviaChildRepr ──────────────┐ Full Childrepresentation
    ///        ↓ pointer cast         │ (with metadata)
    /// GreenTriviaChildReprThin ──────────┤ Normalized layout  
    ///        ↓ transmute            │ (clean structure)
    /// GreenTriviaData ──────────────┘ API interface
    ///
    /// Same bytes, different type views
    /// ```
    #[inline]
    fn deref(&self) -> &GreenTriviaChildData {
        unsafe {
            // Step 1: Get full memory Childrepresentation
            let repr: &ChildRepr = &self.ptr;

            // Step 2: Normalize layout (remove metadata)
            //   &*(ptr as *const A as *const B) pattern:
            //   - Convert to raw pointer
            //   - Reinterpret type
            //   - Dereference and re-borrow
            let repr: &ChildReprThin = &*(repr as *const ChildRepr as *const ChildReprThin);

            // Step 3: Final API view (same bytes, API methods)
            mem::transmute::<&ChildReprThin, &GreenTriviaChildData>(repr)
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
    /// GreenTriviaChildReprThin  
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
    /// GreenTriviaChildReprThin
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
    fn trivia_sizes() {
        assert_eq!(2, std::mem::size_of::<GreenTriviaChildHead>());
        assert_eq!(8, std::mem::size_of::<GreenTriviaChild>());
    }

    #[test]
    fn trivia_child_sizes() {
        assert_eq!(0, std::mem::size_of::<GreenTriviaHead>());
        assert_eq!(8, std::mem::size_of::<GreenTrivia>());
    }
}
