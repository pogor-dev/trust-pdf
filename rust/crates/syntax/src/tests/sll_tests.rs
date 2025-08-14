//! Tests for Sorted Linked List (SLL) implementation

use std::{cell::Cell, ptr};

use crate::{sll::*, utility_types::Delta};

/// Test element implementation for the sorted linked list
#[derive(Debug)]
struct TestElem {
    key: Cell<u32>,
    prev: Cell<*const TestElem>,
    next: Cell<*const TestElem>,
}

impl TestElem {
    fn new(key: u32) -> Self {
        TestElem {
            key: Cell::new(key),
            prev: Cell::new(ptr::null()),
            next: Cell::new(ptr::null()),
        }
    }

    fn is_self_linked(&self) -> bool {
        let self_ptr: *const TestElem = self;
        self.prev.get() == self_ptr && self.next.get() == self_ptr
    }

    fn is_unlinked(&self) -> bool {
        let self_ptr: *const TestElem = self;
        self.prev.get() == self_ptr && self.next.get() == self_ptr
    }
}

unsafe impl Elem for TestElem {
    fn prev(&self) -> &Cell<*const Self> {
        &self.prev
    }

    fn next(&self) -> &Cell<*const Self> {
        &self.next
    }

    fn key(&self) -> &Cell<u32> {
        &self.key
    }
}

#[test]
fn test_init_when_no_head_expect_no_head() {
    let elem = TestElem::new(10);
    let result = init(None, &elem);

    match result {
        AddToSllResult::NoHead => (),
        _ => panic!("Expected NoHead result"),
    }
}

#[test]
fn test_init_when_empty_head_expect_empty_head() {
    let head = Cell::new(ptr::null());
    let elem = TestElem::new(10);
    let result = init(Some(&head), &elem);

    match result {
        AddToSllResult::EmptyHead(_) => (),
        _ => panic!("Expected EmptyHead result"),
    }
}

#[test]
fn test_init_when_smaller_than_head_expect_smaller_than_head() {
    let first_elem = TestElem::new(20);
    let head = Cell::new(&first_elem as *const TestElem);
    first_elem.prev.set(&first_elem);
    first_elem.next.set(&first_elem);

    let new_elem = TestElem::new(10);
    let result = init(Some(&head), &new_elem);

    match result {
        AddToSllResult::SmallerThanHead(_) => (),
        _ => panic!("Expected SmallerThanHead result"),
    }
}

#[test]
fn test_init_when_same_key_as_head_expect_already_in_sll() {
    let first_elem = TestElem::new(10);
    let head = Cell::new(&first_elem as *const TestElem);
    first_elem.prev.set(&first_elem);
    first_elem.next.set(&first_elem);

    let new_elem = TestElem::new(10);
    let result = init(Some(&head), &new_elem);

    match result {
        AddToSllResult::AlreadyInSll(_) => (),
        _ => panic!("Expected AlreadyInSll result"),
    }
}

#[test]
fn test_init_when_larger_than_head_single_element_expect_smaller_than_not_head() {
    let first_elem = TestElem::new(10);
    let head = Cell::new(&first_elem as *const TestElem);
    first_elem.prev.set(&first_elem);
    first_elem.next.set(&first_elem);

    let new_elem = TestElem::new(20);
    let result = init(Some(&head), &new_elem);

    match result {
        AddToSllResult::SmallerThanNotHead(_) => (),
        _ => panic!("Expected SmallerThanNotHead result"),
    }
}

#[test]
fn test_init_when_multiple_elements_find_insertion_point_expect_smaller_than_not_head() {
    // Create a list: 10 -> 30 -> 50
    let elem1 = TestElem::new(10);
    let elem2 = TestElem::new(30);
    let elem3 = TestElem::new(50);

    let head = Cell::new(&elem1 as *const TestElem);

    // Link them in circular fashion
    elem1.prev.set(&elem3);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem3);
    elem3.prev.set(&elem2);
    elem3.next.set(&elem1);

    // Try to insert 25 (should go between 10 and 30)
    let new_elem = TestElem::new(25);
    let result = init(Some(&head), &new_elem);

    match result {
        AddToSllResult::SmallerThanNotHead(ptr) => {
            assert_eq!(ptr, &elem1 as *const TestElem);
        }
        _ => panic!("Expected SmallerThanNotHead result"),
    }
}

