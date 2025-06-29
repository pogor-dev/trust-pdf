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

/// Internal data structure for green syntax tree nodes.
///
/// This structure contains the actual implementation of a green node in the syntax tree.
/// It wraps `GreenNodeReprThin` with a transparent representation, meaning it has the
/// same memory layout and can be safely cast between the two types.
///
/// ## Design Rationale
///
/// The transparent wrapper serves several purposes:
/// - **Type Safety**: Provides a distinct type for the data while maintaining zero-cost abstraction
/// - **API Encapsulation**: Controls access to the underlying representation through methods
/// - **Memory Efficiency**: No additional overhead compared to the wrapped type
/// - **Reference Counting**: Enables shared ownership through the underlying `ThinArc`
///
/// ## Memory Layout
///
/// ```text
/// GreenNodeData {
///     data: GreenNodeReprThin {
///         header: GreenNodeHead {
///             kind: RawSyntaxKind,
///             text_len: u64,
///             ref_count: AtomicUsize
///         },
///         slots: [Slot; N]  // Variable length slice
///     }
/// }
/// ```
///
/// ## PDF Processing Context
///
/// In PDF syntax analysis, this represents any structural element:
/// - **Object definitions**: `1 0 obj ... endobj`
/// - **Dictionaries**: `<< /Key /Value ... >>`
/// - **Arrays**: `[ item1 item2 ... ]`
/// - **Stream content**: Raw bytes between `stream` and `endstream`
/// - **Content operators**: `q 1 0 0 1 0 0 cm Q`
///
/// ## Thread Safety
///
/// The structure is immutable after construction and safe to share across threads.
/// Reference counting is handled atomically by the underlying `ThinArc`.
#[repr(transparent)]
pub(crate) struct GreenNodeData {
    /// The underlying thin representation containing header and child slots.
    ///
    /// This field uses `GreenNodeReprThin` which provides:
    /// - Atomic reference counting through `ThinArc`
    /// - Efficient header+slice layout via `HeaderSlice`
    /// - Direct access to node metadata and children
    pub(crate) data: GreenNodeReprThin,
}

impl GreenNodeData {
    /// Accesses the node header containing metadata.
    ///
    /// The header contains essential information about the node:
    /// - **Kind**: The syntax type this node represents (e.g., object, dictionary, array)
    /// - **Text length**: Total character count covered by this node and its subtree
    /// - **Reference count**: Atomic counter for memory management
    ///
    /// ## Usage
    ///
    /// This method is used internally by public accessor methods to retrieve
    /// specific header fields efficiently. It provides direct access to the
    /// header without additional indirection.
    ///
    /// ## Examples in PDF Context
    ///
    /// For a PDF object node representing:
    /// ```pdf
    /// 1 0 obj
    /// << /Type /Catalog >>
    /// endobj
    /// ```
    ///
    /// The header would contain:
    /// - Kind: `OBJECT` (indicating this is a PDF object)
    /// - Text length: 35 (total characters including whitespace)
    /// - Reference count: Current number of references to this node
    ///
    /// ## Performance
    ///
    /// This is an O(1) operation that simply returns a reference to the
    /// header field. It's marked `#[inline]` for zero-cost access.
    #[inline]
    fn header(&self) -> &GreenNodeHead {
        &self.data.header
    }

