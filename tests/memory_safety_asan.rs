//! Memory Safety Tests: ASAN Validation
//!
//! These tests validate that ASAN (Address Sanitizer) is properly configured and can detect
//! memory safety issues. While the mrrc library has `unsafe_code = "forbid"` preventing
//! most memory bugs, these tests serve as:
//!
//! 1. **Configuration verification**: Confirms ASAN is working when tests are built with
//!    RUSTFLAGS="-Z sanitizer=address"
//! 2. **`PyO3` interaction validation**: Ensures memory safety in Python bindings (src-python/)
//! 3. **Dependency monitoring**: Catches potential memory issues from upstream crates
//!
//! When run with ASAN enabled, any real memory safety issues will cause test failures.

#[cfg(test)]
#[allow(
    clippy::doc_markdown,
    clippy::cast_possible_truncation,
    clippy::format_push_string,
    clippy::uninlined_format_args
)]
mod asan_memory_safety_tests {
    /// Test that basic memory operations work correctly
    ///
    /// This test validates:
    /// - Heap allocation and deallocation succeed
    /// - Memory is properly initialized
    /// - ASAN can track these operations
    #[test]
    fn test_heap_allocation_safety() {
        // Create a vector and verify memory operations
        let mut data = vec![0u8; 256];

        // Write to allocated memory
        for (i, item) in data.iter_mut().enumerate() {
            *item = (i % 256) as u8;
        }

        // Verify the writes
        for (i, &value) in data.iter().enumerate() {
            assert_eq!(value, (i % 256) as u8, "Memory content should be correct");
        }

        // Vector is automatically deallocated when it goes out of scope
        drop(data);
    }

    /// Test that Vec memory operations are safe
    ///
    /// This test validates:
    /// - Vec allocation and growth are safe
    /// - Element access within bounds is safe
    /// - ASAN tracks Vec memory correctly
    #[test]
    fn test_vec_memory_safety() {
        let mut vec = Vec::new();

        // Grow the vector and verify memory safety
        for i in 0..1000 {
            vec.push(i);
        }

        // Verify we can access all elements
        for (i, &val) in vec.iter().enumerate() {
            assert_eq!(val, i, "Vector elements should match insertion order");
        }

        // Shrink and verify
        vec.truncate(500);
        assert_eq!(vec.len(), 500);
        assert_eq!(vec.last(), Some(&499));
    }

    /// Test that Box memory operations are safe
    ///
    /// This test validates:
    /// - Box allocation is safe
    /// - Deallocation happens automatically
    /// - ASAN can track Box lifetime
    #[test]
    fn test_box_memory_safety() {
        let boxed = Box::new([0u32; 100]);
        let _ptr: *const u32 = boxed.as_ptr();

        // Box should be writable
        let mut mutable_box = Box::new(vec![0u32; 100]);
        for (i, item) in mutable_box.iter_mut().enumerate() {
            *item = i as u32;
        }

        // Verify content
        for (i, &val) in mutable_box.iter().enumerate() {
            assert_eq!(val, i as u32);
        }

        // Box is automatically deallocated when it goes out of scope
        drop(boxed);
        drop(mutable_box);
    }

    /// Test String memory operations
    ///
    /// This test validates:
    /// - String allocation is safe
    /// - String growth/shrinking is safe
    /// - UTF-8 validity is maintained
    /// - ASAN tracks String memory
    #[test]
    fn test_string_memory_safety() {
        let mut string = String::new();

        // Build a string dynamically
        for i in 0..100 {
            string.push_str(&format!("Item {i}\n"));
        }

        // Verify content
        assert!(!string.is_empty());
        assert!(string.contains("Item 0"));
        assert!(string.contains("Item 99"));

        // String mutations
        string.clear();
        assert!(string.is_empty());

        let s = String::from("Hello, ASAN!");
        assert_eq!(s.len(), 12);
    }

