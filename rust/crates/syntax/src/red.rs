mod element;
mod node;
mod token;

use std::{cell::Cell, mem::ManuallyDrop, ptr};

use countme::Count;

use crate::{
    GreenToken, SyntaxKind,
    green::{GreenElementRef, GreenNode, GreenNodeData, GreenTokenData},
};

pub use self::{element::SyntaxElement, node::SyntaxNode, token::SyntaxToken};

enum Green {
    Node { ptr: Cell<ptr::NonNull<GreenNodeData>> },
    Token { ptr: ptr::NonNull<GreenTokenData> },
}

struct _SyntaxElement;

struct NodeData {
    _c: Count<_SyntaxElement>,

    rc: Cell<u32>,
    parent: Cell<Option<ptr::NonNull<NodeData>>>,
    index: Cell<u32>,
    green: Green,

    /// Absolute offset for immutable nodes, unused for mutable nodes.
    offset: u64,
}

impl NodeData {
    #[inline]
    fn new(parent: Option<SyntaxNode>, index: u32, offset: u64, green: Green) -> ptr::NonNull<NodeData> {
        let parent = ManuallyDrop::new(parent);
        let res = NodeData {
            _c: Count::new(),
            rc: Cell::new(1),
            parent: Cell::new(parent.as_ref().map(|it| it.ptr)),
            index: Cell::new(index),
            green,
            offset,
        };
        unsafe { ptr::NonNull::new_unchecked(Box::into_raw(Box::new(res))) }
    }

    #[inline]
    fn inc_rc(&self) {
        let rc = match self.rc.get().checked_add(1) {
            Some(it) => it,
            None => std::process::abort(),
        };
        self.rc.set(rc)
    }

    #[inline]
    fn dec_rc(&self) -> bool {
        let rc = self.rc.get() - 1;
        self.rc.set(rc);
        rc == 0
    }

    #[inline]
    fn key(&self) -> (ptr::NonNull<()>, u64) {
        let ptr = match &self.green {
            Green::Node { ptr } => ptr.get().cast(),
            Green::Token { ptr } => ptr.cast(),
        };
        (ptr, self.offset())
    }

    #[inline]
    fn parent_node(&self) -> Option<SyntaxNode> {
        let parent = self.parent()?;
        debug_assert!(matches!(parent.green, Green::Node { .. }));
        parent.inc_rc();
        Some(SyntaxNode {
            ptr: ptr::NonNull::from(parent),
        })
    }

    #[inline]
    fn parent(&self) -> Option<&NodeData> {
        self.parent.get().map(|it| unsafe { &*it.as_ptr() })
    }

    #[inline]
    fn green(&self) -> GreenElementRef<'_> {
        match &self.green {
            Green::Node { ptr } => GreenElementRef::Node(unsafe { &*ptr.get().as_ptr() }),
            Green::Token { ptr } => GreenElementRef::Token(unsafe { ptr.as_ref() }),
        }
    }

    #[inline]
    fn index(&self) -> u32 {
        self.index.get()
    }

    #[inline]
    fn offset(&self) -> u64 {
        self.offset
    }

    #[inline]
    fn kind(&self) -> SyntaxKind {
        self.green().kind()
    }
}

#[inline(never)]
unsafe fn free(mut data: ptr::NonNull<NodeData>) {
    unsafe {
        loop {
            debug_assert_eq!(data.as_ref().rc.get(), 0);
            let node = Box::from_raw(data.as_ptr());
            match node.parent.take() {
                Some(parent) => {
                    debug_assert!(parent.as_ref().rc.get() > 0);
                    if parent.as_ref().dec_rc() {
                        data = parent;
                    } else {
                        break;
                    }
                }
                None => {
                    match &node.green {
                        Green::Node { ptr } => {
                            let _ = GreenNode::from_raw(ptr.get());
                        }
                        Green::Token { ptr } => {
                            let _ = GreenToken::from_raw(*ptr);
                        }
                    }
                    break;
                }
            }
        }
    }
}
