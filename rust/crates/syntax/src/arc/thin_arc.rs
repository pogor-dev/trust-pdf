//! # ThinArc: A Memory-Efficient Reference-Counted Smart Pointer
//!
//! This module provides `ThinArc`, a specialized version of Rust's `Arc` (Atomically Reference Counted)
//! smart pointer that is optimized for storing a header value alongside a dynamically-sized slice.
//!
//! ## What is Arc and Why Do We Need It?
//!
//! In Rust, `Arc` (Atomically Reference Counted) is a smart pointer that allows multiple owners
//! to share the same data safely across threads. Think of it like a shared ownership system:
//! - Multiple variables can "own" the same data
//! - The data is automatically cleaned up when the last owner is dropped
//! - It's thread-safe (multiple threads can access it simultaneously)
//!
//! ## The Problem with Regular Arc for Dynamic Data
//!
//! When you have dynamic data like `Arc<(Header, Vec<T>)>`, Rust stores this as a "fat pointer"
//! - it needs to store both the memory address AND the length of the vector. This takes up more
//!   memory on the stack (16 bytes instead of 8 bytes on 64-bit systems).
//!
//! ## How ThinArc Solves This
//!
//! `ThinArc` stores the length information directly in the heap allocation alongside the data,
//! so the pointer on the stack only needs to store the memory address (8 bytes on 64-bit systems).
//! This makes it "thin" - hence the name.
//!
//! ## When to Use ThinArc
//!
//! - When you need to store many Arc pointers and want to save memory
//! - When interfacing with C code (FFI) that expects simple pointers
//! - When you have a header + slice pattern that you want to share efficiently
//!
//! ## Example Usage
//!
//! ```ignore
//! use crate::arc::thin_arc::ThinArc;
//!
//! // Create a ThinArc with a string header and numeric data
//! let numbers = vec![1, 2, 3, 4, 5];
//! let thin_arc = ThinArc::from_header_and_iter("my_data".to_string(), numbers.into_iter());
//!
//! // Access the header and slice
//! println!("Header: {}", thin_arc.header);  // "my_data"
//! println!("Data: {:?}", thin_arc.slice()); // [1, 2, 3, 4, 5]
//!
//! // Clone creates another reference to the same data (no copying!)
//! let another_ref = thin_arc.clone();
//! // Both `thin_arc` and `another_ref` point to the same memory
//! ```
//!
//! ## Real-World Example: File System Cache
//!
//! Here's a practical example of how you might use `ThinArc` in a file system cache:
//!
//! ```ignore
//! use crate::arc::thin_arc::ThinArc;
//! use std::collections::HashMap;
//!
//! // File metadata that we want to store with the file contents
//! #[derive(Debug, Clone, PartialEq)]
//! struct FileMetadata {
//!     path: String,
//!     size: u64,
//!     modified: u64, // timestamp
//! }
//!
//! // Our file cache entry
//! type CachedFile = ThinArc<FileMetadata, u8>;
//!
//! // A simple file cache
//! struct FileCache {
//!     cache: HashMap<String, CachedFile>,
//! }
//!
//! impl FileCache {
//!     fn new() -> Self {
//!         Self { cache: HashMap::new() }
//!     }
//!
//!     fn store_file(&mut self, path: String, contents: Vec<u8>) {
//!         let metadata = FileMetadata {
//!             size: contents.len() as u64,
//!             modified: 1640995200, // example timestamp
//!             path: path.clone(),
//!         };
//!
//!         let cached_file = ThinArc::from_header_and_iter(metadata, contents.into_iter());
//!         self.cache.insert(path, cached_file);
//!     }
//!
//!     fn get_file(&self, path: &str) -> Option<CachedFile> {
//!         self.cache.get(path).cloned() // This is cheap - just increments ref count!
//!     }
//! }
//!
//! // Usage example
//! let mut cache = FileCache::new();
//!
//! // Store a file (this allocates memory once)
//! cache.store_file("config.txt".to_string(), b"debug=true\nport=8080".to_vec());
//!
//! // Get the file multiple times (no copying, just reference counting)
//! let file1 = cache.get_file("config.txt").unwrap();
//! let file2 = cache.get_file("config.txt").unwrap();
//!
//! // Both point to the same data in memory
//! assert_eq!(file1.header.path, "config.txt");
//! assert_eq!(file2.slice().len(), 18); // length of our config data
//! ```

use std::{
    alloc::{self, Layout},
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::{self, ManuallyDrop, offset_of},
    ops::Deref,
    ptr,
    sync::atomic,
};

use crate::arc::{arc_inner::ArcInner, arc_main::Arc, header_slice::HeaderSlice, thin_to_thick};