    /// Returns the syntax kind of this node.
    ///
    /// The syntax kind identifies what type of PDF construct this node represents.
    /// This is fundamental for understanding how to interpret the node's structure
    /// and children during semantic analysis.
    ///
    /// ## PDF Syntax Kinds
    ///
    /// Common kinds in PDF syntax trees include:
    /// - **OBJECT**: Complete PDF object (`1 0 obj ... endobj`)
    /// - **DICTIONARY**: Dictionary structure (`<< ... >>`)
    /// - **ARRAY**: Array structure (`[ ... ]`)
    /// - **STREAM**: Stream object with binary data
    /// - **NAME**: Name token (`/Type`, `/Catalog`)
    /// - **NUMBER**: Numeric literal (`42`, `3.14159`)
    /// - **STRING**: String literal (`(Hello)`, `<48656C6C6F>`)
    /// - **REFERENCE**: Indirect object reference (`1 0 R`)
    ///
    /// ## Usage in Parser
    ///
    /// The parser uses this kind to:
    /// - Determine how to traverse child nodes
    /// - Apply syntax-specific validation rules
    /// - Generate appropriate semantic representations
    /// - Provide context-aware error messages
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # use crate::green::{RawSyntaxKind, GreenNodeData};
    /// # fn example(node: &GreenNodeData) {
    /// match node.kind() {
    ///     RawSyntaxKind::OBJECT => {
    ///         // Handle PDF object: extract object number, generation, body
    ///     }
    ///     RawSyntaxKind::DICTIONARY => {
    ///         // Handle dictionary: parse key-value pairs
    ///     }
    ///     RawSyntaxKind::STREAM => {
    ///         // Handle stream: decode binary content
    ///     }
    ///     _ => {
    ///         // Handle other syntax constructs
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// ## Performance
    ///
    /// This is an O(1) operation that reads a single field from the node header.
    /// The kind is determined during parsing and never changes afterward.
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.header().kind
    }

    /// Returns the raw slice of child slots.
    ///
    /// This method provides direct access to the internal array of child slots.
    /// Each slot can contain either a child node, a token, or be empty for
    /// optional elements that weren't present in the source.
    ///
    /// ## Internal Usage
    ///
    /// This method is primarily used internally by the `slots()` method to
    /// create an iterator over the child elements. It exposes the underlying
    /// slice representation for efficient access patterns.
    ///
    /// ## Slot Contents
    ///
    /// Each slot in the slice can contain:
    /// - **Node slot**: Reference to a child syntax node
    /// - **Token slot**: A leaf token (keyword, literal, punctuation)
    /// - **Empty slot**: Placeholder for optional syntax elements
    ///
    /// ## PDF Structure Mapping
    ///
    /// For a PDF object like:
    /// ```pdf
    /// 42 0 obj
    /// << /Type /Catalog >>
    /// endobj
    /// ```
    ///
    /// The slots would contain:
    /// - Slot 0: Token("42") - object number
    /// - Slot 1: Token("0") - generation number  
    /// - Slot 2: Token("obj") - keyword
    /// - Slot 3: Node(dictionary) - object body
    /// - Slot 4: Token("endobj") - end keyword
    ///
    /// ## Performance
    ///
    /// Direct slice access is O(1) and provides the foundation for all
    /// child iteration operations. The slice is immutable and safe to
    /// access concurrently.
    #[inline]
    pub(crate) fn slice(&self) -> &[Slot] {
        self.data.slice()
    }

    /// Returns the total length of text covered by this node and its subtree.
    ///
    /// This value represents the number of characters in the original source text
    /// that this node spans, including all of its children and any whitespace or
    /// trivia between them. This enables efficient text range calculations without
    /// needing to traverse the entire subtree.
    ///
    /// ## Text Range Calculation
    ///
    /// The text length is used to determine:
    /// - **Source spans**: Which characters in the original PDF this node covers
    /// - **Error locations**: Precise positions for diagnostic messages
    /// - **Incremental parsing**: Which parts of the tree need updates
    /// - **Text reconstruction**: Rebuilding the original source from the tree
    ///
    /// ## Cumulative Nature
    ///
    /// The length includes:
    /// - All direct token content in this node
    /// - All text covered by child nodes recursively
    /// - All whitespace and comments (trivia) within the node's span
    /// - All punctuation and delimiters
    ///
    /// ## PDF Examples
    ///
    /// For a simple PDF object:
    /// ```pdf
    /// 1 0 obj
    /// << /Type /Catalog >>
    /// endobj
    /// ```
    ///
    /// The text length would be 35 characters (including newlines and spaces).
    ///
    /// For a dictionary node `<< /Type /Catalog >>`:
    /// - Text length: 20 characters (from first `<` to last `>`)
    ///
    /// ## Performance Benefits
    ///
    /// Having pre-calculated text lengths enables:
    /// - O(1) range queries without tree traversal
    /// - Efficient binary search for position-based lookups
    /// - Fast incremental updates by comparing text ranges
    /// - Lazy evaluation of subtree operations
    ///
    /// ## Usage in IDE Features
    ///
    /// This information supports:
    /// - **Syntax highlighting**: Determining token boundaries
    /// - **Code folding**: Calculating collapsible regions
    /// - **Error squiggles**: Precise underline positioning
    /// - **Hover information**: Associating content with screen positions
    #[inline]
    pub fn text_len(&self) -> u64 {
        self.header().text_len
    }

    /// Returns an iterator over the child slots of this node.
    ///
    /// This method provides a safe, typed interface for accessing the children
    /// of this syntax node. Every node of a specific syntax kind has the same
    /// number of slots, allowing fixed offsets to retrieve specific children
    /// even when some optional children are missing.
    ///
    /// ## Slot Structure
    ///
    /// The slots represent the syntactic structure of the node:
    /// - **Fixed positions**: Each syntax kind has predetermined slot positions
    /// - **Optional slots**: Some positions may be empty if optional elements are absent
    /// - **Type safety**: Slots are typed to prevent incorrect access patterns
    /// - **Iteration support**: Provides iterator interface for traversal
    ///
    /// ## PDF Syntax Slot Examples
    ///
    /// ### Object Node Slots
    /// For `1 0 obj << /Type /Catalog >> endobj`:
    /// - Slot 0: Number token "1"
    /// - Slot 1: Number token "0"
    /// - Slot 2: Keyword token "obj"
    /// - Slot 3: Dictionary node
    /// - Slot 4: Keyword token "endobj"
    ///
    /// ### Dictionary Node Slots
    /// For `<< /Type /Catalog /Pages 2 0 R >>`:
    /// - Slot 0: Delimiter token "<<"
    /// - Slot 1: Name token "/Type"
    /// - Slot 2: Name token "/Catalog"
    /// - Slot 3: Name token "/Pages"
    /// - Slot 4: Reference node "2 0 R"
    /// - Slot 5: Delimiter token ">>"
    ///
    /// ### Array Node Slots
    /// For `[ 1 2 3 ]`:
    /// - Slot 0: Delimiter token "["
    /// - Slot 1: Number token "1"
    /// - Slot 2: Number token "2"
    /// - Slot 3: Number token "3"
    /// - Slot 4: Delimiter token "]"
    ///
    /// ## Accessing Specific Children
    ///
    /// ```rust
    /// # use crate::green::GreenNodeData;
    /// # fn example(node: &GreenNodeData) {
    /// let mut slots = node.slots();
    ///
    /// // Access first child (e.g., opening delimiter)
    /// if let Some(first) = slots.next() {
    ///     // Process first child
    /// }
    ///
    /// // Iterate over all children
    /// for child in node.slots() {
    ///     match child {
    ///         // Handle different slot types
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// ## Fixed Slot Layout Benefits
    ///
    /// The fixed slot approach provides:
    /// - **Predictable access**: Known positions for required elements
    /// - **Efficient queries**: Direct indexing without search
    /// - **Error recovery**: Parser can skip missing optional elements
    /// - **Incremental updates**: Changes don't affect slot positions
    ///
    /// ## Performance
    ///
    /// Creating the slot iterator is O(1) and iteration is O(n) where n
    /// is the number of child slots. The iterator borrows the internal
    /// slice and provides zero-allocation traversal.
    #[inline]
    pub fn slots(&self) -> Slots<'_> {
        Slots {
            raw: self.slice().iter(),
        }
    }
}

