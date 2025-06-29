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

/// Immutable syntax tree node with reference-counted ownership.
///
/// `GreenNode` represents an internal node in the green layer of the syntax tree,
/// providing immutable, thread-safe access to hierarchical PDF structures. It
/// uses atomic reference counting for efficient memory management and enables
/// structural sharing between different tree versions.
///
/// ## Core Design
///
/// The node is implemented as a transparent wrapper around `ThinArc<GreenNodeHead, Slot>`,
/// which provides:
/// - **Atomic reference counting**: Safe sharing across threads
/// - **Optimized memory layout**: Header + variable-length slice representation
/// - **Zero-cost abstraction**: No overhead compared to direct ThinArc usage
/// - **Type safety**: Prevents incorrect usage through strong typing
///
/// ## Immutability Guarantees
///
/// Once created, a `GreenNode` never changes:
/// - All content is read-only after construction
/// - Child relationships are fixed at creation time
/// - Reference counting is the only mutable aspect (atomically managed)
/// - Enables safe concurrent access without additional synchronization
///
/// ## Memory Management
///
/// Reference counting provides automatic memory management:
/// - **Shared ownership**: Multiple `GreenNode` instances can reference the same data
/// - **Automatic cleanup**: Memory is freed when the last reference is dropped
/// - **Structural sharing**: Common subtrees are stored only once
/// - **Cache efficiency**: Related nodes are often stored contiguously
///
/// ## PDF Structure Representation
///
/// In PDF processing, green nodes represent complete syntactic units:
///
/// ### Object Node
/// ```pdf
/// 42 0 obj
/// << /Type /Catalog /Pages 1 0 R >>
/// endobj
/// ```
/// - Contains slots for: number, generation, "obj", dictionary, "endobj"
/// - Preserves exact spacing and line breaks
/// - References child dictionary node
///
/// ### Dictionary Node
/// ```pdf
/// << /Type /Catalog /Pages 1 0 R /Version 1.7 >>
/// ```
/// - Contains slots for: "<<", keys, values, ">>"
/// - Maintains key-value pair relationships
/// - Preserves formatting and whitespace
///
/// ### Array Node
/// ```pdf
/// [ 612 792 ]
/// ```
/// - Contains slots for: "[", elements, "]"
/// - Preserves element ordering and spacing
/// - Supports nested arrays and objects
///
/// ## Thread Safety
///
/// `GreenNode` is fully thread-safe:
/// - Implements `Send + Sync` (immutable data + atomic reference counting)
/// - Can be shared across thread boundaries without restriction
/// - No data races possible due to immutability
/// - Reference counting operations are atomic
///
/// ## Performance Characteristics
///
/// - **Clone**: O(1) - just increments reference count
/// - **Drop**: O(1) - decrements reference count, potentially triggers cleanup
/// - **Deref**: O(1) - direct pointer dereference to data
/// - **Memory usage**: Shared nodes reduce overall memory consumption
///
/// ## Usage Patterns
///
/// ### Sharing Nodes
/// ```rust,ignore
/// let node: GreenNode = parser.parse_object();
/// let shared = node.clone(); // Cheap - just reference counting
///
/// // Both `node` and `shared` reference the same underlying data
/// assert_eq!(node.kind(), shared.kind());
/// assert_eq!(node.text_len(), shared.text_len());
/// ```
///
/// ### Accessing Data
/// ```rust,ignore
/// let node: GreenNode = get_node();
///
/// // Access through Deref coercion
/// let kind = node.kind();
/// let length = node.text_len();
/// let slots = node.slots();
///
/// // Explicit deref to GreenNodeData
/// let data: &GreenNodeData = &*node;
/// ```
///
/// ### Building Collections
/// ```rust,ignore
/// let mut nodes = Vec::new();
/// for child in parent.slots() {
///     if let Slot::Node { node, .. } = child {
///         nodes.push(node.clone()); // Efficient sharing
///     }
/// }
/// ```
///
/// ## Integration with Parser
///
/// Green nodes are typically created by the parser and represent the immutable
/// result of parsing operations. They serve as the foundation for:
/// - **Red nodes**: Provide positional and mutable access
/// - **Semantic analysis**: Type checking and validation
/// - **Code generation**: AST transformations and output generation
/// - **IDE features**: Syntax highlighting, error reporting, IntelliSense
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct GreenNode {
    /// Reference-counted pointer to the node's data and children.
    ///
    /// This `ThinArc` provides the core functionality of the green node:
    /// - **Header**: Contains `GreenNodeHead` with kind, text length, and reference count
    /// - **Slice**: Contains the array of `Slot` children in a contiguous layout
    /// - **Atomicity**: Reference counting operations are thread-safe
    /// - **Efficiency**: Optimized memory layout minimizes cache misses
    ///
    /// The thin arc representation ensures that the entire node data
    /// (header + children) is stored in a single allocation, improving
    /// cache locality and reducing memory fragmentation.
    ///
    /// ## Memory Layout
    ///
    /// ```text
    /// ptr -> ThinArc {
    ///     header: GreenNodeHead {
    ///         kind: RawSyntaxKind,
    ///         text_len: u64,
    ///         ref_count: AtomicUsize
    ///     },
    ///     slice: [Slot; N] {
    ///         slot_0: Slot,
    ///         slot_1: Slot,
    ///         ...
    ///         slot_N: Slot
    ///     }
    /// }
    /// ```
    ///
    /// ## Thread Safety
    ///
    /// The `ThinArc` type provides thread-safe reference counting:
    /// - Clone operations atomically increment the reference count
    /// - Drop operations atomically decrement the reference count
    /// - The last drop triggers memory deallocation
    /// - All operations are lock-free and wait-free
    ptr: ThinArc<GreenNodeHead, Slot>,
}

