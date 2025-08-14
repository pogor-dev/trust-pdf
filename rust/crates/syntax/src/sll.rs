//! # Sorted Linked List (SLL)
//!
//! This module implements a circular doubly-linked list that maintains elements in sorted order
//! based on their keys. It's designed for efficient insertion, deletion, and key adjustment
//! operations commonly needed in syntax tree manipulation.
//!
//! ## Structure Overview
//!
//! The SLL is a circular doubly-linked list where:
//! - Elements are always sorted by their `key()` value in ascending order
//! - The list is circular: the last element points back to the first
//! - Each element has both `prev` and `next` pointers
//! - A single `head` pointer always points to the smallest element
//!
//! ```text
//! Single element (self-linked):
//! ┌─────┐
//! │ 10  │◄─┐
//! └─────┘  │
//!   │      │
//!   └──────┘
//!
//! Multiple elements:
//!        head
//!         │
//!         ▼
//!      ┌─────┐    ┌─────┐    ┌─────┐
//!   ┌─►│ 10  │───►│ 20  │───►│ 30  │─┐
//!   │  └─────┘◄───└─────┘◄───└─────┘ │
//!   │                                │
//!   └────────────────────────────────┘
//! ```
//!
//! ## Key Operations
//!
//! ### Insertion
//! New elements are inserted to maintain sorted order:
//!
//! ```text
//! Inserting 15 into [10, 20, 30]:
//!
//! Step 1: Find position (traverse backward from tail)
//!      ┌─────┐    ┌─────┐    ┌─────┐
//!   ┌─►│ 10  │───►│ 20  │───►│ 30  │─┐
//!   │  └─────┘◄───└─────┘◄───└─────┘ │
//!   │             ▲                  │
//!   │             │ 15 < 20          │
//!   └─────────────┴──────────────────┘
//!
//! Step 2: Insert between 10 and 20
//!      ┌─────┐    ┌─────┐    ┌─────┐    ┌─────┐
//!   ┌─►│ 10  │───►│ 15  │───►│ 20  │───►│ 30  │─┐
//!   │  └─────┘◄───└─────┘◄───└─────┘◄───└─────┘ │
//!   │                                           │
//!   └───────────────────────────────────────────┘
//! ```
//!
//! ### Deletion
//! Elements are removed by updating neighboring pointers:
//!
//! ```text
//! Removing 20 from [10, 15, 20, 30]:
//!
//!      ┌─────┐    ┌─────┐    ┌─────┐    ┌─────┐
//!   ┌─►│ 10  │───►│ 15  │───►│ 20  │───►│ 30  │─┐
//!   │  └─────┘◄───└─────┘◄───└─────┘◄───└─────┘ │
//!   │                      ▲                   │
//!   └──────────────────────┴───────────────────┘
//!                          │
//!                      Remove this
//!                          │
//!                          ▼
//!      ┌─────┐    ┌─────┐             ┌─────┐
//!   ┌─►│ 10  │───►│ 15  │────────────►│ 30  │─┐
//!   │  └─────┘◄───└─────┘◄────────────└─────┘ │
//!   │                                         │
//!   └─────────────────────────────────────────┘
//! ```
//!
//! ### Key Adjustment
//! Bulk updates to keys >= threshold:
//!
//! ```text
//! adjust([10, 20, 30, 40], from=25, delta=+5):
//!
//! Before: 10 ─► 20 ─► 30 ─► 40
//!         ▲     ▲     ▲     ▲
//!         │     │     │     │
//!      skip  skip  +5    +5
//!
//! After:  10 ─► 20 ─► 35 ─► 45
//! ```

use std::{cell::Cell, cmp::Ordering, ptr};

use crate::utility_types::Delta;

/// An element that can be stored in a sorted linked list.
///
/// This trait defines the interface for elements that can be linked together
/// in a circular doubly-linked list. Each element must provide:
/// - `prev`: Pointer to the previous element in the circular list
/// - `next`: Pointer to the next element in the circular list  
/// - `key`: The sortable value used to maintain list order
///
/// ## Memory Layout
///
/// ```text
/// Element structure:
/// ┌──────────────┐
/// │ prev: *const │ ──► Previous element
/// │ next: *const │ ──► Next element
/// │ key:  Cell   │ ──► Sortable key value
/// │ ...data...   │ ──► Additional element data
/// └──────────────┘
/// ```
///
/// # Safety
///
/// Implementors of this trait must ensure that the pointers returned by
/// `prev` and `next` are valid and properly initialized. The pointers must
/// point to valid instances of the implementing type or be null pointers.
/// Additionally, the `key` method must return a valid reference to a `Cell<u32>`.
///
/// Failure to uphold these invariants can result in undefined behavior.
pub(crate) unsafe trait Elem {
    /// Returns a reference to the cell containing the pointer to the previous element.
    fn prev(&self) -> &Cell<*const Self>;