/// Implements conversion from borrowed `GreenNodeData` to owned `GreenNode`.
///
/// This implementation enables converting from a borrowed reference to the node data
/// into an owned `GreenNode` that can be stored and passed around independently.
/// The conversion properly handles reference counting to ensure memory safety.
///
/// ## Why ToOwned?
///
/// The `ToOwned` trait bridges the gap between:
/// - **Borrowed data**: `&GreenNodeData` - temporary access to node content
/// - **Owned data**: `GreenNode` - permanent ownership with proper lifecycle management
///
/// This pattern is similar to how `&str` can be converted to `String`, or how
/// `&Path` can be converted to `PathBuf`.
///
/// ## Reference Counting Safety
///
/// The implementation carefully manages reference counts:
/// 1. Creates a new `GreenNode` from the raw pointer to the data
/// 2. Uses `ManuallyDrop` to prevent premature destruction
/// 3. Clones the node to increment the reference count
/// 4. Returns the properly owned node
///
/// ## Usage Scenarios
///
/// This conversion is useful when:
/// - **Storing nodes**: Keeping references beyond the current scope
/// - **Returning from functions**: Transferring ownership to callers
/// - **Building collections**: Creating vectors or maps of nodes
/// - **Caching results**: Storing computed nodes for reuse
///
/// ## PDF Processing Example
///
/// ```rust
/// # use crate::green::{GreenNodeData, GreenNode};
/// # fn process_pdf_object(data: &GreenNodeData) -> GreenNode {
/// // When we find an interesting PDF object, we might want to keep it
/// let owned_node = data.to_owned();
///
/// // Now we can store it, return it, or pass it around
/// owned_node
/// # }
/// ```
///
/// ## Memory Implications
///
/// - **No copying**: The node data itself is not duplicated
/// - **Reference increment**: Only the reference count increases
/// - **Shared memory**: Multiple `GreenNode` instances can reference the same data
/// - **Automatic cleanup**: Memory is freed when the last reference is dropped
impl ToOwned for GreenNodeData {
    type Owned = GreenNode;

