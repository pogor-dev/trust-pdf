use std::{
    borrow::Borrow,
    fmt, iter,
    mem::{self, ManuallyDrop},
    ops, ptr, slice,
};

use crate::{
    GreenToken, GreenTrivia,
    arc::{Arc, HeaderSlice, ThinArc},
    green::{GreenElement, GreenElementRef, token::GreenTokenData, trivia::GreenTriviaData},
};
use countme::Count;

use crate::SyntaxKind;

#[derive(PartialEq, Eq, Hash)]
struct GreenNodeHead {
    kind: SyntaxKind,
    full_width: u32,
    _c: Count<GreenNode>,
}

type Repr = HeaderSlice<GreenNodeHead, [Slot]>;
type ReprThin = HeaderSlice<GreenNodeHead, [Slot; 0]>;

#[repr(transparent)]
pub struct GreenNodeData {
    data: ReprThin,
}

impl GreenNodeData {
    /// Kind of this node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Text of this node.
    #[inline]
    pub fn text(&self) -> Vec<u8> {
        self.write_to(false, false)
    }

    /// Full text of this node.
    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        self.write_to(true, true)
    }

    /// Returns the length of the text covered by this node.
    #[inline]
    pub fn width(&self) -> u32 {
        let first_leading_width = self.first_token().and_then(|t| t.leading_trivia()).map_or(0, |t| t.full_width());
        let last_trailing_width = self.last_token().and_then(|t| t.trailing_trivia()).map_or(0, |t| t.full_width());
        self.full_width() - first_leading_width - last_trailing_width
    }

    /// Full text width of this node.
    #[inline]
    pub fn full_width(&self) -> u32 {
        self.data.header.full_width
    }

    /// The leading trivia of this Node.
    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenNode> {
        self.first_token().and_then(|t| t.leading_trivia())
    }

    /// The trailing trivia of this Node.
    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        self.last_token().and_then(|t| t.trailing_trivia())
    }

    #[inline]
    pub fn slot_count(&self) -> usize {
        self.slots().len()
    }

    #[inline]
    pub fn slots(&self) -> Slots<'_> {
        Slots { raw: self.data.slice().iter() }
    }

    #[inline]
    pub fn slot(&self, index: usize) -> Option<GreenElement> {
        self.slots().nth(index).map(|slot| match slot {
            Slot::Node { node, .. } => GreenElement::Node(node.clone()),
            Slot::Token { token, .. } => GreenElement::Token(token.clone()),
            Slot::Trivia { trivia, .. } => GreenElement::Trivia(trivia.clone()),
        })
    }

    /// Compute the starting offset of slot `index` relative to this node.
    /// (Useful for red position computation.)
    pub fn slot_offset(&self, index: usize) -> Option<u32> {
        if index >= self.slot_count() {
            return None;
        }
        let mut off = 0u32;
        for i in 0..index {
            if let Some(slot) = self.slot(i) {
                off += slot.width();
            } else {
                return None;
            }
        }
        Some(off)
    }

    /// Returns the node's text as a byte vector.
    ///
    /// Similar to Roslyn's WriteTo implementation, uses an explicit stack to avoid
    /// stack overflow on deeply nested structures.
    ///
    /// # Parameters
    /// * `leading` - If true, include the first token's leading trivia
    /// * `trailing` - If true, include the last token's trailing trivia
    fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        fn process_stack(output: &mut Vec<u8>, stack: &mut Vec<(GreenElementRef<'_>, bool, bool)>) {
            while let Some((item, current_leading, current_trailing)) = stack.pop() {
                match item {
                    GreenElementRef::Token(token_data) => {
                        output.extend_from_slice(&token_data.write_to(current_leading, current_trailing));
                    }
                    GreenElementRef::Trivia(trivia_data) => {
                        output.extend_from_slice(&trivia_data.text());
                    }
                    GreenElementRef::Node(node_data) => {
                        let slots = node_data.data.slice();
                        if slots.is_empty() {
                            continue;
                        }

                        let first_index = 0;
                        let last_index = slots.len() - 1;

                        // Push children in reverse so they are processed in forward order.
                        for i in (first_index..=last_index).rev() {
                            let child = &slots[i];
                            let is_first = i == first_index;
                            let is_last = i == last_index;
                            let include_leading = current_leading || !is_first;
                            let include_trailing = current_trailing || !is_last;

                            match child {
                                Slot::Node { node, .. } => {
                                    let node_data: &GreenNodeData = node;
                                    stack.push((GreenElementRef::Node(node_data), include_leading, include_trailing));
                                }
                                Slot::Token { token, .. } => {
                                    let token_data: &GreenTokenData = token;
                                    stack.push((GreenElementRef::Token(token_data), include_leading, include_trailing));
                                }
                                Slot::Trivia { trivia, .. } => {
                                    let trivia_data: &GreenTriviaData = trivia;
                                    stack.push((GreenElementRef::Trivia(trivia_data), include_leading, include_trailing));
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut output = Vec::new();

        // Explicit stack to avoid recursion on deeply nested trees.
        let mut stack: Vec<(GreenElementRef<'_>, bool, bool)> = Vec::with_capacity(64);

        // Seed with this node itself; processing will drill into its slots.
        stack.push((GreenElementRef::Node(self), leading, trailing));

        process_stack(&mut output, &mut stack);
        output
    }

    /// Returns the first terminal token in the node tree
    fn first_token(&self) -> Option<GreenToken> {
        let slots = self.data.slice();
        for child in slots {
            match child {
                Slot::Token { token, .. } => return Some(token.clone()),
                Slot::Node { node, .. } => {
                    if let Some(token) = node.first_token() {
                        return Some(token);
                    }
                }
                Slot::Trivia { .. } => continue,
            }
        }
        None
    }

    /// Returns the last terminal token in the node tree
    fn last_token(&self) -> Option<GreenToken> {
        let slots = self.data.slice();
        for child in slots.iter().rev() {
            match child {
                Slot::Token { token, .. } => return Some(token.clone()),
                Slot::Node { node, .. } => {
                    if let Some(token) = node.last_token() {
                        return Some(token);
                    }
                }
                Slot::Trivia { .. } => continue,
            }
        }
        None
    }
}

impl PartialEq for GreenNodeData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
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

impl fmt::Display for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in &self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenNode")
            .field("kind", &self.kind())
            .field("full_width", &self.full_width())
            .field("slot_count", &self.slot_count())
            .finish()
    }
}

/// Leaf node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenNode {
    ptr: ThinArc<GreenNodeHead, Slot>,
}

impl Borrow<GreenNodeData> for GreenNode {
    #[inline]
    fn borrow(&self) -> &GreenNodeData {
        self
    }
}

impl fmt::Display for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenNodeData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Debug for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenNodeData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenNode {
    /// Creates new Node.
    #[inline]
    pub fn new<I>(kind: SyntaxKind, slots: I) -> GreenNode
    where
        I: IntoIterator<Item = GreenElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut full_width = 0u32;

        let slots = slots.into_iter().map(|el| {
            let rel_offset = full_width;
            full_width += el.full_width();

            match el {
                GreenElement::Node(node) => Slot::Node { rel_offset, node },
                GreenElement::Token(token) => Slot::Token { rel_offset, token },
                GreenElement::Trivia(trivia) => Slot::Trivia { rel_offset, trivia },
            }
        });

        let data = ThinArc::from_header_and_iter(
            GreenNodeHead {
                kind,
                full_width: 0,
                _c: Count::new(),
            },
            slots,
        );

        // XXX: fixup `full_width` after construction, because we can't iterate
        // `slots` twice.
        let data = {
            let mut data = Arc::from_thin(data);
            Arc::get_mut(&mut data)
                .expect("Arc should have unique ownership after construction")
                .header
                .full_width = full_width;
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

    /// # Safety
    ///
    /// This function uses `unsafe` code to create an `Arc` from a raw pointer and then transmutes it into a `ThinArc`.
    ///
    /// - The raw pointer must be valid and correctly aligned for the type `ReprThin`.
    /// - The lifetime of the raw pointer must outlive the lifetime of the `Arc` created from it.
    /// - The transmute operation must be safe, meaning that the memory layout of `Arc<ReprThin>` must be compatible with `ThinArc<GreenNodeHead, Slot>`.
    ///
    /// Failure to uphold these invariants can lead to undefined behavior.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenNodeData>) -> GreenNode {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenNodeHead, Slot>>(arc)
        };
        GreenNode { ptr: arc }
    }
}

impl ops::Deref for GreenNode {
    type Target = GreenNodeData;

    #[inline]
    fn deref(&self) -> &GreenNodeData {
        unsafe {
            let repr: &Repr = &*self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenNodeData>(repr)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Slot {
    Node { rel_offset: u32, node: GreenNode },
    Token { rel_offset: u32, token: GreenToken },
    Trivia { rel_offset: u32, trivia: GreenTrivia },
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Slot::Node { node, .. } => std::fmt::Display::fmt(node, f),
            Slot::Token { token, .. } => std::fmt::Display::fmt(token, f),
            Slot::Trivia { trivia, .. } => std::fmt::Display::fmt(trivia, f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Slots<'a> {
    pub(crate) raw: slice::Iter<'a, Slot>,
}

impl ExactSizeIterator for Slots<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.raw.len()
    }
}

impl<'a> Iterator for Slots<'a> {
    type Item = &'a Slot;

    #[inline]
    fn next(&mut self) -> Option<&'a Slot> {
        self.raw.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.raw.count()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth(n)
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let mut accum = init;
        for x in self {
            accum = f(accum, x);
        }
        accum
    }
}

impl<'a> DoubleEndedIterator for Slots<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth_back(n)
    }

    #[inline]
    fn rfold<Acc, Fold>(mut self, init: Acc, mut f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let mut accum = init;
        while let Some(x) = self.next_back() {
            accum = f(accum, x);
        }
        accum
    }
}

impl iter::FusedIterator for Slots<'_> {}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_node_head_memory_layout() {
        // GreenNodeHead: kind (2 bytes) + full_width (4 bytes) + _c (0 bytes)
        assert!(std::mem::size_of::<GreenNodeHead>() >= 6);
    }

    #[test]
    fn test_green_node_data_memory_layout() {
        // GreenNodeData is transparent wrapper around ReprThin
        assert!(std::mem::size_of::<GreenNodeData>() >= std::mem::size_of::<GreenNodeHead>());
    }

    #[test]
    fn test_green_node_memory_layout() {
        // GreenNode wraps ThinArc pointer (8 bytes on 64-bit)
        assert_eq!(std::mem::size_of::<GreenNode>(), std::mem::size_of::<usize>());
        assert_eq!(std::mem::align_of::<GreenNode>(), std::mem::align_of::<usize>());
    }
}