    /// Returns a reference to the cell containing the pointer to the next element.
    fn next(&self) -> &Cell<*const Self>;

    /// Returns a reference to the cell containing the sortable key.
    fn key(&self) -> &Cell<u32>;
}

/// Result of analyzing where a new element should be inserted in the sorted linked list.
///
/// This enum represents the different scenarios encountered when determining
/// where to insert a new element while maintaining sorted order.
///
/// ## Insertion Scenarios
///
/// ```text
/// Case 1: NoHead - No list to insert into
/// (no list exists)
///
/// Case 2: EmptyHead - List exists but is empty
/// head → null
///
/// Case 3: SmallerThanHead - New element becomes the new head
/// [new_elem] should become: new_elem → [old_head] → ...
///
/// Case 4: SmallerThanNotHead - Insert after a specific element
/// ... → [curr] → [new_elem] → [next] → ...
///
/// Case 5: AlreadyInSll - Element with same key already exists
/// ... → [existing_elem_with_same_key] → ...
/// ```
pub(crate) enum AddToSllResult<'a, E: Elem> {
    /// No head pointer provided - cannot insert anywhere
    NoHead,
    /// List head exists but points to null (empty list)
    EmptyHead(&'a Cell<*const E>),
    /// New element should become the new head (smaller than current head)
    SmallerThanHead(&'a Cell<*const E>),
    /// New element should be inserted after the given element
    SmallerThanNotHead(*const E),
    /// An element with the same key already exists
    AlreadyInSll(*const E),
}

impl<E: Elem> AddToSllResult<'_, E> {
    /// Performs the actual insertion of an element into the sorted linked list.
    ///
    /// This method takes the result of link analysis and executes the appropriate
    /// insertion strategy to maintain the circular doubly-linked structure.
    ///
    /// ## Implementation Details
    ///
    /// All elements start as self-linked before insertion:
    /// ```text
    /// Initial state: elem ←→ elem (self-linked)
    /// ```
    ///
    /// Then based on the variant:
    /// - `EmptyHead`: Simply update head to point to the element
    /// - `SmallerThanHead`: Insert as new head, update all links
    /// - `SmallerThanNotHead`: Insert after specified element
    /// - `NoHead`/`AlreadyInSll`: No operation performed
    ///
    /// # Arguments
    /// * `elem_ptr` - Raw pointer to the element being inserted
    pub(crate) fn add_to_sll(&self, elem_ptr: *const E) {
        unsafe {
            (*elem_ptr).prev().set(elem_ptr);
            (*elem_ptr).next().set(elem_ptr);

            match self {
                // Case 1: empty head, replace it.
                AddToSllResult::EmptyHead(head) => head.set(elem_ptr),

                // Case 2: we are smaller than the head, replace it.
                AddToSllResult::SmallerThanHead(head) => {
                    let old_head = head.get();
                    let prev = (*old_head).prev().replace(elem_ptr);
                    (*prev).next().set(elem_ptr);
                    (*elem_ptr).next().set(old_head);
                    (*elem_ptr).prev().set(prev);
                    head.set(elem_ptr);
                }

                // Case 3: insert in place found by looping
                AddToSllResult::SmallerThanNotHead(curr) => {
                    let next = (**curr).next().replace(elem_ptr);
                    (*next).prev().set(elem_ptr);
                    (*elem_ptr).prev().set(*curr);
                    (*elem_ptr).next().set(next);
                }
                AddToSllResult::NoHead | AddToSllResult::AlreadyInSll(_) => (),
            }
        }
    }
}

/// Initializes the linking process for adding an element to the sorted linked list.
///
/// This is the entry point for adding elements. It determines if a list exists
/// and delegates to `link()` for position analysis.
///
/// # Arguments
/// * `head` - Optional reference to the list head pointer
/// * `elem` - Element to be inserted
///
/// # Returns
/// Analysis result indicating where/how the element should be inserted
#[cold]
pub(crate) fn init<'a, E: Elem>(
    head: Option<&'a Cell<*const E>>,
    elem: &E,
) -> AddToSllResult<'a, E> {
    if let Some(head) = head {
        link(head, elem)
    } else {
        AddToSllResult::NoHead
    }
}

