use countme::Count;

use crate::{GreenNodeElement, GreenFlags, SyntaxKind, arc::HeaderSlice};

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenNodeHead {
    full_width: u32,   // 4 bytes
    kind: SyntaxKind,  // 2 bytes
    flags: GreenFlags, // 1 byte
    _c: Count<GreenNode>,
}

type Repr = HeaderSlice<GreenNodeHead, [GreenNodeElement]>;
type ReprThin = HeaderSlice<GreenNodeHead, [GreenNodeElement; 0]>;

#[repr(transparent)]
pub struct GreenNodeData {
    data: ReprThin,
}

impl GreenNodeData {
    /// Kind of this token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Text of this token.
    #[inline]
    pub fn text(&self) -> &[u8] {
        self.kind().get_text()
    }

    /// Full text of this token, including leading and trailing trivia.
    #[inline]
    pub fn full_text(&self) -> &[u8] {
        self.kind().get_text()
    }

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn width(&self) -> u8 {
        self.kind().get_text().len() as u8
    }

    /// Returns the full width of this token, including leading and trailing trivia.
    #[inline]
    pub fn full_width(&self) -> u32 {
        self.data.header.full_width
    }

    /// Returns the flags of this token.
    #[inline]
    pub fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }
}