#[cfg(test)]
mod green_node_tests {
    use super::*;
    use crate::GreenTrivia;
    use pretty_assertions::assert_eq;

    fn empty_trivia_list() -> Option<GreenNode> {
        Some(GreenNode::new(SyntaxKind::List, vec![]))
    }

    #[test]
    fn test_new_node_empty() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        assert_eq!(node.kind(), SyntaxKind::List);
        assert_eq!(node.slot_count(), 0);
    }

    #[test]
    fn test_new_node_with_tokens() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"99", empty_trivia_list(), empty_trivia_list());

        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        assert_eq!(node.kind(), SyntaxKind::ArrayExpression);
        assert_eq!(node.slot_count(), 2);
    }

    #[test]
    fn test_kind() {
        let node = GreenNode::new(SyntaxKind::DictionaryExpression, vec![]);
        assert_eq!(node.kind(), SyntaxKind::DictionaryExpression);
    }

    #[test]
    fn test_slot_count() {
        let token1 = GreenToken::new(SyntaxKind::NameLiteralToken, b"Name", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());

        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];

        let node = GreenNode::new(SyntaxKind::DictionaryExpression, slots);
        assert_eq!(node.slot_count(), 2);
    }

    #[test]
    fn test_slot_access() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token.clone())];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let slot0 = node.slot(0);
        assert!(slot0.is_some());

        let slot1 = node.slot(1);
        assert!(slot1.is_none());
    }

    #[test]
    fn test_slot_access_with_node() {
        let inner_token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let inner_node = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(inner_token)]);
        let outer_node = GreenNode::new(SyntaxKind::DictionaryExpression, vec![GreenElement::Node(inner_node.clone())]);

        let slot0 = outer_node.slot(0);
        assert!(slot0.is_some());
        match slot0.expect("slot should exist") {
            GreenElement::Node(n) => assert_eq!(n.kind(), SyntaxKind::ArrayExpression),
            _ => panic!("Expected Node element"),
        }
    }

    #[test]
    fn test_slot_access_with_trivia() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let node = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(trivia.clone())]);

        let slot0 = node.slot(0);
        assert!(slot0.is_some());
        match slot0.expect("slot should exist") {
            GreenElement::Trivia(t) => assert_eq!(t.kind(), SyntaxKind::WhitespaceTrivia),
            _ => panic!("Expected Trivia element"),
        }
    }

    #[test]
    fn test_full_width() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"999", empty_trivia_list(), empty_trivia_list());

        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        assert_eq!(node.full_width(), 5); // 2 + 3
    }

    #[test]
    fn test_text() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"99", empty_trivia_list(), empty_trivia_list());

        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        assert_eq!(node.text(), b"4299");
    }

    #[test]
    fn test_full_text() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());

        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        assert_eq!(node.full_text(), b"12");
    }

    #[test]
    fn test_clone() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token)];

        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        let node2 = node1.clone();

        assert_eq!(node1.kind(), node2.kind());
        assert_eq!(node1.slot_count(), node2.slot_count());
        assert_eq!(node1.full_width(), node2.full_width());
    }

    #[test]
    fn test_display() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token)];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        let display_str = node.to_string();
        assert_eq!(display_str, "42");
    }

    #[test]
    fn test_debug() {
        let node = GreenNode::new(SyntaxKind::ArrayExpression, vec![]);
        let debug_str = format!("{:?}", node);
        assert_eq!(debug_str, "GreenNode { kind: ArrayExpression, full_width: 0, slot_count: 0 }");
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token)];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        let ptr = GreenNode::into_raw(node.clone());
        let reconstructed = unsafe { GreenNode::from_raw(ptr) };

        assert_eq!(node.kind(), reconstructed.kind());
        assert_eq!(node.slot_count(), reconstructed.slot_count());
    }

    #[test]
    fn test_borrow() {
        let node = GreenNode::new(SyntaxKind::ArrayExpression, vec![]);
        let borrowed: &GreenNodeData = node.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::ArrayExpression);
    }

    #[test]
    fn test_to_owned() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token)];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        let data: &GreenNodeData = &*node;
        let owned = data.to_owned();

        assert_eq!(node.kind(), owned.kind());
        assert_eq!(node.slot_count(), owned.slot_count());
    }

    #[test]
    fn test_nested_nodes() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());

        let inner_slots = vec![GreenElement::Token(token1)];
        let inner_node = GreenNode::new(SyntaxKind::ArrayExpression, inner_slots);

        let outer_slots = vec![GreenElement::Node(inner_node), GreenElement::Token(token2)];
        let outer_node = GreenNode::new(SyntaxKind::DictionaryExpression, outer_slots);

        assert_eq!(outer_node.kind(), SyntaxKind::DictionaryExpression);
        assert_eq!(outer_node.slot_count(), 2);
        assert_eq!(outer_node.text(), b"12");
    }

    #[test]
    fn test_width() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        assert_eq!(node.width(), 2);
    }

    #[test]
    fn test_width_with_trivia() {
        let leading_trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let trailing_trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let leading = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(leading_trivia)]);
        let trailing = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(trailing_trivia)]);
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", Some(leading), Some(trailing));
        let slots = vec![GreenElement::Token(token)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        assert_eq!(node.width(), 2); // Only token text, not trivia
        assert_eq!(node.full_width(), 5); // Token + leading + trailing
    }

    #[test]
    fn test_leading_trivia() {
        let leading_trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let leading = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(leading_trivia)]);
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", Some(leading.clone()), empty_trivia_list());
        let slots = vec![GreenElement::Token(token)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let trivia = node.leading_trivia();
        assert!(trivia.is_some());
        assert_eq!(trivia.expect("leading trivia should exist").full_width(), 1);
    }

    #[test]
    fn test_trailing_trivia() {
        let trailing_trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let trailing = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(trailing_trivia)]);
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), Some(trailing.clone()));
        let slots = vec![GreenElement::Token(token)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let trivia = node.trailing_trivia();
        assert!(trivia.is_some());
        assert_eq!(trivia.expect("trailing trivia should exist").full_width(), 2);
    }

    #[test]
    fn test_slot_offset() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"999", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        assert_eq!(node.slot_offset(0), Some(0));
        assert_eq!(node.slot_offset(1), Some(2)); // After "42"
        assert_eq!(node.slot_offset(2), None); // Out of bounds
    }

    #[test]
    fn test_slots_iterator() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let slot_vec: Vec<_> = node.slots().collect();
        assert_eq!(slot_vec.len(), 2);
    }

    #[test]
    fn test_slots_double_ended_iterator() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let token3 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"3", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2), GreenElement::Token(token3)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let mut iter = node.slots();
        assert!(iter.next_back().is_some()); // Gets last
        assert!(iter.next().is_some()); // Gets first
        assert!(iter.next_back().is_some()); // Gets middle
        assert!(iter.next().is_none()); // Exhausted
    }

    #[test]
    fn test_slot_display() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token.clone())];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        if let Some(slot) = node.slots().next() {
            let display_str = format!("{}", slot);
            assert_eq!(display_str, "42");
        }
    }

    #[test]
    fn test_write_to_with_deeply_nested_nodes() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let token3 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"3", empty_trivia_list(), empty_trivia_list());

        let inner1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token1)]);
        let inner2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token2)]);
        let middle = GreenNode::new(SyntaxKind::DictionaryExpression, vec![GreenElement::Node(inner1), GreenElement::Node(inner2)]);
        let outer = GreenNode::new(SyntaxKind::List, vec![GreenElement::Node(middle), GreenElement::Token(token3)]);

        assert_eq!(outer.text(), b"123");
        assert_eq!(outer.full_text(), b"123");
    }

    #[test]
    fn test_eq_different_nodes() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"99", empty_trivia_list(), empty_trivia_list());

        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token1)]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token2)]);

        assert_ne!(node1, node2);
    }

    #[test]
    fn test_eq_same_content() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());

        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token1)]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token2)]);

        assert_eq!(node1, node2);
    }

    #[test]
    fn test_green_node_data_eq() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());

        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token1)]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token2)]);

        let data1: &GreenNodeData = &*node1;
        let data2: &GreenNodeData = &*node2;
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_first_token_with_nested_nodes() {
        let inner_token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let inner_node = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(inner_token.clone())]);
        let outer_node = GreenNode::new(SyntaxKind::DictionaryExpression, vec![GreenElement::Node(inner_node)]);

        let first = outer_node.first_token();
        assert!(first.is_some());
        assert_eq!(first.expect("first token should exist").text(), b"42");
    }

    #[test]
    fn test_last_token_with_nested_nodes() {
        let inner_token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"99", empty_trivia_list(), empty_trivia_list());
        let inner_node = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(inner_token.clone())]);
        let outer_node = GreenNode::new(SyntaxKind::DictionaryExpression, vec![GreenElement::Node(inner_node)]);

        let last = outer_node.last_token();
        assert!(last.is_some());
        assert_eq!(last.expect("last token should exist").text(), b"99");
    }

    #[test]
    fn test_first_token_with_trivia_only() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let node = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(trivia)]);

        let first = node.first_token();
        assert!(first.is_none());
    }

    #[test]
    fn test_last_token_with_trivia_only() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let node = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(trivia)]);

        let last = node.last_token();
        assert!(last.is_none());
    }

    #[test]
    fn test_first_token_empty_node() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);

        let first = node.first_token();
        assert!(first.is_none());
    }

    #[test]
    fn test_last_token_empty_node() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);

        let last = node.last_token();
        assert!(last.is_none());
    }

    #[test]
    fn test_first_token_with_trivia_before_token() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let node = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(trivia), GreenElement::Token(token.clone())]);

        let first = node.first_token();
        assert!(first.is_some());
        assert_eq!(first.expect("first token should exist").text(), b"42");
    }

    #[test]
    fn test_last_token_with_trivia_after_token() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"99", empty_trivia_list(), empty_trivia_list());
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let node = GreenNode::new(SyntaxKind::List, vec![GreenElement::Token(token.clone()), GreenElement::Trivia(trivia)]);

        let last = node.last_token();
        assert!(last.is_some());
        assert_eq!(last.expect("last token should exist").text(), b"99");
    }

    #[test]
    fn test_text_with_trivia_list() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let trivia2 = GreenTrivia::new(SyntaxKind::CommentTrivia, b"% comment");
        let trivia3 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n");

        let node = GreenNode::new(
            SyntaxKind::List,
            vec![GreenElement::Trivia(trivia1), GreenElement::Trivia(trivia2), GreenElement::Trivia(trivia3)],
        );

        assert_eq!(node.text(), b"  % comment\n");
        assert_eq!(node.full_text(), b"  % comment\n");
    }

    #[test]
    fn test_text_with_mixed_trivia_and_tokens() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");

        let node = GreenNode::new(
            SyntaxKind::List,
            vec![GreenElement::Trivia(trivia1), GreenElement::Token(token), GreenElement::Trivia(trivia2)],
        );

        assert_eq!(node.text(), b"  42 ");
        assert_eq!(node.full_text(), b"  42 ");
    }
}

