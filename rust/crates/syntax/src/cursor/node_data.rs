//! Internal node data structure for cursor implementation.
//!
//! ```text
//!     🎯 NodeData (Internal)
//!    ┌─────────────┐
//!    │ ┌─────────┐ │   Core cursor data:
//!    │ │ RefCnt  │ │   • reference counting
//!    │ │ Parent  │ │   • parent linkage  
//!    │ │ Green   │ │   • green tree ref
//!    │ └─────────┘ │   • mutation support
//!    └─────────────┘   
//! ```

use std::{
    cell::Cell,
    mem::{self, ManuallyDrop},
    ops::Range,
    ptr, slice,
};

use countme::Count;

use crate::{
    GreenNode, GreenToken, NodeOrToken, SyntaxKind,
    cursor::{Green, free, node::SyntaxNode, syntax_element::SyntaxElement},
    green::{GreenChild, GreenElementRef},
    sll,
    utility_types::Delta,
};
struct _SyntaxElement;

pub(super) struct NodeData {
    _c: Count<_SyntaxElement>,

    pub(super) rc: Cell<u32>,
    pub(super) parent: Cell<Option<ptr::NonNull<NodeData>>>,
    pub(super) index: Cell<u32>,
    pub(super) green: Green,

    /// Invariant: never changes after NodeData is created.
    pub(super) mutable: bool,
    /// Absolute offset for immutable nodes, unused for mutable nodes.
    pub(super) offset: u32,
    // The following links only have meaning when `mutable` is true.
    pub(super) first: Cell<*const NodeData>,
    /// Invariant: never null if mutable.
    pub(super) next: Cell<*const NodeData>,
    /// Invariant: never null if mutable.
    pub(super) prev: Cell<*const NodeData>,
}

impl NodeData {
    /// Creates a new NodeData with the given parent, index, offset, and green tree reference.
    #[inline]
    pub(super) fn new(
        parent: Option<SyntaxNode>,
        index: u32,
        offset: u32,
        green: Green,
        mutable: bool,
    ) -> ptr::NonNull<NodeData> {
        let parent = ManuallyDrop::new(parent);
        let res = NodeData {
            _c: Count::new(),
            rc: Cell::new(1),
            parent: Cell::new(parent.as_ref().map(|it| it.ptr)),
            index: Cell::new(index),
            green,

            mutable,
            offset,
            first: Cell::new(ptr::null()),
            next: Cell::new(ptr::null()),
            prev: Cell::new(ptr::null()),
        };
        unsafe {
            if mutable {
                let res_ptr: *const NodeData = &res;
                match sll::init(
                    (*res_ptr).parent().map(|it| &it.first),
                    res_ptr.as_ref().unwrap(),
                ) {
                    sll::AddToSllResult::AlreadyInSll(node) => {
                        if cfg!(debug_assertions) {
                            assert_eq!((*node).index(), (*res_ptr).index());
                            match ((*node).green(), (*res_ptr).green()) {
                                (NodeOrToken::Node(lhs), NodeOrToken::Node(rhs)) => {
                                    assert!(ptr::eq(lhs, rhs))
                                }
                                (NodeOrToken::Token(lhs), NodeOrToken::Token(rhs)) => {
                                    assert!(ptr::eq(lhs, rhs))
                                }
                                it => {
                                    panic!("node/token confusion: {:?}", it)
                                }
                            }
                        }

                        ManuallyDrop::into_inner(parent);
                        let res = node as *mut NodeData;
                        (*res).inc_rc();
                        return ptr::NonNull::new_unchecked(res);
                    }
                    it => {
                        let res = Box::into_raw(Box::new(res));
                        it.add_to_sll(res);
                        return ptr::NonNull::new_unchecked(res);
                    }
                }
            }
            ptr::NonNull::new_unchecked(Box::into_raw(Box::new(res)))
        }
    }

    /// Increments the reference count.
    #[inline]
    pub(super) fn inc_rc(&self) {
        let rc = match self.rc.get().checked_add(1) {
            Some(it) => it,
            None => std::process::abort(),
        };
        self.rc.set(rc)
    }

    /// Decrements the reference count, returns true if count reaches zero.
    #[inline]
    pub(super) fn dec_rc(&self) -> bool {
        let rc = self.rc.get() - 1;
        self.rc.set(rc);
        rc == 0
    }

    /// Returns a unique key for this node data based on green pointer and offset.
    #[inline]
    pub(super) fn key(&self) -> (ptr::NonNull<()>, u32) {
        let ptr = match &self.green {
            Green::Node { ptr } => ptr.get().cast(),
            Green::Token { ptr } => ptr.cast(),
        };
        (ptr, self.offset())
    }

