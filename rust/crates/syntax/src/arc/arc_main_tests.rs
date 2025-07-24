use crate::arc::arc_inner::ArcInner;
use crate::arc::arc_main::Arc;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

// Helper function to create an Arc for testing
// Note: This creates proper Arc instances that manage their own memory
fn create_test_arc<T>(data: T) -> Arc<T> {
    let inner = Box::new(ArcInner {
        count: AtomicUsize::new(1),
        data,
    });
    let ptr = Box::into_raw(inner);
    Arc {
        pointer: unsafe { NonNull::new_unchecked(ptr) },
        phantom: PhantomData,
    }
}
#[test]
fn test_arc_structure_creation() {
    let arc = create_test_arc(42i32);

    // Verify we can access the inner data
    unsafe {
        let inner = arc.pointer.as_ref();
        assert_eq!(inner.count.load(Ordering::Relaxed), 1);
        assert_eq!(inner.data, 42);
    }
    // Arc will automatically clean itself up when dropped
}

#[test]
fn test_arc_with_string() {
    let test_string = String::from("Hello, Arc!");
    let arc = create_test_arc(test_string.clone());

    unsafe {
        let inner = arc.pointer.as_ref();
        assert_eq!(inner.count.load(Ordering::Relaxed), 1);
        assert_eq!(inner.data, test_string);

        // Arc automatically cleans up memory when dropped
    }
}

#[test]
fn test_arc_with_vec() {
    let test_vec = vec![1, 2, 3, 4, 5];
    let arc = create_test_arc(test_vec.clone());

    unsafe {
        let inner = arc.pointer.as_ref();
        assert_eq!(inner.count.load(Ordering::Relaxed), 1);
        assert_eq!(inner.data, test_vec);

        // Arc automatically cleans up memory when dropped
    }
}

#[test]
fn test_arc_memory_layout() {
    // Test that Arc<T> has the expected memory layout
    use std::mem;

    // Arc should be the size of a pointer due to #[repr(transparent)]
    assert_eq!(mem::size_of::<Arc<i32>>(), mem::size_of::<*const ()>());
    assert_eq!(mem::size_of::<Arc<String>>(), mem::size_of::<*const ()>());

    // Arc should have the same alignment as a pointer
    assert_eq!(mem::align_of::<Arc<i32>>(), mem::align_of::<*const ()>());
    assert_eq!(mem::align_of::<Arc<String>>(), mem::align_of::<*const ()>());
}
#[test]
fn test_arc_pointer_operations() {
    let arc = create_test_arc(100u32);

    // Test that we can get raw pointer
    let raw_ptr = arc.pointer.as_ptr();

    // Test that we can access data through pointer
    unsafe {
        let inner_ref = &*raw_ptr;
        assert_eq!(inner_ref.data, 100u32);
        assert_eq!(inner_ref.count.load(Ordering::Relaxed), 1);

        // Arc automatically cleans up memory when dropped
    }
}

#[test]
fn test_arc_send_sync_properties() {
    // Test that Arc implements Send and Sync for appropriate types
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    // Test with types that are Send + Sync
    assert_send::<Arc<i32>>();
    assert_sync::<Arc<i32>>();

    assert_send::<Arc<String>>();
    assert_sync::<Arc<String>>();

    assert_send::<Arc<Vec<i32>>>();
    assert_sync::<Arc<Vec<i32>>>();
}

#[test]
fn test_arc_phantom_data() {
    let arc = create_test_arc("phantom test");

    // PhantomData should be zero-sized
    assert_eq!(std::mem::size_of_val(&arc.phantom), 0);

    // PhantomData should not affect the total size due to #[repr(transparent)]
    assert_eq!(
        std::mem::size_of::<Arc<&str>>(),
        std::mem::size_of::<NonNull<ArcInner<&str>>>()
    );
}

#[test]
fn test_arc_with_unsized_type() {
    // Test with boxed slice (unsized type)
    let data: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
    let arc = create_test_arc(data);

    unsafe {
        let inner = arc.pointer.as_ref();
        assert_eq!(inner.count.load(Ordering::Relaxed), 1);
        assert_eq!(&*inner.data, &[1, 2, 3][..]);

        // Arc automatically cleans up memory when dropped
    }
}
#[test]
fn test_arc_nonnull_properties() {
    let arc = create_test_arc(42i32);

    // Test that we can create a reference safely
    // NonNull guarantees the pointer is never null
    unsafe {
        let _inner_ref = arc.pointer.as_ref();
        // If we get here without panic, NonNull is working correctly

        // Arc automatically cleans up memory when dropped
    }
}

#[test]
fn test_arc_multiple_instances() {
    // Test creating multiple Arc instances
    let arc1 = create_test_arc(1i32);
    let arc2 = create_test_arc(2i32);
    let arc3 = create_test_arc(3i32);

    unsafe {
        // Verify each has correct data
        assert_eq!(arc1.pointer.as_ref().data, 1);
        assert_eq!(arc2.pointer.as_ref().data, 2);
        assert_eq!(arc3.pointer.as_ref().data, 3);

        // Verify they point to different memory locations
        assert_ne!(arc1.pointer.as_ptr(), arc2.pointer.as_ptr());
        assert_ne!(arc2.pointer.as_ptr(), arc3.pointer.as_ptr());
        assert_ne!(arc1.pointer.as_ptr(), arc3.pointer.as_ptr());

        // Arc automatically cleans up memory when dropped
        // Arc automatically cleans up memory when dropped
        // Arc automatically cleans up memory when dropped
    }
}

