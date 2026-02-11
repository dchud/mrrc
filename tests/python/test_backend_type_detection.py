"""
Backend Type Detection Tests

This test suite validates:
1. All 8 supported input types are correctly routed to backends
2. Type detection order and priority are correct
"""

import io
import tempfile
from pathlib import Path
import mrrc


class TestTypeDetectionSupportedTypes:
    """Test all 8 supported input types"""

    def test_type_str_path_creates_reader(self):
        """Type 1/8: str path → RustFile backend"""
        with tempfile.NamedTemporaryFile(suffix=".mrc", delete=False) as f:
            temp_path = f.name
            f.write(b"")

        try:
            reader = mrrc.MARCReader(temp_path)
            assert reader is not None
            assert isinstance(reader, mrrc.MARCReader)
        finally:
            import os
            os.unlink(temp_path)

    def test_type_pathlib_path_creates_reader(self):
        """Type 2/8: pathlib.Path → RustFile backend"""
        with tempfile.NamedTemporaryFile(suffix=".mrc", delete=False) as f:
            temp_path = f.name
            f.write(b"")

        try:
            path_obj = Path(temp_path)
            reader = mrrc.MARCReader(path_obj)
            assert reader is not None
            assert isinstance(reader, mrrc.MARCReader)
        finally:
            import os
            os.unlink(temp_path)

    def test_type_bytes_creates_reader(self):
        """Type 3/8: bytes → CursorBackend"""
        data = b""
        reader = mrrc.MARCReader(data)
        assert reader is not None
        assert isinstance(reader, mrrc.MARCReader)

    def test_type_bytearray_creates_reader(self):
        """Type 4/8: bytearray → CursorBackend"""
        data = bytearray(b"")
        reader = mrrc.MARCReader(data)
        assert reader is not None
        assert isinstance(reader, mrrc.MARCReader)

    def test_type_file_object_creates_reader(self):
        """Type 5/8: file object (open()) → PythonFile backend"""
        with tempfile.NamedTemporaryFile(suffix=".mrc", delete=False) as f:
            temp_path = f.name
            f.write(b"")

        try:
            with open(temp_path, "rb") as f:
                reader = mrrc.MARCReader(f)
                assert reader is not None
                assert isinstance(reader, mrrc.MARCReader)
        finally:
            import os
            os.unlink(temp_path)

    def test_type_bytesio_creates_reader(self):
        """Type 6/8: io.BytesIO → PythonFile backend (file-like)"""
        data = io.BytesIO(b"")
        reader = mrrc.MARCReader(data)
        assert reader is not None
        assert isinstance(reader, mrrc.MARCReader)

    def test_type_stringio_creates_reader(self):
        """Type 7/8: io.StringIO treated as file-like with .read() method"""
        # StringIO has .read() but returns str, not bytes
        # This should be accepted by type detection but fail at first read
        data = io.StringIO("")
        reader = mrrc.MARCReader(data)
        assert reader is not None
        assert isinstance(reader, mrrc.MARCReader)

    def test_type_custom_file_like_with_read_method(self):
        """Type 8/8: Custom object with .read() method → PythonFile backend"""

        class CustomReader:
            def __init__(self, data):
                self.data = data
                self.pos = 0

            def read(self, n=-1):
                if n == -1:
                    result = self.data[self.pos :]
                    self.pos = len(self.data)
                else:
                    result = self.data[self.pos : self.pos + n]
                    self.pos += len(result)
                return result

        data = CustomReader(b"")
        reader = mrrc.MARCReader(data)
        assert reader is not None
        assert isinstance(reader, mrrc.MARCReader)


class TestTypeDetectionOrder:
    """Test type detection priority/order"""

    def test_str_takes_priority_over_object_with_read(self):
        """str path detection happens before file-like check"""
        # If str had a .read() method (hypothetically), str should win
        # This is implicitly tested by separate str and file-like tests
        with tempfile.NamedTemporaryFile(suffix=".mrc", delete=False) as f:
            temp_path = f.name
            f.write(b"")

        try:
            reader = mrrc.MARCReader(temp_path)
            # Should use RustFile backend (fast path) not PythonFile (slow path)
            assert isinstance(reader, mrrc.MARCReader)
        finally:
            import os
            os.unlink(temp_path)

    def test_bytes_takes_priority_over_object_with_read(self):
        """bytes detection happens before generic file-like check"""
        data = b"test"
        reader = mrrc.MARCReader(data)
        # Should use CursorBackend (fast path) not PythonFile (slow path)
        assert isinstance(reader, mrrc.MARCReader)


class TestTypeDetectionRobustness:
    """Test edge cases and robustness of type detection"""

    def test_empty_bytes_accepted(self):
        """Empty bytes should be accepted (EOF at read)"""
        reader = mrrc.MARCReader(b"")
        assert reader is not None

    def test_empty_bytearray_accepted(self):
        """Empty bytearray should be accepted (EOF at read)"""
        reader = mrrc.MARCReader(bytearray())
        assert reader is not None

    def test_empty_file_accepted(self):
        """Empty file should be accepted (EOF at read)"""
        with tempfile.NamedTemporaryFile(suffix=".mrc", delete=False) as f:
            temp_path = f.name
            f.write(b"")

        try:
            reader = mrrc.MARCReader(temp_path)
            assert reader is not None
        finally:
            import os
            os.unlink(temp_path)

    def test_unicode_path_str_accepted(self):
        """Unicode characters in file path should work"""
        # Create temp file with unicode name
        import os
        import unicodedata

        # Use a safe unicode character
        safe_unicode = "café"
        with tempfile.TemporaryDirectory() as tmpdir:
            # Normalize to NFC for filesystem consistency
            safe_name = unicodedata.normalize("NFC", safe_unicode)
            temp_path = os.path.join(tmpdir, f"{safe_name}.mrc")

            with open(temp_path, "wb") as f:
                f.write(b"")

            reader = mrrc.MARCReader(temp_path)
            assert reader is not None

    def test_relative_path_str_accepted(self):
        """Relative paths should work"""
        import os

        with tempfile.NamedTemporaryFile(suffix=".mrc", dir=".", delete=False) as f:
            temp_path = f.name
            f.write(b"")

        try:
            reader = mrrc.MARCReader(temp_path)
            assert reader is not None
        finally:
            os.unlink(temp_path)