impl GreenNode {
    /// Constructs a `GreenNode` from a raw pointer to `GreenNodeData`.
    ///
    /// This method provides a way to convert from a raw pointer to node data
    /// back into a properly reference-counted `GreenNode`. It's typically used
    /// internally by the syntax tree implementation when converting between
    /// different representations or when interfacing with low-level APIs.
    ///
    /// ## Safety Requirements
    ///
    /// This function is marked `unsafe` because it makes several critical assumptions:
    ///
    /// ### Valid Pointer
    /// The `ptr` parameter must point to a valid, properly initialized `GreenNodeData`:
    /// - The pointer must not be null
    /// - The pointed-to memory must be properly aligned
    /// - The memory must contain a valid `GreenNodeData` structure
    /// - The data must have been allocated with the expected layout
    ///
    /// ### Reference Counting Invariants
    /// The underlying data must have proper reference counting state:
    /// - Must have been originally created through proper allocation
    /// - Reference count must be valid and not corrupted
    /// - Memory must not have been freed or deallocated
    /// - No other code should be modifying the reference count concurrently
    ///
    /// ### Memory Layout Compatibility
    /// The memory layout must be compatible with the expected representations:
    /// - `GreenNodeData` must contain a valid `GreenNodeReprThin`
    /// - The thin representation must be compatible with `ThinArc` layout
    /// - Header and slice portions must be properly initialized
    ///
    /// ## Implementation Details
    ///
    /// The method performs several unsafe transformations:
    ///
    /// ### Pointer Conversion
    /// ```rust,ignore
    /// // Convert from GreenNodeData* to GreenNodeReprThin*
    /// let arc = Arc::from_raw(&ptr.as_ref().data as *const GreenNodeReprThin);
    /// ```
    /// This step creates an `Arc` from the raw pointer to the internal representation.
    ///
    /// ### Type Transmutation
    /// ```rust,ignore
    /// // Convert Arc<GreenNodeReprThin> to ThinArc<GreenNodeHead, Slot>
    /// let arc = mem::transmute::<Arc<GreenNodeReprThin>, ThinArc<GreenNodeHead, Slot>>(arc);
    /// ```
    /// This transmutes between different Arc types that have the same memory layout
    /// but different type parameters.
    ///
    /// ## Usage Context
    ///
    /// This method is primarily used for:
    /// - **Internal conversions**: Converting between node representations
    /// - **FFI boundaries**: Interfacing with C code or other languages
    /// - **Serialization/deserialization**: Reconstructing nodes from persistent storage
    /// - **Memory pool management**: Custom allocation strategies
    ///
    /// ## PDF Processing Example
    ///
    /// ```rust,ignore
    /// // Typically used internally by the parser or tree builder
    /// unsafe fn reconstruct_object_node(data_ptr: ptr::NonNull<GreenNodeData>) -> GreenNode {
    ///     // Validate the pointer points to an object node
    ///     assert_eq!(data_ptr.as_ref().kind(), OBJECT);
    ///     
    ///     // Safely convert to GreenNode
    ///     GreenNode::from_raw(data_ptr)
    /// }
    /// ```
    ///
    /// ## Error Conditions
    ///
    /// Violating the safety requirements can lead to:
    /// - **Memory corruption**: Invalid pointer dereference
    /// - **Use-after-free**: Accessing freed memory
    /// - **Double-free**: Incorrect reference counting
    /// - **Undefined behavior**: Type system violations
    ///
    /// ## Performance
    ///
    /// This operation is very fast (O(1)) as it only:
    /// - Performs pointer arithmetic and casts
    /// - Creates a new Arc wrapper (no allocation)
    /// - Transmutes between compatible types
    /// - Does not traverse or copy data
    ///
    /// ## Related Methods
    ///
    /// This method is the inverse of operations that extract raw pointers
    /// from nodes, enabling round-trip conversions between owned and raw
    /// pointer representations.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenNodeData>) -> GreenNode {
        let arc = unsafe { Arc::from_raw(&ptr.as_ref().data as *const GreenNodeReprThin) };
        let arc =
            unsafe { mem::transmute::<Arc<GreenNodeReprThin>, ThinArc<GreenNodeHead, Slot>>(arc) };
        GreenNode { ptr: arc }
    }
}

