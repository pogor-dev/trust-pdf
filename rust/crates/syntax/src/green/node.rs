use std::{
    borrow::{self, Cow},
    fmt, iter,
    mem::{self, ManuallyDrop},
    ops::{self, Range},
    ptr, slice,
};

use countme::Count;

use crate::{
    GreenToken, GreenTrivia, NodeOrToken, SyntaxKind,
    arc::{Arc, HeaderSlice, ThinArc},
    green::{GreenElementRef, element::GreenElement},
};

type Repr = HeaderSlice<GreenNodeHead, [Slot]>;
type ReprThin = HeaderSlice<GreenNodeHead, [Slot; 0]>;

#[derive(PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    kind: SyntaxKind,
    full_text_len: u32,
    _c: Count<GreenNode>,
}

#[repr(transparent)]
pub struct GreenNodeData {
    data: ReprThin,
}

impl GreenNodeData {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    #[inline]
    pub(crate) fn slice(&self) -> &[Slot] {
        self.data.slice()
    }

    #[inline]
    pub fn text_len(&self) -> u32 {
        self.full_text_len() - self.leading_trivia_len() - self.trailing_trivia_len()
    }

    #[inline]
    pub fn full_text_len(&self) -> u32 {
        self.data.header.full_text_len
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenTrivia> {
        if self.data.header.full_text_len == 0 {
            return None;
        }

        let slot = Slot::Node {
            rel_offset: 0,
            node: self.to_owned(),
        };

        get_first_terminal(&slot).and_then(|t| t.leading_trivia()).cloned()
    }

    #[inline]
    pub fn leading_trivia_len(&self) -> u32 {
        if self.data.header.full_text_len == 0 {
            return 0;
        }

        let slot = Slot::Node {
            rel_offset: 0,
            node: self.to_owned(),
        };

        get_first_terminal(&slot).map_or(0, |t| t.leading_trivia_len())
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenTrivia> {
        if self.data.header.full_text_len == 0 {
            return None;
        }

        let slot = Slot::Node {
            rel_offset: 0,
            node: self.to_owned(),
        };

        get_last_terminal(&slot).and_then(|t| t.trailing_trivia()).cloned()
    }

    #[inline]
    pub fn trailing_trivia_len(&self) -> u32 {
        if self.data.header.full_text_len == 0 {
            return 0;
        }

        let slot = Slot::Node {
            rel_offset: 0,
            node: self.to_owned(),
        };

        get_last_terminal(&slot).map_or(0, |t| t.trailing_trivia_len())
    }

    /// Returns the slots of this node. Every node of a specific kind has the same number of slots
    /// to allow using fixed offsets to retrieve a specific child even if some other child is missing.
    #[inline]
    pub(crate) fn slots(&self) -> Slots<'_> {
        Slots { raw: self.slice().iter() }
    }

    /// Finds the child element that contains the given text range using binary search.
    ///
    /// Uses range overlap detection to efficiently locate which child contains the target range.
    /// Essential for position-based queries in syntax trees.
    ///
    /// # Range Comparison Logic
    ///
    /// |             | Less                     | Greater                 | Equal (Overlap)         | Equal (Contains)         |
    /// |-------------|--------------------------|-------------------------|-------------------------|--------------------------|
    /// | child_range |   ███████ [3..8)         |          █████ [20..25) |         ██████ [8..18)  |     ████████████ [5..25) |
    /// | rel_range   |            ████ [12..18) | ███████ [5..12)         |           ████ [12..18) |         ██ [10..15)      |
    /// | result      | search ->                | search <-               | found                   | found                    |
    pub(crate) fn child_at_range(&self, rel_range: Range<u64>) -> Option<(usize, u64, GreenElementRef<'_>)> {
        let idx = self
            .slice()
            .binary_search_by(|child| {
                let child_range = child.rel_range();
                if child_range.end <= rel_range.start {
                    std::cmp::Ordering::Less
                } else if child_range.start >= rel_range.end {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            // XXX: this handles empty ranges
            .unwrap_or_else(|it| it.saturating_sub(1));

        let child = &self.slice().get(idx).filter(|it| {
            let child_range = it.rel_range();
            child_range.start <= rel_range.start && rel_range.end <= child_range.end
        })?;

        Some((idx, child.rel_offset(), child.as_ref()))
    }

    #[must_use]
    pub fn replace_child(&self, index: usize, new_child: GreenElement) -> GreenNode {
        let mut replacement = Some(new_child);
        let children = self.slots().enumerate().map(|(i, child)| {
            if i == index {
                replacement.take().unwrap()
            } else {
                GreenElement::from(child).to_owned()
            }
        });
        GreenNode::new_list(self.kind(), children)
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
        let mut children: Vec<_> = self.slots().map(|it| GreenElement::from(it).to_owned()).collect();
        children.splice(range, replace_with);
        GreenNode::new_list(self.kind(), children)
    }
}

impl From<Cow<'_, GreenNodeData>> for GreenNode {
    #[inline]
    fn from(cow: Cow<'_, GreenNodeData>) -> Self {
        cow.into_owned()
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
            .field("full_text_len", &self.full_text_len())
            .finish()
    }
}

impl fmt::Display for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for child in self.slots() {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenNode {
    ptr: ThinArc<GreenNodeHead, Slot>,
}

impl GreenNode {
    /// Creates a new node containing the passed in pieces
    pub fn new_list<I>(kind: SyntaxKind, slots: I) -> Self
    where
        I: IntoIterator<Item = GreenElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut full_text_len = 0;
        let children = slots.into_iter().map(|el| {
            let rel_offset = full_text_len;
            full_text_len += el.full_text_len();
            match el {
                NodeOrToken::Node(node) => Slot::Node { rel_offset, node },
                NodeOrToken::Token(token) => Slot::Token { rel_offset, token },
            }
        });

        let data = ThinArc::from_header_and_iter(
            GreenNodeHead {
                kind,
                full_text_len: 0,
                _c: Count::new(),
            },
            children,
        );

        // XXX: fixup `full_text_len` after construction, because we can't iterate
        // `children` twice.
        let data = {
            let mut data = Arc::from_thin(data);
            Arc::get_mut(&mut data).unwrap().header.full_text_len = full_text_len;
            Arc::into_thin(data)
        };

        GreenNode { ptr: data }
    }

    /// Creates a single piece of node from the given text.
    pub fn new_single(kind: SyntaxKind, slot: GreenElement) -> Self {
        GreenNode::new_list(kind, std::iter::once(slot))
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
    /// - The transmute operation must be safe, meaning that the memory layout of `Arc<ReprThin>` must be compatible with `ThinArc<GreenTokenHead, u8>`.
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

impl borrow::Borrow<GreenNodeData> for GreenNode {
    #[inline]
    fn borrow(&self) -> &GreenNodeData {
        self
    }
}

impl ops::Deref for GreenNode {
    type Target = GreenNodeData;

    #[inline]
    fn deref(&self) -> &GreenNodeData {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenNodeData>(repr)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Slot {
    Node { rel_offset: u32, node: GreenNode },
    Token { rel_offset: u32, token: GreenToken },
}

impl Slot {
    #[inline]
    pub(crate) fn as_ref(&self) -> GreenElementRef {
        match self {
            Slot::Node { node, .. } => NodeOrToken::Node(node),
            Slot::Token { token, .. } => NodeOrToken::Token(token),
        }
    }
    #[inline]
    pub(crate) fn rel_offset(&self) -> u64 {
        match self {
            Slot::Node { rel_offset, .. } | Slot::Token { rel_offset, .. } => *rel_offset as u64,
        }
    }
    #[inline]
    fn rel_range(&self) -> Range<u64> {
        let len = self.as_ref().text_len();
        let start = self.rel_offset();
        let end = start + len as u64;
        start..end
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Slot::Node { node, .. } => fmt::Display::fmt(node, f),
            Slot::Token { token, .. } => fmt::Display::fmt(token, f),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Slots<'a> {
    pub(crate) raw: slice::Iter<'a, Slot>,
}

// NB: forward everything stable that iter::Slice specializes as of Rust 1.39.0
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

fn get_first_terminal(node: &Slot) -> Option<&GreenToken> {
    get_terminal(node, |len| 0..len)
}

fn get_last_terminal(node: &Slot) -> Option<&GreenToken> {
    get_terminal(node, |len| (0..len).rev())
}
/// Performs a depth-first search for the first/last terminal token in the given node.
fn get_terminal<I>(node: &Slot, indices: impl Fn(usize) -> I) -> Option<&GreenToken>
where
    I: Iterator<Item = usize>,
{
    let mut node = Some(node);

    loop {
        let current = node?;
        let mut next_child = None;

        match current {
            Slot::Node { node: n, .. } => {
                let slots = n.slice();
                for i in indices(slots.len()) {
                    if let Some(child) = slots.get(i) {
                        next_child = Some(child);
                        break;
                    }
                }
                node = next_child;
            }
            Slot::Token { token, .. } => {
                return Some(token);
            }
        }
    }
}

#[cfg(test)]
mod node_tests {
    use rstest::rstest;

    use super::*;

    fn create_whitespace_trivia() -> GreenTrivia {
        GreenTrivia::new_single(SyntaxKind(0), b" ")
    }

    fn create_eol_trivia() -> GreenTrivia {
        GreenTrivia::new_single(SyntaxKind(1), b"\n")
    }

    #[rstest]
    fn test_new_single_token() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        assert_eq!(node.kind(), SyntaxKind(3));
    }

    #[rstest]
    fn test_new_single_nested_node() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let child_node = GreenNode::new_single(SyntaxKind(4), GreenElement::from(token.clone()));
        let parent_node = GreenNode::new_single(SyntaxKind(3), NodeOrToken::Node(child_node.clone()));
        assert_eq!(parent_node.kind(), SyntaxKind(3));
    }

    #[rstest]
    fn test_new_list_tokens() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token1", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token2", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![GreenElement::from(token1.clone()), GreenElement::from(token2.clone())]);
        assert_eq!(node.kind(), SyntaxKind(3));
    }

    #[rstest]
    fn test_new_list_nested_nodes() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let child_node = GreenNode::new_single(SyntaxKind(4), GreenElement::from(token.clone()));
        let parent_node = GreenNode::new_list(SyntaxKind(3), vec![NodeOrToken::Node(child_node.clone())]);
        assert_eq!(parent_node.kind(), SyntaxKind(3));
    }

    #[rstest]
    #[allow(useless_ptr_null_checks)]
    fn test_into_raw_pointer() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        let ptr: ptr::NonNull<GreenNodeData> = GreenNode::into_raw(node.clone());
        assert!(!ptr.as_ptr().is_null());
    }
    #[rstest]
    fn test_from_raw_pointer() {
        let kind = SyntaxKind(3);
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(kind, GreenElement::from(token.clone()));
        let ptr: ptr::NonNull<GreenNodeData> = GreenNode::into_raw(node.clone());
        let recovered = unsafe { GreenNode::from_raw(ptr) };
        assert_eq!(recovered.kind(), kind);
    }

    #[rstest]
    fn test_fmt_debug() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        let formatted = format!("{:?}", node);
        assert_eq!(formatted, "GreenNode { kind: SyntaxKind(3), full_text_len: 7 }");
    }

    #[rstest]
    fn test_fmt_display() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        let formatted = format!("{}", node);
        assert_eq!(formatted, " token\n");
    }

