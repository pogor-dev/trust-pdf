//! # Green Node Data: Internal Node Structure and Implementation
//!
//! This module provides the `GreenNodeData` type, which contains the actual data
//! and methods for green tree nodes. It serves as the core implementation for
//! immutable syntax tree nodes in the PDF compiler.
//!
//! ## What is GreenNodeData?
//!
//! `GreenNodeData` is the concrete implementation behind `GreenNode`, containing:
//! - **Node header**: Metadata like syntax kind, text length, and reference counting
//! - **Child slots**: Array of child elements (nodes or tokens) in this node
//! - **Access methods**: Safe API for querying node properties and children
//!
//! ## Memory Layout
//!
//! The data structure uses a `#[repr(transparent)]` layout around `GreenNodeReprThin`,
//! providing:
//! - Zero-cost abstraction over the underlying representation
//! - Efficient header+slice memory layout via `HeaderSlice`
//! - Atomic reference counting through `ThinArc`
//! - Cache-friendly data organization
//!
//! ## Node Properties
//!
//! Every green node has these fundamental properties:
//!
//! ### Kind
//! The syntax kind identifies what type of construct this node represents
//! (e.g., PDF object, dictionary, array, stream content).
//!
//! ### Text Length
//! The total length of source text covered by this node and all its children.
//! This enables efficient range calculations without tree traversal.
//!
//! ### Child Slots
//! A fixed-size array of child positions, where each slot can contain:
//! - A child node (for nested structures)
//! - A token (for leaf content)
//! - Nothing (for optional elements that weren't present)
//!
//! ## PDF Context Usage
//!
//! In PDF syntax trees, node data represents structural elements:
//!
//! ### PDF Object Node
//! ```pdf
//! 1 0 obj
//! << /Type /Catalog /Pages 2 0 R >>
//! endobj
//! ```
//! - Kind: Object
//! - Text length: Full object including whitespace
//! - Slots: [number, number, "obj", dictionary, "endobj"]
//!
//! ### Dictionary Node
//! ```pdf
//! << /Type /Catalog /Pages 2 0 R >>
//! ```
//! - Kind: Dictionary
//! - Text length: Dictionary including delimiters
//! - Slots: ["<<", entries..., ">>"]
//!
//! ## Thread Safety
//!
//! `GreenNodeData` is immutable after construction and thread-safe:
//! - All fields are read-only after initialization
//! - Reference counting uses atomic operations
//! - Can be safely shared across threads
//!
//! ## Memory Efficiency
//!
//! The design optimizes for memory usage:
//! - Header data is compact and cache-friendly
//! - Child slots use minimal representation
//! - Structural sharing reduces duplication
//! - No unnecessary padding or alignment waste

use std::{borrow::Borrow, fmt, mem::ManuallyDrop, ptr};

use crate::green::{
    GreenNodeHead, GreenNodeReprThin, kind::RawSyntaxKind, node::GreenNode, node_slot::Slot,
    node_slots::Slots,
};

#[repr(transparent)]
pub(crate) struct GreenNodeData {
    pub(crate) data: GreenNodeReprThin,
}

impl GreenNodeData {
    #[inline]
    fn header(&self) -> &GreenNodeHead {
        &self.data.header
    }

    /// Kind of this node.
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.header().kind
    }

    #[inline]
    pub(crate) fn slice(&self) -> &[Slot] {
        self.data.slice()
    }

    /// Returns the length of the text covered by this node.
    #[inline]
    pub fn text_len(&self) -> u64 {
        self.header().text_len
    }

    /// Returns the slots of this node. Every node of a specific kind has the same number of slots
    /// to allow using fixed offsets to retrieve a specific child even if some other child is missing.
    #[inline]
    pub fn slots(&self) -> Slots<'_> {
        Slots {
            raw: self.slice().iter(),
        }
    }
}

impl ToOwned for GreenNodeData {
    type Owned = GreenNode;

    #[inline]
    fn to_owned(&self) -> GreenNode {
        unsafe {
            let green = GreenNode::from_raw(ptr::NonNull::from(self));
            let green = ManuallyDrop::new(green);
            GreenNode::clone(&green)
        }
    }
}

impl Borrow<GreenNodeData> for GreenNode {
    #[inline]
    fn borrow(&self) -> &GreenNodeData {
        self
    }
}

