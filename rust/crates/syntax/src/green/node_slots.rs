//! # Node Slots: Efficient Iterator for Child Elements
//!
//! This module provides the `Slots` iterator for traversing child elements
//! within green tree nodes. It offers efficient, type-safe iteration over
//! the heterogeneous child elements (nodes, tokens, empty slots) that make
//! up PDF syntax structures.
//!
//! ## What are Node Slots?
//!
//! Node slots represent the child positions within a parent green node.
//! The `Slots` iterator provides a way to traverse these children efficiently
//! while maintaining all the performance optimizations of the underlying
//! slice iterator.
//!
//! ## Design Benefits
//!
//! ### Zero-Cost Iteration
//! The `Slots` iterator is a thin wrapper around `slice::Iter` that forwards
//! all the specialized optimizations from the standard library, ensuring
//! optimal performance for tree traversal operations.
//!
//! ### Type Safety
//! Provides strongly-typed access to `Slot` elements, preventing confusion
//! between different types of child elements while maintaining performance.
//!
//! ### Memory Efficiency
//! Iteration operates directly over the underlying slot array without
//! additional allocations or temporary data structures.
//!
//! ## PDF Processing Context
//!
//! In PDF syntax trees, slot iteration enables:
//! - **Dictionary traversal**: Iterating over key-value pairs in PDF dictionaries
//! - **Array processing**: Walking through array elements sequentially
//! - **Object analysis**: Examining components of PDF objects (header, content, footer)
//! - **Stream parsing**: Processing stream dictionaries and content sections
//!
//! ## Performance Characteristics
//!
//! - **O(1) size operations**: Length calculation with no traversal
//! - **Cache-friendly**: Sequential access pattern optimal for CPU caches
//! - **Fused iteration**: Optimized iteration that can be vectorized
//! - **Exact size**: Known length enables additional optimizations
//!
//! ## Iterator Specializations
//!
//! This iterator forwards all stable specializations from `slice::Iter`,
//! including vectorized operations and other performance optimizations
//! available in the Rust standard library.

use std::{iter::FusedIterator, slice};

use crate::green::node_slot::Slot;

/// Efficient iterator over child slots within a green tree node.
///
/// `Slots` provides a type-safe, high-performance way to iterate over
/// the child elements of a green node. It wraps the underlying slice
/// iterator while maintaining all performance optimizations.
///
/// ## Iteration Guarantees
///
/// - **Exact size**: The iterator knows its length without traversal
/// - **Fused**: Continues to return `None` after exhaustion
/// - **Double-ended**: Can iterate from both ends (when underlying slice supports it)
/// - **Memory safe**: No bounds checking overhead during iteration
///
/// ## PDF Use Cases
///
/// Common patterns when iterating over slots:
/// ```rust,ignore
/// for slot in node.slots() {
///     match slot {
///         Slot::Node { node, .. } => process_child_node(node),
///         Slot::Token { token, .. } => process_token(token),
///         Slot::Empty { .. } => handle_missing_element(),
///     }
/// }
/// ```
///
/// This enables processing of heterogeneous PDF structures like:
/// - Dictionary entries mixing names, values, and sub-dictionaries
/// - Arrays containing various data types and nested structures
/// - Object definitions with headers, content, and metadata
#[derive(Debug, Clone)]
pub(crate) struct Slots<'a> {
    /// The underlying slice iterator over slot elements.
    ///
    /// This raw iterator provides all the performance optimizations
    /// and specializations available in the standard library's
    /// slice iteration implementation.
    pub(crate) raw: slice::Iter<'a, Slot>,
}

/// Exact size iterator implementation for efficient length queries.
///
/// This trait implementation forwards the exact size calculation to the
/// underlying slice iterator, providing O(1) length queries without
/// needing to traverse the iterator. This is particularly useful for:
/// - Pre-allocating collections of the right size
/// - Optimizing algorithms that need to know iteration bounds
/// - Enabling vectorization and other compiler optimizations
// NB: forward everything stable that iter::Slice specializes as of Rust 1.39.0
impl ExactSizeIterator for Slots<'_> {
    /// Returns the exact number of slots remaining in the iterator.
    ///
    /// This is a zero-cost operation that directly queries the underlying
    /// slice length, providing immediate access to the iteration bounds
    /// without any traversal or computation overhead.
    ///
    /// ## PDF Processing Benefits
    ///
    /// Knowing the exact size enables optimizations such as:
    /// - Pre-allocating result collections for dictionary key-value pairs
    /// - Vectorizing operations over array elements
    /// - Optimizing memory usage for large object processing
    #[inline(always)]
    fn len(&self) -> usize {
        self.raw.len()
    }
}