    #[rstest]
    fn test_fmt_display_nested_node() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let child_node = GreenNode::new_single(SyntaxKind(4), GreenElement::from(token.clone()));
        let parent_node = GreenNode::new_single(SyntaxKind(3), NodeOrToken::Node(child_node.clone()));
        let formatted = format!("{}", parent_node);
        assert_eq!(formatted, " token\n");
    }

    #[rstest]
    fn test_borrowing() {
        use std::borrow::Borrow;
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        let borrowed = node.borrow();
        let data: &GreenNodeData = &borrowed;
        let owned = data.to_owned();
        assert_eq!(owned.kind(), borrowed.kind());
    }

    #[rstest]
    fn test_kind() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        assert_eq!(node.kind(), SyntaxKind(3));
    }

    #[rstest]
    fn test_text_len() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        assert_eq!(node.text_len(), 5);
    }

    #[rstest]
    fn test_full_text_len() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        assert_eq!(node.full_text_len(), 7); // 1 (whitespace) + 5 (token) + 1 (eol)
    }

    #[rstest]
    fn test_leading_trivia_len() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        assert_eq!(node.leading_trivia_len(), 1); // whitespace
    }

    #[rstest]
    fn test_leading_trivia_len_empty_node() {
        let node = GreenNode::new_list(SyntaxKind(3), vec![]);
        assert_eq!(node.leading_trivia_len(), 0);
    }

    #[rstest]
    fn test_trailing_trivia_len() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        assert_eq!(node.trailing_trivia_len(), 1); // eol
    }

    #[rstest]
    fn test_trailing_trivia_len_empty_node() {
        let node = GreenNode::new_list(SyntaxKind(3), vec![]);
        assert_eq!(node.trailing_trivia_len(), 0);
    }

    #[rstest]
    fn test_leading_trivia() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        let leading_trivia = node.leading_trivia().unwrap();
        assert_eq!(leading_trivia.full_text_len(), 1);
        assert_eq!(leading_trivia.full_text(), b" ");
    }

    #[rstest]
    fn test_leading_trivia_empty_node() {
        let node = GreenNode::new_list(SyntaxKind(3), vec![]);
        assert!(node.leading_trivia().is_none());
    }

    #[rstest]
    fn test_trailing_trivia() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        let trailing_trivia = node.trailing_trivia().unwrap();
        assert_eq!(trailing_trivia.full_text_len(), 1);
        assert_eq!(trailing_trivia.full_text(), b"\n");
    }

    #[rstest]
    fn test_trailing_trivia_empty_node() {
        let node = GreenNode::new_list(SyntaxKind(3), vec![]);
        assert!(node.trailing_trivia().is_none());
    }

    #[rstest]
    fn test_insert_child() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token1", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token2", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![GreenElement::from(token1.clone())]);
        let new_node = node.insert_child(1, GreenElement::from(token2.clone()));
        assert_eq!(new_node.slots().count(), 2);
    }

    #[rstest]
    fn test_remove_child() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token1", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token2", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![GreenElement::from(token1.clone()), GreenElement::from(token2.clone())]);
        let new_node = node.remove_child(0);
        assert_eq!(new_node.slots().count(), 1);
    }

    #[rstest]
    fn test_replace_child() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token1", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token2", create_whitespace_trivia(), create_eol_trivia());
        let token3 = GreenToken::new_with_trivia(SyntaxKind(2), b"token3", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![GreenElement::from(token1.clone()), GreenElement::from(token2.clone())]);
        let new_node = node.replace_child(0, GreenElement::from(token3.clone()));
        assert_eq!(new_node.slots().count(), 2);
        assert_eq!(
            new_node.slots().nth(0).unwrap(),
            &Slot::Token {
                rel_offset: 0,
                token: token3.clone()
            }
        );
    }

    #[rstest]
    #[case::empty_range(
        vec![b"token1".as_slice(), b"token2".as_slice()],
        0..0,
        Some((0, 0, 0))
    )]
    #[case::range_outside_all_children(
        vec![b"abc".as_slice(), b"defghijk".as_slice()],
        12..18,
        None  // Range 12..18 is outside children ranges 0..3 and 3..11
    )]
    #[case::range_contained_in_second_child(
        vec![b"short".as_slice(), b"verylongtoken".as_slice()],
        5..12,
        Some((1, 5, 1))
    )]
    #[case::range_extends_beyond_child(
        vec![b"first".as_slice(), b"overlapping".as_slice()],
        12..19,
        None  // Range 12..19 extends beyond child 1 range 5..16
    )]
    #[case::range_contained_in_large_token(
        vec![b"verylongtokenthatcontains".as_slice()],
        10..15,
        Some((0, 0, 0))
    )]
    #[case::range_contained_in_second_token(
        vec![b"abc".as_slice(), b"def".as_slice()],
        4..6,
        Some((1, 3, 1))  // Range 4..6 is contained in child 1 range 3..6
    )]
    #[case::range_spans_multiple_children(
        vec![b"abc".as_slice(), b"def".as_slice()],
        2..8,
        None
    )]
    fn test_child_at_range_scenarios(
        #[case] token_texts: Vec<&[u8]>,
        #[case] target_range: Range<u64>,
        #[case] expected: Option<(usize, u64, usize)>, // (index, rel_offset, token_index)
    ) {
        // Create tokens with trivia
        let tokens: Vec<GreenToken> = token_texts.iter().map(|text| GreenToken::new(SyntaxKind(2), text)).collect();

        // Create node with tokens
        let elements: Vec<GreenElement> = tokens.iter().map(|t| GreenElement::from(t.clone())).collect();
        let node = GreenNode::new_list(SyntaxKind(3), elements);

        // Test the function
        let result = node.child_at_range(target_range);

        // Match both actual and expected results
        match (result, expected) {
            // Both are Some - compare the values
            (Some((actual_idx, actual_offset, actual_child)), Some((expected_idx, expected_offset, expected_token_idx))) => {
                assert_eq!(actual_idx, expected_idx, "Child index mismatch");
                assert_eq!(actual_offset, expected_offset, "Relative offset mismatch");
                assert_eq!(
                    actual_child.to_owned(),
                    GreenElement::from(tokens[expected_token_idx].clone()),
                    "Child element mismatch"
                );
            }
            // Both are None - test passes
            (None, None) => {
                // Both are None, test passes
            }
            // Mismatched results - test fails
            (Some(actual), None) => {
                panic!("Expected None, but got Some({:?})", actual);
            }
            (None, Some(expected)) => {
                panic!("Expected Some({:?}), but got None", expected);
            }
        }
    }

    #[rstest]
    fn test_cow_from() {
        let token = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_single(SyntaxKind(3), GreenElement::from(token.clone()));
        let cow: Cow<'_, GreenNodeData> = Cow::Borrowed(&node);
        let owned: GreenNode = cow.into();
        assert_eq!(owned.kind(), node.kind());
    }
}

