use std::{fmt, hash, ptr};

#[derive(Debug)]
pub(crate) struct SyntaxTrivia {
    // ptr: ptr::NonNull<NodeData>,
}

impl SyntaxTrivia {
    // fn new(green: &GreenTokenData, parent: SyntaxToken, index: u32, offset: u32) -> SyntaxTrivia {}

    // #[inline]
    // fn data(&self) -> &NodeData {}
    // #[inline]
    // pub fn kind(&self) -> SyntaxKind {}
    // #[inline]
    // pub fn text(&self) -> &[u8] {}
    // #[inline]
    // pub fn width(&self) -> usize {}
    // #[inline]
    // pub fn span(&self) -> Range<isize> {}
    // #[inline]
    // pub fn green(&self) -> &GreenTokenData {}
    // #[inline]
    // pub fn parent(&self) -> Option<SyntaxToken> {}
}

// impl Clone for SyntaxTrivia {
//     #[inline]
//     fn clone(&self) -> Self {}
// }

// impl Drop for SyntaxTrivia {
//     #[inline]
//     fn drop(&mut self) {}
// }

// // Identity semantics for hash & eq
// impl PartialEq for SyntaxTrivia {
//     #[inline]
//     fn eq(&self, other: &SyntaxTrivia) -> bool {}
// }

// impl Eq for SyntaxTrivia {}

// impl hash::Hash for SyntaxTrivia {
//     #[inline]
//     fn hash<H: hash::Hasher>(&self, state: &mut H) {}
// }

// impl fmt::Display for SyntaxTrivia {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
// }