/// Primary iterator implementation for traversing child slots.
///
/// This implementation forwards all operations to the underlying slice
/// iterator while maintaining type safety and performance optimizations.
/// The iterator yields references to slots, allowing efficient examination
/// of child elements without ownership transfer.
impl<'a> Iterator for Slots<'a> {
    /// The type of items yielded by this iterator.
    ///
    /// Each iteration yields a reference to a `Slot`, which can be
    /// a node, token, or empty slot representing the child element
    /// at that position within the parent.
    type Item = &'a Slot;

    /// Advances the iterator and returns the next slot.
    ///
    /// This method provides sequential access to child elements,
    /// forwarding directly to the optimized slice iterator implementation.
    /// Returns `None` when all slots have been consumed.
    ///
    /// ## PDF Processing Pattern
    ///
    /// Typically used in loops to process heterogeneous child elements:
    /// ```rust,ignore
    /// while let Some(slot) = slots.next() {
    ///     match slot {
    ///         Slot::Token { token, .. } => process_pdf_token(token),
    ///         Slot::Node { node, .. } => recurse_into_structure(node),
    ///         Slot::Empty { .. } => handle_optional_element(),
    ///     }
    /// }
    /// ```
    #[inline]
    fn next(&mut self) -> Option<&'a Slot> {
        self.raw.next()
    }

    /// Provides size hints for iterator optimization.
    ///
    /// Returns a tuple indicating the lower and upper bounds of remaining
    /// iterations. For slice-based iteration, these bounds are exact,
    /// enabling compiler optimizations and efficient memory pre-allocation.
    ///
    /// ## Optimization Benefits
    ///
    /// Accurate size hints enable:
    /// - **Collection pre-sizing**: Avoid reallocation during collection
    /// - **Vectorization**: Enable SIMD optimizations for bulk operations
    /// - **Memory planning**: Allocate appropriate buffer sizes for PDF processing
    ///
    /// ## Return Value
    ///
    /// Returns `(lower_bound, Some(upper_bound))` where both bounds are
    /// equal for slice iteration, providing exact iteration count.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }

    /// Consumes the iterator and returns the total number of slots.
    ///
    /// This method provides an efficient way to count all remaining slots
    /// without manually iterating through them. For slice-based iterators,
    /// this is optimized to simply return the remaining length.
    ///
    /// ## Performance
    ///
    /// This is an O(1) operation for slice iterators, making it much more
    /// efficient than manually counting with a loop. The method forwards
    /// to the optimized slice implementation.
    ///
    /// ## PDF Processing Usage
    ///
    /// Useful for metrics and validation:
    /// ```rust,ignore
    /// let slot_count = node.slots().count();
    /// println!("PDF object has {} child elements", slot_count);
    ///
    /// // Validate expected structure
    /// if dictionary_slots.count() % 2 != 0 {
    ///     return Err("Dictionary must have even number of slots (key-value pairs)");
    /// }
    /// ```
    ///
    /// ## Consumption Note
    ///
    /// This method consumes the iterator, so it cannot be used afterward.
    /// If you need both the count and to iterate, use `len()` instead.
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.raw.count()
    }

    /// Consumes the iterator and returns the last slot, if any.
    ///
    /// This method efficiently finds the last slot without iterating through
    /// all elements. For double-ended iterators like this one, it's implemented
    /// by calling `next_back()` once, providing O(1) access to the final element.
    ///
    /// ## PDF Processing Applications
    ///
    /// Useful for examining structural endings:
    /// ```rust,ignore
    /// // Check if object properly ends with "endobj"
    /// if let Some(last_slot) = object_slots.last() {
    ///     match last_slot {
    ///         Slot::Token { token, .. } if token.text() == "endobj" => {
    ///             // Object is properly terminated
    ///         }
    ///         _ => return Err("Object missing 'endobj' terminator"),
    ///     }
    /// }
    ///
    /// // Validate array closing bracket
    /// if let Some(last_slot) = array_slots.last() {
    ///     // Should be a "]" token
    /// }
    /// ```
    ///
    /// ## Performance Benefits
    ///
    /// - **O(1) complexity**: Direct access to last element via reverse iteration
    /// - **No traversal**: Doesn't iterate through preceding elements
    /// - **Memory efficient**: Uses the existing double-ended iterator capability
    ///
    /// ## Return Value
    ///
    /// Returns `Some(&Slot)` if the iterator contains any elements,
    /// or `None` if the iterator is empty.
    ///
    /// ## Consumption Note
    ///
    /// This method consumes the iterator, making it unavailable for further use.
    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    /// Returns the nth slot from the current position.
    ///
    /// This method efficiently skips `n` slots and returns the next one,
    /// providing indexed access into the remaining iterator elements.
    /// It advances the iterator past the skipped elements.
    ///
    /// ## Parameters
    ///
    /// - `n`: The number of slots to skip (0-indexed from current position)
    ///
    /// ## Return Value
    ///
    /// - `Some(&Slot)`: The slot at position n (relative to current iterator position)
    /// - `None`: If n is beyond the remaining elements
    ///
    /// ## PDF Processing Examples
    ///
    /// Useful for accessing specific structural elements:
    /// ```rust,ignore
    /// let mut slots = object_slots;
    ///
    /// // Skip object number and generation, get the "obj" keyword
    /// if let Some(obj_keyword) = slots.nth(2) {
    ///     assert_eq!(obj_keyword.token_text(), "obj");
    /// }
    ///
    /// // For dictionary slots: skip "<<", get first key
    /// let mut dict_slots = dictionary.slots();
    /// if let Some(first_key) = dict_slots.nth(1) {
    ///     // Process first dictionary key
    /// }
    ///
    /// // Skip to specific array element
    /// let mut array_slots = array.slots();
    /// array_slots.nth(0); // Skip opening "["
    /// if let Some(third_element) = array_slots.nth(2) {
    ///     // Process third array element (skipped first two)
    /// }
    /// ```
    ///
    /// ## Performance
    ///
    /// - **O(n) complexity**: Must advance through n elements
    /// - **Early termination**: Stops immediately if iterator exhausted
    /// - **Memory efficient**: No allocation, direct iterator advancement
    ///
    /// ## Iterator State
    ///
    /// After calling `nth(n)`, the iterator position is advanced by n+1 elements.
    /// Subsequent calls to `next()` will continue from that new position.
    ///
    /// ## Edge Cases
    ///
    /// - `nth(0)` is equivalent to `next()`
    /// - `nth(n)` where n >= remaining elements returns `None`
    /// - The iterator position advances even when returning `None`
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth(n)
    }

    /// Performs a left-associative fold over all remaining slots.
    ///
    /// This method applies a closure to each slot in sequence, accumulating
    /// the results into a single value. It's a fundamental operation for
    /// processing collections of slots in a functional programming style.
    ///
    /// ## Parameters
    ///
    /// - `init`: The initial accumulator value
    /// - `f`: A closure that takes `(accumulator, slot)` and returns new accumulator
    ///
    /// ## Return Value
    ///
    /// The final accumulated value after processing all slots.
    ///
    /// ## PDF Processing Applications
    ///
    /// Fold is powerful for analyzing and transforming PDF structures:
    ///
    /// ### Text Length Calculation
    /// ```rust,ignore
    /// let total_length = slots.fold(0u64, |acc, slot| {
    ///     acc + slot.text_len()
    /// });
    /// ```
    ///
    /// ### Dictionary Key Collection
    /// ```rust,ignore
    /// let mut dict_slots = dictionary.slots();
    /// dict_slots.nth(0); // Skip "<<"
    ///
    /// let keys = dict_slots.fold(Vec::new(), |mut acc, slot| {
    ///     if let Slot::Token { token, .. } = slot {
    ///         if token.text().starts_with('/') {
    ///             acc.push(token.text().to_string());
    ///         }
    ///     }
    ///     acc
    /// });
    /// ```
    ///
    /// ### Validation State Accumulation
    /// ```rust,ignore
    /// #[derive(Default)]
    /// struct ValidationState {
    ///     has_type: bool,
    ///     has_pages: bool,
    ///     error_count: usize,
    /// }
    ///
    /// let state = slots.fold(ValidationState::default(), |mut state, slot| {
    ///     match slot {
    ///         Slot::Token { token, .. } => {
    ///             match token.text() {
    ///                 "/Type" => state.has_type = true,
    ///                 "/Pages" => state.has_pages = true,
    ///                 _ => {}
    ///             }
    ///         }
    ///         Slot::Node { node, .. } => {
    ///             // Validate nested structure
    ///         }
    ///         _ => {}
    ///     }
    ///     state
    /// });
    /// ```
    ///
    /// ### Object Reference Collection
    /// ```rust,ignore
    /// let references = slots.fold(HashSet::new(), |mut refs, slot| {
    ///     if let Slot::Node { node, .. } = slot {
    ///         if node.kind() == REFERENCE {
    ///             refs.insert(extract_object_number(node));
    ///         }
    ///     }
    ///     refs
    /// });
    /// ```
    ///
    /// ## Performance Characteristics
    ///
    /// - **O(n) complexity**: Visits each slot exactly once
    /// - **Single pass**: Efficient for one-time processing
    /// - **Memory efficient**: No intermediate collections unless explicitly created
    /// - **Tail call optimization**: Can be optimized by the compiler
    ///
    /// ## Functional Programming Benefits
    ///
    /// Fold enables composable, declarative processing:
    /// - **Immutable patterns**: Accumulate without mutation
    /// - **Composable**: Can be chained with other iterator operations
    /// - **Testable**: Pure functions are easier to test
    /// - **Readable**: Intent is clear from the accumulation logic
    ///
    /// ## Alternative Approaches
    ///
    /// While fold is powerful, consider alternatives for specific use cases:
    /// - Use `collect()` for simple collection building
    /// - Use `find()` for early termination searches
    /// - Use `all()` or `any()` for boolean conditions
    /// - Use `reduce()` when no initial value is needed
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