#[cfg(test)]
mod green_node_data_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn empty_trivia_list() -> Option<GreenNode> {
        Some(GreenNode::new(SyntaxKind::List, vec![]))
    }

    #[test]
    fn test_to_owned() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token)];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        let data: &GreenNodeData = &*node;
        let owned = data.to_owned();

        assert_eq!(node.kind(), owned.kind());
        assert_eq!(node.slot_count(), owned.slot_count());
    }

    #[test]
    fn test_eq() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());

        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token1)]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token2)]);

        let data1: &GreenNodeData = &*node1;
        let data2: &GreenNodeData = &*node2;
        assert_eq!(data1, data2);
    }
}

#[cfg(test)]
mod slot_tests {
    use super::*;
    use crate::GreenTrivia;
    use pretty_assertions::assert_eq;

    fn empty_trivia_list() -> Option<GreenNode> {
        Some(GreenNode::new(SyntaxKind::List, vec![]))
    }

    #[test]
    fn test_slot_count() {
        let token1 = GreenToken::new(SyntaxKind::NameLiteralToken, b"Name", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());

        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];

        let node = GreenNode::new(SyntaxKind::DictionaryExpression, slots);
        assert_eq!(node.slot_count(), 2);
    }

    #[test]
    fn test_slot_access() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token.clone())];

        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let slot0 = node.slot(0);
        assert!(slot0.is_some());

        let slot1 = node.slot(1);
        assert!(slot1.is_none());
    }

    #[test]
    fn test_slot_access_with_node() {
        let inner_token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let inner_node = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(inner_token)]);
        let outer_node = GreenNode::new(SyntaxKind::DictionaryExpression, vec![GreenElement::Node(inner_node.clone())]);

        let slot0 = outer_node.slot(0);
        assert!(slot0.is_some());
        match slot0.expect("slot should exist") {
            GreenElement::Node(n) => assert_eq!(n.kind(), SyntaxKind::ArrayExpression),
            _ => panic!("Expected Node element"),
        }
    }

    #[test]
    fn test_slot_access_with_trivia() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let node = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(trivia.clone())]);

        let slot0 = node.slot(0);
        assert!(slot0.is_some());
        match slot0.expect("slot should exist") {
            GreenElement::Trivia(t) => assert_eq!(t.kind(), SyntaxKind::WhitespaceTrivia),
            _ => panic!("Expected Trivia element"),
        }
    }

    #[test]
    fn test_slot_offset() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"999", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        assert_eq!(node.slot_offset(0), Some(0));
        assert_eq!(node.slot_offset(1), Some(2)); // After "42"
        assert_eq!(node.slot_offset(2), None); // Out of bounds
    }

    #[test]
    fn test_slot_display() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token.clone())];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        if let Some(slot) = node.slots().next() {
            let display_str = format!("{}", slot);
            assert_eq!(display_str, "42");
        }
    }
}

