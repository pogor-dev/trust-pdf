use std::{borrow::Cow, cell::Cell, ptr};

use crate::{
    SyntaxKind,
    green::{GreenNode, GreenNodeData},
    red::{Green, NodeData, free},
};

pub struct SyntaxNode {
    pub(super) ptr: ptr::NonNull<NodeData>,
}

impl SyntaxNode {
    pub fn new_root(green: GreenNode) -> SyntaxNode {
        let green = GreenNode::into_raw(green);
        let green = Green::Node { ptr: Cell::new(green) };
        SyntaxNode {
            ptr: NodeData::new(None, 0, 0u64, green),
        }
    }

    pub(super) fn new_child(green: &GreenNodeData, parent: SyntaxNode, index: u32, offset: u64) -> SyntaxNode {
        let green = Green::Node { ptr: Cell::new(green.into()) };
        SyntaxNode {
            ptr: NodeData::new(Some(parent), index, offset, green),
        }
    }

    #[inline]
    fn data(&self) -> &NodeData {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub(super) fn can_take_ptr(&self) -> bool {
        self.data().rc.get() == 1
    }

    #[inline]
    pub(super) fn take_ptr(self) -> ptr::NonNull<NodeData> {
        assert!(self.can_take_ptr());
        let ret = self.ptr;
        // don't change the refcount when self gets dropped
        std::mem::forget(self);
        ret
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data().kind()
    }

    #[inline]
    fn offset(&self) -> u64 {
        self.data().offset()
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.data().index()
    }

    #[inline]
    pub fn green(&self) -> Cow<'_, GreenNodeData> {
        let green_ref = self.green_ref();
        Cow::Borrowed(green_ref)
    }

    #[inline]
    fn green_ref(&self) -> &GreenNodeData {
        self.data().green().into_node().unwrap()
    }

    #[inline]
    pub fn parent(&self) -> Option<SyntaxNode> {
        self.data().parent_node()
    }
}

impl Clone for SyntaxNode {
    #[inline]
    fn clone(&self) -> Self {
        self.data().inc_rc();
        SyntaxNode { ptr: self.ptr }
    }
}

impl Drop for SyntaxNode {
    #[inline]
    fn drop(&mut self) {
        if self.data().dec_rc() {
            unsafe { free(self.ptr) }
        }
    }
}
