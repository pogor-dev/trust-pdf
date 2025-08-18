//! # Green Token - PDF Lexical Token Management
//!
//! Immutable, shareable PDF tokens with zero-cost conversions and efficient memory layout.

use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use countme::Count;

use crate::{
    GreenTrivia, SyntaxKind,
    arc::{arc_main::Arc, header_slice::HeaderSlice, thin_arc::ThinArc},
};

type Repr = HeaderSlice<GreenTokenHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;

/// Immutable PDF token with efficient sharing and zero-cost data access.
///
/// Represents a single lexical token from PDF content, preserving exact bytes
/// for round-trip fidelity. Supports efficient cloning via reference counting.
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
    #[inline]
    pub(crate) fn into_raw(this: GreenToken) -> ptr::NonNull<GreenTokenData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTokenData = &green;
        ptr::NonNull::from(green)
    }

    /// Reconstructs owned token from raw pointer created by `into_raw()`.
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
    #[inline]
    fn deref(&self) -> &GreenTokenData {
        unsafe {
            // Step 1: Get full memory representation
            let repr: &Repr = &self.ptr;

            // Step 2: Normalize layout (remove ThinArc metadata)
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);

            // Step 3: Final API view (same bytes, API methods)
            mem::transmute::<&ReprThin, &GreenTokenData>(repr)
        }
    }
}

impl GreenTokenData {
    /// Returns the semantic kind of this token element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Returns the raw byte content of this token element.
    #[inline]
    pub fn text(&self) -> &[u8] {
        let slice = self.data.slice();
        unsafe { std::slice::from_raw_parts(slice.as_ptr(), slice.len()) }
    }

    /// Returns the complete text content including leading trivia, token text, and trailing trivia.
    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.full_width() as usize);

        // Concatenate leading trivia + token text + trailing trivia
        result.extend_from_slice(&self.leading_trivia().text());
        result.extend_from_slice(self.text());
        result.extend_from_slice(&self.trailing_trivia().text());

        result
    }

    /// Returns the byte width (length) of this token element.
    #[inline]
    pub fn width(&self) -> usize {
        self.text().len()
    }

    /// Returns the total byte width including leading and trailing trivia.
    #[inline]
    pub fn full_width(&self) -> usize {
        let leading = self.leading_trivia().width();
        let trailing = self.trailing_trivia().width();
        self.width() + leading + trailing
    }

    /// Returns the leading trivia associated with this token.
    #[inline]
    pub fn leading_trivia(&self) -> &GreenTrivia {
        &self.data.header.leading
    }

    /// Returns the trailing trivia associated with this token.
    #[inline]
    pub fn trailing_trivia(&self) -> &GreenTrivia {
        &self.data.header.trailing
    }
}

impl PartialEq for GreenTokenData {
    /// Compares tokens for semantic equality (kind + content + trivia).
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
            && self.text() == other.text()
            && self.leading_trivia() == other.leading_trivia()
            && self.trailing_trivia() == other.trailing_trivia()
    }
}

impl ToOwned for GreenTokenData {
    type Owned = GreenToken;

    /// Creates an owned token from borrowed token data.
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