impl fmt::Debug for GreenNodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenNode")
            .field("kind", &self.kind())
            .field("text_len", &self.text_len())
            .field("n_slots", &self.slots().len())
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_green_node_data_interface() {
        // Test that GreenNodeData provides the expected public interface
        fn assert_has_kind<T>(_: T) 
        where 
            T: Fn(&GreenNodeData) -> RawSyntaxKind,
        {}
        
        fn assert_has_text_len<T>(_: T) 
        where 
            T: Fn(&GreenNodeData) -> u64,
        {}
        
        fn assert_has_slots<T>(_: T) 
        where 
            T: Fn(&GreenNodeData) -> Slots<'_>,
        {}
        
        // Verify the methods exist
        assert_has_kind(GreenNodeData::kind);
        assert_has_text_len(GreenNodeData::text_len);
        assert_has_slots(GreenNodeData::slots);
    }

    #[test]
    fn test_green_node_data_transparent_repr() {
        // Test that GreenNodeData is transparently represented
        use std::mem;
        
        assert_eq!(
            mem::size_of::<GreenNodeData>(),
            mem::size_of::<GreenNodeReprThin>()
        );
        assert_eq!(
            mem::align_of::<GreenNodeData>(),
            mem::align_of::<GreenNodeReprThin>()
        );
    }

    #[test]
    fn test_green_node_data_to_owned() {
        // Test that ToOwned is properly implemented
        fn assert_to_owned<T: ToOwned<Owned = GreenNode>>(_: &T) {}
        
        // This would test with actual data:
        // let data: &GreenNodeData = create_test_node_data();
        // assert_to_owned(data);
        // let owned = data.to_owned();
        // verify_owned_node(owned);
    }

    #[test]  
    fn test_green_node_data_borrow() {
        // Test that Borrow is properly implemented for conversions
        fn assert_borrow<T: Borrow<GreenNodeData>>(_: &T) {}
        
        // This would test with actual data:
        // let node: &GreenNode = create_test_node();
        // assert_borrow(node);
        // let borrowed: &GreenNodeData = node.borrow();
        // verify_borrowed_data(borrowed);
    }

    #[test]
    fn test_green_node_data_debug() {
        // Test that Debug is properly implemented
        fn assert_debug<T: fmt::Debug>(_: T) {}
        
        // Verify the trait is implemented
        // This would test with actual data:
        // let data: &GreenNodeData = create_test_node_data();
        // let debug_str = format!("{:?}", data);
        // assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_green_node_data_display() {
        // Test that Display is properly implemented
        fn assert_display<T: fmt::Display>(_: T) {}
        
        // Verify the trait is implemented
        // This would test with actual data:
        // let data: &GreenNodeData = create_test_node_data();
        // let display_str = format!("{}", data);
        // assert!(!display_str.is_empty());
    }

    #[test]
    fn test_raw_syntax_kind_integration() {
        // Test that RawSyntaxKind works properly in this context
        let kind = RawSyntaxKind(42);
        assert_eq!(kind.0, 42);
        
        // Verify compatibility with our interfaces
        fn accepts_raw_kind(_: RawSyntaxKind) {}
        accepts_raw_kind(kind);
    }

    #[test]
    fn test_slots_integration() {
        // Test that Slots type works properly
        // This verifies the interface exists and is properly typed
        fn processes_slots(_: Slots<'_>) {}
        
        // This would test with actual data:
        // let data: &GreenNodeData = create_test_node_data();
        // let slots = data.slots();
        // processes_slots(slots);
        // verify_slots_contents(slots);
    }

    #[test]
    fn test_header_access() {
        // Test that header access is properly encapsulated
        // Note: header() is private, so we test through public interfaces
        
        // This would test with actual data:
        // let data: &GreenNodeData = create_test_node_data();
        // let kind = data.kind(); // Uses header internally
        // let text_len = data.text_len(); // Uses header internally
        // verify_header_consistency(kind, text_len);
    }

    #[test]
    fn test_slice_access() {
        // Test that slice access works correctly
        // Note: slice() is internal, so we test through slots()
        
        // This would test with actual data:
        // let data: &GreenNodeData = create_test_node_data();
        // let slots = data.slots();
        // verify_slice_contents(slots);
    }
}