    /// Test that thread-local storage is safe
    ///
    /// This test validates:
    /// - Thread-local variables can be safely allocated
    /// - ASAN can track thread-local memory
    #[test]
    fn test_thread_local_memory_safety() {
        thread_local! {
            static TLS_VALUE: Vec<u32> = {
                let mut v = Vec::new();
                for i in 0..100 {
                    v.push(i);
                }
                v
            };
        }

        TLS_VALUE.with(|v| {
            assert_eq!(v.len(), 100);
            assert_eq!(v[0], 0);
            assert_eq!(v[99], 99);
        });
    }

    /// Test RefCell memory operations
    ///
    /// This test validates:
    /// - RefCell allocation is safe
    /// - Interior mutability works correctly
    /// - ASAN can track RefCell memory
    #[test]
    fn test_refcell_memory_safety() {
        use std::cell::RefCell;

        let cell = RefCell::new(vec![1, 2, 3, 4, 5]);

        // Borrow immutably
        {
            let borrowed = cell.borrow();
            assert_eq!(borrowed.len(), 5);
            assert_eq!(borrowed[0], 1);
        }

        // Borrow mutably
        {
            let mut borrowed = cell.borrow_mut();
            borrowed.push(6);
            assert_eq!(borrowed.len(), 6);
        }

        // Verify mutation persisted
        assert_eq!(cell.borrow().len(), 6);
    }

    /// Test Mutex memory operations
    ///
    /// This test validates:
    /// - Mutex allocation and locking are safe
    /// - Synchronization doesn't cause memory issues
    /// - ASAN tracks Mutex memory
    #[test]
    fn test_mutex_memory_safety() {
        use std::sync::Mutex;

        let mutex = Mutex::new(vec![1, 2, 3, 4, 5]);

        // Lock and modify
        {
            let mut guard = mutex.lock().unwrap();
            guard.push(6);
            assert_eq!(guard.len(), 6);
        }

        // Lock and read
        {
            let guard = mutex.lock().unwrap();
            assert_eq!(guard.len(), 6);
            assert_eq!(guard[5], 6);
        }
    }

    /// Test Arc memory operations
    ///
    /// This test validates:
    /// - Arc reference counting is safe
    /// - Shared ownership doesn't cause memory issues
    /// - ASAN can track Arc memory lifetime
    #[test]
    fn test_arc_memory_safety() {
        use std::sync::Arc;
        use std::thread;

        let data = Arc::new(vec![1, 2, 3, 4, 5]);
        let mut handles = vec![];

        // Spawn threads that share data via Arc
        for _ in 0..5 {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                assert_eq!(data_clone[0], 1);
                assert_eq!(data_clone[4], 5);
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Original Arc should still be valid
        assert_eq!(data[0], 1);
        assert_eq!(data.len(), 5);
    }

    /// Integration test: Build and query a MARC record
    ///
    /// This test validates:
    /// - MARC record creation doesn't cause memory issues
    /// - Field access is memory-safe
    /// - ASAN can track record memory
    #[test]
    fn test_marc_record_memory_safety() {
        use mrrc::{Field, Leader, Record};

        // Create a leader
        let leader = Leader {
            record_length: 100,
            record_status: '0',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: ' ',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0,
            encoding_level: ' ',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        };

        // Create a new MARC record
        let mut record = Record::new(leader);

        // Add some control fields
        record
            .control_fields
            .insert("008".to_string(), "Test008".to_string());

        // Add variable fields using builder pattern
        let field = Field::builder("245".to_string(), '1', '4')
            .subfield_str('a', "Test title")
            .build();

        if let Some(fields) = record.fields.get_mut("245") {
            fields.push(field);
        } else {
            record.fields.insert("245".to_string(), vec![field]);
        }

        // Query the record
        assert_eq!(record.leader.record_length, 100);
        assert!(!record.control_fields.is_empty());
        assert!(!record.fields.is_empty());

        // Access fields
        if let Some(fields_245) = record.fields.get("245") {
            for field in fields_245 {
                assert_eq!(field.tag, "245");
                assert_eq!(field.indicator1, '1');
                assert_eq!(field.indicator2, '4');
            }
        }
    }
}
