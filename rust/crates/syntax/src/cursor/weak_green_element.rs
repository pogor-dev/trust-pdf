//! # Weak Green Element: Memory-Efficient Tree References
//!
//! This module provides weak references to green tree elements that don't
//! participate in reference counting. These are used by child nodes in the
//! red-green tree to prevent reference cycles while maintaining access to
//! the underlying green tree data.
//!
//! ## What are Weak Green Elements?
//!
//! Weak green elements are raw pointers to green tree data that:
//! - **Don't affect reference counts**: No impact on memory management
//! - **Prevent cycles**: Child nodes can't create memory leaks through back-references
//! - **Enable access**: Still provide access to green tree data when valid
//! - **Require safety guarantees**: Must be used only when the data is known to be alive
//!
//! ## Safety Model
//!
//! Unlike `std::sync::Weak`, these are unsafe raw pointers with no automatic
//! safety checks. The safety is guaranteed by the red-green tree architecture:
//!
//! ### Safety Invariant
//! A child node's weak reference is only valid as long as:
//! 1. The child node's parent chain leads to a root node
//! 2. The root node holds a strong reference to the green tree
//! 3. The green tree contains the referenced element
//!
//! This invariant is maintained by the red-green tree construction and
//! navigation algorithms, making the unsafe operations statically safe.
//!
//! ## Memory Management Benefits
//!
//! This design provides several advantages:
//! - **No cycle detection needed**: Cycles are structurally prevented
//! - **Simplified cleanup**: Only root nodes manage green tree lifetime
//! - **Performance**: No weak reference counting overhead
//! - **Memory efficiency**: No extra metadata for weak reference tracking
//!
//! ## PDF Processing Context
//!
//! In PDF syntax trees, weak references are used for:
//! - Child dictionary entries referencing their green data
//! - Array elements pointing to their syntax representations
//! - Nested structures that don't own their green tree portions
//! - Any non-root node that needs green tree access
//!
//! ## Usage Pattern
//!
//! Weak green elements are typically:
//! 1. Created when constructing child nodes from parent contexts
//! 2. Dereferenced temporarily for property access or traversal
//! 3. Converted to owned references when longer-term access is needed

use std::ptr;

use crate::{
    cursor::{element::GreenElement, green_element::GreenElementRef},
    green::{node_data::GreenNodeData, token_data::GreenTokenData},
    utility_types::node_or_token::NodeOrToken,
};

/// Memory-efficient weak reference to green tree elements.
///
/// `WeakGreenElement` provides access to green tree data without participating
/// in reference counting, enabling child nodes to reference their green data
/// without creating memory cycles. This is a key component of the red-green
/// tree's memory management strategy.
///
/// ## Safety Guarantees
///
/// The safety of these raw pointers is guaranteed by the red-green tree
/// invariants. Child nodes can only exist while their parent chain leads
/// to a root node that owns the green tree, ensuring the referenced data
/// remains valid.
///
/// ## Variants
///
/// - **Node**: Weak reference to a green node (internal tree element)
/// - **Token**: Weak reference to a green token (leaf element with text)
///
/// ## Performance Characteristics
///
/// - **Zero-cost**: No reference counting overhead
/// - **Compact**: Just a raw pointer, minimal memory usage  
/// - **Fast access**: Direct pointer dereferencing
/// - **No cleanup**: No destructor or cleanup logic needed
#[derive(Debug, Clone)]
pub(crate) enum WeakGreenElement {
    /// Weak reference to a green node (internal tree element).
    ///
    /// Points to green node data that represents complex PDF structures
    /// like dictionaries, arrays, or object definitions. The referenced
    /// node contains child elements and structural information.
    Node { 
        /// Raw pointer to the green node data.
        /// Safety: Valid as long as parent chain leads to owning root node.
        ptr: ptr::NonNull<GreenNodeData> 
    },
    
    /// Weak reference to a green token (leaf element with text).
    ///
    /// Points to green token data that represents actual text content
    /// like keywords, values, or structural tokens. The referenced
    /// token contains the actual PDF text and associated trivia.
    Token { 
        /// Raw pointer to the green token data.
        /// Safety: Valid as long as parent chain leads to owning root node.
        ptr: ptr::NonNull<GreenTokenData> 
    },
}

impl WeakGreenElement {
    /// Creates a new weak reference from a borrowed green element reference.
    ///
    /// This constructor extracts a raw pointer from the borrowed reference,
    /// creating a weak reference that can be stored long-term without
    /// affecting reference counts.
    ///
    /// ## Safety
    ///
    /// The returned weak reference is only safe to use as long as the
    /// original green element remains alive through some other strong
    /// reference (typically held by a root node).
    ///
    /// ## Parameters
    ///
    /// * `green` - Borrowed reference to the green element to weakly reference
    ///
    /// ## Returns
    ///
    /// A weak reference that points to the same green data without
    /// participating in reference counting.
    pub(crate) fn new(green: GreenElementRef) -> Self {
        match green {
            NodeOrToken::Node(ptr) => Self::Node {
                ptr: ptr::NonNull::from(ptr),
            },
            NodeOrToken::Token(ptr) => Self::Token {
                ptr: ptr::NonNull::from(ptr),
            },
        }
    }

    /// Safely dereferences the weak reference to access green element data.
    ///
    /// This method provides temporary access to the green tree data without
    /// creating strong references. It's the primary way to access green
    /// element properties and structure from child nodes.
    ///
    /// ## Safety
    ///
    /// This method uses unsafe pointer dereferencing internally, but the
    /// operation is safe due to the red-green tree invariants. The green
    /// data is guaranteed to be alive as long as this weak reference is
    /// reachable from a root node.
    ///
    /// ## Performance
    ///
    /// This is a zero-cost operation that compiles to a direct pointer
    /// dereference after optimization.
    ///
    /// ## Returns
    ///
    /// A borrowed reference to the green element data that can be used
    /// for property access, tree traversal, or other read operations.
    pub(crate) fn as_deref(&self) -> GreenElementRef {
        match self {
            WeakGreenElement::Node { ptr } => GreenElementRef::Node(unsafe { ptr.as_ref() }),
            WeakGreenElement::Token { ptr } => GreenElementRef::Token(unsafe { ptr.as_ref() }),
        }
    }

    /// Converts the weak reference to an owned green element.
    ///
    /// This method creates a strong reference to the green element data,
    /// which can be stored long-term and will participate in reference
    /// counting. Useful when you need to hold onto green tree data
    /// beyond the lifetime of the current tree navigation.
    ///
    /// ## Safety
    ///
    /// Like `as_deref`, this method uses unsafe pointer operations but
    /// is safe due to red-green tree invariants. The green data must
    /// be alive when this method is called.
    ///
    /// ## Performance Impact
    ///
    /// This method increments reference counts, so it's more expensive
    /// than `as_deref`. Use sparingly when you actually need owned
    /// references.
    ///
    /// ## Returns
    ///
    /// An owned green element that participates in reference counting
    /// and can be stored independently of the original tree structure.
    #[allow(dead_code)] // Used for tree ownership management, may not be called directly yet
    pub(crate) fn to_owned(&self) -> GreenElement {
        match self {
            WeakGreenElement::Node { ptr } => {
                GreenElement::Node(unsafe { ptr.as_ref().to_owned() })
            }
            WeakGreenElement::Token { ptr } => {
                GreenElement::Token(unsafe { ptr.as_ref().to_owned() })
            }
        }
    }
}