    /// Returns the parent node as a SyntaxNode wrapper.
    #[inline]
    pub(super) fn parent_node(&self) -> Option<SyntaxNode> {
        let parent = self.parent()?;
        debug_assert!(matches!(parent.green, Green::Node { .. }));
        parent.inc_rc();
        Some(SyntaxNode {
            ptr: ptr::NonNull::from(parent),
        })
    }

    /// Returns a raw reference to the parent NodeData.
    #[inline]
    pub(super) fn parent(&self) -> Option<&NodeData> {
        self.parent.get().map(|it| unsafe { &*it.as_ptr() })
    }

    /// Returns a reference to the underlying green tree element.
    #[inline]
    pub(super) fn green(&self) -> GreenElementRef<'_> {
        match &self.green {
            Green::Node { ptr } => GreenElementRef::Node(unsafe { &*ptr.get().as_ptr() }),
            Green::Token { ptr } => GreenElementRef::Token(unsafe { ptr.as_ref() }),
        }
    }

    /// Returns an iterator over sibling green children.
    #[inline]
    pub(super) fn green_siblings(&self) -> slice::Iter<'_, GreenChild> {
        match &self.parent().map(|it| &it.green) {
            Some(Green::Node { ptr }) => unsafe { &*ptr.get().as_ptr() }.children().raw,
            Some(Green::Token { .. }) => {
                debug_assert!(false);
                [].iter()
            }
            None => [].iter(),
        }
    }

    /// Returns the index of this node among its siblings.
    #[inline]
    pub(super) fn index(&self) -> u32 {
        self.index.get()
    }

    /// Returns the absolute byte offset of this node in the text.
    #[inline]
    pub(super) fn offset(&self) -> u32 {
        if self.mutable {
            self.offset_mut()
        } else {
            self.offset
        }
    }

    #[cold]
    fn offset_mut(&self) -> u32 {
        let mut res = 0u32;

        let mut node = self;
        while let Some(parent) = node.parent() {
            let green = parent.green().into_node().unwrap();
            res += green
                .children()
                .raw
                .nth(node.index() as usize)
                .unwrap()
                .rel_offset();
            node = parent;
        }

        res
    }

    /// Returns the text span (excluding trivia) of this node.
    #[inline]
    pub(super) fn span(&self) -> Range<u32> {
        let offset = self.offset();
        let len = self.green().width();
        offset..(offset + len)
    }

    /// Returns the full text span (including trivia) of this node.
    #[inline]
    pub(super) fn full_span(&self) -> Range<u32> {
        let offset = self.offset();
        let len = self.green().full_width();
        offset..(offset + len)
    }

    /// Returns the syntax kind of this node.
    #[inline]
    pub(super) fn kind(&self) -> SyntaxKind {
        self.green().kind()
    }

    /// Returns the next sibling node if it exists.
    pub(super) fn next_sibling(&self) -> Option<SyntaxNode> {
        let siblings = self.green_siblings().enumerate();
        let index = self.index() as usize;

        siblings.skip(index + 1).find_map(|(index, child)| {
            child.as_ref().into_node().and_then(|green| {
                let parent = self.parent_node()?;
                let offset = parent.offset() + child.rel_offset();
                Some(SyntaxNode::new_child(green, parent, index as u32, offset))
            })
        })
    }

    /// Returns the next sibling node matching the given kind predicate.
    pub(super) fn next_sibling_by_kind(
        &self,
        matcher: &impl Fn(SyntaxKind) -> bool,
    ) -> Option<SyntaxNode> {
        let siblings = self.green_siblings().enumerate();
        let index = self.index() as usize;

        siblings.skip(index + 1).find_map(|(index, child)| {
            if !matcher(child.as_ref().kind()) {
                return None;
            }
            child.as_ref().into_node().and_then(|green| {
                let parent = self.parent_node()?;
                let offset = parent.offset() + child.rel_offset();
                Some(SyntaxNode::new_child(green, parent, index as u32, offset))
            })
        })
    }

    /// Returns the previous sibling node if it exists.
    pub(super) fn prev_sibling(&self) -> Option<SyntaxNode> {
        let rev_siblings = self.green_siblings().enumerate().rev();
        let index = rev_siblings.len().checked_sub(self.index() as usize)?;

        rev_siblings.skip(index).find_map(|(index, child)| {
            child.as_ref().into_node().and_then(|green| {
                let parent = self.parent_node()?;
                let offset = parent.offset() + child.rel_offset();
                Some(SyntaxNode::new_child(green, parent, index as u32, offset))
            })
        })
    }

    /// Returns the next sibling element (node or token) if it exists.
    pub(super) fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        let mut siblings = self.green_siblings().enumerate();
        let index = self.index() as usize + 1;

        siblings.nth(index).and_then(|(index, child)| {
            let parent = self.parent_node()?;
            let offset = parent.offset() + child.rel_offset();
            Some(SyntaxElement::new(
                child.as_ref(),
                parent,
                index as u32,
                offset,
            ))
        })
    }

    /// Returns the next sibling element matching the given kind predicate.
    pub(super) fn next_sibling_or_token_by_kind(
        &self,
        matcher: &impl Fn(SyntaxKind) -> bool,
    ) -> Option<SyntaxElement> {
        let siblings = self.green_siblings().enumerate();
        let index = self.index() as usize;

        siblings.skip(index + 1).find_map(|(index, child)| {
            if !matcher(child.as_ref().kind()) {
                return None;
            }
            let parent = self.parent_node()?;
            let offset = parent.offset() + child.rel_offset();
            Some(SyntaxElement::new(
                child.as_ref(),
                parent,
                index as u32,
                offset,
            ))
        })
    }

    /// Returns the previous sibling element (node or token) if it exists.
    pub(super) fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        let mut siblings = self.green_siblings().enumerate();
        let index = self.index().checked_sub(1)? as usize;

        siblings.nth(index).and_then(|(index, child)| {
            let parent = self.parent_node()?;
            let offset = parent.offset() + child.rel_offset();
            Some(SyntaxElement::new(
                child.as_ref(),
                parent,
                index as u32,
                offset,
            ))
        })
    }

    /// Detaches this node from its parent (mutable trees only).
    pub(super) fn detach(&self) {
        assert!(self.mutable);
        assert!(self.rc.get() > 0);
        let parent_ptr = match self.parent.take() {
            Some(parent) => parent,
            None => return,
        };

        sll::adjust(self, self.index() + 1, Delta::Sub(1));
        let parent = unsafe { parent_ptr.as_ref() };
        sll::unlink(&parent.first, self);

        // Add strong ref to green
        match self.green().to_owned() {
            NodeOrToken::Node(it) => {
                GreenNode::into_raw(it);
            }
            NodeOrToken::Token(it) => {
                GreenToken::into_raw(it);
            }
        }

        match parent.green() {
            NodeOrToken::Node(green) => {
                let green = green.remove_child(self.index() as usize);
                unsafe { parent.respine(green) }
            }
            NodeOrToken::Token(_) => unreachable!(),
        }

        if parent.dec_rc() {
            unsafe { free(parent_ptr) }
        }
    }

    /// Attaches a child node at the given index (mutable trees only).
    pub(super) fn attach_child(&self, index: usize, child: &NodeData) {
        assert!(self.mutable && child.mutable && child.parent().is_none());
        assert!(self.rc.get() > 0 && child.rc.get() > 0);

        child.index.set(index as u32);
        child.parent.set(Some(self.into()));
        self.inc_rc();

        if !self.first.get().is_null() {
            sll::adjust(unsafe { &*self.first.get() }, index as u32, Delta::Add(1));
        }

        match sll::link(&self.first, child) {
            sll::AddToSllResult::AlreadyInSll(_) => {
                panic!("Child already in sorted linked list")
            }
            it => it.add_to_sll(child),
        }

        match self.green() {
            NodeOrToken::Node(green) => {
                // Child is root, so it owns the green node. Steal it!
                let child_green = match &child.green {
                    Green::Node { ptr } => unsafe { GreenNode::from_raw(ptr.get()).into() },
                    Green::Token { ptr } => unsafe { GreenToken::from_raw(*ptr).into() },
                };

                let green = green.insert_child(index, child_green);
                unsafe { self.respine(green) };
            }
            NodeOrToken::Token(_) => unreachable!(),
        }
    }

    unsafe fn respine(&self, mut new_green: GreenNode) {
        unsafe {
            let mut node = self;
            loop {
                let old_green = match &node.green {
                    Green::Node { ptr } => ptr.replace(ptr::NonNull::from(&*new_green)),
                    Green::Token { .. } => unreachable!(),
                };
                match node.parent() {
                    Some(parent) => match parent.green() {
                        NodeOrToken::Node(parent_green) => {
                            new_green =
                                parent_green.replace_child(node.index() as usize, new_green.into());
                            node = parent;
                        }
                        _ => unreachable!(),
                    },
                    None => {
                        mem::forget(new_green);
                        let _ = GreenNode::from_raw(old_green);
                        break;
                    }
                }
            }
        }
    }
}

unsafe impl sll::Elem for NodeData {
    fn prev(&self) -> &Cell<*const Self> {
        &self.prev
    }

    fn next(&self) -> &Cell<*const Self> {
        &self.next
    }

    fn key(&self) -> &Cell<u32> {
        &self.index
    }
}