/// Removes an element from the sorted linked list.
///
/// This function safely removes an element by updating the neighboring elements'
/// pointers to bypass the removed element, maintaining the circular structure.
///
/// ## Removal Process
/// ```text
/// Before: A ←→ B ←→ C (removing B)
/// After:  A ←───────→ C
/// ```
///
/// Special case - removing the head updates the head pointer to the next element,
/// or sets it to null if the list becomes empty.
///
/// # Arguments
/// * `head` - Reference to the list head pointer
/// * `elem` - Element to remove from the list
///
/// # Panics
/// Panics in debug builds if the head pointer is null or if link integrity is violated
#[cold]
pub(crate) fn unlink<E: Elem>(head: &Cell<*const E>, elem: &E) {
    debug_assert!(!head.get().is_null(), "invalid linked list head");

    let elem_ptr: *const E = elem;

    let prev = elem.prev().replace(elem_ptr);
    let next = elem.next().replace(elem_ptr);
    unsafe {
        debug_assert_eq!((*prev).next().get(), elem_ptr, "invalid linked list links");
        debug_assert_eq!((*next).prev().get(), elem_ptr, "invalid linked list links");
        (*prev).next().set(next);
        (*next).prev().set(prev);
    }

    if head.get() == elem_ptr {
        head.set(if next == elem_ptr { ptr::null() } else { next })
    }
}

/// Analyzes where a new element should be inserted to maintain sorted order.
///
/// This function examines the existing sorted linked list and determines the
/// correct insertion point for a new element based on its key value.
///
/// ## Algorithm
/// 1. Check if list is empty → `EmptyHead`
/// 2. Check if element is smaller than head → `SmallerThanHead`  
/// 3. Search backward from tail to find insertion point
/// 4. Compare keys to determine exact position or detect duplicates
///
/// ## Backward Traversal Strategy
/// ```text
/// List: [10] ←→ [20] ←→ [30] ←→ [40]
///        ↑head                  ↑tail
///
/// To insert 25, start from tail (40) and walk backward:
/// 40 > 25? Yes, continue ←
/// 30 > 25? Yes, continue ←
/// 20 < 25? Found position! Insert after 20
/// ```
///
/// Backward traversal is efficient because:
/// - Most insertions happen near the end of the list
/// - We can stop as soon as we find a smaller key
/// - Circular structure guarantees we won't loop infinitely
///
/// # Arguments
/// * `head` - Reference to the list head pointer  
/// * `elem` - Element to analyze for insertion
///
/// # Returns
/// Analysis result indicating insertion strategy
#[cold]
pub(crate) fn link<'a, E: Elem>(head: &'a Cell<*const E>, elem: &E) -> AddToSllResult<'a, E> {
    let old_head = head.get();
    // Case 1: empty head, replace it.
    if old_head.is_null() {
        return AddToSllResult::EmptyHead(head);
    }
    unsafe {
        // Case 2: we are smaller than the head, replace it.
        if elem.key() < (*old_head).key() {
            return AddToSllResult::SmallerThanHead(head);
        }

        // Case 3: loop *backward* until we find insertion place. Because of
        // Case 2, we can't loop beyond the head.
        let mut curr = (*old_head).prev().get();
        loop {
            match (*curr).key().cmp(elem.key()) {
                Ordering::Less => return AddToSllResult::SmallerThanNotHead(curr),
                Ordering::Equal => return AddToSllResult::AlreadyInSll(curr),
                Ordering::Greater => curr = (*curr).prev().get(),
            }
        }
    }
}

/// Adjusts keys of all elements in the list that meet the threshold condition.
///
/// This function traverses the entire circular list once and updates keys
/// that are greater than or equal to the specified threshold.
///
/// ## Use Cases
/// - Text editing: Update character positions after insertions/deletions
/// - Syntax trees: Adjust node offsets when content changes
/// - Version control: Update line numbers after modifications
///
/// ## Algorithm
/// ```text
/// Given list: [10] → [20] → [30] → [40]
/// adjust(from=25, delta=Delta::Add(5))
///
/// Traverse: 10 >= 25? No,  skip
///          20 >= 25? No,  skip  
///          30 >= 25? Yes, 30+5=35
///          40 >= 25? Yes, 40+5=45
///
/// Result:   [10] → [20] → [35] → [45]
/// ```
///
/// # Arguments
/// * `elem` - Any element in the list (traversal starts from here)
/// * `from` - Threshold value for key comparison
/// * `by` - Delta to apply (Add or Subtract)
pub(crate) fn adjust<E: Elem>(elem: &E, from: u32, by: Delta<u32>) {
    let elem_ptr: *const E = elem;

    unsafe {
        let mut curr = elem_ptr;
        loop {
            let mut key = (*curr).key().get();
            if key >= from {
                key += by;
                (*curr).key().set(key);
            }
            curr = (*curr).next().get();
            if curr == elem_ptr {
                break;
            }
        }
    }
}
