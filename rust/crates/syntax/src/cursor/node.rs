use std::{hash, ptr};

pub struct SyntaxNode {
    ptr: ptr::NonNull<NodeData>,
}

impl SyntaxNode {
    pub fn new_root(green: GreenNode) -> SyntaxNode {}
    #[inline]
    fn data(&self) -> &NodeData {}
    #[inline]
    pub fn kind(&self) -> SyntaxKind {}
    #[inline]
    fn offset(&self) -> u32 {}
    #[inline]
    pub fn text(&self) -> SyntaxText {}
    #[inline]
    pub fn green(&self) -> Cow<'_, GreenNodeData> {}
    #[inline]
    fn green_ref(&self) -> &GreenNodeData {}
}

impl Clone for SyntaxNode {
    #[inline]
    fn clone(&self) -> Self {}
}

impl Drop for SyntaxNode {
    #[inline]
    fn drop(&mut self) {}
}

// Identity semantics for hash & eq
impl PartialEq for SyntaxNode {
    #[inline]
    fn eq(&self, other: &SyntaxNode) -> bool {}
}

impl Eq for SyntaxNode {}

impl hash::Hash for SyntaxNode {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {}
}

impl fmt::Debug for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}
