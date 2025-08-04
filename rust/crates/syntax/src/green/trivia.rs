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
    arc::{arc_main::Arc, header_slice::HeaderSlice, thin_arc::ThinArc},
    green::trivia_child::GreenTriviaChild,
};

type ReprThin = HeaderSlice<GreenTriviaHead, [GreenTriviaChild; 0]>;
type Repr = HeaderSlice<GreenTriviaHead, [GreenTriviaChild]>;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(0, std::mem::size_of::<GreenTriviaHead>());
        assert_eq!(8, std::mem::size_of::<GreenTrivia>());
    }
}
