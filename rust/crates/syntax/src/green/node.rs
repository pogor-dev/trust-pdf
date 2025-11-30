use std::{fmt, ptr::NonNull, slice};

use countme::Count;

use crate::{GreenToken, GreenTriviaList, NodeOrToken, SyntaxKind};

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct GreenNodeHead {
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
#[repr(C)]
pub(super) struct GreenNodeData {
    head: GreenNodeHead,       // 18 bytes
    children: [GreenChild; 0], // 0 bytes, actual children are stored inline after this struct
}

#[derive(Clone, Copy)]
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

    /// Returns the bytes excluding the first token's leading trivia and last token's trailing trivia
    #[inline]
    pub fn bytes(&self) -> Vec<u8> {
        self.write_to(false, false)
    }

    /// Returns the full bytes including all trivia
    #[inline]
    pub fn full_bytes(&self) -> Vec<u8> {
        self.write_to(true, true)
    }

    /// Returns the width excluding the first token's leading trivia and last token's trailing trivia.
    /// This is similar to Roslyn's approach for calculating the "true" width of a node's content.
    #[inline]
    pub fn width(&self) -> u32 {
        let first_leading_width = self.first_token().map(|t| t.leading_trivia().full_width()).unwrap_or(0);
        let last_trailing_width = self.last_token().map(|t| t.trailing_trivia().full_width()).unwrap_or(0);
        self.full_width() - first_leading_width - last_trailing_width
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

    /// Returns the leading trivia from the first terminal token in the node tree
    #[inline]
    pub fn leading_trivia(&self) -> Option<&GreenTriviaList> {
        self.first_token().map(|token| token.leading_trivia())
    }

    /// Returns the trailing trivia from the last terminal token in the node tree
    #[inline]
    pub fn trailing_trivia(&self) -> Option<&GreenTriviaList> {
        self.last_token().map(|token| token.trailing_trivia())
    }

    /// Returns the node's text as a byte vector.
    ///
    /// Similar to Roslyn's WriteTo implementation, uses an explicit stack to avoid
    /// stack overflow on deeply nested structures.
    ///
    /// # Parameters
    /// * `leading` - If true, include the first token's leading trivia
    /// * `trailing` - If true, include the last token's trailing trivia
    fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        let mut output = Vec::new();

        // Use explicit stack to handle deeply recursive structures without stack overflow
        let mut stack: Vec<(NodeOrToken<&GreenNode, &GreenToken>, bool, bool)> = Vec::new();
        stack.push((NodeOrToken::Node(self), leading, trailing));

        while let Some((item, current_leading, current_trailing)) = stack.pop() {
            match item {
                NodeOrToken::Token(token) => {
                    output.extend_from_slice(&token.write_to(current_leading, current_trailing));
                }
                NodeOrToken::Node(node) => {
                    let children = node.children();
                    if children.is_empty() {
                        continue;
                    }

                    let first_index = 0;
                    let last_index = children.len() - 1;

                    // Process children in reverse order (last to first), pushing to stack
                    // so they're popped in correct order (first to last)
                    for i in (first_index..=last_index).rev() {
                        let child = &children[i];
                        let is_first = i == first_index;
                        let is_last = i == last_index;
                        let include_leading = current_leading || !is_first;
                        let include_trailing = current_trailing || !is_last;

                        match child {
                            GreenChild::Node { node: child_node, .. } => {
                                stack.push((NodeOrToken::Node(child_node), include_leading, include_trailing));
                            }
                            GreenChild::Token { token, .. } => {
                                stack.push((NodeOrToken::Token(token), include_leading, include_trailing));
                            }
                        }
                    }
                }
            }
        }

        output
    }

    /// Returns the first terminal token in the node tree
    fn first_token(&self) -> Option<&GreenToken> {
        self.children().first().and_then(|child| match child {
            GreenChild::Token { token, .. } => Some(token),
            GreenChild::Node { node, .. } => node.first_token(),
        })
    }

    /// Returns the last terminal token in the node tree
    fn last_token(&self) -> Option<&GreenToken> {
        self.children().last().and_then(|child| match child {
            GreenChild::Token { token, .. } => Some(token),
            GreenChild::Node { node, .. } => node.last_token(),
        })
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

impl PartialEq for GreenNode {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.full_width() == other.full_width() && self.children() == other.children()
    }
}

impl Eq for GreenNode {}

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
        let bytes = self.full_bytes();
        for &byte in &bytes {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

// SAFETY: The pointer is valid.
unsafe impl Send for GreenNode {}
unsafe impl Sync for GreenNode {}

#[derive(Debug, PartialEq, Eq, Clone)]
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
mod memory_layout_tests {

    use super::*;

    #[test]
    fn test_memory_layout() {
        assert_eq!(std::mem::size_of::<GreenNodeHead>(), 8); // 6 bytes + 2 bytes padding
        assert_eq!(std::mem::align_of::<GreenNodeHead>(), 4); // 4 bytes alignment

        assert_eq!(std::mem::size_of::<GreenNodeData>(), 8); // 6 bytes + 2 bytes padding
        assert_eq!(std::mem::align_of::<GreenNodeData>(), 8); // 8 bytes alignment

        assert_eq!(std::mem::size_of::<GreenChild>(), 16); // 12 bytes + 4 bytes padding
        assert_eq!(std::mem::align_of::<GreenChild>(), 8); // 8 bytes alignment
    }
}

#[cfg(test)]
mod node_tests {

    use crate::green::arena::GreenTree;

    use super::*;

    const TOKEN_KIND: SyntaxKind = SyntaxKind(1);
    const NODE_KIND: SyntaxKind = SyntaxKind(100);
    const TRIVIA_KIND: SyntaxKind = SyntaxKind(200);

    #[test]
    fn test_kind() {
        let mut arena = GreenTree::new();
        let empty_trivia = arena.alloc_trivia_list(&[]);
        let token = arena.alloc_token(TOKEN_KIND, b"test", empty_trivia, empty_trivia);
        let node = arena.alloc_node(NODE_KIND, token.full_width(), 1, [GreenChild::Token { token, rel_offset: 0 }].into_iter());
        assert_eq!(node.kind(), NODE_KIND);
    }
}
