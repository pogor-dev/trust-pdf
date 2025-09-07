use std::{borrow::Cow, cell::Cell, fmt, hash, iter, ops::Range, ptr};

use crate::{
    NodeOrToken, SyntaxKind, SyntaxText,
    green::{GreenNode, GreenNodeData},
    red::{
        Preorder, PreorderWithTokens, SyntaxElement, SyntaxToken,
        node_data::{Green, NodeData, free},
    },
    utils::{Direction, TokenAtOffset, WalkEvent},
};

pub struct SyntaxNode {
    pub(super) ptr: ptr::NonNull<NodeData>,
}

impl SyntaxNode {
    pub fn new_root(green: GreenNode) -> SyntaxNode {
        let green = GreenNode::into_raw(green);
        let green = Green::Node { ptr: Cell::new(green) };
        SyntaxNode {
            ptr: NodeData::new(None, 0, 0, green),
        }
    }

    pub(super) fn new_child(green: &GreenNodeData, parent: SyntaxNode, index: u32, offset: u64) -> SyntaxNode {
        let green = Green::Node { ptr: Cell::new(green.into()) };
        SyntaxNode {
            ptr: NodeData::new(Some(parent), index, offset, green),
        }
    }

    pub fn clone_subtree(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green().into())
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

    pub fn replace_with(&self, replacement: GreenNode) -> GreenNode {
        assert_eq!(self.kind(), replacement.kind());
        match &self.parent() {
            None => replacement,
            Some(parent) => {
                let new_parent = parent.green_ref().replace_child(self.data().index() as usize, replacement.into());
                parent.replace_with(new_parent)
            }
        }
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data().kind()
    }

    #[inline]
    pub(super) fn offset(&self) -> u64 {
        self.data().offset()
    }

    #[inline]
    pub fn text_range(&self) -> Range<u64> {
        self.data().text_range()
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.data().index() as usize
    }

    #[inline]
    pub fn full_text(&self) -> SyntaxText {
        SyntaxText::new(self.clone())
    }