#[cfg(test)]
mod slots_tests {
    use rstest::rstest;

    use super::*;

    fn create_whitespace_trivia() -> GreenTrivia {
        GreenTrivia::new_single(SyntaxKind(0), b" ")
    }

    fn create_eol_trivia() -> GreenTrivia {
        GreenTrivia::new_single(SyntaxKind(1), b"\n")
    }

    #[rstest]
    fn test_slots_iterator() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token1", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token2", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![GreenElement::from(token1.clone()), GreenElement::from(token2.clone())]);

        assert_eq!(node.slots().count(), 2);
        assert_eq!(
            node.slots().last().unwrap(),
            &Slot::Token {
                rel_offset: 8,
                token: token2.clone()
            }
        );
        assert_eq!(
            node.slots().nth(0).unwrap(),
            &Slot::Token {
                rel_offset: 0,
                token: token1.clone()
            }
        );
        assert_eq!(node.slots().fold(0, |acc, _| acc + 1), 2);
        assert_eq!(
            node.slots().next().unwrap(),
            &Slot::Token {
                rel_offset: 0,
                token: token1.clone()
            }
        );
        assert_eq!(node.slots().size_hint(), (2, Some(2)));
    }

    #[rstest]
    fn test_slots_double_ended_iterator() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token1", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token2", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![GreenElement::from(token1.clone()), GreenElement::from(token2.clone())]);

        assert_eq!(
            node.slots().next_back().unwrap(),
            &Slot::Token {
                rel_offset: 8,
                token: token2.clone()
            }
        );
        assert_eq!(
            node.slots().nth_back(1).unwrap(),
            &Slot::Token {
                rel_offset: 0,
                token: token1.clone()
            }
        );
        assert_eq!(node.slots().rfold(0, |acc, _| acc + 1), 2);
    }

    #[rstest]
    fn test_slots_exact_size_iterator() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token1", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token2", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![GreenElement::from(token1.clone()), GreenElement::from(token2.clone())]);

        assert_eq!(node.slots().len(), 2);
    }
}