#[cfg(test)]
mod slots_iterator_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn empty_trivia_list() -> Option<GreenNode> {
        Some(GreenNode::new(SyntaxKind::List, vec![]))
    }

    #[test]
    fn test_slots_iterator() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let slot_vec: Vec<_> = node.slots().collect();
        assert_eq!(slot_vec.len(), 2);
    }

    #[test]
    fn test_slots_len() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        assert_eq!(node.slots().len(), 2);
    }

    #[test]
    fn test_slots_size_hint() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let (lower, upper) = node.slots().size_hint();
        assert_eq!(lower, 1);
        assert_eq!(upper, Some(1));
    }

    #[test]
    fn test_slots_count() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        assert_eq!(node.slots().count(), 2);
    }

    #[test]
    fn test_slots_last() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let last = node.slots().last();
        assert!(last.is_some());
    }

    #[test]
    fn test_slots_nth() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let token3 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"3", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2), GreenElement::Token(token3)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let mut iter = node.slots();
        let nth = iter.nth(1);
        assert!(nth.is_some());
    }

    #[test]
    fn test_slots_fold() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let count = node.slots().fold(0, |acc, _| acc + 1);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_slots_double_ended_iterator() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let token3 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"3", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2), GreenElement::Token(token3)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let mut iter = node.slots();
        assert!(iter.next_back().is_some()); // Gets last
        assert!(iter.next().is_some()); // Gets first
        assert!(iter.next_back().is_some()); // Gets middle
        assert!(iter.next().is_none()); // Exhausted
    }

    #[test]
    fn test_slots_nth_back() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let token3 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"3", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2), GreenElement::Token(token3)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let mut iter = node.slots();
        let nth_back = iter.nth_back(1);
        assert!(nth_back.is_some());
    }

    #[test]
    fn test_slots_rfold() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list());
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list());
        let slots = vec![GreenElement::Token(token1), GreenElement::Token(token2)];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);

        let count = node.slots().rfold(0, |acc, _| acc + 1);
        assert_eq!(count, 2);
    }
}