    #[inline]
    pub fn green(&self) -> Cow<'_, GreenNodeData> {
        let green_ref = self.green_ref();
        Cow::Borrowed(green_ref)
    }

    #[inline]
    pub(super) fn green_ref(&self) -> &GreenNodeData {
        self.data().green().into_node().unwrap()
    }

    #[inline]
    pub fn parent(&self) -> Option<SyntaxNode> {
        self.data().parent_node()
    }

    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        iter::successors(Some(self.clone()), SyntaxNode::parent)
    }

    // #[inline]
    // pub fn children(&self) -> SyntaxNodeChildren {
    //     SyntaxNodeChildren::new(self.clone())
    // }

    // #[inline]
    // pub fn children_with_tokens(&self) -> SyntaxElementChildren {
    //     SyntaxElementChildren::new(self.clone())
    // }

    pub fn first_child(&self) -> Option<SyntaxNode> {
        self.green_ref().slots().raw.enumerate().find_map(|(index, child)| {
            child
                .as_ref()
                .into_node()
                .map(|green| SyntaxNode::new_child(green, self.clone(), index as u32, self.offset() + child.rel_offset()))
        })
    }

    pub fn first_child_by_kind(&self, matcher: &impl Fn(SyntaxKind) -> bool) -> Option<SyntaxNode> {
        self.green_ref().slots().raw.enumerate().find_map(|(index, child)| {
            if !matcher(child.as_ref().kind()) {
                return None;
            }
            child
                .as_ref()
                .into_node()
                .map(|green| SyntaxNode::new_child(green, self.clone(), index as u32, self.offset() + child.rel_offset()))
        })
    }

    pub fn last_child(&self) -> Option<SyntaxNode> {
        self.green_ref().slots().raw.enumerate().rev().find_map(|(index, child)| {
            child
                .as_ref()
                .into_node()
                .map(|green| SyntaxNode::new_child(green, self.clone(), index as u32, self.offset() + child.rel_offset()))
        })
    }

    pub fn first_child_or_token(&self) -> Option<SyntaxElement> {
        self.green_ref()
            .slots()
            .raw
            .next()
            .map(|child| SyntaxElement::new(child.as_ref(), self.clone(), 0, self.offset() + child.rel_offset()))
    }

    pub fn first_child_or_token_by_kind(&self, matcher: &impl Fn(SyntaxKind) -> bool) -> Option<SyntaxElement> {
        self.green_ref().slots().raw.enumerate().find_map(|(index, child)| {
            if !matcher(child.as_ref().kind()) {
                return None;
            }

            Some(SyntaxElement::new(
                child.as_ref(),
                self.clone(),
                index as u32,
                self.offset() + child.rel_offset(),
            ))
        })
    }

    pub fn last_child_or_token(&self) -> Option<SyntaxElement> {
        self.green_ref()
            .slots()
            .raw
            .enumerate()
            .next_back()
            .map(|(index, child)| SyntaxElement::new(child.as_ref(), self.clone(), index as u32, self.offset() + child.rel_offset()))
    }

    // if possible (i.e. unshared), consume self and advance it to point to the next sibling
    // this way, we can reuse the previously allocated buffer
    pub fn to_next_sibling(self) -> Option<SyntaxNode> {
        if !self.can_take_ptr() {
            // cannot mutate in-place
            return self.next_sibling();
        }

        let mut ptr = self.take_ptr();
        let data = unsafe { ptr.as_mut() };
        assert!(data.rc.get() == 1);

        let parent = data.parent_node()?;
        let parent_offset = parent.offset();
        let siblings = parent.green_ref().slots().raw.enumerate();
        let index = data.index() as usize;

        siblings
            .skip(index + 1)
            .find_map(|(index, child)| child.as_ref().into_node().map(|green| (green, index as u32, child.rel_offset())))
            .map(|(green, index, rel_offset)| {
                data.index.set(index);
                data.offset = parent_offset + rel_offset;
                data.green = Green::Node { ptr: Cell::new(green.into()) };
                SyntaxNode { ptr }
            })
            .or_else(|| {
                data.dec_rc();
                unsafe { free(ptr) };
                None
            })
    }

    pub fn next_sibling(&self) -> Option<SyntaxNode> {
        self.data().next_sibling()
    }

    pub fn next_sibling_by_kind(&self, matcher: &impl Fn(SyntaxKind) -> bool) -> Option<SyntaxNode> {
        self.data().next_sibling_by_kind(matcher)
    }

    pub fn prev_sibling(&self) -> Option<SyntaxNode> {
        self.data().prev_sibling()
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.data().next_sibling_or_token()
    }

    pub fn next_sibling_or_token_by_kind(&self, matcher: &impl Fn(SyntaxKind) -> bool) -> Option<SyntaxElement> {
        self.data().next_sibling_or_token_by_kind(matcher)
    }

    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.data().prev_sibling_or_token()
    }

    pub fn first_token(&self) -> Option<SyntaxToken> {
        self.first_child_or_token()?.first_token()
    }
    pub fn last_token(&self) -> Option<SyntaxToken> {
        self.last_child_or_token()?.last_token()
    }

    #[inline]
    pub fn siblings(&self, direction: Direction) -> impl Iterator<Item = SyntaxNode> + use<> {
        iter::successors(Some(self.clone()), move |node| match direction {
            Direction::Next => node.next_sibling(),
            Direction::Prev => node.prev_sibling(),
        })
    }

    #[inline]
    pub fn siblings_with_tokens(&self, direction: Direction) -> impl Iterator<Item = SyntaxElement> + use<> {
        let me: SyntaxElement = self.clone().into();
        iter::successors(Some(me), move |el| match direction {
            Direction::Next => el.next_sibling_or_token(),
            Direction::Prev => el.prev_sibling_or_token(),
        })
    }

    #[inline]
    pub fn descendants(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        self.preorder().filter_map(|event| match event {
            WalkEvent::Enter(node) => Some(node),
            WalkEvent::Leave(_) => None,
        })
    }

    #[inline]
    pub fn descendants_with_tokens(&self) -> impl Iterator<Item = SyntaxElement> + use<> {
        self.preorder_with_tokens().filter_map(|event| match event {
            WalkEvent::Enter(it) => Some(it),
            WalkEvent::Leave(_) => None,
        })
    }

    #[inline]
    pub fn preorder(&self) -> Preorder {
        Preorder::new(self.clone())
    }

    #[inline]
    pub fn preorder_with_tokens(&self) -> PreorderWithTokens {
        PreorderWithTokens::new(self.clone())
    }

    // pub fn token_at_offset(&self, offset: u64) -> TokenAtOffset<SyntaxToken> {
    //     // TODO: this could be faster if we first drill-down to node, and only
    //     // then switch to token search. We should also replace explicit
    //     // recursion with a loop.
    //     let range = self.text_range();
    //     assert!(
    //         range.start <= offset && offset <= range.end,
    //         "Bad offset: range {:?} offset {:?}",
    //         range,
    //         offset
    //     );
    //     if range.is_empty() {
    //         return TokenAtOffset::None;
    //     }

    //     let mut children = self.children_with_tokens().filter(|child| {
    //         let child_range = child.text_range();
    //         !child_range.is_empty() && (child_range.start() <= offset && offset <= child_range.end())
    //     });

    //     let left = children.next().unwrap();
    //     let right = children.next();
    //     assert!(children.next().is_none());

    //     if let Some(right) = right {
    //         match (left.token_at_offset(offset), right.token_at_offset(offset)) {
    //             (TokenAtOffset::Single(left), TokenAtOffset::Single(right)) => TokenAtOffset::Between(left, right),
    //             _ => unreachable!(),
    //         }
    //     } else {
    //         left.token_at_offset(offset)
    //     }
    // }

    pub fn covering_element(&self, range: Range<u64>) -> SyntaxElement {
        let mut res: SyntaxElement = self.clone().into();
        loop {
            assert!(
                res.text_range().start <= range.start && range.end <= res.text_range().end,
                "Bad range: node range {:?}, range {:?}",
                res.text_range(),
                range,
            );
            res = match &res {
                NodeOrToken::Token(_) => return res,
                NodeOrToken::Node(node) => match node.child_or_token_at_range(range.clone()) {
                    Some(it) => it,
                    None => return res,
                },
            };
        }
    }

    pub fn child_or_token_at_range(&self, range: Range<u64>) -> Option<SyntaxElement> {
        let offset = self.offset();
        let rel_range = (range.start - offset)..(range.end - offset);
        self.green_ref()
            .child_at_range(rel_range)
            .map(|(index, rel_offset, green)| SyntaxElement::new(green, self.clone(), index as u32, self.offset() + rel_offset))
    }

    // pub fn splice_children<I: IntoIterator<Item = SyntaxElement>>(&self, to_delete: Range<usize>, to_insert: I) {
    //     for (i, child) in self.children_with_tokens().enumerate() {
    //         if to_delete.contains(&i) {
    //             child.detach();
    //         }
    //     }
    //     let mut index = to_delete.start;
    //     for child in to_insert {
    //         self.attach_child(index, child);
    //         index += 1;
    //     }
    // }

    pub fn detach(&self) {
        self.data().detach()
    }

    fn attach_child(&self, index: usize, child: SyntaxElement) {
        child.detach();
        let data = match &child {
            NodeOrToken::Node(it) => it.data(),
            NodeOrToken::Token(it) => it.data(),
        };
        self.data().attach_child(index, data)
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

impl PartialEq for SyntaxNode {
    #[inline]
    fn eq(&self, other: &SyntaxNode) -> bool {
        self.data().key() == other.data().key()
    }
}

impl Eq for SyntaxNode {}

impl hash::Hash for SyntaxNode {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.data().key().hash(state);
    }
}

impl fmt::Debug for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxNode")
            .field("kind", &self.kind())
            .field("text_range", &self.text_range())
            .finish()
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.preorder_with_tokens()
            .filter_map(|event| match event {
                WalkEvent::Enter(NodeOrToken::Token(token)) => Some(token),
                _ => None,
            })
            .try_for_each(|it| fmt::Display::fmt(&it, f))
    }
}
