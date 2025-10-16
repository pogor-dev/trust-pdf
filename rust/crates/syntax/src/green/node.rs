use std::{fmt, ptr::NonNull, slice};

use countme::Count;

use crate::{GreenToken, SyntaxKind};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    // TODO: change to u64? Do we want to support files > 4GB? Some cfg for that?
    full_width: u32,   // 4 bytes
    kind: SyntaxKind,  // 2 bytes
    children_len: u16, // 2 bytes
    _c: Count<GreenNode>,
}

impl GreenNodeHead {
    #[inline]
    pub(super) fn new(kind: SyntaxKind, full_width: u32, children_len: u16) -> Self {
        Self {
            kind,
            full_width,
            children_len,
            _c: Count::new(),
        }
    }

    #[inline]
    pub(super) fn layout(children_len: u16) -> std::alloc::Layout {
        std::alloc::Layout::new::<GreenNodeHead>()
            .extend(std::alloc::Layout::array::<GreenChild>(children_len as usize).expect("too big node"))
            .expect("too big node")
            .0
            .pad_to_align()
    }
}

/// This is used to store the node in the arena.
/// The actual text is stored inline after the head.
#[derive(Debug, Eq, PartialEq, Hash)]
#[repr(C)]
pub(super) struct GreenNodeData {
    head: GreenNodeHead,       // 18 bytes
    children: [GreenChild; 0], // 0 bytes, actual children are stored inline after this struct
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenNode {
    /// INVARIANT: This points at a valid `GreenNodeData` followed by `children_len` `GreenChild`s,
    /// with `#[repr(C)]`.
    pub(super) data: NonNull<GreenNodeData>,
}

impl GreenNode {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.header().kind
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        self.header().full_width
    }

    #[inline]
    pub fn children_len(&self) -> u16 {
        self.header().children_len
    }

    #[inline]
    pub(crate) fn children(&self) -> &[GreenChild] {
        // SAFETY: `data`'s invariant.
        unsafe { slice::from_raw_parts(self.children_ptr_mut(), self.header().children_len as usize) }
    }

    #[inline]
    fn header(&self) -> &GreenNodeHead {
        // SAFETY: `data` points to a valid `GreenNodeData`.
        unsafe { &self.data.as_ref().head }
    }

    /// Does not require the pointer to be valid.
    #[inline]
    pub(super) fn header_ptr_mut(&self) -> *mut GreenNodeHead {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { &raw mut (*self.data.as_ptr()).head }
    }

    #[inline]
    pub(super) fn children_ptr_mut(&self) -> *mut GreenChild {
        // SAFETY: `&raw mut` doesn't require the data to be valid, only allocated.
        unsafe { (&raw mut (*self.data.as_ptr()).children).cast::<GreenChild>() }
    }
}

impl fmt::Debug for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenNode")
            .field("kind", &self.kind())
            .field("full_width", &self.full_width())
            .field("children_len", &self.children_len())
            .finish()
    }
}

impl fmt::Display for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in self.children() {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenNode {}
unsafe impl Sync for GreenNode {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum GreenChild {
    Node { node: GreenNode, rel_offset: u32 },
    Token { token: GreenToken, rel_offset: u32 },
}

impl GreenChild {
    #[inline]
    pub(crate) fn kind(&self) -> SyntaxKind {
        match self {
            GreenChild::Node { node, .. } => node.kind(),
            GreenChild::Token { token, .. } => token.kind(),
        }
    }

    #[inline]
    pub(crate) fn as_node(&self) -> Option<&GreenNode> {
        match self {
            GreenChild::Node { node, .. } => Some(node),
            GreenChild::Token { .. } => None,
        }
    }

    #[inline]
    pub(crate) fn as_token(&self) -> Option<&GreenToken> {
        match self {
            GreenChild::Node { .. } => None,
            GreenChild::Token { token, .. } => Some(token),
        }
    }
}

impl fmt::Display for GreenChild {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node { rel_offset: _, node } => fmt::Display::fmt(node, f),
            Self::Token { rel_offset: _, token } => fmt::Display::fmt(token, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenNodeHead>(), 8); // 6 bytes + 2 bytes padding
        assert_eq!(std::mem::align_of::<GreenNodeHead>(), 4); // 4 bytes alignment

        assert_eq!(std::mem::size_of::<GreenNodeData>(), 8); // 6 bytes + 2 bytes padding
        assert_eq!(std::mem::align_of::<GreenNodeData>(), 8); // 8 bytes alignment

        assert_eq!(std::mem::size_of::<GreenChild>(), 16); // 12 bytes + 4 bytes padding
        assert_eq!(std::mem::align_of::<GreenChild>(), 8); // 8 bytes alignment
    }
}
