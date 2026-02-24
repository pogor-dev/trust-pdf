use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use countme::Count;

use crate::{
    GreenFlags, GreenNodeElement, GreenNodeElementRef, GreenTokenData, GreenTokenElement, GreenTokenElementRef, GreenTriviaData, SyntaxKind,
    arc::{Arc, HeaderSlice, ThinArc},
};

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenNodeHead {
    full_width: u32,   // 4 bytes
    kind: SyntaxKind,  // 2 bytes
    flags: GreenFlags, // 1 byte
    _c: Count<GreenNode>,
}

type Repr = HeaderSlice<GreenNodeHead, [GreenNodeElement]>;
type ReprThin = HeaderSlice<GreenNodeHead, [GreenNodeElement; 0]>;

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

    /// Full text of this node, including leading and trailing trivia.
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

    /// Returns the full width of this node, including leading and trailing trivia.
    #[inline]
    pub fn full_width(&self) -> u32 {
        self.data.header.full_width
    }

    /// Returns the flags of this node.
    #[inline]
    pub fn flags(&self) -> GreenFlags {
        self.data.header.flags
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
        self.data.slice().len()
    }

    #[inline]
    fn slots(&self) -> &[GreenNodeElement] {
        self.data.slice()
    }

    #[inline]
    fn slot(&self, index: usize) -> Option<&GreenNodeElement> {
        self.slots().get(index)
    }

    /// Compute the starting offset of slot `index` relative to this node.
    /// (Useful for red position computation.)
    fn slot_offset(&self, index: usize) -> Option<u32> {
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
    /// * `leading` - If true, include the first node's leading trivia
    /// * `trailing` - If true, include the last node's trailing trivia
    fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        fn process_stack(output: &mut Vec<u8>, stack: &mut Vec<(GreenNodeElementRef<'_>, bool, bool)>) {
            while let Some((item, current_leading, current_trailing)) = stack.pop() {
                match item {
                    GreenNodeElementRef::Token(token_data) => {
                        if current_leading && let Some(leading_trivia) = token_data.leading_trivia() {
                            output.extend_from_slice(&leading_trivia.full_text());
                        }

                        output.extend_from_slice(&token_data.text());

                        if current_trailing && let Some(trailing_trivia) = token_data.trailing_trivia() {
                            output.extend_from_slice(&trailing_trivia.full_text());
                        }
                    }
                    GreenNodeElementRef::Trivia(trivia_data) => {
                        output.extend_from_slice(&trivia_data.text());
                    }
                    GreenNodeElementRef::Node(node_data) => {
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
                                GreenNodeElement::Node(node) => {
                                    let node_data: &GreenNodeData = node;
                                    stack.push((GreenNodeElementRef::Node(node_data), include_leading, include_trailing));
                                }
                                GreenNodeElement::Token(token) => {
                                    let token_data: GreenTokenElementRef = token.as_deref();
                                    stack.push((GreenNodeElementRef::Token(token_data), include_leading, include_trailing));
                                }
                                GreenNodeElement::Trivia(trivia) => {
                                    let trivia_data: &GreenTriviaData = trivia;
                                    stack.push((GreenNodeElementRef::Trivia(trivia_data), include_leading, include_trailing));
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut output = Vec::new();

        // Explicit stack to avoid recursion on deeply nested trees.
        let mut stack: Vec<(GreenNodeElementRef<'_>, bool, bool)> = Vec::with_capacity(64);

        // Seed with this node itself; processing will drill into its slots.
        stack.push((GreenNodeElementRef::Node(self), leading, trailing));

        process_stack(&mut output, &mut stack);
        output
    }

    /// Returns the first terminal node in the node tree
    fn first_token(&self) -> Option<&GreenTokenElement> {
        for child in self.slots() {
            match child {
                GreenNodeElement::Token(token) => return Some(token),
                GreenNodeElement::Node(node) => {
                    if let Some(token) = node.first_token() {
                        return Some(token);
                    }
                }
                GreenNodeElement::Trivia(_) => continue,
            }
        }
        None
    }

    /// Returns the last terminal node in the node tree
    fn last_token(&self) -> Option<&GreenTokenElement> {
        for child in self.slots().iter().rev() {
            match child {
                GreenNodeElement::Token(token) => return Some(token),
                GreenNodeElement::Node(node) => {
                    if let Some(token) = node.last_token() {
                        return Some(token);
                    }
                }
                GreenNodeElement::Trivia(_) => continue,
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
        for &byte in &self.full_text() {
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
    ptr: ThinArc<GreenNodeHead, GreenNodeElement>,
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
        I: IntoIterator<Item = GreenNodeElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut full_width = 0u32;
        let slots = slots.into_iter().map(|el| {
            full_width += el.full_width();
            el
        });

        let data = ThinArc::from_header_and_iter(
            GreenNodeHead {
                kind,
                full_width: 0,
                flags: GreenFlags::NONE,
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
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenNodeHead, GreenNodeElement>>(arc)
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