#[test]
fn test_arc_clone_behavior() {
    let arc1 = create_test_arc(42i32);
    let arc2 = arc1.clone();

    unsafe {
        // Verify both point to the same memory
        assert_eq!(arc1.pointer.as_ptr(), arc2.pointer.as_ptr());

        // Verify reference count increased
        let count = arc1.pointer.as_ref().count.load(Ordering::Relaxed);
        assert_eq!(count, 2);

        // Arc automatically cleans up memory when dropped
        // Arc automatically cleans up memory when dropped
    }
}

#[test]
fn test_arc_ptr_eq() {
    let arc1 = create_test_arc(42i32);
    let arc2 = arc1.clone();
    let arc3 = create_test_arc(42i32);

    // arc1 and arc2 should point to the same allocation
    assert!(Arc::ptr_eq(&arc1, &arc2));

    // arc1 and arc3 should point to different allocations
    assert!(!Arc::ptr_eq(&arc1, &arc3));
}

#[test]
fn test_arc_is_unique() {
    let arc1 = create_test_arc(42i32);

    // Should be unique initially
    assert!(arc1.is_unique());

    let arc2 = arc1.clone();

    // Should not be unique after clone
    assert!(!arc1.is_unique());
    assert!(!arc2.is_unique());

    drop(arc2);

    // Should be unique again after dropping the clone
    assert!(arc1.is_unique());
}

#[test]
fn test_arc_get_mut() {
    let mut arc1 = create_test_arc(42i32);

    // Should be able to get mutable reference when unique
    let data_ref = Arc::get_mut(&mut arc1);
    assert!(data_ref.is_some());
    *data_ref.unwrap() = 100;

    let _arc2 = arc1.clone();

    // Should not be able to get mutable reference when not unique
    let data_ref = Arc::get_mut(&mut arc1);
    assert!(data_ref.is_none());
}

#[test]
fn test_arc_deref() {
    let arc = create_test_arc("hello".to_string());

    // Test Deref functionality
    assert_eq!(*arc, "hello".to_string());
    assert_eq!(arc.len(), 5);
    assert_eq!(arc.chars().count(), 5);
}

#[test]
fn test_arc_drop_behavior() {
    let arc1 = create_test_arc(vec![1, 2, 3]);
    let pointer = arc1.pointer.as_ptr();

    unsafe {
        // Initial reference count should be 1
        assert_eq!((*pointer).count.load(Ordering::Relaxed), 1);
    }

    let _arc2 = arc1.clone();

    unsafe {
        // Reference count should be 2 after clone
        assert_eq!((*pointer).count.load(Ordering::Relaxed), 2);
    }

    drop(arc1);

    unsafe {
        // Reference count should be 1 after dropping one Arc
        assert_eq!((*pointer).count.load(Ordering::Relaxed), 1);

        // Arc automatically cleans up memory when dropped
    }
}

#[test]
fn test_arc_partial_eq() {
    let arc1 = create_test_arc(42i32);
    let arc2 = arc1.clone();
    let arc3 = create_test_arc(42i32);
    let arc4 = create_test_arc(100i32);

    // Same pointer should be equal
    assert!(arc1 == arc2);

    // Different pointers with same value should be equal
    assert!(arc1 == arc3);

    // Different values should not be equal
    assert!(arc1 != arc4);
}

#[test]
fn test_arc_partial_ord() {
    let arc1 = create_test_arc(10i32);
    let arc2 = create_test_arc(20i32);
    let arc3 = create_test_arc(10i32);

    // Test ordering
    assert!(arc1 < arc2);
    assert!(arc2 > arc1);
    assert!(arc1 <= arc2);
    assert!(arc2 >= arc1);
    assert!(arc1 <= arc3);
    assert!(arc1 >= arc3);

    use std::cmp::Ordering;
    assert_eq!(arc1.partial_cmp(&arc2), Some(Ordering::Less));
    assert_eq!(arc2.partial_cmp(&arc1), Some(Ordering::Greater));
    assert_eq!(arc1.partial_cmp(&arc3), Some(Ordering::Equal));
}

#[test]
fn test_arc_ord() {
    let arc1 = create_test_arc(10i32);
    let arc2 = create_test_arc(20i32);
    let arc3 = create_test_arc(10i32);

    use std::cmp::Ordering;
    assert_eq!(arc1.cmp(&arc2), Ordering::Less);
    assert_eq!(arc2.cmp(&arc1), Ordering::Greater);
    assert_eq!(arc1.cmp(&arc3), Ordering::Equal);
}

#[test]
fn test_arc_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let arc1 = create_test_arc("test".to_string());
    let arc2 = create_test_arc("test".to_string());
    let arc3 = create_test_arc("different".to_string());

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    let mut hasher3 = DefaultHasher::new();

    arc1.hash(&mut hasher1);
    arc2.hash(&mut hasher2);
    arc3.hash(&mut hasher3);

    // Equal values should have equal hashes
    assert_eq!(hasher1.finish(), hasher2.finish());

    // Different values should (usually) have different hashes
    assert_ne!(hasher1.finish(), hasher3.finish());
}

#[test]
fn test_arc_with_zero_sized_type() {
    let arc = create_test_arc(());

    // Verify we can access the ZST
    assert_eq!(*arc, ());
}

#[test]
fn test_arc_complex_type() {
    #[derive(Debug, PartialEq, Clone)]
    struct ComplexType {
        id: u32,
        name: String,
        data: Vec<i32>,
    }

    let complex = ComplexType {
        id: 123,
        name: "test".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };

    let arc = create_test_arc(complex.clone());

    // Verify all fields are accessible
    assert_eq!(arc.id, 123);
    assert_eq!(arc.name, "test");
    assert_eq!(arc.data, vec![1, 2, 3, 4, 5]);
}