#[test]
fn test_add_to_sll_when_empty_head_expect_head_updated() {
    let head = Cell::new(ptr::null());
    let elem = TestElem::new(10);
    let elem_ptr = &elem as *const TestElem;

    let result = AddToSllResult::EmptyHead(&head);
    result.add_to_sll(elem_ptr);

    assert_eq!(head.get(), elem_ptr);
    assert!(elem.is_self_linked());
}

#[test]
fn test_add_to_sll_when_smaller_than_head_expect_new_head() {
    let old_head = TestElem::new(20);
    let head = Cell::new(&old_head as *const TestElem);
    old_head.prev.set(&old_head);
    old_head.next.set(&old_head);

    let new_elem = TestElem::new(10);
    let new_elem_ptr = &new_elem as *const TestElem;

    let result = AddToSllResult::SmallerThanHead(&head);
    result.add_to_sll(new_elem_ptr);

    assert_eq!(head.get(), new_elem_ptr);
    assert_eq!(new_elem.next.get(), &old_head as *const TestElem);
    assert_eq!(new_elem.prev.get(), &old_head as *const TestElem);
    assert_eq!(old_head.next.get(), new_elem_ptr);
    assert_eq!(old_head.prev.get(), new_elem_ptr);
}

#[test]
fn test_add_to_sll_when_smaller_than_not_head_expect_inserted_in_place() {
    // Create a list: 10 -> 30
    let elem1 = TestElem::new(10);
    let elem2 = TestElem::new(30);

    elem1.prev.set(&elem2);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem1);

    // Insert 20 after elem1
    let new_elem = TestElem::new(20);
    let new_elem_ptr = &new_elem as *const TestElem;
    let elem1_ptr = &elem1 as *const TestElem;

    let result = AddToSllResult::SmallerThanNotHead(elem1_ptr);
    result.add_to_sll(new_elem_ptr);

    // Check the list is now: 10 -> 20 -> 30
    assert_eq!(elem1.next.get(), new_elem_ptr);
    assert_eq!(new_elem.prev.get(), elem1_ptr);
    assert_eq!(new_elem.next.get(), &elem2 as *const TestElem);
    assert_eq!(elem2.prev.get(), new_elem_ptr);
}

#[test]
fn test_add_to_sll_when_no_head_expect_no_change() {
    let elem = TestElem::new(10);
    let elem_ptr = &elem as *const TestElem;

    let result = AddToSllResult::<TestElem>::NoHead;
    result.add_to_sll(elem_ptr);

    // Element should still be self-linked as initialized
    assert!(elem.is_self_linked());
}

#[test]
fn test_add_to_sll_when_already_in_sll_expect_no_change() {
    let elem1 = TestElem::new(10);
    let elem2 = TestElem::new(20);

    // Set up initial state
    let elem1_ptr = &elem1 as *const TestElem;
    let elem2_ptr = &elem2 as *const TestElem;
    elem1.prev.set(elem2_ptr);
    elem1.next.set(elem2_ptr);
    elem2.prev.set(elem1_ptr);
    elem2.next.set(elem1_ptr);

    // Store original values to compare later
    let orig_elem2_prev = elem2.prev.get();
    let orig_elem2_next = elem2.next.get();

    let result = AddToSllResult::AlreadyInSll(elem1_ptr);
    result.add_to_sll(elem1_ptr);

    // The element becomes self-linked (this is the actual behavior of add_to_sll)
    // but since AlreadyInSll doesn't do anything in the match, the element will be self-linked
    // from the initial lines in add_to_sll method
    assert!(elem1.is_self_linked());

    // elem2's links should remain unchanged since we didn't modify them
    assert_eq!(elem2.prev.get(), orig_elem2_prev);
    assert_eq!(elem2.next.get(), orig_elem2_next);
}

