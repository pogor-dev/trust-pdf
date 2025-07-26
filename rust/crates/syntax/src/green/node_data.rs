use std::{
    fmt, iter,
    mem::ManuallyDrop,
    ops::{self},
    ptr,
};

use crate::{
    SyntaxKind,
    green::{
        GreenNodeHead, GreenNodeReprThin, element::GreenElement, node::GreenNode,
        node_child::GreenChild, node_children::NodeChildren,
    },
};

#[repr(transparent)]
pub struct GreenNodeData {
    pub(super) data: GreenNodeReprThin,
}

impl GreenNodeData {
    #[inline]
    fn header(&self) -> &GreenNodeHead {
        &self.data.header
    }

    #[inline]
    fn slice(&self) -> &[GreenChild] {
        self.data.slice()
    }

    /// Kind of this node.
    #[inline]
    pub(crate) fn kind(&self) -> SyntaxKind {
        self.header().kind
    }

    #[inline]
    pub(crate) fn width(&self) -> u32 {
        self.header().full_width
    }

    #[inline]
    pub(crate) fn full_width(&self) -> u32 {
        self.header().full_width
    }

    /// Children of this node.
    #[inline]
    pub(crate) fn children(&self) -> NodeChildren<'_> {
        NodeChildren {
            raw: self.slice().iter(),
        }
    }

    #[must_use]
    pub(crate) fn replace_child(&self, index: usize, new_child: GreenElement) -> GreenNode {
        let mut replacement = Some(new_child);
        let children = self.children().enumerate().map(|(i, child)| {
            if i == index {
                replacement.take().unwrap()
            } else {
                child.to_owned()
            }
        });
        GreenNode::new(self.kind(), children)
    }

    #[must_use]
    pub(crate) fn insert_child(&self, index: usize, new_child: GreenElement) -> GreenNode {
        self.splice_children(index..index, iter::once(new_child))
    }

    #[must_use]
    pub(crate) fn remove_child(&self, index: usize) -> GreenNode {
        self.splice_children(index..=index, iter::empty())
    }

    #[must_use]
    pub(crate) fn splice_children<R, I>(&self, range: R, replace_with: I) -> GreenNode
    where
        R: ops::RangeBounds<usize>,
        I: IntoIterator<Item = GreenElement>,
    {
        let mut children: Vec<_> = self.children().map(|it| it.to_owned()).collect();
        children.splice(range, replace_with);
        GreenNode::new(self.kind(), children)
    }
}

impl PartialEq for GreenNodeData {
    fn eq(&self, other: &Self) -> bool {
        self.header() == other.header() && self.slice() == other.slice()
    }
}

impl ToOwned for GreenNodeData {
    type Owned = GreenNode;

    #[inline]
    fn to_owned(&self) -> GreenNode {
        let green = unsafe { GreenNode::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenNode::clone(&green)
    }
}

impl fmt::Debug for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenNode")
            .field("kind", &self.kind())
            .field("full_width", &self.full_width())
            .field("n_children", &self.children().len())
            .finish()
    }
}

impl fmt::Display for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in self.children() {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}
