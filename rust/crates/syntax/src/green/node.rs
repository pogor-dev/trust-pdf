use std::{
    borrow::{Borrow, Cow},
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    NodeOrToken, SyntaxKind,
    arc::{arc_main::Arc, thin_arc::ThinArc},
    green::{
        GreenNodeHead, GreenNodeRepr, GreenNodeReprThin, element::GreenElement,
        node_child::GreenChild, node_data::GreenNodeData,
    },
};

/// Internal node in the immutable tree.
/// It has other nodes and tokens as children.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GreenNode {
    ptr: ThinArc<GreenNodeHead, GreenChild>,
}

impl GreenNode {
    /// Creates new Node.
    #[inline]
    pub(crate) fn new<I>(kind: SyntaxKind, children: I) -> GreenNode
    where
        I: IntoIterator<Item = GreenElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut width: u32 = 0;
        let mut full_width: u32 = 0;
        let children = children.into_iter().map(|el| {
            let rel_offset = full_width;
            width += el.width();
            full_width += el.full_width();
            match el {
                NodeOrToken::Node(node) => GreenChild::Node { rel_offset, node },
                NodeOrToken::Token(token) => GreenChild::Token { rel_offset, token },
            }
        });

        let data = ThinArc::from_header_and_iter(GreenNodeHead::new(kind, 0, 0), children);

        // XXX: fixup `full_width` after construction, because we can't iterate
        // `children` twice.
        let data = {
            let mut data = Arc::from_thin(data);
            Arc::get_mut(&mut data).unwrap().header.width = width;
            Arc::get_mut(&mut data).unwrap().header.full_width = full_width;
            Arc::into_thin(data)
        };

        GreenNode { ptr: data }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenNode) -> ptr::NonNull<GreenNodeData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenNodeData = &green;
        ptr::NonNull::from(green)
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenNodeData>) -> GreenNode {
        unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const GreenNodeReprThin);
            let arc =
                mem::transmute::<Arc<GreenNodeReprThin>, ThinArc<GreenNodeHead, GreenChild>>(arc);
            GreenNode { ptr: arc }
        }
    }
}

impl Borrow<GreenNodeData> for GreenNode {
    #[inline]
    fn borrow(&self) -> &GreenNodeData {
        self
    }
}

impl From<Cow<'_, GreenNodeData>> for GreenNode {
    #[inline]
    fn from(cow: Cow<'_, GreenNodeData>) -> Self {
        cow.into_owned()
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

impl ops::Deref for GreenNode {
    type Target = GreenNodeData;

    #[inline]
    fn deref(&self) -> &GreenNodeData {
        let repr: &GreenNodeRepr = &self.ptr;
        unsafe {
            let repr: &GreenNodeReprThin =
                &*(repr as *const GreenNodeRepr as *const GreenNodeReprThin);
            mem::transmute::<&GreenNodeReprThin, &GreenNodeData>(repr)
        }
    }
}