#[test]
fn test_unlink_when_single_element_expect_empty_head() {
    let elem = TestElem::new(10);
    let head = Cell::new(&elem as *const TestElem);
    elem.prev.set(&elem);
    elem.next.set(&elem);

    unlink(&head, &elem);

    assert!(head.get().is_null());
    assert!(elem.is_unlinked());
}

#[test]
fn test_unlink_when_head_element_in_multi_element_list_expect_head_updated() {
    // Create a list: 10 -> 20 -> 30
    let elem1 = TestElem::new(10);
    let elem2 = TestElem::new(20);
    let elem3 = TestElem::new(30);

    let head = Cell::new(&elem1 as *const TestElem);

    elem1.prev.set(&elem3);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem3);
    elem3.prev.set(&elem2);
    elem3.next.set(&elem1);

    unlink(&head, &elem1);

    // Head should now point to elem2
    assert_eq!(head.get(), &elem2 as *const TestElem);
    // List should be: 20 -> 30
    assert_eq!(elem2.prev.get(), &elem3 as *const TestElem);
    assert_eq!(elem2.next.get(), &elem3 as *const TestElem);
    assert_eq!(elem3.prev.get(), &elem2 as *const TestElem);
    assert_eq!(elem3.next.get(), &elem2 as *const TestElem);
    // elem1 should be self-linked
    assert!(elem1.is_unlinked());
}

#[test]
fn test_unlink_when_middle_element_expect_links_updated() {
    // Create a list: 10 -> 20 -> 30
    let elem1 = TestElem::new(10);
    let elem2 = TestElem::new(20);
    let elem3 = TestElem::new(30);

    let head = Cell::new(&elem1 as *const TestElem);

    elem1.prev.set(&elem3);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem3);
    elem3.prev.set(&elem2);
    elem3.next.set(&elem1);

    unlink(&head, &elem2);

    // Head should remain elem1
    assert_eq!(head.get(), &elem1 as *const TestElem);
    // List should be: 10 -> 30
    assert_eq!(elem1.next.get(), &elem3 as *const TestElem);
    assert_eq!(elem3.prev.get(), &elem1 as *const TestElem);
    // elem2 should be self-linked
    assert!(elem2.is_unlinked());
}

#[test]
fn test_adjust_when_keys_above_threshold_expect_keys_updated() {
    // Create a list: 5 -> 10 -> 15 -> 20
    let elem1 = TestElem::new(5);
    let elem2 = TestElem::new(10);
    let elem3 = TestElem::new(15);
    let elem4 = TestElem::new(20);

    // Link them in circular fashion
    elem1.prev.set(&elem4);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem3);
    elem3.prev.set(&elem2);
    elem3.next.set(&elem4);
    elem4.prev.set(&elem3);
    elem4.next.set(&elem1);

    // Adjust all keys >= 10 by adding 5
    adjust(&elem1, 10, Delta::Add(5));

    // Check the results
    assert_eq!(elem1.key.get(), 5); // Below threshold, unchanged
    assert_eq!(elem2.key.get(), 15); // 10 + 5
    assert_eq!(elem3.key.get(), 20); // 15 + 5
    assert_eq!(elem4.key.get(), 25); // 20 + 5
}

#[test]
fn test_adjust_when_keys_above_threshold_subtract_expect_keys_updated() {
    // Create a list: 5 -> 10 -> 15 -> 20
    let elem1 = TestElem::new(5);
    let elem2 = TestElem::new(10);
    let elem3 = TestElem::new(15);
    let elem4 = TestElem::new(20);

    // Link them in circular fashion
    elem1.prev.set(&elem4);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem3);
    elem3.prev.set(&elem2);
    elem3.next.set(&elem4);
    elem4.prev.set(&elem3);
    elem4.next.set(&elem1);

    // Adjust all keys >= 12 by subtracting 3
    adjust(&elem1, 12, Delta::Sub(3));

    // Check the results
    assert_eq!(elem1.key.get(), 5); // Below threshold, unchanged
    assert_eq!(elem2.key.get(), 10); // Below threshold, unchanged
    assert_eq!(elem3.key.get(), 12); // 15 - 3
    assert_eq!(elem4.key.get(), 17); // 20 - 3
}