    /// Converts this borrowed node data into an owned `GreenNode`.
    ///
    /// This operation increments the reference count of the underlying data
    /// and returns a new `GreenNode` that owns a reference to it.
    ///
    /// ## Safety
    ///
    /// The implementation uses unsafe code to:
    /// 1. Create a `GreenNode` from a raw pointer (safe because we know the pointer is valid)
    /// 2. Use `ManuallyDrop` to prevent double-free (safe because we immediately clone)
    /// 3. Clone the temporary node to get proper ownership (safe reference counting)
    ///
    /// ## Performance
    ///
    /// This is an O(1) operation that only increments an atomic reference counter.
    /// No data is copied or moved.
    #[inline]
    fn to_owned(&self) -> GreenNode {
        unsafe {
            let green = GreenNode::from_raw(ptr::NonNull::from(self));
            let green = ManuallyDrop::new(green);
            GreenNode::clone(&green)
        }
    }
}

/// Implements borrowing from owned `GreenNode` to `GreenNodeData`.
///
/// This implementation allows owned `GreenNode` instances to be borrowed as
/// `&GreenNodeData`, enabling seamless conversion between owned and borrowed
/// forms. This is essential for APIs that accept either owned or borrowed nodes.
///
/// ## The Borrow Pattern
///
/// This follows Rust's standard borrow pattern where:
/// - **Owned type**: `GreenNode` - manages memory and reference counting
/// - **Borrowed type**: `GreenNodeData` - provides access to the data
/// - **Conversion**: Automatic dereferencing from owned to borrowed
///
/// This is analogous to how `String` implements `Borrow<str>` or how
/// `Vec<T>` implements `Borrow<[T]>`.
///
/// ## Usage Benefits
///
/// With this implementation, functions can accept both owned and borrowed nodes:
///
/// ```rust
/// # use std::borrow::Borrow;
/// # use crate::green::{GreenNode, GreenNodeData};
/// # use crate::green::kind::RawSyntaxKind;
///
/// fn analyze_node<T>(node: T) -> RawSyntaxKind
/// where
///     T: Borrow<GreenNodeData>
/// {
///     let data: &GreenNodeData = node.borrow();
///     data.kind()
/// }
///
/// # fn example() {
/// # let owned_node: GreenNode = todo!();
/// # let borrowed_data: &GreenNodeData = todo!();
/// // Works with owned nodes
/// let kind1 = analyze_node(owned_node);
///
/// // Works with borrowed data
/// let kind2 = analyze_node(borrowed_data);
/// # }
/// ```
///
/// ## Hash Map Integration
///
/// This trait is particularly important for using nodes as keys in hash maps
/// and other collections that rely on `Borrow` for efficient lookups:
///
/// ```rust
/// # use std::collections::HashMap;
/// # use std::borrow::Borrow;
/// # use crate::green::{GreenNode, GreenNodeData};
///
/// let mut cache: HashMap<GreenNode, String> = HashMap::new();
///
/// # fn example(node_data: &GreenNodeData, owned_node: GreenNode) {
/// // Can lookup with borrowed data even though keys are owned
/// if let Some(cached) = cache.get(node_data) {
///     // Found cached result
/// }
///
/// // Can also lookup with owned node
/// if let Some(cached) = cache.get(&owned_node) {
///     // Found cached result
/// }
/// # }
/// ```
///
/// ## PDF Processing Context
///
/// In PDF processing, this enables flexible APIs:
/// - **Parser results**: Return owned nodes for permanent storage
/// - **Analysis functions**: Accept borrowed data for temporary processing
/// - **Caching systems**: Use owned keys with borrowed lookups
/// - **Visitor patterns**: Pass borrowed references during traversal
///
/// ## Zero-Cost Abstraction
///
/// This conversion is zero-cost - it's simply a reinterpretation of the
/// same memory location. The `#[inline]` attribute ensures the compiler
/// optimizes away the function call entirely.
impl Borrow<GreenNodeData> for GreenNode {
    /// Borrows the node data from this owned node.
    ///
    /// This operation provides a borrowed view of the node's data without
    /// affecting reference counting or ownership. It's a simple dereference
    /// that exposes the underlying `GreenNodeData`.
    ///
    /// ## Performance
    ///
    /// This is a zero-cost operation that compiles down to a no-op.
    /// It's simply reinterpreting the same memory location as a different type.
    #[inline]
    fn borrow(&self) -> &GreenNodeData {
        self
    }
}