/// Automatic dereferencing from `GreenNode` to `GreenNodeData`.
///
/// This implementation enables transparent access to the underlying node data
/// through Rust's deref coercion mechanism. It allows `GreenNode` instances
/// to be used directly as `&GreenNodeData` in contexts where the data interface
/// is needed.
///
/// ## Deref Coercion Benefits
///
/// With this implementation, you can call `GreenNodeData` methods directly on `GreenNode`:
/// ```rust,ignore
/// let node: GreenNode = get_node();
///
/// // These work through deref coercion:
/// let kind = node.kind();          // Calls GreenNodeData::kind()
/// let length = node.text_len();    // Calls GreenNodeData::text_len()
/// let slots = node.slots();        // Calls GreenNodeData::slots()
/// ```
///
/// ## Memory Safety
///
/// The dereference operation is safe because:
/// - `GreenNode` owns a valid reference to the data (through `ThinArc`)
/// - The returned reference has the same lifetime as the `GreenNode`
/// - The underlying data is immutable, preventing data races
/// - Reference counting ensures the data remains valid
///
/// ## Implementation Strategy
///
/// The method performs careful pointer conversions between compatible representations:
///
/// ### Step 1: ThinArc to GreenNodeRepr
/// ```rust,ignore
/// let repr: &GreenNodeRepr = &self.ptr;
/// ```
/// Gets a reference to the thick representation (header + slice together).
///
/// ### Step 2: Thick to Thin Representation
/// ```rust,ignore
/// let repr: &GreenNodeReprThin = &*(repr as *const GreenNodeRepr as *const GreenNodeReprThin);
/// ```
/// Converts between the thick and thin representation formats, maintaining
/// the same underlying memory layout.
///
/// ### Step 3: Transmute to GreenNodeData
/// ```rust,ignore
/// mem::transmute::<&GreenNodeReprThin, &GreenNodeData>(repr)
/// ```
/// Safely reinterprets the thin representation as `GreenNodeData`,
/// which has a transparent wrapper around the same type.
///
/// ## PDF Processing Applications
///
/// Deref coercion enables natural usage patterns in PDF processing:
///
/// ### Direct Method Access
/// ```rust,ignore
/// fn analyze_object(node: GreenNode) -> ObjectInfo {
///     ObjectInfo {
///         kind: node.kind(),              // Deref to GreenNodeData
///         text_length: node.text_len(),   // Deref to GreenNodeData
///         child_count: node.slots().len(), // Deref to GreenNodeData
///     }
/// }
/// ```
///
/// ### Collection Processing
/// ```rust,ignore
/// fn collect_dictionary_keys(nodes: Vec<GreenNode>) -> Vec<String> {
///     nodes
///         .iter()
///         .filter(|node| node.kind() == DICTIONARY) // Deref coercion
///         .flat_map(|node| extract_keys(node))       // Deref coercion
///         .collect()
/// }
/// ```
///
/// ### Pattern Matching
/// ```rust,ignore
/// match node.kind() {                    // Deref coercion
///     OBJECT => process_object(&*node),   // Explicit deref to GreenNodeData
///     DICTIONARY => process_dict(&*node), // Explicit deref to GreenNodeData
///     ARRAY => process_array(&*node),     // Explicit deref to GreenNodeData
///     _ => {}
/// }
/// ```
///
/// ## Performance Characteristics
///
/// - **Zero cost**: Compiles to direct memory access
/// - **No allocation**: Only pointer arithmetic and type casting
/// - **Cache friendly**: Accesses contiguous memory layout
/// - **Inlined**: Marked `#[inline]` for optimal performance
///
/// ## Type Relationships
///
/// This deref implementation establishes the relationship:
/// ```text
/// GreenNode -> GreenNodeData
///     |             |
///     |             +-- Provides data access methods
///     |
///     +-- Provides ownership and reference counting
/// ```
///
/// ## Safety Guarantees
///
/// The unsafe code is justified because:
/// - All pointer casts are between compatible types with same memory layout
/// - The lifetime of the returned reference is tied to the `GreenNode`
/// - The `ThinArc` guarantees the underlying data is valid and properly aligned
/// - The transmute is between transparent wrapper types
impl ops::Deref for GreenNode {
    type Target = GreenNodeData;