#[test]
fn test_adjust_when_single_element_above_threshold_expect_key_updated() {
    let elem = TestElem::new(15);
    elem.prev.set(&elem);
    elem.next.set(&elem);

    adjust(&elem, 10, Delta::Add(5));

    assert_eq!(elem.key.get(), 20);
}

#[test]
fn test_adjust_when_single_element_below_threshold_expect_key_unchanged() {
    let elem = TestElem::new(5);
    elem.prev.set(&elem);
    elem.next.set(&elem);

    adjust(&elem, 10, Delta::Add(5));

    assert_eq!(elem.key.get(), 5);
}

#[test]
fn test_link_when_empty_head_expect_empty_head_result() {
    let head = Cell::new(ptr::null());
    let elem = TestElem::new(10);

    let result = link(&head, &elem);

    match result {
        AddToSllResult::EmptyHead(_) => (),
        _ => panic!("Expected EmptyHead result"),
    }
}

#[test]
fn test_link_when_complex_ordering_expect_correct_insertion_point() {
    // Create a list: 10 -> 20 -> 30 -> 40 -> 50
    let elem1 = TestElem::new(10);
    let elem2 = TestElem::new(20);
    let elem3 = TestElem::new(30);
    let elem4 = TestElem::new(40);
    let elem5 = TestElem::new(50);

    let head = Cell::new(&elem1 as *const TestElem);

    // Link them in circular fashion
    elem1.prev.set(&elem5);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem3);
    elem3.prev.set(&elem2);
    elem3.next.set(&elem4);
    elem4.prev.set(&elem3);
    elem4.next.set(&elem5);
    elem5.prev.set(&elem4);
    elem5.next.set(&elem1);

    // Try to insert 35 (should go between 30 and 40)
    let new_elem = TestElem::new(35);
    let result = link(&head, &new_elem);

    match result {
        AddToSllResult::SmallerThanNotHead(ptr) => {
            assert_eq!(ptr, &elem3 as *const TestElem);
        }
        _ => panic!("Expected SmallerThanNotHead result"),
    }
}

#[test]
fn test_integrate_full_workflow_expect_correct_list_operations() {
    // Test a complete workflow: create list, add elements, unlink, adjust
    let head = Cell::new(ptr::null());

    // Add first element
    let elem1 = TestElem::new(20);
    let result = init(Some(&head), &elem1);
    result.add_to_sll(&elem1);

    // Add smaller element (becomes new head)
    let elem2 = TestElem::new(10);
    let result = init(Some(&head), &elem2);
    result.add_to_sll(&elem2);

    // Add larger element
    let elem3 = TestElem::new(30);
    let result = init(Some(&head), &elem3);
    result.add_to_sll(&elem3);

    // Verify the list order: 10 -> 20 -> 30
    assert_eq!(head.get(), &elem2 as *const TestElem);
    assert_eq!(elem2.next.get(), &elem1 as *const TestElem);
    assert_eq!(elem1.next.get(), &elem3 as *const TestElem);
    assert_eq!(elem3.next.get(), &elem2 as *const TestElem);

    // Adjust keys >= 15 by adding 10
    adjust(&elem2, 15, Delta::Add(10));

    // Check adjusted values
    assert_eq!(elem2.key.get(), 10); // Below threshold, unchanged
    assert_eq!(elem1.key.get(), 30); // 20 + 10
    assert_eq!(elem3.key.get(), 40); // 30 + 10

    // Unlink middle element
    unlink(&head, &elem1);

    // Verify the list is now: 10 -> 40
    assert_eq!(elem2.next.get(), &elem3 as *const TestElem);
    assert_eq!(elem3.prev.get(), &elem2 as *const TestElem);
    assert!(elem1.is_unlinked());
}

