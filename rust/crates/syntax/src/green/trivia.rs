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
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    arc::{arc_main::Arc, thin_arc::ThinArc},
    green::{
        GreenTriviaRepr, GreenTriviaReprThin, trivia_child::GreenTriviaChild,
        trivia_data::GreenTriviaData, trivia_head::GreenTriviaHead,
    },
};

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
pub(crate) struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, GreenTriviaChild>,
}

impl GreenTrivia {
    /// Creates a new trivia collection from an iterator of trivia children.
    ///
    /// The iterator must provide an exact size hint for efficient memory allocation.
    /// All trivia children are stored contiguously in memory for cache efficiency.
    #[inline]
    pub(crate) fn new<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = GreenTriviaChild>,
        I::IntoIter: ExactSizeIterator,
    {
        let data = ThinArc::from_header_and_iter(GreenTriviaHead::new(), pieces.into_iter());

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
            let arc = Arc::from_raw(&ptr.as_ref().data as *const GreenTriviaReprThin);
            mem::transmute::<Arc<GreenTriviaReprThin>, ThinArc<GreenTriviaHead, GreenTriviaChild>>(
                arc,
            )
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