/// A memory-efficient reference-counted smart pointer for header + slice data.
///
/// `ThinArc<H, T>` is a specialized version of `Arc` that efficiently stores a header value
/// of type `H` alongside a dynamically-sized slice of elements of type `T`. Unlike a regular
/// `Arc<(H, Vec<T>)>`, `ThinArc` uses only 8 bytes on the stack (on 64-bit systems) by storing
/// the slice length in the heap allocation rather than in the pointer itself.
///
/// ## Key Benefits
///
/// - **Memory Efficient**: Uses only one pointer-sized field on the stack
/// - **Thread Safe**: Multiple threads can safely share the same data
/// - **Zero-Copy Cloning**: Cloning increments a reference count, no data copying
/// - **Automatic Cleanup**: Data is freed when the last reference is dropped
///
/// ## Memory Layout
///
/// ```text
/// Stack:           Heap:
/// ┌─────────────┐  ┌─────────────┐
/// │   pointer   │─▶│ ref_count   │
/// └─────────────┘  │ header: H   │
///                  │ length: u32 │
///                  │ data: [T]   │
///                  └─────────────┘
/// ```
///
/// ## Example
///
/// ```ignore
/// # use crate::arc::thin_arc::ThinArc;
/// // Store file metadata with the file contents
/// let file_contents = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello" in bytes
/// let file_data = ThinArc::from_header_and_iter(
///     "hello.txt".to_string(),
///     file_contents.into_iter()
/// );
///
/// println!("Filename: {}", file_data.header);
/// println!("Size: {} bytes", file_data.length);
/// println!("First byte: 0x{:02x}", file_data.slice()[0]);
///
/// // Share the data with another part of your program
/// let shared_data = file_data.clone(); // No copying, just increment reference count
/// ```
///
/// ## Type Parameters
///
/// - `H`: The type of the header data (can be any type: `String`, `u32`, custom structs, etc.)
/// - `T`: The type of elements in the slice (must not be a zero-sized type)
#[repr(transparent)]
pub(crate) struct ThinArc<H, T> {
    /// Pointer to the heap-allocated data containing reference count, header, and slice.
    /// This is the only field, making ThinArc exactly one pointer in size.
    pub(super) pointer: ptr::NonNull<ArcInner<HeaderSlice<H, [T; 0]>>>,

    /// Zero-sized marker that tells Rust about our ownership of H and T types.
    /// This doesn't take up any space but helps the compiler with type checking.
    pub(super) phantom: PhantomData<(H, T)>,
}

unsafe impl<H: Sync + Send, T: Sync + Send> Send for ThinArc<H, T> {}
unsafe impl<H: Sync + Send, T: Sync + Send> Sync for ThinArc<H, T> {}

