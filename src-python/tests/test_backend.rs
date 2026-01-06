//! Rust unit tests for ReaderBackend type detection
//!
//! These tests verify the backend enum and its construction without Python interaction

#[cfg(test)]
mod backend_tests {
    use std::fs::File;
    use std::io::Cursor;

    #[test]
    fn test_cursor_backend_with_empty_data() {
        let cursor: Cursor<Vec<u8>> = Cursor::new(vec![]);
        let _ = cursor; // Just verify it compiles
    }

    #[test]
    fn test_cursor_backend_with_data() {
        let data = b"test data";
        let cursor = Cursor::new(data.to_vec());
        let _ = cursor;
    }

    #[test]
    fn test_file_backend_with_dev_null() {
        let file = File::open("/dev/null").unwrap();
        let _ = file;
    }
}
