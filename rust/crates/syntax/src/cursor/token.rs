use std::{fmt, hash, ops::Range, ptr};

use crate::{GreenTokenData, cursor::node_data::NodeData};

#[derive(Debug)]
pub struct SyntaxToken {
    ptr: ptr::NonNull<NodeData>,
}

impl SyntaxToken {
    fn new(green: &GreenTokenData, parent: SyntaxNode, index: u32, offset: u32) -> SyntaxToken {}

    #[inline]
    fn data(&self) -> &NodeData {}
    #[inline]
    pub fn kind(&self) -> SyntaxKind {}
    #[inline]
    pub fn text(&self) -> &[u8] {}
    #[inline]
    pub fn full_text(&self) -> &[u8] {}
    #[inline]
    pub fn width(&self) -> usize {}
    #[inline]
    pub fn full_width(&self) -> usize {}
    #[inline]
    pub fn span(&self) -> Range<usize> {}
    #[inline]
    pub fn full_span(&self) -> Range<usize> {}
    #[inline]
    pub fn green(&self) -> &GreenTokenData {}
    #[inline]
    pub fn parent(&self) -> Option<SyntaxNode> {}
}

impl Clone for SyntaxToken {
    #[inline]
    fn clone(&self) -> Self {}
}

impl Drop for SyntaxToken {
    #[inline]
    fn drop(&mut self) {}
}

// Identity semantics for hash & eq
impl PartialEq for SyntaxToken {
    #[inline]
    fn eq(&self, other: &SyntaxToken) -> bool {}
}

impl Eq for SyntaxToken {}

impl hash::Hash for SyntaxToken {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {}
}

impl fmt::Display for SyntaxToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}
