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
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    SyntaxKind,
    arc::{arc_main::Arc, thin_arc::ThinArc},
    green::{
        GreenTriviaRepr, GreenTriviaReprThin, trivia_data::GreenTriviaData,
        trivia_head::GreenTriviaHead,
    },
};

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
pub(crate) struct GreenTrivia {
    /// Single allocation for metadata + text content
    ptr: ThinArc<GreenTriviaHead, u8>,
}

impl GreenTrivia {
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
    pub(crate) fn new(kind: SyntaxKind, text: &[u8]) -> GreenTrivia {
        let head = GreenTriviaHead::new(kind);
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenTrivia { ptr }
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
    pub(crate) fn into_raw(this: GreenTrivia) -> ptr::NonNull<GreenTriviaData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaData = &green;
        ptr::NonNull::from(green)
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaData>) -> GreenTrivia {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const GreenTriviaReprThin);
            mem::transmute::<Arc<GreenTriviaReprThin>, ThinArc<GreenTriviaHead, u8>>(arc)
        };
        GreenTrivia { ptr: arc }
    }
}

impl Borrow<GreenTriviaData> for GreenTrivia {
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
    fn borrow(&self) -> &GreenTriviaData {
        self
    }
}

impl ops::Deref for GreenTrivia {
    type Target = GreenTriviaData;

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
    fn deref(&self) -> &GreenTriviaData {
        unsafe {
            // Step 1: Get full memory representation
            let repr: &GreenTriviaRepr = &self.ptr;

            // Step 2: Normalize layout (remove metadata)
            //   &*(ptr as *const A as *const B) pattern:
            //   - Convert to raw pointer
            //   - Reinterpret type
            //   - Dereference and re-borrow
            let repr: &GreenTriviaReprThin =
                &*(repr as *const GreenTriviaRepr as *const GreenTriviaReprThin);

            // Step 3: Final API view (same bytes, API methods)
            mem::transmute::<&GreenTriviaReprThin, &GreenTriviaData>(repr)
        }
    }
}

impl std::fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the Deref trait to access GreenTriviaData and its Debug impl
        (**self).fmt(f)
    }
}