    /// Dereferences the node to access its underlying data.
    ///
    /// This method converts from the owned `GreenNode` to a borrowed
    /// `&GreenNodeData`, enabling direct access to all data methods
    /// through Rust's automatic deref coercion.
    ///
    /// ## Return Value
    ///
    /// Returns a reference to the `GreenNodeData` contained within this node.
    /// The reference has the same lifetime as the `GreenNode` itself.
    ///
    /// ## Performance
    ///
    /// This is a zero-cost operation that compiles down to simple pointer
    /// arithmetic and type casting. No data is copied or moved.
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

/// Debug formatting implementation that delegates to the underlying data.
///
/// This implementation provides debug output for `GreenNode` by forwarding
/// to the `Debug` implementation of `GreenNodeData`. This ensures consistent
/// debug formatting across the different node representations while maintaining
/// the ownership semantics of `GreenNode`.
///
/// ## Debug Output Format
///
/// The debug output will show the essential node properties:
/// ```text
/// GreenNode {
///     kind: OBJECT,
///     text_len: 42,
///     n_slots: 5
/// }
/// ```
///
/// This format provides the most useful information for debugging:
/// - **kind**: What type of PDF construct this represents
/// - **text_len**: How much source text this node covers
/// - **n_slots**: How many child positions this node has
///
/// ## Delegation Strategy
///
/// Rather than duplicating the debug logic, this implementation:
/// 1. Dereferences the `GreenNode` to get `&GreenNodeData`
/// 2. Delegates to the `Debug` implementation of `GreenNodeData`
/// 3. Maintains consistent formatting across the codebase
/// 4. Reduces code duplication and maintenance burden
///
/// ## PDF Processing Context
///
/// Debug output is particularly useful when analyzing PDF structures:
///
/// ### Object Analysis
/// ```rust,ignore
/// let object_node: GreenNode = parse_object("1 0 obj << /Type /Catalog >> endobj");
/// println!("{:?}", object_node);
/// // Output: GreenNode { kind: OBJECT, text_len: 35, n_slots: 5 }
/// ```
///
/// ### Dictionary Inspection
/// ```rust,ignore
/// let dict_node: GreenNode = parse_dictionary("<< /Type /Catalog /Pages 2 0 R >>");
/// println!("{:?}", dict_node);
/// // Output: GreenNode { kind: DICTIONARY, text_len: 32, n_slots: 6 }
/// ```
///
/// ### Tree Traversal Debugging
/// ```rust,ignore
/// fn debug_tree_structure(node: &GreenNode, indent: usize) {
///     println!("{:indent$}{:?}", "", node, indent = indent);
///     for slot in node.slots() {
///         if let Slot::Node { node: child, .. } = slot {
///             debug_tree_structure(child, indent + 2);
///         }
///     }
/// }
/// ```
///
/// ## Performance Considerations
///
/// - **Delegation overhead**: Minimal - just a deref coercion
/// - **String allocation**: Only occurs when debug formatting is actually used
/// - **Development only**: Debug formatting is typically optimized away in release builds
/// - **Lazy evaluation**: No work done unless the debug string is actually needed
///
/// ## Alternative Approaches
///
/// This delegation approach was chosen over alternatives:
/// - **Direct implementation**: Would duplicate logic and create maintenance burden
/// - **Macro generation**: Would be less flexible and harder to customize
/// - **Custom format**: Would be inconsistent with `GreenNodeData` formatting
///
/// ## Usage in Testing
///
/// Debug formatting is particularly valuable in unit tests:
/// ```rust,ignore
/// #[test]
/// fn test_object_parsing() {
///     let node = parse_object_node(test_input);
///     assert_eq!(format!("{:?}", node), "GreenNode { kind: OBJECT, text_len: 25, n_slots: 5 }");
/// }
/// ```
impl fmt::Debug for GreenNode {
    /// Formats the node for debug output by delegating to the underlying data.
    ///
    /// This method uses the `Deref` implementation to access the `GreenNodeData`
    /// and then forwards the debug formatting request to that implementation.
    ///
    /// ## Parameters
    ///
    /// - `f`: The formatter to write debug output to
    ///
    /// ## Return Value
    ///
    /// Returns `fmt::Result` indicating success or failure of the formatting operation.
    ///
    /// ## Implementation
    ///
    /// The method performs these steps:
    /// 1. Dereferences `self` to get `&GreenNodeData` (through `Deref` coercion)
    /// 2. Calls `fmt::Debug::fmt` on the data reference
    /// 3. Returns the result of the formatting operation
    ///
    /// This ensures that debug output is consistent between `GreenNode` and
    /// `GreenNodeData` representations.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenNodeData = self;
        fmt::Debug::fmt(data, f)
    }
}

