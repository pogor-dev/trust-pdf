use std::{
    fmt,
    ops::{self, Range},
};

use crate::{
    SyntaxKind,
    green::{
        GreenNodeHead, GreenNodeReprThin, element::GreenElement, element_ref::GreenElementRef,
        node::GreenNode, node_child::GreenChild, node_children::NodeChildren,
    },
};

#[repr(transparent)]
pub struct GreenNodeData {
    data: GreenNodeReprThin,
}

impl GreenNodeData {
    #[inline]
    fn header(&self) -> &GreenNodeHead {}

    #[inline]
    fn slice(&self) -> &[GreenChild] {}

    /// Kind of this node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {}

    /// Returns the length of the text covered by this node.
    #[inline]
    pub fn width(&self) -> u32 {}

    #[inline]
    pub fn full_width(&self) -> u32 {}

    /// Children of this node.
    #[inline]
    pub fn children(&self) -> NodeChildren<'_> {}

    pub(crate) fn child_at_range(
        &self,
        rel_range: Range,
    ) -> Option<(usize, u32, GreenElementRef<'_>)> {
    }

    #[must_use]
    pub fn replace_child(&self, index: usize, new_child: GreenElement) -> GreenNode {}

    #[must_use]
    pub fn insert_child(&self, index: usize, new_child: GreenElement) -> GreenNode {}

    #[must_use]
    pub fn remove_child(&self, index: usize) -> GreenNode {}

    #[must_use]
    pub fn splice_children<R, I>(&self, range: R, replace_with: I) -> GreenNode
    where
        R: ops::RangeBounds<usize>,
        I: IntoIterator<Item = GreenElement>,
    {
    }
}

impl PartialEq for GreenNodeData {
    fn eq(&self, other: &Self) -> bool {}
}

impl ToOwned for GreenNodeData {
    type Owned = GreenNode;

    #[inline]
    fn to_owned(&self) -> GreenNode {}
}

impl fmt::Debug for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl fmt::Display for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}
