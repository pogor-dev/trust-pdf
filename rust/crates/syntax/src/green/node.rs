use std::{
    borrow::{Borrow, Cow},
    fmt, iter,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use countme::Count;

use crate::{
    NodeOrToken, SyntaxKind,
    arc::{arc_main::Arc, header_slice::HeaderSlice, thin_arc::ThinArc},
    green::{element::GreenElement, node_child::GreenChild, node_children::NodeChildren},
};

type Repr = HeaderSlice<GreenNodeHead, [GreenChild]>;
type ReprThin = HeaderSlice<GreenNodeHead, [GreenChild; 0]>;

/// Internal node in the immutable tree.
/// It has other nodes and tokens as children.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GreenNode {
    ptr: ThinArc<GreenNodeHead, GreenChild>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GreenNodeHead {
    kind: SyntaxKind,
    width: u32,
    full_width: u32,
    _c: Count<GreenNode>,
}

impl GreenNode {
    /// Creates new Node.
    #[inline]
    pub fn new<I>(kind: SyntaxKind, children: I) -> GreenNode
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

        let head = GreenNodeHead {
            kind,
            width: 0,
            full_width: 0,
            _c: Count::new(),
        };
        let data = ThinArc::from_header_and_iter(head, children);

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
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            let arc = mem::transmute::<Arc<ReprThin>, ThinArc<GreenNodeHead, GreenChild>>(arc);
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
        let repr: &Repr = &self.ptr;
        unsafe {
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenNodeData>(repr)
        }
    }
}

#[repr(transparent)]
pub struct GreenNodeData {
    data: ReprThin,
}

impl GreenNodeData {
    #[inline]
    pub fn header(&self) -> &GreenNodeHead {
        &self.data.header
    }

    #[inline]
    pub fn slice(&self) -> &[GreenChild] {
        self.data.slice()
    }

    /// Kind of this node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.header().kind
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.header().width
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        self.header().full_width
    }

    /// Children of this node.
    #[inline]
    pub fn children(&self) -> NodeChildren<'_> {
        NodeChildren {
            raw: self.slice().iter(),
        }
    }

    #[must_use]
    pub fn replace_child(&self, index: usize, new_child: GreenElement) -> GreenNode {
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
    pub fn insert_child(&self, index: usize, new_child: GreenElement) -> GreenNode {
        self.splice_children(index..index, iter::once(new_child))
    }

    #[must_use]
    pub fn remove_child(&self, index: usize) -> GreenNode {
        self.splice_children(index..=index, iter::empty())
    }

    #[must_use]
    pub fn splice_children<R, I>(&self, range: R, replace_with: I) -> GreenNode
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