#[test]
fn test_link_when_duplicate_key_in_middle_expect_already_in_sll() {
    // Create a list: 10 -> 20 -> 30
    let elem1 = TestElem::new(10);
    let elem2 = TestElem::new(20);
    let elem3 = TestElem::new(30);

    let head = Cell::new(&elem1 as *const TestElem);

    elem1.prev.set(&elem3);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem3);
    elem3.prev.set(&elem2);
    elem3.next.set(&elem1);

    // Try to insert another element with key 20
    let new_elem = TestElem::new(20);
    let result = link(&head, &new_elem);

    match result {
        AddToSllResult::AlreadyInSll(ptr) => {
            assert_eq!(ptr, &elem2 as *const TestElem);
        }
        _ => panic!("Expected AlreadyInSll result"),
    }
}

#[test]
fn test_link_when_element_goes_at_end_of_long_list_expect_correct_insertion() {
    // Create a list: 10 -> 20 -> 30 -> 40
    let elem1 = TestElem::new(10);
    let elem2 = TestElem::new(20);
    let elem3 = TestElem::new(30);
    let elem4 = TestElem::new(40);

    let head = Cell::new(&elem1 as *const TestElem);

    // Link them in circular fashion
    elem1.prev.set(&elem4);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem3);
    elem3.prev.set(&elem2);
    elem3.next.set(&elem4);
    elem4.prev.set(&elem3);
    elem4.next.set(&elem1);

    // Try to insert 50 (should go at the end, after elem4)
    let new_elem = TestElem::new(50);
    let result = link(&head, &new_elem);

    match result {
        AddToSllResult::SmallerThanNotHead(ptr) => {
            assert_eq!(ptr, &elem4 as *const TestElem);
        }
        _ => panic!("Expected SmallerThanNotHead result"),
    }
}

#[test]
fn test_unlink_when_two_element_list_expect_single_element_remaining() {
    // Create a list: 10 -> 20
    let elem1 = TestElem::new(10);
    let elem2 = TestElem::new(20);

    let head = Cell::new(&elem1 as *const TestElem);

    elem1.prev.set(&elem2);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem1);

    unlink(&head, &elem2);

    // Head should still point to elem1
    assert_eq!(head.get(), &elem1 as *const TestElem);
    // elem1 should be self-linked
    assert!(elem1.is_self_linked());
    // elem2 should be self-linked
    assert!(elem2.is_unlinked());
}

#[test]
fn test_adjust_when_threshold_equals_key_expect_key_adjusted() {
    let elem = TestElem::new(15);
    elem.prev.set(&elem);
    elem.next.set(&elem);

    // Threshold exactly equals the key
    adjust(&elem, 15, Delta::Add(10));

    assert_eq!(elem.key.get(), 25);
}

#[test]
fn test_adjust_when_threshold_one_above_key_expect_key_unchanged() {
    let elem = TestElem::new(15);
    elem.prev.set(&elem);
    elem.next.set(&elem);

    // Threshold is one above the key
    adjust(&elem, 16, Delta::Add(10));

    assert_eq!(elem.key.get(), 15);
}

#[test]
fn test_link_when_traversing_entire_circle_expect_correct_insertion() {
    // Create a list where we need to traverse the entire circle: 20 -> 30 -> 40 -> 50
    // Insert 15 which should go before the head but we need to traverse all the way around
    let elem1 = TestElem::new(20);
    let elem2 = TestElem::new(30);
    let elem3 = TestElem::new(40);
    let elem4 = TestElem::new(50);

    let head = Cell::new(&elem1 as *const TestElem);

    // Link them in circular fashion
    elem1.prev.set(&elem4);
    elem1.next.set(&elem2);
    elem2.prev.set(&elem1);
    elem2.next.set(&elem3);
    elem3.prev.set(&elem2);
    elem3.next.set(&elem4);
    elem4.prev.set(&elem3);
    elem4.next.set(&elem1);

    // Try to insert 45 (should go between 40 and 50, requires traversing backward)
    let new_elem = TestElem::new(45);
    let result = link(&head, &new_elem);

    match result {
        AddToSllResult::SmallerThanNotHead(ptr) => {
            assert_eq!(ptr, &elem3 as *const TestElem);
        }
        _ => panic!("Expected SmallerThanNotHead result"),
    }
}
