use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops::{self},
    ptr,
};

use countme::Count;

use crate::green::diagnostics;
use crate::{
    GreenDiagnostic, GreenFlags, GreenNodeElement, GreenNodeElementRef, GreenTokenElement, GreenTokenElementRef, GreenTriviaData, SyntaxKind,
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
    pub(crate) fn flags(&self) -> GreenFlags {
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
                        output.extend_from_slice(&token_data.write_to(current_leading, current_trailing));
                    }
                    GreenNodeElementRef::Trivia(trivia_data) => {
                        output.extend_from_slice(&trivia_data.text());
                    }
                    GreenNodeElementRef::Node(node_data) => {
                        let slots = node_data.slots();
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
    /// Determines if this node is structurally equivalent to another node.
    ///
    /// This performs a deep structural comparison that handles the special case where
    /// a single-element list can be represented either as a List node with one child
    /// or as just the child node directly. Based on Roslyn's EquivalentToInternal.
    ///
    /// Two nodes are equivalent if:
    /// - Their kinds match (after normalizing single-element lists)
    /// - Their full widths are equal
    /// - Their slot counts match
    /// - All corresponding children are recursively equivalent
    fn eq(&self, other: &Self) -> bool {
        let (mut kind1, mut node1) = (self.kind(), self);
        let (mut kind2, mut node2) = (other.kind(), other);

        // Normalize single-element lists: unwrap the child if this is a List with one slot
        if kind1 != kind2 {
            if kind1 == SyntaxKind::List && node1.slot_count() == 1 {
                if let Some(GreenNodeElement::Node(child)) = node1.slot(0) {
                    kind1 = child.kind();
                    node1 = child;
                }
            }

            if kind2 == SyntaxKind::List && node2.slot_count() == 1 {
                if let Some(GreenNodeElement::Node(child)) = node2.slot(0) {
                    kind2 = child.kind();
                    node2 = child;
                }
            }

            if kind1 != kind2 {
                return false;
            }
        }

        // Check full width
        if node1.full_width() != node2.full_width() {
            return false;
        }

        // Check slot count
        let slot_count = node1.slot_count();
        if slot_count != node2.slot_count() {
            return false;
        }

        // Recursively check all children
        for i in 0..slot_count {
            let child1 = node1.slot(i);
            let child2 = node2.slot(i);

            match (child1, child2) {
                (Some(GreenNodeElement::Node(n1)), Some(GreenNodeElement::Node(n2))) => {
                    if n1 != n2 {
                        return false;
                    }
                }
                (Some(GreenNodeElement::Token(t1)), Some(GreenNodeElement::Token(t2))) => {
                    if t1 != t2 {
                        return false;
                    }
                }
                (Some(GreenNodeElement::Trivia(tr1)), Some(GreenNodeElement::Trivia(tr2))) => {
                    if tr1 != tr2 {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        true
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

impl GreenNode {
    /// Creates new Node.
    #[inline]
    pub fn new<I>(kind: SyntaxKind, slots: I) -> GreenNode
    where
        I: IntoIterator<Item = GreenNodeElement>,
        I::IntoIter: ExactSizeIterator,
    {
        Self::create_full(kind, slots, GreenFlags::NONE, Vec::new())
    }

    #[inline]
    pub fn new_with_diagnostic<I>(kind: SyntaxKind, slots: I, diagnostics: Vec<GreenDiagnostic>) -> GreenNode
    where
        I: IntoIterator<Item = GreenNodeElement>,
        I::IntoIter: ExactSizeIterator,
    {
        Self::create_full(kind, slots, GreenFlags::NONE, diagnostics)
    }

    #[inline]
    fn create_full<I>(kind: SyntaxKind, slots: I, base_flags: GreenFlags, diagnostics: Vec<GreenDiagnostic>) -> GreenNode
    where
        I: IntoIterator<Item = GreenNodeElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let has_diagnostics = !diagnostics.is_empty();
        let flags = match has_diagnostics {
            true => base_flags | GreenFlags::CONTAINS_DIAGNOSTIC,
            false => base_flags,
        };

        let mut full_width = 0u32;
        let slots = slots.into_iter().map(|el| {
            full_width += el.full_width();
            el
        });

        let data = ThinArc::from_header_and_iter(
            GreenNodeHead {
                kind,
                full_width: 0,
                flags,
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

        let node = GreenNode { ptr: data };

        if has_diagnostics {
            let key = node.diagnostics_key();
            diagnostics::insert_diagnostics(key, diagnostics);
        }

        node
    }
}

impl_green_boilerplate!(GreenNodeHead, GreenNodeData, GreenNode, GreenNodeElement);

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_node_head_memory_layout() {
        // GreenNodeHead: full_width (4 bytes) + kind (2 bytes) + flags (1 byte) + _c (0 bytes)
        // Expected: 4 + 2 + 1 + padding = 8 bytes (aligned to 4-byte boundary for u32)
        assert_eq!(std::mem::size_of::<GreenNodeHead>(), 8);
        assert_eq!(std::mem::align_of::<GreenNodeHead>(), 4);
    }

    #[test]
    fn test_green_node_data_memory_layout() {
        // GreenNodeData is transparent wrapper around HeaderSlice<GreenNodeHead, [GreenNodeElement; 0]>
        // HeaderSlice = header + length(usize)
        // On 64-bit: 8 bytes (header) + 8 bytes (length) = 16 bytes
        // On 32-bit: 8 bytes (header) + 4 bytes (length) = 12 bytes
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenNodeData>(), 16);
            assert_eq!(std::mem::align_of::<GreenNodeData>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenNodeData>(), 12);
            assert_eq!(std::mem::align_of::<GreenNodeData>(), 4);
        }
    }

    #[test]
    fn test_green_node_memory_layout() {
        // GreenNode wraps a ThinArc pointer
        // On 64-bit: pointer is 8 bytes
        // On 32-bit: pointer is 4 bytes
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenNode>(), 8);
            assert_eq!(std::mem::align_of::<GreenNode>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenNode>(), 4);
            assert_eq!(std::mem::align_of::<GreenNode>(), 4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::green::diagnostics;
    use crate::{DiagnosticKind, DiagnosticSeverity, GreenToken};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_when_empty_expect_node_with_zero_width() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        assert_eq!(node.kind(), SyntaxKind::List);
        assert_eq!(node.full_width(), 0);
        assert_eq!(node.width(), 0);
        assert_eq!(node.slot_count(), 0);
    }

    #[test]
    fn test_new_when_single_token_expect_width_from_token() {
        let token = GreenToken::new(SyntaxKind::OpenBracketToken);
        let node = GreenNode::new(SyntaxKind::ArrayExpression, vec![token.into()]);
        assert_eq!(node.kind(), SyntaxKind::ArrayExpression);
        assert_eq!(node.full_width(), 1);
        assert_eq!(node.width(), 1);
        assert_eq!(node.slot_count(), 1);
    }

    #[test]
    fn test_new_when_multiple_tokens_expect_total_width() {
        let token1 = GreenToken::new(SyntaxKind::OpenBracketToken);
        let token2 = GreenToken::new(SyntaxKind::CloseBracketToken);
        let slots: Vec<GreenNodeElement> = vec![token1.into(), token2.into()];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        assert_eq!(node.full_width(), 2);
        assert_eq!(node.slot_count(), 2);
    }

    #[test]
    fn test_kind_when_node_expect_reflected_kind() {
        let node = GreenNode::new(SyntaxKind::DictionaryExpression, vec![]);
        assert_eq!(node.kind(), SyntaxKind::DictionaryExpression);
    }

    #[test]
    fn test_full_width_when_node_with_children_expect_sum_of_widths() {
        let token1 = GreenToken::new(SyntaxKind::OpenDictToken);
        let token2 = GreenToken::new(SyntaxKind::CloseDictToken);
        let slots: Vec<GreenNodeElement> = vec![token1.into(), token2.into()];
        let node = GreenNode::new(SyntaxKind::DictionaryExpression, slots);
        assert_eq!(node.full_width(), 4);
    }

    #[test]
    fn test_text_when_node_with_tokens_expect_concatenated_text() {
        let token1 = GreenToken::new(SyntaxKind::OpenBracketToken);
        let token2 = GreenToken::new(SyntaxKind::CloseBracketToken);
        let slots: Vec<GreenNodeElement> = vec![token1.into(), token2.into()];
        let node = GreenNode::new(SyntaxKind::ArrayExpression, slots);
        assert_eq!(node.text(), b"[]");
    }

    #[test]
    fn test_full_text_when_empty_node_expect_empty_bytes() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        assert_eq!(node.full_text().len(), 0);
    }

    #[test]
    fn test_slot_count_when_three_slots_expect_three() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::FalseKeyword);
        let token3 = GreenToken::new(SyntaxKind::NullKeyword);
        let slots: Vec<GreenNodeElement> = vec![token1.into(), token2.into(), token3.into()];
        let node = GreenNode::new(SyntaxKind::List, slots);
        assert_eq!(node.slot_count(), 3);
    }

    #[test]
    fn test_flags_when_node_created_expect_flags_none() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        assert_eq!(node.flags(), GreenFlags::NONE);
    }

    #[test]
    fn test_clone_when_node_expect_equal_kind_and_width() {
        let token = GreenToken::new(SyntaxKind::IndirectObjectKeyword);
        let node1 = GreenNode::new(SyntaxKind::IndirectObjectExpression, vec![token.into()]);
        let node2 = node1.clone();
        assert_eq!(node1.kind(), node2.kind());
        assert_eq!(node1.full_width(), node2.full_width());
    }

    #[test]
    fn test_equality_when_same_kind_and_text_expect_equal() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::TrueKeyword);
        let node1 = GreenNode::new(SyntaxKind::List, vec![token1.into()]);
        let node2 = GreenNode::new(SyntaxKind::List, vec![token2.into()]);
        assert_eq!(node1, node2);
    }

    #[test]
    fn test_debug_when_node_expect_struct_debug_format() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        let debug_str = format!("{:?}", node);
        assert!(debug_str.contains("GreenNode"));
        assert!(debug_str.contains("kind"));
    }

    #[test]
    fn test_display_when_node_with_token_expect_token_text() {
        let token = GreenToken::new(SyntaxKind::NullKeyword);
        let node = GreenNode::new(SyntaxKind::NullLiteralExpression, vec![token.into()]);
        let display_str = format!("{}", node);
        assert_eq!(display_str, "null");
    }

    #[test]
    fn test_first_token_when_node_with_tokens_expect_first_token() {
        let token1 = GreenToken::new(SyntaxKind::IndirectObjectKeyword);
        let token2 = GreenToken::new(SyntaxKind::IndirectEndObjectKeyword);
        let slots: Vec<GreenNodeElement> = vec![token1.clone().into(), token2.into()];
        let node = GreenNode::new(SyntaxKind::IndirectObjectExpression, slots);
        let first = unsafe { &*(node.first_token().unwrap() as *const GreenTokenElement) };
        assert_eq!(first.kind(), SyntaxKind::IndirectObjectKeyword);
    }

    #[test]
    fn test_nested_nodes_when_parent_child_expect_correct_widths() {
        let token1 = GreenToken::new(SyntaxKind::OpenDictToken);
        let child = GreenNode::new(SyntaxKind::DictionaryExpression, vec![token1.into()]);
        let parent = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![GreenNodeElement::Node(child)]);
        assert_eq!(parent.full_width(), 2);
        assert_eq!(parent.slot_count(), 1);
    }

    #[test]
    fn test_hash_when_same_node_expect_consistent_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let token = GreenToken::new(SyntaxKind::FalseKeyword);
        let node = GreenNode::new(SyntaxKind::FalseLiteralExpression, vec![token.into()]);

        let mut hasher1 = DefaultHasher::new();
        node.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        node.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_trivia_when_no_trivia_expect_none() {
        let token = GreenToken::new(SyntaxKind::StreamKeyword);
        let node = GreenNode::new(SyntaxKind::StreamExpression, vec![token.into()]);
        assert!(node.leading_trivia().is_none());
        assert!(node.trailing_trivia().is_none());
    }

    #[test]
    fn test_slot_access_when_index_within_bounds_expect_some() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::FalseKeyword);
        let slots: Vec<GreenNodeElement> = vec![token1.into(), token2.into()];
        let node = GreenNode::new(SyntaxKind::List, slots);

        // Accessing valid indices
        assert!(node.slot(0).is_some());
        assert!(node.slot(1).is_some());
        assert!(node.slot(2).is_none());
    }

    #[test]
    fn test_slot_access_with_nested_node_expect_node_element() {
        let inner_token = GreenToken::new(SyntaxKind::NumericLiteralToken);
        let inner_node = GreenNode::new(SyntaxKind::ArrayExpression, vec![inner_token.into()]);
        let outer_node = GreenNode::new(SyntaxKind::DictionaryExpression, vec![GreenNodeElement::Node(inner_node.clone())]);

        let slot = outer_node.slot(0);
        assert!(slot.is_some());
        match slot {
            Some(GreenNodeElement::Node(n)) => {
                assert_eq!(n.kind(), SyntaxKind::ArrayExpression);
            }
            _ => panic!("Expected Node element"),
        }
    }

    #[test]
    fn test_borrow_when_node_expect_data_access() {
        use std::borrow::Borrow;
        let node = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![]);
        let borrowed: &GreenNodeData = node.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::DirectObjectExpression);
    }

    #[test]
    fn test_to_owned_when_data_expect_new_node() {
        let token = GreenToken::new(SyntaxKind::IndirectObjectKeyword);
        let node1 = GreenNode::new(SyntaxKind::IndirectObjectExpression, vec![token.into()]);
        let data: &GreenNodeData = &*node1;
        let node2 = data.to_owned();

        assert_eq!(node1.kind(), node2.kind());
        assert_eq!(node1.slot_count(), node2.slot_count());
        assert_eq!(node1.full_width(), node2.full_width());
    }

    #[test]
    fn test_into_raw_and_from_raw_expect_roundtrip() {
        let token = GreenToken::new(SyntaxKind::OpenBracketToken);
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token.into()]);
        let ptr = GreenNode::into_raw(node1.clone());
        let node2 = unsafe { GreenNode::from_raw(ptr) };

        assert_eq!(node1.kind(), node2.kind());
        assert_eq!(node1.slot_count(), node2.slot_count());
        assert_eq!(node1.full_width(), node2.full_width());
    }

    #[test]
    fn test_width_without_trivia_expect_token_width_only() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken);
        let node = GreenNode::new(SyntaxKind::ArrayExpression, vec![token.into()]);

        // GreenToken without explicit text has width derived from SyntaxKind
        assert_eq!(node.width(), node.full_width());
    }

    #[test]
    fn test_deref_coercion_expect_data_access() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        let data: &GreenNodeData = &*node;
        assert_eq!(data.kind(), SyntaxKind::List);
        assert_eq!(data.slot_count(), 0);
    }

    #[test]
    fn test_partial_eq_when_different_kinds_expect_not_equal() {
        let node1 = GreenNode::new(SyntaxKind::List, vec![]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![]);
        assert_ne!(node1, node2);
    }

    #[test]
    fn test_is_equivalent_to_when_identical_nodes_expect_true() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::TrueKeyword);
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token1.into()]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token2.into()]);

        assert_eq!(node1, node2);
        assert_eq!(node2, node1);
    }

    #[test]
    fn test_is_equivalent_to_when_different_kinds_expect_false() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token1.into()]);
        let node2 = GreenNode::new(SyntaxKind::DictionaryExpression, vec![]);

        assert_ne!(node1, node2);
    }

    #[test]
    fn test_is_equivalent_to_when_different_full_width_expect_false() {
        let token1 = GreenToken::new(SyntaxKind::OpenBracketToken);
        let token2 = GreenToken::new(SyntaxKind::OpenBracketToken);
        let token3 = GreenToken::new(SyntaxKind::CloseBracketToken);
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token1.into()]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token2.into(), token3.into()]);

        assert_ne!(node1, node2);
    }

    #[test]
    fn test_is_equivalent_to_when_different_slot_count_expect_false() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token3 = GreenToken::new(SyntaxKind::FalseKeyword);
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token1.into()]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token2.into(), token3.into()]);

        assert_ne!(node1, node2);
    }

    #[test]
    fn test_is_equivalent_to_when_different_token_kinds_expect_false() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::FalseKeyword);
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token1.into()]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token2.into()]);

        assert_ne!(node1, node2);
    }

    #[test]
    fn test_is_equivalent_to_when_single_element_list_expect_equivalent() {
        // A single-element list should be equivalent to its child node
        let token = GreenToken::new(SyntaxKind::OpenBracketToken);
        let token_elem: GreenNodeElement = token.into();

        // Create a child node
        let child = GreenNode::new(SyntaxKind::ArrayExpression, vec![token_elem.clone()]);

        // Create a List with the child
        let list = GreenNode::new(SyntaxKind::List, vec![GreenNodeElement::Node(child.clone())]);

        // List and child should be equivalent due to normalization
        assert_eq!(&*list, &*child);
        assert_eq!(&*child, &*list);
    }

    #[test]
    fn test_is_equivalent_to_when_nested_nodes_expect_equivalent() {
        let token1 = GreenToken::new(SyntaxKind::OpenDictToken);
        let token2 = GreenToken::new(SyntaxKind::OpenDictToken);
        let child1 = GreenNode::new(SyntaxKind::DictionaryExpression, vec![token1.into()]);
        let child2 = GreenNode::new(SyntaxKind::DictionaryExpression, vec![token2.into()]);

        let parent1 = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![GreenNodeElement::Node(child1)]);
        let parent2 = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![GreenNodeElement::Node(child2)]);

        assert_eq!(parent1, parent2);
    }

    #[test]
    fn test_is_equivalent_to_when_nested_nodes_with_different_children_expect_not_equivalent() {
        let token1 = GreenToken::new(SyntaxKind::OpenDictToken);
        let token2 = GreenToken::new(SyntaxKind::CloseDictToken);
        let child1 = GreenNode::new(SyntaxKind::DictionaryExpression, vec![token1.into()]);
        let child2 = GreenNode::new(SyntaxKind::DictionaryExpression, vec![token2.into()]);

        let parent1 = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![GreenNodeElement::Node(child1)]);
        let parent2 = GreenNode::new(SyntaxKind::DirectObjectExpression, vec![GreenNodeElement::Node(child2)]);

        assert_ne!(parent1, parent2);
    }

    #[test]
    fn test_is_equivalent_to_when_multiple_children_all_match_expect_true() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::FalseKeyword);
        let token3 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token4 = GreenToken::new(SyntaxKind::FalseKeyword);
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token1.into(), token2.into()]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token3.into(), token4.into()]);

        assert_eq!(node1, node2);
    }

    #[test]
    fn test_is_equivalent_to_when_multiple_children_one_differs_expect_false() {
        let token1 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token2 = GreenToken::new(SyntaxKind::FalseKeyword);
        let token3 = GreenToken::new(SyntaxKind::TrueKeyword);
        let token4 = GreenToken::new(SyntaxKind::NullKeyword);
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token1.into(), token2.into()]);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![token3.into(), token4.into()]);

        assert_ne!(node1, node2);
    }

    #[test]
    fn test_is_equivalent_to_when_empty_nodes_expect_true() {
        let node1 = GreenNode::new(SyntaxKind::List, vec![]);
        let node2 = GreenNode::new(SyntaxKind::List, vec![]);

        assert_eq!(node1, node2);
    }

    #[test]
    fn test_new_with_diagnostic_when_created_expect_accessible_and_cleared_on_drop() {
        let diagnostic = GreenDiagnostic::new(DiagnosticKind::Unknown, DiagnosticSeverity::Warning, "node diag");
        let key;

        {
            let node = GreenNode::new_with_diagnostic(SyntaxKind::List, vec![], vec![diagnostic.clone()]);
            assert!(node.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
            let diagnostics = node.diagnostics().expect("diagnostics should exist");
            assert_eq!(diagnostics, vec![diagnostic]);

            key = (&*node as *const GreenNodeData) as usize;
            assert!(diagnostics::contains_diagnostics(key));
        }

        assert!(!diagnostics::contains_diagnostics(key));
    }

    #[test]
    fn test_new_with_diagnostic_when_empty_expect_same_as_new_without_diagnostic_flag() {
        let node = GreenNode::new_with_diagnostic(SyntaxKind::List, vec![], vec![]);
        assert_eq!(node.flags(), GreenFlags::NONE);
        assert!(!node.flags().contains(GreenFlags::CONTAINS_DIAGNOSTIC));
        assert!(node.diagnostics().is_none());
    }
}
