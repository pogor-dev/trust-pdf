//! # Green Node - PDF Composite Syntax Structures
//!
//! Immutable, shareable internal nodes representing PDF composite structures.

use std::{
    borrow::{Borrow, Cow},
    fmt,
    iter::{self, FusedIterator},
    mem::{self, ManuallyDrop},
    ops::{self, Range},
    ptr, slice,
};

use countme::Count;

use crate::{
    NodeOrToken, SyntaxKind,
    arc::{arc_main::Arc, header_slice::HeaderSlice, thin_arc::ThinArc},
    green::{
        element::{GreenElement, GreenElementRef},
        token::{GreenToken, GreenTokenData},
    },
};

/// Internal representation of child nodes with positional metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(crate) enum GreenChild {
    /// Child node with relative byte offset from parent start
    Node { rel_offset: u32, node: GreenNode },
    /// Child token with relative byte offset from parent start  
    Token { rel_offset: u32, token: GreenToken },
}

type Repr = HeaderSlice<GreenNodeHead, [GreenChild]>;
type ReprThin = HeaderSlice<GreenNodeHead, [GreenChild; 0]>;

/// Immutable PDF composite node with efficient sharing and zero-cost data access.
///
/// Represents PDF structures that contain child elements (objects, dictionaries, arrays).
/// Supports efficient cloning via reference counting and structural sharing.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GreenNode {
    ptr: ThinArc<GreenNodeHead, GreenChild>,
}

/// Metadata header for green nodes containing size and classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GreenNodeHead {
    /// PDF syntax kind (Object, Dictionary, Array, etc.)
    kind: SyntaxKind,
    /// Total width including all trivia elements
    full_width: u32,
    /// Reference counting for memory management
    _c: Count<GreenNode>,
}

impl GreenNode {
    /// Creates a new PDF composite node from child elements.
    ///
    /// Constructs immutable node with calculated width metrics and relative child offsets.
    #[inline]
    pub fn new<I>(kind: SyntaxKind, children: I) -> GreenNode
    where
        I: IntoIterator<Item = GreenElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut full_width: u32 = 0;
        let children = children.into_iter().map(|el| {
            let rel_offset = full_width;
            full_width += el.full_width();
            match el {
                NodeOrToken::Node(node) => GreenChild::Node { rel_offset, node },
                NodeOrToken::Token(token) => GreenChild::Token { rel_offset, token },
            }
        });

        let head = GreenNodeHead {
            kind,
            full_width: 0,
            _c: Count::new(),
        };
        let data = ThinArc::from_header_and_iter(head, children);

        // XXX: fixup `full_width` after construction, because we can't iterate
        // `children` twice.
        let data = {
            let mut data = Arc::from_thin(data);
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

/// Data access layer for green nodes providing zero-cost API methods.
#[repr(transparent)]
pub struct GreenNodeData {
    data: ReprThin,
}

impl GreenNodeData {
    #[inline]
    fn header(&self) -> &GreenNodeHead {
        &self.data.header
    }

    #[inline]
    fn slice(&self) -> &[GreenChild] {
        self.data.slice()
    }

    /// Returns the semantic classification of this PDF node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.header().kind
    }

    /// Returns the byte length of text content excluding trivia.
    ///
    /// Calculated as total width minus leading and trailing trivia, similar to Roslyn's approach.
    #[inline]
    pub fn width(&self) -> u32 {
        if self.full_width() == 0 {
            return 0;
        }

        let leading_width = self.get_leading_trivia_width();
        let trailing_width = self.get_trailing_trivia_width();

        self.full_width()
            .saturating_sub(leading_width + trailing_width)
    }

    /// Returns the total byte span including all child trivia.
    #[inline]
    pub fn full_width(&self) -> u32 {
        self.header().full_width
    }

    /// Returns the width of leading trivia from the first terminal node.
    ///
    /// Similar to Roslyn's GetLeadingTriviaWidth, finds the leftmost terminal
    /// and returns its leading trivia width.
    #[inline]
    pub fn get_leading_trivia_width(&self) -> u32 {
        if self.full_width() == 0 {
            return 0;
        }

        if let Some(first_terminal) = self.get_first_terminal() {
            return first_terminal.leading_trivia().width();
        }

        0
    }

    /// Returns the width of trailing trivia from the last terminal node.
    ///
    /// Similar to Roslyn's GetTrailingTriviaWidth, finds the rightmost terminal
    /// and returns its trailing trivia width.
    #[inline]
    pub fn get_trailing_trivia_width(&self) -> u32 {
        if self.full_width() == 0 {
            return 0;
        }

        if let Some(last_terminal) = self.get_last_terminal() {
            return last_terminal.trailing_trivia().width();
        }

        0
    }

    /// Returns an iterator over all immediate child elements.
    #[inline]
    pub fn children(&self) -> NodeChildren<'_> {
        NodeChildren {
            raw: self.slice().iter(),
        }
    }

    /// Creates a new node with one child replaced.
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

    /// Creates a new node with a child inserted at the specified position.
    #[must_use]
    pub fn insert_child(&self, index: usize, new_child: GreenElement) -> GreenNode {
        self.splice_children(index..index, iter::once(new_child))
    }

    /// Creates a new node with a child removed at the specified position.
    #[must_use]
    pub fn remove_child(&self, index: usize) -> GreenNode {
        self.splice_children(index..=index, iter::empty())
    }

    /// Creates a new node with children replaced in the specified range.
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