impl<H, T> ThinArc<H, T> {
    /// Temporarily converts this `ThinArc` into a regular `Arc` for advanced operations.
    ///
    /// This method allows you to perform operations that require a full `Arc` without
    /// changing the reference count. The provided closure receives a temporary `Arc`
    /// that points to the same data.
    ///
    /// # Parameters
    /// - `f`: A closure that receives the temporary `Arc` and returns a value
    ///
    /// # Returns
    /// Whatever the closure returns
    ///
    /// # Example
    /// ```ignore
    /// # use crate::arc::thin_arc::ThinArc;
    /// let thin_arc = ThinArc::from_header_and_iter("test".to_string(), vec![1, 2, 3].into_iter());
    ///
    /// let result = thin_arc.with_arc(|arc| {
    ///     // Now we can use arc-specific methods
    ///     format!("Header: {}, Length: {}", arc.header, arc.length)
    /// });
    /// ```
    #[inline]
    pub(crate) fn with_arc<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&Arc<HeaderSlice<H, [T]>>) -> U,
    {
        // Synthesize transient Arc, which never touches the refcount of the ArcInner.
        let transient = unsafe {
            ManuallyDrop::new(Arc {
                pointer: ptr::NonNull::new_unchecked(thin_to_thick(self.pointer.as_ptr())),
                phantom: PhantomData,
            })
        };

        // Expose the transient Arc to the callback, which may clone it if it wants.
        f(&transient)
    }

    /// Creates a new `ThinArc` from a header and an iterator of slice elements.
    ///
    /// This is the primary way to create a `ThinArc`. It allocates memory on the heap
    /// to store the reference count, header, length, and all the slice elements together.
    ///
    /// # Parameters
    /// - `header`: The header data of type `H` (can be any type)
    /// - `items`: An iterator that knows its exact size (`ExactSizeIterator`)
    ///
    /// # Returns
    /// A new `ThinArc` containing the header and slice data
    ///
    /// # Panics
    /// - If `T` is a zero-sized type (like `()`)
    /// - If the iterator lies about its size (reports wrong length)
    /// - If memory allocation fails
    /// - If the total size would overflow
    ///
    /// # Example
    /// ```ignore
    /// # use crate::arc::thin_arc::ThinArc;
    /// // Create a ThinArc with a string header and number slice
    /// let numbers = vec![10, 20, 30, 40];
    /// let data = ThinArc::from_header_and_iter("numbers".to_string(), numbers.into_iter());
    ///
    /// assert_eq!(data.header, "numbers");
    /// assert_eq!(data.slice(), &[10, 20, 30, 40]);
    ///
    /// // With different types
    /// let metadata = ("file.txt", 1024u64); // (filename, size)
    /// let bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
    /// let file = ThinArc::from_header_and_iter(metadata, bytes.into_iter());
    /// ```
    ///
    /// # Memory Layout
    /// The method allocates a single block of memory containing:
    /// ```text
    /// [reference_count][header_data][slice_length][element_0][element_1]...[element_n]
    /// ```
    pub(crate) fn from_header_and_iter<I>(header: H, mut items: I) -> Self
    where
        I: Iterator<Item = T> + ExactSizeIterator,
    {
        // Zero-sized types would break our memory layout calculations
        assert_ne!(mem::size_of::<T>(), 0, "Need to think about ZST");

        let num_items = items.len();

        // Calculate where different parts of our data structure will live in memory
        // Memory layout: [ArcInner][HeaderSlice][actual slice data]

        // Offset from start of allocation to the HeaderSlice.data field
        let inner_to_data_offset = offset_of!(ArcInner<HeaderSlice<H, [T; 0]>>, data);
        // Offset from HeaderSlice to where the actual slice data starts
        let data_to_slice_offset = offset_of!(HeaderSlice<H, [T; 0]>, slice);
        // Total offset to where we'll store our T elements
        let slice_offset = inner_to_data_offset + data_to_slice_offset;

        // Calculate how much memory we need for the slice data
        let slice_size = mem::size_of::<T>()
            .checked_mul(num_items)
            .expect("size overflows");
        // Total memory needed (structure + slice data)
        let usable_size = slice_offset
            .checked_add(slice_size)
            .expect("size overflows");

        // Round up to proper alignment (required by Rust's allocator)
        let align = mem::align_of::<ArcInner<HeaderSlice<H, [T; 0]>>>();
        let size = usable_size.wrapping_add(align - 1) & !(align - 1);
        assert!(size >= usable_size, "size overflows");
        let layout = Layout::from_size_align(size, align).expect("invalid layout");

        let ptr: *mut ArcInner<HeaderSlice<H, [T; 0]>>;
        unsafe {
            // Allocate the memory block
            let buffer = alloc::alloc(layout);

            if buffer.is_null() {
                alloc::handle_alloc_error(layout);
            }

            ptr = buffer as *mut _;

            // Initialize reference count to 1 (this ThinArc)
            let count = atomic::AtomicUsize::new(1);

            // Write all the data into the allocated memory
            // Order matters here - we're initializing the memory layout piece by piece

            // 1. Write the reference count
            ptr::write(ptr::addr_of_mut!((*ptr).count), count);
            // 2. Write the header data
            ptr::write(ptr::addr_of_mut!((*ptr).data.header), header);
            // 3. Write the slice length
            ptr::write(ptr::addr_of_mut!((*ptr).data.length), num_items);

            // 4. Write each element of the slice
            if num_items != 0 {
                let mut current = ptr::addr_of_mut!((*ptr).data.slice) as *mut T;
                debug_assert_eq!(current as usize - buffer as usize, slice_offset);

                // Copy elements from iterator into our allocated memory
                for _ in 0..num_items {
                    ptr::write(
                        current,
                        items
                            .next()
                            .expect("ExactSizeIterator over-reported length"),
                    );
                    current = current.offset(1);
                }

                // Verify the iterator gave us exactly the number of items it promised
                assert!(
                    items.next().is_none(),
                    "ExactSizeIterator under-reported length"
                );

                // Sanity check: we should have used exactly the memory we calculated
                debug_assert_eq!(current as *mut u8, buffer.add(usable_size));
            }

            // Final check: iterator should be exhausted
            assert!(
                items.next().is_none(),
                "ExactSizeIterator under-reported length"
            );
        }

        ThinArc {
            pointer: unsafe { ptr::NonNull::new_unchecked(ptr) },
            phantom: PhantomData,
        }
    }
}

impl<H, T> Deref for ThinArc<H, T> {
    type Target = HeaderSlice<H, [T]>;

