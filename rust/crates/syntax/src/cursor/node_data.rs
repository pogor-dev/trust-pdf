use std::{ops::Range, ptr, rc::Rc};

use countme::Count;

use crate::cursor::{
    green_element::GreenElementRef, node_kind::NodeKind, weak_green_element::WeakGreenElement,
};

#[derive(Debug)]
pub(crate) struct NodeData {
    _c: Count<_SyntaxElement>,

    pub(crate) kind: NodeKind,
    pub(crate) slot: u32,
    /// Absolute offset for immutable nodes, unused for mutable nodes.
    pub(crate) offset: u64,
}

impl NodeData {
    #[inline]
    fn new(kind: NodeKind, slot: u32, offset: u64) -> Rc<NodeData> {
        let res = NodeData {
            _c: Count::new(),
            kind,
            slot,
            offset,
        };

        Rc::new(res)
    }

    #[inline]
    pub(crate) fn key(&self) -> (ptr::NonNull<()>, u64) {
        let weak = match &self.kind {
            NodeKind::Root { green } => WeakGreenElement::new(green.as_deref()),
            NodeKind::Child { green, .. } => green.clone(),
        };
        let ptr = match weak {
            WeakGreenElement::Node { ptr } => ptr.cast(),
            WeakGreenElement::Token { ptr } => ptr.cast(),
        };
        (ptr, self.offset())
    }

    #[inline]
    pub(crate) fn offset(&self) -> u64 {
        self.offset
    }

    #[inline]
    pub(crate) fn text_range(&self) -> Range<u64> {
        let offset = self.offset();
        let len = self.green().text_len();
        Range {
            start: offset,
            end: len,
        }
    }

    #[inline]
    pub(crate) fn green(&self) -> GreenElementRef<'_> {
        match &self.kind {
            NodeKind::Root { green } => green.as_deref(),
            NodeKind::Child { green, .. } => green.as_deref(),
        }
    }
}

#[derive(Debug)]
struct _SyntaxElement;

pub(crate) fn has_live() -> bool {
    countme::get::<_SyntaxElement>().live > 0
}