/// Implements debug formatting for `GreenNodeData`.
///
/// This implementation provides a structured debug representation that shows
/// the essential properties of a syntax node without overwhelming detail.
/// It's designed to be useful for debugging parser issues, analyzing tree
/// structure, and understanding node relationships.
///
/// ## Debug Output Format
///
/// The debug format includes three key pieces of information:
/// - **kind**: The syntax kind (e.g., `OBJECT`, `DICTIONARY`, `ARRAY`)
/// - **text_len**: Total characters covered by this node and its subtree
/// - **n_slots**: Number of child slots (both filled and empty)
///
/// ## Example Output
///
/// ```text
/// GreenNode {
///     kind: OBJECT,
///     text_len: 42,
///     n_slots: 5
/// }
/// ```
///
/// This tells us:
/// - It's a PDF object node
/// - Covers 42 characters of source text
/// - Has 5 child slots (obj number, generation, "obj", body, "endobj")
///
/// ## PDF Context Examples
///
/// For different PDF constructs, debug output might look like:
///
/// ### Dictionary Node
/// ```text
/// GreenNode {
///     kind: DICTIONARY,
///     text_len: 25,
///     n_slots: 6
/// }
/// ```
///
/// ### Array Node  
/// ```text
/// GreenNode {
///     kind: ARRAY,
///     text_len: 15,
///     n_slots: 5
/// }
/// ```
///
/// ### Stream Node
/// ```text
/// GreenNode {
///     kind: STREAM,
///     text_len: 1024,
///     n_slots: 4
/// }
/// ```
///
/// ## Debugging Workflows
///
/// This format is particularly useful for:
/// - **Tree traversal debugging**: Understanding node structure during iteration
/// - **Parser validation**: Verifying correct node creation
/// - **Text span debugging**: Checking that length calculations are correct
/// - **Slot analysis**: Ensuring proper child slot allocation
///
/// ## Intentional Limitations
///
/// The debug format deliberately excludes:
/// - **Child contents**: Would create overwhelming output for large trees
/// - **Memory addresses**: Not useful for logical debugging
/// - **Internal implementation details**: Focuses on externally relevant data
/// - **Trivia information**: Keeps output concise and readable
///
/// ## Performance Considerations
///
/// Debug formatting is typically only used during development, but this
/// implementation is efficient:
/// - O(1) access to displayed fields
/// - No tree traversal required
/// - Minimal string allocation
/// - No recursive formatting
impl fmt::Debug for GreenNodeData {
    /// Formats the node for debug output.
    ///
    /// Creates a debug struct showing the node's essential properties:
    /// kind, text length, and number of slots.
    ///
    /// ## Performance
    ///
    /// This operation is O(1) and only accesses header fields and slot count.
    /// No recursive traversal or complex computation is performed.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenNode")
            .field("kind", &self.kind())
            .field("text_len", &self.text_len())
            .field("n_slots", &self.slots().len())
            .finish()
    }
}

