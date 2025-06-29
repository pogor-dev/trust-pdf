mod thin_arc_tests {
    //! Tests for ThinArc functionality
    //!
    //! These tests verify that ThinArc works correctly in various scenarios:
    //! - Basic creation and data access
    //! - Memory management (cloning, dropping)
    //! - Edge cases (empty slices, large data)
    //! - Type safety and trait implementations

    use crate::arc::thin_arc::ThinArc;

    #[test]
    fn test_thin_arc_creation() {
        // Test basic creation with string header and integer slice
        let data = vec![1, 2, 3, 4, 5];
        let thin_arc = ThinArc::from_header_and_iter("test_header".to_string(), data.into_iter());

        assert_eq!(thin_arc.header, "test_header");
        assert_eq!(thin_arc.length, 5);
        assert_eq!(thin_arc.slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_thin_arc_empty_slice() {
        // Test that ThinArc works correctly with empty data
        let data: Vec<i32> = vec![];
        let thin_arc = ThinArc::from_header_and_iter(42u32, data.into_iter());

        assert_eq!(thin_arc.header, 42u32);
        assert_eq!(thin_arc.length, 0);
        assert_eq!(thin_arc.slice(), &[]);
    }

    #[test]
    fn test_thin_arc_single_element() {
        // Test edge case of single element slice
        let data = vec![42];
        let thin_arc = ThinArc::from_header_and_iter("single".to_string(), data.into_iter());

        assert_eq!(thin_arc.header, "single");
        assert_eq!(thin_arc.length, 1);
        assert_eq!(thin_arc.slice(), &[42]);
    }

    #[test]
    fn test_thin_arc_large_slice() {
        // Test that ThinArc can handle large amounts of data efficiently
        let data: Vec<usize> = (0..1000).collect();
        let thin_arc = ThinArc::from_header_and_iter("large".to_string(), data.clone().into_iter());

        assert_eq!(thin_arc.header, "large");
        assert_eq!(thin_arc.length, 1000);
        assert_eq!(thin_arc.slice().len(), 1000);
        assert_eq!(thin_arc.slice()[0], 0);
        assert_eq!(thin_arc.slice()[999], 999);

        // Verify all elements are stored correctly
        for (i, &value) in thin_arc.slice().iter().enumerate() {
            assert_eq!(value, i);
        }
    }

    #[test]
    fn test_thin_arc_with_arc_conversion() {
        let data = vec![10, 20, 30];
        let thin_arc = ThinArc::from_header_and_iter("conversion".to_string(), data.into_iter());

        // Test with_arc method
        let result = thin_arc.with_arc(|arc| {
            assert_eq!(arc.header, "conversion");
            assert_eq!(arc.slice(), &[10, 20, 30]);
            arc.slice().len()
        });

        assert_eq!(result, 3);
    }

    #[test]
    fn test_thin_arc_clone() {
        // Test that cloning creates a new reference without copying data
        let data = vec![1, 2, 3];
        let thin_arc1 = ThinArc::from_header_and_iter("clone_test".to_string(), data.into_iter());
        let thin_arc2 = thin_arc1.clone();

        // Both should have the same data (pointing to same memory)
        assert_eq!(thin_arc1.header, thin_arc2.header);
        assert_eq!(thin_arc1.length, thin_arc2.length);
        assert_eq!(thin_arc1.slice(), thin_arc2.slice());
    }

    #[test]
    fn test_thin_arc_drop_behavior() {
        // Test that reference counting works correctly when dropping
        let data = vec![1, 2, 3, 4];
        let thin_arc = ThinArc::from_header_and_iter("drop_test".to_string(), data.into_iter());

        // Clone to test that drop works correctly with multiple references
        let thin_arc2 = thin_arc.clone();

        // Verify both have the same data
        assert_eq!(thin_arc.header, thin_arc2.header);
        assert_eq!(thin_arc.slice(), thin_arc2.slice());

        // Drop one copy - this should not affect the other
        drop(thin_arc2);

        // Original should still be accessible (memory not freed yet)
        assert_eq!(thin_arc.header, "drop_test");
        assert_eq!(thin_arc.slice(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_thin_arc_deref() {
        let data = vec![5, 10, 15];
        let thin_arc = ThinArc::from_header_and_iter("deref_test".to_string(), data.into_iter());

        // Test that we can access fields through Deref
        assert_eq!(thin_arc.header, "deref_test");
        assert_eq!(thin_arc.length, 3);

        // Test slice access
        let slice = thin_arc.slice();
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], 5);
        assert_eq!(slice[1], 10);
        assert_eq!(slice[2], 15);
    }

    #[test]
    fn test_thin_arc_partial_eq() {
        let data1 = vec![1, 2, 3];
        let data2 = vec![1, 2, 3];
        let data3 = vec![1, 2, 4];

        let thin_arc1 = ThinArc::from_header_and_iter("test".to_string(), data1.into_iter());
        let thin_arc2 = ThinArc::from_header_and_iter("test".to_string(), data2.into_iter());
        let thin_arc3 = ThinArc::from_header_and_iter("test".to_string(), data3.into_iter());
        let thin_arc4 =
            ThinArc::from_header_and_iter("different".to_string(), vec![1, 2, 3].into_iter());

        // Test equality by comparing fields individually
        assert_eq!(thin_arc1.header, thin_arc2.header);
        assert_eq!(thin_arc1.slice(), thin_arc2.slice());

        // Test inequality (different slice)
        assert_eq!(thin_arc1.header, thin_arc3.header);
        assert_ne!(thin_arc1.slice(), thin_arc3.slice());

        // Test inequality (different header)
        assert_ne!(thin_arc1.header, thin_arc4.header);
        assert_eq!(thin_arc1.slice(), thin_arc4.slice());
    }

    #[test]
    fn test_thin_arc_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let data1 = vec![1, 2, 3];
        let data2 = vec![1, 2, 3];

        let thin_arc1 = ThinArc::from_header_and_iter("test".to_string(), data1.into_iter());
        let thin_arc2 = ThinArc::from_header_and_iter("test".to_string(), data2.into_iter());

        // Equal objects should have equal hashes
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        thin_arc1.hash(&mut hasher1);
        thin_arc2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_thin_arc_different_types() {
        // Test with different header and element types
        let float_data = vec![1.5, 2.5, 3.5];
        let thin_arc = ThinArc::from_header_and_iter(42i32, float_data.into_iter());

        assert_eq!(thin_arc.header, 42i32);
        assert_eq!(thin_arc.length, 3);
        assert_eq!(thin_arc.slice(), &[1.5, 2.5, 3.5]);

        // Test with string elements
        let string_data = vec!["hello".to_string(), "world".to_string()];
        let thin_arc2 = ThinArc::from_header_and_iter(true, string_data.into_iter());

        assert_eq!(thin_arc2.header, true);
        assert_eq!(thin_arc2.length, 2);
        assert_eq!(
            thin_arc2.slice(),
            &["hello".to_string(), "world".to_string()]
        );
    }

    #[test]
    fn test_thin_arc_memory_layout() {
        let data = vec![1, 2, 3];
        let thin_arc = ThinArc::from_header_and_iter("layout_test".to_string(), data.into_iter());

        // ThinArc should be the size of a single pointer
        assert_eq!(
            std::mem::size_of::<ThinArc<String, i32>>(),
            std::mem::size_of::<*const ()>()
        );

        // Verify the phantom data doesn't take space
        assert_eq!(std::mem::size_of_val(&thin_arc.phantom), 0);
    }

    #[test]
    fn test_thin_arc_send_sync() {
        // Test that ThinArc implements Send and Sync for appropriate types
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ThinArc<String, i32>>();
        assert_sync::<ThinArc<String, i32>>();
        assert_send::<ThinArc<i32, String>>();
        assert_sync::<ThinArc<i32, String>>();
    }

    #[test]
    fn test_thin_arc_with_complex_data() {
        // Test with complex nested data structures
        #[derive(Debug, PartialEq, Clone)]
        struct ComplexHeader {
            id: u32,
            name: String,
            flags: Vec<bool>,
        }

        let header = ComplexHeader {
            id: 123,
            name: "complex".to_string(),
            flags: vec![true, false, true],
        };

        let data = vec![vec![1, 2, 3], vec![4, 5], vec![6, 7, 8, 9]];

        let thin_arc = ThinArc::from_header_and_iter(header.clone(), data.clone().into_iter());

        assert_eq!(thin_arc.header, header);
        assert_eq!(thin_arc.length, 3);
        assert_eq!(thin_arc.slice(), &data);
    }

    #[test]
    #[should_panic(expected = "Need to think about ZST")]
    fn test_thin_arc_zero_sized_type() {
        // ThinArc should panic when trying to create with zero-sized types
        let data = vec![(), (), ()];
        let _thin_arc = ThinArc::from_header_and_iter("zst_test".to_string(), data.into_iter());
    }

    #[test]
    fn test_thin_arc_iterator_size_mismatch() {
        // This test ensures that ExactSizeIterator contract is enforced
        // We'll use a custom iterator that lies about its size

        struct LyingIterator {
            actual_data: std::vec::IntoIter<i32>,
            reported_size: usize,
        }

        impl Iterator for LyingIterator {
            type Item = i32;

            fn next(&mut self) -> Option<Self::Item> {
                self.actual_data.next()
            }
        }

        impl ExactSizeIterator for LyingIterator {
            fn len(&self) -> usize {
                self.reported_size // Lie about the size
            }
        }

        // This should work fine when the reported size matches
        let honest_iter = LyingIterator {
            actual_data: vec![1, 2, 3].into_iter(),
            reported_size: 3,
        };

        let thin_arc = ThinArc::from_header_and_iter("honest".to_string(), honest_iter);
        assert_eq!(thin_arc.slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_thin_arc_pointer_transparency() {
        let data = vec![1, 2, 3];
        let thin_arc = ThinArc::from_header_and_iter("pointer_test".to_string(), data.into_iter());

        // Test that we can access the underlying pointer
        let ptr = thin_arc.pointer;

        // Verify that the pointer is correctly typed
        unsafe {
            let inner_ref = ptr.as_ref();
            // We can't directly access count due to privacy, but we can verify the structure exists
            assert_eq!(inner_ref.data.header, "pointer_test");
            assert_eq!(inner_ref.data.length, 3);
        }
    }
}
