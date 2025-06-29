use std::{borrow::Borrow, fmt, mem::ManuallyDrop, ptr};

use crate::green::{
    GreenNodeHead, GreenNodeReprThin, kind::RawSyntaxKind, node::GreenNode, node_slot::Slot,
    node_slots::Slots,
};

#[repr(transparent)]
pub(crate) struct GreenNodeData {
    pub(crate) data: GreenNodeReprThin,
}

impl GreenNodeData {
    #[inline]
    fn header(&self) -> &GreenNodeHead {
        &self.data.header
    }

    /// Kind of this node.
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.header().kind
    }

    #[inline]
    pub(crate) fn slice(&self) -> &[Slot] {
        self.data.slice()
    }

    /// Returns the length of the text covered by this node.
    #[inline]
    pub fn text_len(&self) -> u64 {
        self.header().text_len
    }

    /// Returns the slots of this node. Every node of a specific kind has the same number of slots
    /// to allow using fixed offsets to retrieve a specific child even if some other child is missing.
    #[inline]
    pub fn slots(&self) -> Slots<'_> {
        Slots {
            raw: self.slice().iter(),
        }
    }
}

impl ToOwned for GreenNodeData {
    type Owned = GreenNode;

    #[inline]
    fn to_owned(&self) -> GreenNode {
        unsafe {
            let green = GreenNode::from_raw(ptr::NonNull::from(self));
            let green = ManuallyDrop::new(green);
            GreenNode::clone(&green)
        }
    }
}

impl Borrow<GreenNodeData> for GreenNode {
    #[inline]
    fn borrow(&self) -> &GreenNodeData {
        self
    }
}

impl fmt::Debug for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenNode")
            .field("kind", &self.kind())
            .field("text_len", &self.text_len())
            .field("n_slots", &self.slots().len())
            .finish()
    }
}

impl fmt::Display for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in self.slots() {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}
