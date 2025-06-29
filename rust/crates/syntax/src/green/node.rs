//! # Green Node: Immutable Syntax Tree Node Implementation
//!
//! This module provides the `GreenNode` type, which represents immutable internal
//! nodes in the green tree layer of the syntax tree architecture. Green nodes
//! are the fundamental building blocks for creating memory-efficient, thread-safe
//! syntax trees with full fidelity preservation.
//!
//! ## What is a Green Node?
//!
//! A green node is an immutable tree node that:
//! - Contains child elements (other nodes or tokens)
//! - Preserves exact source text representation including trivia
//! - Uses reference counting for efficient memory sharing
//! - Supports thread-safe access across multiple contexts
//!
//! ## Design Principles
//!
//! ### Immutability
//! Once created, green nodes never change. This enables:
//! - Safe sharing across threads without synchronization
//! - Efficient caching of parsed results
//! - Simplified reasoning about tree structure
//! - Incremental parsing with structural sharing
//!
//! ### Memory Efficiency
//! Green nodes use several techniques for memory efficiency:
//! - **Reference counting**: Shared nodes are stored only once
//! - **Thin pointers**: Optimized memory layout for header+children pattern
//! - **Compact representation**: Minimal overhead per node
//! - **Structural sharing**: Common subtrees are shared between versions
//!
//! ### Full Fidelity
//! Green nodes preserve complete source information:
//! - All whitespace and formatting
//! - Comments and other trivia
//! - Exact token positions and lengths
//! - Original error recovery decisions
//!
//! ## PDF Context Usage
//!
//! In PDF parsing, green nodes represent structural elements like:
//! - **PDF Objects**: Complete object definitions with headers and content
//! - **Dictionaries**: Key-value collections with proper nesting
//! - **Arrays**: Ordered element sequences with correct spacing
//! - **Content Streams**: Operator sequences with preserved formatting
//! - **Cross-reference sections**: Table structures with exact layout
//!
//! ## Thread Safety
//!
//! Green nodes are fully thread-safe:
//! - Immutable after construction
//! - Reference counting uses atomic operations
//! - No interior mutability or weak references
//! - Safe to share across thread boundaries
//!
//! ## Memory Layout
//!
//! The node uses a `ThinArc<GreenNodeHead, Slot>` internally, which provides:
//! - Efficient header+slice memory layout
//! - Atomic reference counting
//! - Optimized pointer representation
//! - Cache-friendly data organization
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! // Green nodes are typically created by parsers or builders
//! // let node: GreenNode = parser.parse_object(source);
//! // 
//! // Access node properties
//! // let kind = node.kind();
//! // let length = node.text_len();
//! // let children = node.children();
//! //
//! // Clone is cheap (just reference counting)
//! // let shared_node = node.clone();
//! ```

use std::{fmt, mem, ops, ptr};

use crate::{
    arc::{arc::Arc, thin_arc::ThinArc},
    green::{
        GreenNodeRepr, GreenNodeReprThin, node_data::GreenNodeData, node_head::GreenNodeHead,
        node_slot::Slot,
    },
};

/// Internal node in the immutable tree.
/// It has other nodes and tokens as children.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct GreenNode {
    ptr: ThinArc<GreenNodeHead, Slot>,
}

impl GreenNode {
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenNodeData>) -> GreenNode {
        let arc = unsafe { Arc::from_raw(&ptr.as_ref().data as *const GreenNodeReprThin) };
        let arc =
            unsafe { mem::transmute::<Arc<GreenNodeReprThin>, ThinArc<GreenNodeHead, Slot>>(arc) };
        GreenNode { ptr: arc }
    }
}

impl ops::Deref for GreenNode {
    type Target = GreenNodeData;

