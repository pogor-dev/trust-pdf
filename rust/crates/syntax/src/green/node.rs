use std::{fmt, mem, ops, ptr};

use crate::{
    arc::{arc::Arc, thin_arc::ThinArc},
    green::{
        GreenNodeRepr, GreenNodeReprThin, node_data::GreenNodeData, node_head::GreenNodeHead,
        node_slot::Slot,
    },
};

/// Internal node in the immutable tree.
/// It has other nodes and tokens as children.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct GreenNode {
    ptr: ThinArc<GreenNodeHead, Slot>,
}

impl GreenNode {
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenNodeData>) -> GreenNode {
        let arc = unsafe { Arc::from_raw(&ptr.as_ref().data as *const GreenNodeReprThin) };
        let arc =
            unsafe { mem::transmute::<Arc<GreenNodeReprThin>, ThinArc<GreenNodeHead, Slot>>(arc) };
        GreenNode { ptr: arc }
    }
}

impl ops::Deref for GreenNode {
    type Target = GreenNodeData;

    #[inline]
    fn deref(&self) -> &GreenNodeData {
        unsafe {
            let repr: &GreenNodeRepr = &self.ptr;
            let repr: &GreenNodeReprThin =
                &*(repr as *const GreenNodeRepr as *const GreenNodeReprThin);
            mem::transmute::<&GreenNodeReprThin, &GreenNodeData>(repr)
        }
    }
}

impl fmt::Debug for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenNodeData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenNodeData = self;
        fmt::Display::fmt(data, f)
    }
}