/// Double-ended iterator implementation for bidirectional slot traversal.
///
/// This implementation enables iteration from both ends of the slot collection,
/// providing efficient access to both the beginning and end of PDF structures.
/// This is particularly valuable for PDF processing where both prefix and
/// suffix elements may need special handling.
///
/// ## Bidirectional Benefits
///
/// Double-ended iteration enables powerful patterns:
/// - **Validation**: Check both opening and closing delimiters
/// - **Optimization**: Process from most relevant end first
/// - **Structure analysis**: Examine boundaries of PDF constructs
/// - **Error recovery**: Find valid boundaries when parsing fails
///
/// ## PDF Processing Applications
///
/// ### Object Boundary Validation
/// ```rust,ignore
/// let mut slots = object.slots();
///
/// // Check first element (should be object number)
/// if let Some(first) = slots.next() {
///     validate_object_number(first);
/// }
///
/// // Check last element (should be "endobj")
/// if let Some(last) = slots.next_back() {
///     validate_endobj_keyword(last);
/// }
///
/// // Process middle content
/// for slot in slots {
///     process_object_content(slot);
/// }
/// ```
///
/// ### Dictionary Processing
/// ```rust,ignore
/// let mut dict_slots = dictionary.slots();
///
/// // Verify opening delimiter
/// assert_eq!(dict_slots.next().unwrap().token_text(), "<<");
///
/// // Verify closing delimiter  
/// assert_eq!(dict_slots.next_back().unwrap().token_text(), ">>");
///
/// // Process key-value pairs in middle
/// while let (Some(key), Some(value)) = (dict_slots.next(), dict_slots.next()) {
///     process_dictionary_entry(key, value);
/// }
/// ```
///
/// ### Array Boundary Handling
/// ```rust,ignore
/// let mut array_slots = array.slots();
///
/// // Skip array delimiters from both ends
/// array_slots.next();      // Skip "["
/// array_slots.next_back(); // Skip "]"
///
/// // Process array elements
/// for element in array_slots {
///     process_array_element(element);
/// }
/// ```
///
/// ## Performance Characteristics
///
/// - **O(1) access**: Both ends accessible in constant time
/// - **Cache efficiency**: Can optimize access patterns based on usage
/// - **Memory layout**: Takes advantage of slice's contiguous memory
/// - **Branch prediction**: Predictable iteration patterns
impl<'a> DoubleEndedIterator for Slots<'a> {
    /// Advances the iterator from the back and returns the previous slot.
    ///
    /// This method provides reverse iteration capability, allowing efficient
    /// access to slots from the end of the collection. Combined with `next()`,
    /// it enables powerful bidirectional processing patterns.
    ///
    /// ## Return Value
    ///
    /// - `Some(&Slot)`: The next slot from the back of the remaining iterator
    /// - `None`: When the iterator is exhausted (meets the forward iterator)
    ///
    /// ## PDF Structure Applications
    ///
    /// Reverse iteration is particularly useful for PDF validation and parsing:
    ///
    /// ### Object Termination Validation
    /// ```rust,ignore
    /// let mut slots = object.slots();
    ///
    /// if let Some(terminator) = slots.next_back() {
    ///     match terminator {
    ///         Slot::Token { token, .. } if token.text() == "endobj" => {
    ///             // Valid object termination
    ///         }
    ///         _ => return Err("Object must end with 'endobj'"),
    ///     }
    /// }
    /// ```
    ///
    /// ### Dictionary Closing Validation
    /// ```rust,ignore
    /// let mut dict_slots = dictionary.slots();
    ///
    /// if let Some(closer) = dict_slots.next_back() {
    ///     assert_eq!(closer.token_text(), ">>");
    /// }
    /// ```
    ///
    /// ### Stream Length Calculation
    /// ```rust,ignore
    /// let mut stream_slots = stream.slots();
    ///
    /// // Find "endstream" from the back
    /// while let Some(slot) = stream_slots.next_back() {
    ///     if let Slot::Token { token, .. } = slot {
    ///         if token.text() == "endstream" {
    ///             break;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// ## Bidirectional Processing Pattern
    ///
    /// ```rust,ignore
    /// let mut slots = node.slots();
    ///
    /// loop {
    ///     match (slots.next(), slots.next_back()) {
    ///         (Some(front), Some(back)) => {
    ///             process_from_both_ends(front, back);
    ///         }
    ///         (Some(middle), None) => {
    ///             process_remaining_middle(middle);
    ///             break;
    ///         }
    ///         (None, Some(_)) => {
    ///             // This shouldn't happen with proper double-ended iterator
    ///             unreachable!();
    ///         }
    ///         (None, None) => break,
    ///     }
    /// }
    /// ```
    ///
    /// ## Performance Benefits
    ///
    /// - **O(1) complexity**: Direct access to next element from back
    /// - **Cache locality**: Sequential memory access in reverse direction
    /// - **Early termination**: Can stop processing when condition met
    /// - **Memory efficiency**: No additional storage required
    ///
    /// ## Iterator State
    ///
    /// Each call to `next_back()` decreases the remaining iterator length.
    /// The iterator maintains internal pointers to track both forward and
    /// backward positions, ensuring they never cross.
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw.next_back()
    }

    /// Returns the nth slot from the back of the iterator.
    ///
    /// This method efficiently skips `n` slots from the back and returns
    /// the next one in reverse order. It's the backward equivalent of `nth()`,
    /// providing indexed access from the end of the slot collection.
    ///
    /// ## Parameters
    ///
    /// - `n`: Number of slots to skip from the back (0-indexed from current back position)
    ///
    /// ## Return Value
    ///
    /// - `Some(&Slot)`: The slot at position n from the back
    /// - `None`: If n exceeds remaining elements from the back
    ///
    /// ## PDF Processing Examples
    ///
    /// Useful for accessing specific structural elements from the end:
    ///
    /// ### Object Structure Analysis
    /// ```rust,ignore
    /// let mut slots = object.slots();
    ///
    /// // Get "endobj" (should be last)
    /// let endobj = slots.nth_back(0); // equivalent to next_back()
    ///
    /// // Get the object content (skip "endobj", get content)
    /// let content = slots.nth_back(1);
    ///
    /// // Get "obj" keyword (skip "endobj" and content)
    /// let obj_keyword = slots.nth_back(2);
    /// ```
    ///
    /// ### Dictionary Value Access
    /// ```rust,ignore
    /// let mut dict_slots = dictionary.slots();
    ///
    /// // Skip ">>" delimiter, get last value
    /// if let Some(last_value) = dict_slots.nth_back(1) {
    ///     process_dictionary_value(last_value);
    /// }
    ///
    /// // Skip ">>" and last value, get last key
    /// if let Some(last_key) = dict_slots.nth_back(2) {
    ///     assert!(last_key.token_text().starts_with('/'));
    /// }
    /// ```
    ///
    /// ### Array Element Indexing
    /// ```rust,ignore
    /// let mut array_slots = array.slots();
    ///
    /// // Skip closing "]", get last array element
    /// if let Some(last_element) = array_slots.nth_back(1) {
    ///     process_array_element(last_element);
    /// }
    ///
    /// // Get second-to-last element
    /// if let Some(second_last) = array_slots.nth_back(2) {
    ///     process_array_element(second_last);
    /// }
    /// ```
    ///
    /// ### Stream Processing
    /// ```rust,ignore
    /// let mut stream_slots = stream.slots();
    ///
    /// // Skip "endstream", get the actual stream data
    /// if let Some(stream_data) = stream_slots.nth_back(1) {
    ///     process_stream_content(stream_data);
    /// }
    ///
    /// // Skip "endstream" and data, get stream dictionary
    /// if let Some(stream_dict) = stream_slots.nth_back(2) {
    ///     process_stream_dictionary(stream_dict);
    /// }
    /// ```
    ///
    /// ## Performance Characteristics
    ///
    /// - **O(n) complexity**: Must advance backward through n elements
    /// - **Early termination**: Stops if iterator exhausted before reaching n
    /// - **Memory efficient**: Direct slice access, no intermediate allocation
    /// - **Cache friendly**: Sequential memory access in reverse
    ///
    /// ## Iterator State
    ///
    /// After calling `nth_back(n)`, the back position advances by n+1 elements.
    /// This reduces the remaining iterator length and affects subsequent operations.
    ///
    /// ## Edge Cases
    ///
    /// - `nth_back(0)` is equivalent to `next_back()`
    /// - `nth_back(n)` where n >= remaining elements returns `None`
    /// - The iterator back position advances even when returning `None`
    ///
    /// ## Bidirectional Interaction
    ///
    /// This method works in conjunction with forward iteration methods.
    /// The forward and backward positions never cross, ensuring iterator safety.
    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.raw.nth_back(n)
    }

    /// Performs a right-associative fold over slots in reverse order.
    ///
    /// This method applies a closure to each slot from back to front,
    /// accumulating results in reverse order. It's the backward equivalent
    /// of `fold()`, enabling processing patterns that work naturally from
    /// the end of PDF structures.
    ///
    /// ## Parameters
    ///
    /// - `init`: The initial accumulator value
    /// - `f`: A closure that takes `(accumulator, slot)` and returns new accumulator
    ///
    /// ## Return Value
    ///
    /// The final accumulated value after processing all slots in reverse order.
    ///
    /// ## PDF Processing Applications
    ///
    /// Reverse folding is particularly useful for PDF structures where
    /// processing from the end provides better performance or correctness:
    ///
    /// ### Object Validation (End-to-Start)
    /// ```rust,ignore
    /// let validation_result = slots.rfold(ValidationState::new(), |mut state, slot| {
    ///     match slot {
    ///         Slot::Token { token, .. } => {
    ///             match token.text() {
    ///                 "endobj" => state.found_terminator = true,
    ///                 "obj" => state.found_start = true,
    ///                 _ => {}
    ///             }
    ///         }
    ///         _ => {}
    ///     }
    ///     state
    /// });
    /// ```
    ///
    /// ### Dictionary Value Collection (Reverse Order)
    /// ```rust,ignore
    /// let mut dict_slots = dictionary.slots();
    /// dict_slots.next_back(); // Skip ">>"
    ///
    /// let values_reversed = dict_slots.rfold(Vec::new(), |mut values, slot| {
    ///     // Collect values in reverse order (useful for certain algorithms)
    ///     if is_dictionary_value(slot) {
    ///         values.push(extract_value(slot));
    ///     }
    ///     values
    /// });
    /// ```
    ///
    /// ### Error Context Building
    /// ```rust,ignore
    /// let error_context = slots.rfold(String::new(), |mut context, slot| {
    ///     if let Slot::Token { token, .. } = slot {
    ///         if !context.is_empty() {
    ///             context.push(' ');
    ///         }
    ///         context.insert_str(0, token.text()); // Build context backwards
    ///     }
    ///     context
    /// });
    /// ```
    ///
    /// ### Reference Resolution (Bottom-Up)
    /// ```rust,ignore
    /// let resolved_refs = slots.rfold(HashMap::new(), |mut refs, slot| {
    ///     if let Slot::Node { node, .. } = slot {
    ///         if node.kind() == REFERENCE {
    ///             let obj_num = extract_object_number(node);
    ///             refs.insert(obj_num, resolve_reference(obj_num));
    ///         }
    ///     }
    ///     refs
    /// });
    /// ```
    ///
    /// ### Stream Length Calculation (Reverse)
    /// ```rust,ignore
    /// let stream_info = stream_slots.rfold(StreamInfo::default(), |mut info, slot| {
    ///     match slot {
    ///         Slot::Token { token, .. } => {
    ///             match token.text() {
    ///                 "endstream" => info.has_end = true,
    ///                 "stream" => info.has_start = true,
    ///                 _ => info.content_length += token.text().len(),
    ///             }
    ///         }
    ///         Slot::Node { .. } => {
    ///             info.content_length += slot.text_len() as usize;
    ///         }
    ///         _ => {}
    ///     }
    ///     info
    /// });
    /// ```
    ///
    /// ## Performance Characteristics
    ///
    /// - **O(n) complexity**: Visits each slot exactly once, in reverse
    /// - **Single pass**: Efficient reverse processing
    /// - **Memory efficient**: No intermediate storage unless explicitly created
    /// - **Cache optimization**: Can leverage reverse iteration optimizations
    ///
    /// ## Comparison with Forward Fold
    ///
    /// Choose `rfold` over `fold` when:
    /// - Processing naturally flows from end to beginning
    /// - Building data structures that benefit from reverse order
    /// - Validation requires checking termination before content
    /// - Error recovery is easier from known endpoints
    ///
    /// ## Functional Programming Benefits
    ///
    /// Reverse fold enables different algorithmic approaches:
    /// - **Bottom-up processing**: Build solutions from endpoints
    /// - **Validation chains**: Check conditions in reverse dependency order
    /// - **Context accumulation**: Build error messages from specific to general
    /// - **Optimization**: Some algorithms are more efficient in reverse
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