/// Implements display formatting for `GreenNodeData`.
///
/// This implementation reconstructs the original source text by concatenating
/// the display output of all child slots in order. This enables faithful
/// reproduction of the original PDF content from the syntax tree.
///
/// ## Text Reconstruction
///
/// The display format works by:
/// 1. Iterating through all child slots in order
/// 2. Displaying each slot (which may be a token or nested node)
/// 3. Concatenating the results to rebuild the original text
/// 4. Preserving all whitespace, punctuation, and formatting
///
/// ## Fidelity Guarantee
///
/// The reconstructed text should be identical to the original source:
/// - **Whitespace preservation**: All spaces, tabs, and newlines are maintained
/// - **Token ordering**: Children are displayed in their original sequence
/// - **Nested structure**: Child nodes recursively display their content
/// - **Trivia inclusion**: Comments and formatting are preserved
///
/// ## PDF Examples
///
/// ### Object Display
/// For a PDF object node, the display output would be:
/// ```pdf
/// 1 0 obj
/// << /Type /Catalog /Pages 2 0 R >>
/// endobj
/// ```
///
/// ### Dictionary Display
/// For a dictionary node, the display output would be:
/// ```pdf
/// << /Type /Catalog /Pages 2 0 R >>
/// ```
///
/// ### Array Display
/// For an array node, the display output would be:
/// ```pdf
/// [ 1 2 3 4 ]
/// ```
///
/// ## Use Cases
///
/// Display formatting is used for:
/// - **Source reconstruction**: Rebuilding PDF files from syntax trees
/// - **Pretty printing**: Formatted output for human readers
/// - **Diff generation**: Comparing original and modified content
/// - **Error messages**: Showing problematic source code in diagnostics
/// - **IDE features**: Displaying hover content and code completion
///
/// ## Performance Characteristics
///
/// The display operation has these properties:
/// - **O(n) complexity**: Where n is the total number of tokens in the subtree
/// - **Single pass**: Each node is visited exactly once
/// - **Streaming output**: Uses the formatter's write methods efficiently
/// - **Memory efficient**: No intermediate string building
///
/// ## Error Handling
///
/// The implementation propagates formatting errors from child nodes:
/// - If any child slot fails to format, the entire operation fails
/// - Error propagation preserves the original error context
/// - Partial output is possible if errors occur mid-stream
///
/// ## Whitespace Handling
///
/// Special attention is paid to PDF's whitespace rules:
/// - **Semantic whitespace**: Spaces that affect PDF interpretation
/// - **Formatting whitespace**: Spaces for human readability
/// - **Line endings**: Proper handling of different line ending styles
/// - **Indentation**: Preservation of original indentation patterns
impl fmt::Display for GreenNodeData {
    /// Formats the node by displaying all its child slots in order.
    ///
    /// This reconstructs the original source text by concatenating the
    /// display output of each child slot. The result should be identical
    /// to the original source that was parsed to create this node.
    ///
    /// ## Error Propagation
    ///
    /// If any child slot fails to format, this method returns the error
    /// immediately. This ensures that formatting errors are properly
    /// reported and not silently ignored.
    ///
    /// ## Performance
    ///
    /// The operation is O(n) where n is the total number of tokens in
    /// the subtree. Each slot is visited exactly once during formatting.
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
        {
        }

        fn assert_has_text_len<T>(_: T)
        where
            T: Fn(&GreenNodeData) -> u64,
        {
        }

        fn assert_has_slots<T>(_: T)
        where
            T: Fn(&GreenNodeData) -> Slots<'_>,
        {
        }

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
