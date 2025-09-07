use core::slice;
use std::{
    cell::Cell,
    mem::{self, ManuallyDrop},
    ops::Range,
    ptr,
};

use countme::Count;

use crate::{
    GreenToken, NodeOrToken, SyntaxKind,
    green::{GreenElementRef, GreenNode, GreenNodeData, GreenTokenData, Slot},
    red::{SyntaxElement, SyntaxNode},
};

pub(super) enum Green {
    Node { ptr: Cell<ptr::NonNull<GreenNodeData>> },
    Token { ptr: ptr::NonNull<GreenTokenData> },
}

struct _SyntaxElement;

pub(super) struct NodeData {
    _c: Count<_SyntaxElement>,

    pub(super) rc: Cell<u32>,
    pub(super) parent: Cell<Option<ptr::NonNull<NodeData>>>,
    pub(super) index: Cell<u32>,
    pub(super) green: Green,

    /// Absolute offset for immutable nodes, unused for mutable nodes.
    pub(super) offset: u64,
}

impl NodeData {
    #[inline]
    pub(super) fn new(parent: Option<SyntaxNode>, index: u32, offset: u64, green: Green) -> ptr::NonNull<NodeData> {
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
    pub(super) fn inc_rc(&self) {
        let rc = match self.rc.get().checked_add(1) {
            Some(it) => it,
            None => std::process::abort(),
        };
        self.rc.set(rc)
    }

    #[inline]
    pub(super) fn dec_rc(&self) -> bool {
        let rc = self.rc.get() - 1;
        self.rc.set(rc);
        rc == 0
    }

    #[inline]
    pub(super) fn key(&self) -> (ptr::NonNull<()>, u64) {
        let ptr = match &self.green {
            Green::Node { ptr } => ptr.get().cast(),
            Green::Token { ptr } => ptr.cast(),
        };
        (ptr, self.offset())
    }

    #[inline]
    pub(super) fn parent_node(&self) -> Option<SyntaxNode> {
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
    pub(super) fn green(&self) -> GreenElementRef<'_> {
        match &self.green {
            Green::Node { ptr } => GreenElementRef::Node(unsafe { &*ptr.get().as_ptr() }),
            Green::Token { ptr } => GreenElementRef::Token(unsafe { ptr.as_ref() }),
        }
    }

    #[inline]
    fn green_siblings(&self) -> slice::Iter<Slot> {
        match &self.parent().map(|it| &it.green) {
            Some(Green::Node { ptr }) => unsafe { &*ptr.get().as_ptr() }.slots().raw,
            Some(Green::Token { .. }) => {
                debug_assert!(false);
                [].iter()
            }
            None => [].iter(),
        }
    }

    #[inline]
    pub(super) fn index(&self) -> u32 {
        self.index.get()
    }

    #[inline]
    pub(super) fn offset(&self) -> u64 {
        self.offset
    }

    #[cold]
    fn offset_mut(&self) -> u64 {
        let mut res = 0;

        let mut node = self;
        while let Some(parent) = node.parent() {
            let green = parent.green().into_node().unwrap();
            res += green.slots().raw.nth(node.index() as usize).unwrap().rel_offset();
            node = parent;
        }

        res
    }

    #[inline]
    pub(super) fn text_range(&self) -> Range<u64> {
        let offset = self.offset();
        let len = self.green().text_len() as u64;
        offset..(offset + len)
    }

    #[inline]
    pub(super) fn kind(&self) -> SyntaxKind {
        self.green().kind()
    }

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

    pub(super) fn next_sibling_by_kind(&self, matcher: &impl Fn(SyntaxKind) -> bool) -> Option<SyntaxNode> {
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

    pub(super) fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        let mut siblings = self.green_siblings().enumerate();
        let index = self.index() as usize + 1;

        siblings.nth(index).and_then(|(index, child)| {
            let parent = self.parent_node()?;
            let offset = parent.offset() + child.rel_offset();
            Some(SyntaxElement::new(child.as_ref(), parent, index as u32, offset))
        })
    }

    pub(super) fn next_sibling_or_token_by_kind(&self, matcher: &impl Fn(SyntaxKind) -> bool) -> Option<SyntaxElement> {
        let siblings = self.green_siblings().enumerate();
        let index = self.index() as usize;

        siblings.skip(index + 1).find_map(|(index, child)| {
            if !matcher(child.as_ref().kind()) {
                return None;
            }
            let parent = self.parent_node()?;
            let offset = parent.offset() + child.rel_offset();
            Some(SyntaxElement::new(child.as_ref(), parent, index as u32, offset))
        })
    }

    pub(super) fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        let mut siblings = self.green_siblings().enumerate();
        let index = self.index().checked_sub(1)? as usize;

        siblings.nth(index).and_then(|(index, child)| {
            let parent = self.parent_node()?;
            let offset = parent.offset() + child.rel_offset();
            Some(SyntaxElement::new(child.as_ref(), parent, index as u32, offset))
        })
    }

    pub(super) fn detach(&self) {
        assert!(self.rc.get() > 0);
        let parent_ptr = match self.parent.take() {
            Some(parent) => parent,
            None => return,
        };

        let parent = unsafe { parent_ptr.as_ref() };

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
    pub(super) fn attach_child(&self, index: usize, child: &NodeData) {
        assert!(child.parent().is_none());
        assert!(self.rc.get() > 0 && child.rc.get() > 0);

        child.index.set(index as u32);
        child.parent.set(Some(self.into()));
        self.inc_rc();

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
                            new_green = parent_green.replace_child(node.index() as usize, new_green.into());
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

#[inline(never)]
pub(super) unsafe fn free(mut data: ptr::NonNull<NodeData>) {
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