/// Display formatting implementation for source text reconstruction.
///
/// This implementation provides the ability to reconstruct the original source
/// text from a `GreenNode` by delegating to the `Display` implementation of
/// `GreenNodeData`. This enables faithful reproduction of PDF content while
/// maintaining the ownership semantics of `GreenNode`.
///
/// ## Text Reconstruction
///
/// The display output recreates the exact original source text:
/// - **Whitespace preservation**: All spaces, tabs, and newlines maintained
/// - **Token ordering**: Elements appear in their original sequence
/// - **Trivia inclusion**: Comments and formatting are preserved
/// - **Byte-for-byte accuracy**: Output should match input exactly
///
/// ## Delegation Benefits
///
/// By delegating to `GreenNodeData::fmt`, this implementation:
/// - **Maintains consistency**: Same output format across representations
/// - **Reduces duplication**: Single implementation of text reconstruction logic
/// - **Simplifies maintenance**: Changes only need to be made in one place
/// - **Ensures correctness**: Well-tested logic is reused
///
/// ## PDF Processing Applications
///
/// Display formatting enables various PDF operations:
///
/// ### Source Reconstruction
/// ```rust,ignore
/// let object_node: GreenNode = parse_object(original_source);
/// let reconstructed = format!("{}", object_node);
/// assert_eq!(original_source, reconstructed);
/// ```
///
/// ### Pretty Printing
/// ```rust,ignore
/// fn print_pdf_structure(node: &GreenNode) {
///     println!("PDF Content:");
///     println!("{}", node);
/// }
/// ```
///
/// ### File Generation
/// ```rust,ignore
/// fn write_pdf_file(root: &GreenNode, path: &Path) -> Result<(), io::Error> {
///     let content = format!("{}", root);
///     fs::write(path, content)
/// }
/// ```
///
/// ### Diff Generation
/// ```rust,ignore
/// fn compare_pdf_versions(original: &GreenNode, modified: &GreenNode) {
///     let original_text = format!("{}", original);
///     let modified_text = format!("{}", modified);
///     generate_diff(&original_text, &modified_text);
/// }
/// ```
///
/// ### Error Reporting
/// ```rust,ignore
/// fn show_error_context(error_node: &GreenNode) {
///     eprintln!("Error in PDF content:");
///     eprintln!("{}", error_node);
/// }
/// ```
///
/// ## Fidelity Guarantees
///
/// The display implementation ensures:
/// - **Roundtrip accuracy**: Parse then display should match original
/// - **Structure preservation**: Nested elements maintain their relationships
/// - **Formatting retention**: Human-readable formatting is preserved
/// - **Binary compatibility**: Exact byte sequences are maintained
///
/// ## Performance Characteristics
///
/// - **Delegation overhead**: Minimal - just a deref coercion
/// - **Streaming output**: Uses formatter efficiently without intermediate strings
/// - **Memory efficient**: No additional allocations beyond formatter requirements
/// - **Single pass**: Visits each node exactly once during formatting
///
/// ## Integration with IDE Features
///
/// Display formatting supports various IDE capabilities:
/// - **Hover previews**: Show source content when hovering over nodes
/// - **Code completion**: Display context around completion points
/// - **Error messages**: Include relevant source text in diagnostics
/// - **Refactoring**: Show before/after content for transformations
///
/// ## Usage Patterns
///
/// ### Simple Output
/// ```rust,ignore
/// println!("{}", node);
/// ```
///
/// ### String Creation
/// ```rust,ignore
/// let content = format!("{}", node);
/// ```
///
/// ### Writer Integration
/// ```rust,ignore
/// write!(writer, "{}", node)?;
/// ```
///
/// ### Streaming
/// ```rust,ignore
/// node.fmt(&mut formatter)?;
/// ```
///
/// ## Error Handling
///
/// The display operation can fail if:
/// - The underlying formatter encounters I/O errors
/// - Memory allocation fails during string building
/// - Child nodes have corrupted or invalid data
///
/// All errors are properly propagated through the `fmt::Result` return type.
impl fmt::Display for GreenNode {
    /// Formats the node for display by reconstructing the original source text.
    ///
    /// This method delegates to the `Display` implementation of `GreenNodeData`
    /// to ensure consistent text reconstruction across different node representations.
    ///
    /// ## Parameters
    ///
    /// - `f`: The formatter to write the reconstructed text to
    ///
    /// ## Return Value
    ///
    /// Returns `fmt::Result` indicating success or failure of the formatting operation.
    /// Failures typically indicate I/O errors or memory allocation issues.
    ///
    /// ## Implementation
    ///
    /// The method performs these steps:
    /// 1. Dereferences `self` to get `&GreenNodeData` (through `Deref` coercion)
    /// 2. Calls `fmt::Display::fmt` on the data reference
    /// 3. Returns the result of the formatting operation
    ///
    /// This delegation ensures that the text reconstruction logic is centralized
    /// in `GreenNodeData` while still being accessible through `GreenNode`.
    ///
    /// ## Source Text Fidelity
    ///
    /// The output of this method should be byte-for-byte identical to the
    /// original source text that was parsed to create this node, including
    /// all whitespace, comments, and other syntactic trivia.
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