    /// Finds the first terminal (leaf token) in the subtree rooted at this node.
    ///
    /// Traverses the tree depth-first, always taking the leftmost child,
    /// until reaching a terminal token. Returns the first token found.
    pub fn get_first_terminal(&self) -> Option<&GreenTokenData> {
        let mut current = self;

        loop {
            // Look for the first child
            for child in current.children() {
                match child {
                    NodeOrToken::Token(token) => {
                        // Found a token - this is our terminal
                        return Some(token);
                    }
                    NodeOrToken::Node(node) => {
                        // Found a node - continue traversing into it
                        current = node;
                        break;
                    }
                }
            }

            // If we get here and didn't find any children, there are no tokens
            if current.children().len() == 0 {
                return None;
            }
        }
    }

    /// Finds the last terminal (leaf token) in the subtree rooted at this node.
    ///
    /// Traverses the tree depth-first, always taking the rightmost child,
    /// until reaching a terminal token. Returns the last token found.
    pub fn get_last_terminal(&self) -> Option<&GreenTokenData> {
        let mut current = self;

        loop {
            // Look for the last child (iterate backwards)
            for child in current.children().rev() {
                match child {
                    NodeOrToken::Token(token) => {
                        // Found a token - this is our terminal
                        return Some(token);
                    }
                    NodeOrToken::Node(node) => {
                        // Found a node - continue traversing into it
                        current = node;
                        break;
                    }
                }
            }

            // If we get here and didn't find any children, there are no tokens
            if current.children().len() == 0 {
                return None;
            }
        }
    }

    /// Returns the text content without leading and trailing trivia.
    ///
    /// Similar to Roslyn's ToString(), this excludes boundary trivia but preserves
    /// internal whitespace and formatting between tokens.
    pub fn text(&self) -> Vec<u8> {
        let mut result = Vec::new();
        self.write_text_to(&mut result, false, false);
        result
    }

    /// Returns the full text content including all trivia.
    ///
    /// Similar to Roslyn's ToFullString(), this includes all leading and trailing
    /// trivia, providing the complete textual representation of the node.
    pub fn full_text(&self) -> Vec<u8> {
        let mut result = Vec::new();
        self.write_text_to(&mut result, true, true);
        result
    }

    /// Writes text content to a byte vector with trivia control.
    ///
    /// This is the core text generation function that handles the recursive
    /// traversal of the syntax tree and trivia inclusion logic.
    ///
    /// Parameters:
    /// - `include_leading`: Whether to include leading trivia from the first terminal token
    /// - `include_trailing`: Whether to include trailing trivia from the last terminal token
    fn write_text_to(&self, writer: &mut Vec<u8>, include_leading: bool, include_trailing: bool) {
        let children: Vec<_> = self.children().collect();

        if children.is_empty() {
            return;
        }

        let first_index = 0;
        let last_index = children.len() - 1;

        for (i, child) in children.iter().enumerate() {
            let is_first = i == first_index;
            let is_last = i == last_index;

            match child {
                NodeOrToken::Token(token) => {
                    // For tokens, write trivia based on position and flags
                    // Leading trivia: include unless this is the first token and include_leading is false
                    let write_leading = !(is_first && !include_leading);
                    // Trailing trivia: include unless this is the last token and include_trailing is false
                    let write_trailing = !(is_last && !include_trailing);

                    if write_leading {
                        let leading_text = token.leading_trivia().text();
                        writer.extend_from_slice(&leading_text);
                    }

                    writer.extend_from_slice(token.text());

                    if write_trailing {
                        let trailing_text = token.trailing_trivia().text();
                        writer.extend_from_slice(&trailing_text);
                    }
                }
                NodeOrToken::Node(node) => {
                    // For nodes, propagate trivia flags based on position
                    // Only the first child should respect include_leading
                    // Only the last child should respect include_trailing
                    let node_leading = if is_first { include_leading } else { true };
                    let node_trailing = if is_last { include_trailing } else { true };
                    node.write_text_to(writer, node_leading, node_trailing);
                }
            }
        }
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

impl GreenChild {
    #[inline]
    pub(crate) fn as_ref(&self) -> GreenElementRef {
        match self {
            GreenChild::Node { node, .. } => NodeOrToken::Node(node),
            GreenChild::Token { token, .. } => NodeOrToken::Token(token),
        }
    }

    #[inline]
    pub(crate) fn rel_offset(&self) -> u32 {
        match self {
            GreenChild::Node { rel_offset, .. } | GreenChild::Token { rel_offset, .. } => {
                *rel_offset
            }
        }
    }

    #[inline]
    pub(crate) fn rel_range(&self) -> Range<u32> {
        let len = self.as_ref().full_width();
        let rel_offset = self.rel_offset();
        rel_offset..(rel_offset + len)
    }
}

/// Iterator over PDF node children with efficient access patterns.
#[derive(Debug, Clone)]
pub struct NodeChildren<'a> {
    pub(crate) raw: slice::Iter<'a, GreenChild>,
}

impl ExactSizeIterator for NodeChildren<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.raw.len()
    }
}

impl<'a> Iterator for NodeChildren<'a> {
    type Item = GreenElementRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<GreenElementRef<'a>> {
        self.raw.next().map(GreenChild::as_ref)
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
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth(n).map(GreenChild::as_ref)
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
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

impl DoubleEndedIterator for NodeChildren<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw.next_back().map(GreenChild::as_ref)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth_back(n).map(GreenChild::as_ref)
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

impl FusedIterator for NodeChildren<'_> {}