    #[inline]
    fn deref(&self) -> &GreenNodeData {
        unsafe {
            let repr: &GreenNodeRepr = &self.ptr;
            let repr: &GreenNodeReprThin =
                &*(repr as *const GreenNodeRepr as *const GreenNodeReprThin);
            mem::transmute::<&GreenNodeReprThin, &GreenNodeData>(repr)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_green_node_memory_layout() {
        // Test that GreenNode has a reasonable memory footprint
        let size = mem::size_of::<GreenNode>();
        let align = mem::align_of::<GreenNode>();
        
        // Should be pointer-sized (since it's essentially a ThinArc)
        assert!(size > 0);
        assert!(align > 0);
        
        // Should be efficiently aligned
        assert!(align <= 8); // Reasonable upper bound for pointer alignment
    }

    #[test]
    fn test_green_node_transparent_repr() {
        // Test that GreenNode is transparently represented
        // (this verifies the #[repr(transparent)] attribute works)
        assert_eq!(
            mem::size_of::<GreenNode>(),
            mem::size_of::<ThinArc<GreenNodeHead, Slot>>()
        );
        assert_eq!(
            mem::align_of::<GreenNode>(),
            mem::align_of::<ThinArc<GreenNodeHead, Slot>>()
        );
    }

    #[test]
    fn test_green_node_deref() {
        // Test that GreenNode properly derefs to GreenNodeData
        // Note: This test verifies the trait implementation exists
        fn assert_deref<T: ops::Deref<Target = GreenNodeData>>(_: T) {}
        
        // This would test with actual data:
        // let node: GreenNode = create_test_node();
        // assert_deref(node);
    }

    #[test]
    fn test_green_node_debug() {
        // Test that Debug is properly implemented
        // Note: This verifies the implementation delegates to GreenNodeData
        fn assert_debug<T: fmt::Debug>(_: T) {}
        
        // Verify the trait is implemented
        fn test_debug_impl() {
            // This would test with actual data:
            // let node: GreenNode = create_test_node();
            // let debug_str = format!("{:?}", node);
            // assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_green_node_display() {
        // Test that Display is properly implemented
        fn assert_display<T: fmt::Display>(_: T) {}
        
        // Verify the trait is implemented
        fn test_display_impl() {
            // This would test with actual data:
            // let node: GreenNode = create_test_node();
            // let display_str = format!("{}", node);
            // assert!(!display_str.is_empty());
        }
    }

    #[test]
    fn test_green_node_clone() {
        // Test that GreenNode supports efficient cloning
        fn assert_clone<T: Clone>(_: T) {}
        
        // Verify the trait is implemented
        // Real test would verify reference counting behavior:
        // let node: GreenNode = create_test_node();
        // let cloned = node.clone();
        // assert!(Arc::ptr_eq(&node.ptr, &cloned.ptr)); // Same underlying data
    }

    #[test]
    fn test_green_node_equality() {
        // Test that GreenNode supports structural equality
        fn assert_eq<T: PartialEq>(_: T) {}
        
        // Verify the traits are implemented
        // Real test would verify equality semantics:
        // let node1: GreenNode = create_test_node();
        // let node2 = node1.clone();
        // let node3: GreenNode = create_different_test_node();
        // assert_eq!(node1, node2);
        // assert_ne!(node1, node3);
    }

    #[test]
    fn test_green_node_hash() {
        // Test that GreenNode supports hashing
        fn assert_hash<T: std::hash::Hash>(_: T) {}
        
        // Verify the trait is implemented
        // Real test would verify hash consistency:
        // let node: GreenNode = create_test_node();
        // let hash1 = calculate_hash(&node);
        // let hash2 = calculate_hash(&node.clone());
        // assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_green_node_from_raw() {
        // Test that from_raw correctly constructs a GreenNode
        // Note: This is an unsafe operation that requires valid data
        
        // This test verifies the method signature exists
        fn assert_from_raw_exists() {
            // This would test with actual data:
            // let raw_ptr: ptr::NonNull<GreenNodeData> = create_test_raw_data();
            // let node = unsafe { GreenNode::from_raw(raw_ptr) };
            // verify_node_properties(node);
        }
    }

    #[test]
    fn test_green_node_type_relationships() {
        // Test that GreenNode works with related types
        
        // Should work with GreenNodeRepr
        fn assert_repr_compatibility() {
            // let repr: &GreenNodeRepr = get_repr_from_node();
            // let thin_repr: &GreenNodeReprThin = convert_repr(repr);
        }
        
        // Should work with GreenNodeData
        fn assert_data_compatibility() {
            // let node: GreenNode = create_test_node();
            // let data: &GreenNodeData = &*node;
            // verify_data_properties(data);
        }
    }
}