    /// Provides access to the underlying `HeaderSlice` containing the header and slice data.
    ///
    /// This allows you to use `thin_arc.header` and `thin_arc.slice()` directly,
    /// as if the `ThinArc` was the `HeaderSlice` itself.
    ///
    /// # Example
    /// ```ignore
    /// # use crate::arc::thin_arc::ThinArc;
    /// let data = ThinArc::from_header_and_iter("test".to_string(), vec![1, 2, 3].into_iter());
    ///
    /// // These work because of Deref:
    /// println!("Header: {}", data.header);     // Access header directly
    /// println!("Length: {}", data.length);     // Access length directly  
    /// println!("Slice: {:?}", data.slice());   // Access slice directly
    /// ```
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &(*thin_to_thick(self.pointer.as_ptr())).data }
    }
}

impl<H, T> Clone for ThinArc<H, T> {
    /// Creates a new reference to the same data without copying.
    ///
    /// This is very efficient - it only increments the reference count and returns
    /// a new `ThinArc` pointing to the same heap allocation. No data is copied.
    ///
    /// # Example
    /// ```ignore
    /// # use crate::arc::thin_arc::ThinArc;
    /// let original = ThinArc::from_header_and_iter("shared".to_string(), vec![1, 2, 3].into_iter());
    /// let copy = original.clone(); // Very fast - no data copying!
    ///
    /// // Both point to the same data
    /// assert_eq!(original.slice(), copy.slice());
    /// assert_eq!(original.header, copy.header);
    /// ```
    #[inline]
    fn clone(&self) -> Self {
        ThinArc::with_arc(self, |a| Arc::into_thin(a.clone()))
    }
}

impl<H, T> Drop for ThinArc<H, T> {
    /// Decrements the reference count and frees memory if this was the last reference.
    ///
    /// When you drop a `ThinArc`, it decrements the reference count. If the count
    /// reaches zero (meaning this was the last `ThinArc` pointing to the data),
    /// the memory is freed and the header and all slice elements are properly dropped.
    ///
    /// This happens automatically when the `ThinArc` goes out of scope.
    ///
    /// # Example
    /// ```ignore
    /// # use crate::arc::thin_arc::ThinArc;
    /// {
    ///     let data = ThinArc::from_header_and_iter("temp".to_string(), vec![1, 2, 3].into_iter());
    ///     let copy = data.clone();
    ///     // `data` is dropped here, but memory is not freed because `copy` still exists
    /// }
    /// // `copy` is dropped here, and since it's the last reference, memory is freed
    /// ```
    #[inline]
    fn drop(&mut self) {
        let _ = Arc::from_thin(ThinArc {
            pointer: self.pointer,
            phantom: PhantomData,
        });
    }
}

impl<H: PartialEq, T: PartialEq> PartialEq for ThinArc<H, T> {
    /// Compares two `ThinArc`s for equality by comparing their contents.
    ///
    /// Two `ThinArc`s are equal if both their headers and slices are equal.
    /// This compares the actual data, not the memory addresses.
    ///
    /// # Example
    /// ```ignore
    /// # use crate::arc::thin_arc::ThinArc;
    /// let data1 = ThinArc::from_header_and_iter("test".to_string(), vec![1, 2, 3].into_iter());
    /// let data2 = ThinArc::from_header_and_iter("test".to_string(), vec![1, 2, 3].into_iter());
    /// let data3 = ThinArc::from_header_and_iter("different".to_string(), vec![1, 2, 3].into_iter());
    ///
    /// assert_eq!(data1, data2); // Same content = equal
    /// assert_ne!(data1, data3); // Different header = not equal
    /// ```
    #[inline]
    fn eq(&self, other: &ThinArc<H, T>) -> bool {
        **self == **other
    }
}

impl<H: Eq, T: Eq> Eq for ThinArc<H, T> {}

impl<H: Hash, T: Hash> Hash for ThinArc<H, T> {
    /// Computes a hash of the `ThinArc`'s contents.
    ///
    /// This hashes both the header and the slice data, allowing `ThinArc` to be
    /// used as a key in `HashMap`, `HashSet`, etc.
    ///
    /// # Example
    /// ```ignore
    /// # use crate::arc::thin_arc::ThinArc;
    /// # use std::collections::HashMap;
    /// let data1 = ThinArc::from_header_and_iter("key1".to_string(), vec![1, 2, 3].into_iter());
    /// let data2 = ThinArc::from_header_and_iter("key2".to_string(), vec![4, 5, 6].into_iter());
    ///
    /// let mut map = HashMap::new();
    /// map.insert(data1, "value1");
    /// map.insert(data2, "value2");
    /// ```
    fn hash<HSR: Hasher>(&self, state: &mut HSR) {
        (**self).hash(state)
    }
}