/// Fused iterator implementation ensuring consistent behavior after exhaustion.
///
/// This trait implementation guarantees that once the iterator returns `None`
/// from `next()` or `next_back()`, all subsequent calls will also return `None`.
/// This provides predictable behavior and enables certain optimizations in
/// iterator-consuming code.
///
/// ## Fused Iterator Benefits
///
/// The fused property provides several guarantees:
/// - **Predictable exhaustion**: No undefined behavior after `None`
/// - **Optimization opportunities**: Consumers can rely on consistent behavior
/// - **Safe chaining**: Can be used safely in iterator chains
/// - **Error prevention**: Eliminates potential infinite loop scenarios
///
/// ## PDF Processing Reliability
///
/// For PDF processing, fused iteration ensures:
/// - **Consistent parsing**: Parser state remains stable after exhaustion
/// - **Error handling**: Graceful handling of malformed structures
/// - **Resource management**: Predictable cleanup of iterator resources
/// - **Debugging**: Reliable behavior during development and testing
///
/// ## Example Scenarios
///
/// Without fused guarantee, an iterator might:
/// ```rust,ignore
/// let mut iter = some_iterator();
/// assert_eq!(iter.next(), None);
/// // Non-fused iterator might return Some() here!
/// assert_eq!(iter.next(), None); // This is guaranteed with FusedIterator
/// ```
///
/// With fused guarantee:
/// ```rust,ignore
/// let mut slots = node.slots();
///
/// // Iterate until exhausted
/// while let Some(slot) = slots.next() {
///     process_slot(slot);
/// }
///
/// // All subsequent calls return None
/// assert_eq!(slots.next(), None);
/// assert_eq!(slots.next(), None);
/// assert_eq!(slots.next_back(), None);
/// ```
///
/// ## Implementation Details
///
/// The `Slots` iterator inherits its fused property from the underlying
/// `slice::Iter`, which is guaranteed to be fused. This means:
/// - No additional runtime overhead
/// - Behavior is inherited from the well-tested standard library
/// - Consistent with other slice-based iterators
/// - Optimal performance characteristics maintained
///
/// ## Usage in PDF Compiler
///
/// Fused iteration is particularly valuable for:
/// - **Parser robustness**: Handling incomplete or malformed PDF structures
/// - **Iterator chains**: Safely combining multiple iterator operations
/// - **Error recovery**: Predictable behavior when parsing fails
/// - **Performance optimization**: Compilers can optimize based on fused guarantee
impl FusedIterator for Slots<'_> {}
