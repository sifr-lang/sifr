# wave_psp_d1 Review Pass 1

## Executive Summary

wave_psp_d1 (filesystem, pathlib, glob, shutil, tempfile, gzip, zipfile) has **substantial implementation** in the codebase. However, this review identifies several CPython parity gaps, root-cause issues, and production-grade concerns that should be addressed before considering this wave complete.

**Recommendation**: The wave is **not production-ready** for CPython parity closure. The implementation provides a functional subset but has significant gaps compared to Python's stdlib surfaces.

---

## Implementation Status

### What EXISTS

1. **Core Intrinsics** (`crates/sifr_hir/src/stdlib/sys_fs.rs`):
   - File I/O: `read_text`, `write_text`, `read_lines`, `append_text`, `open_file`, file handle operations
   - Directory ops: `listdir`, `mkdir`, `rmdir`, `iterdir`, `walk_dir`, `chdir`, `getcwd`
   - Path operations: `exists`, `is_file`, `is_dir`, `rename`, `copy_file`, `touch`, `resolve_path`
   - Glob: `glob_pattern`, `rglob_pattern`

2. **Compression Intrinsics** (`crates/sifr_hir/src/stdlib/platform_misc.rs`):
   - gzip: `gzip_compress`, `gzip_decompress`
   - zipfile: `zip_create`, `zip_add_file`, `zip_read_file`, `zip_namelist`

3. **Sifr Modules** (`lib/sifr/`):
   - `io.sifr`: FileHandle class
   - `pathlib.sifr`: Path class with 20+ methods
   - `glob.sifr`: glob function
   - `shutil.sifr`: copy, move_file, rmtree, which, disk_usage
   - `tempfile.sifr`: mktemp_path, mkstemp, mkdtemp
   - `gzip.sifr`: compress, decompress
   - `zipfile.sifr`: ZipFile class

4. **Codegen** (`crates/sifr_codegen/src/intrinsics/`):
   - gzip.rs, zipfile.rs full implementations

5. **Tests**:
   - All consolidated tests pass: stdlib_pathlib, stdlib_shutil, stdlib_tempfile, stdlib_io, stdlib_glob, stdlib_gzip, stdlib_zipfile
   - CPython subset tests exist: cpython_pathlib, cpython_shutil, cpython_tempfile, cpython_io, cpython_glob

---

## Issues Found

### 1. Root-Cause Issue: pathlib Path Type Inconsistency (HIGH)

**Location**: `lib/sifr/pathlib.sifr`

**Problem**: Several pathlib methods return `str` instead of `Path` objects, violating Python's pathlib API:

```sifr
# Line 75-76: parent() returns str instead of Path
def parent(self) -> str:
    return dirname(self._path)

# Line 105-106: joinpath() returns str instead of Path
def joinpath(self, child: str) -> str:
    return join_path(self._path, child)

# Lines 126-137: with_name() and with_suffix() return str instead of Path
def with_name(self, name: str) -> str:
    ...
def with_suffix(self, suffix: str) -> str:
    ...
```

**Expected CPython behavior**:
```python
p = Path("/tmp/file.txt")
print(type(p.parent()))        # <class 'pathlib.Path'>
print(type(p.joinpath("sub"))) # <class 'pathlib.Path'>
print(type(p.with_name("x")))  # <class 'pathlib.Path'>
```

**Impact**: Users expecting Python-like pathlib behavior will get string types, leading to:
- Missing Path methods on return values
- Inconsistent type annotations
- API surface divergence

---

### 2. Missing CPython Parity: io Module (MEDIUM)

**Location**: `lib/sifr/io.sifr`

**Missing**:
- `StringIO` class (in-memory text stream)
- `BytesIO` class (in-memory binary stream)
- `BufferedReader`, `BufferedWriter`, `BufferedRandom`
- `TextIOWrapper`
- Builtin `open()` function is not exposed as Python's built-in
- `io.BytesIO` not available
- `io.StringIO` not available
- `io.FileIO` not available
- `io.RawIOBase`, `io.BufferedIOBase`, `io.TextIOBase` ABCs

**Current workaround**: Uses `_sifr.fs` intrinsics wrapped in FileHandle class

**CPython test coverage missing**: No `test_io.py` subset tests exist

---

### 3. Missing CPython Parity: glob Module (MEDIUM)

**Location**: `lib/sifr/glob.sifr`

**Missing**:
- `iglob()` - iterator version that yields paths one at a time (memory efficient for large directories)
- `glob.escape()` - escape special characters in path
- `glob.has_magic()` - check if pattern contains magic characters
- Recursive `**` pattern support is not explicit (relies on `rglob` in pathlib)
- `recursive` parameter (Python 3.6+)

**Current implementation** only does single-level glob with fnmatch patterns.

---

### 4. Missing CPython Parity: shutil Module (MEDIUM)

**Location**: `lib/sifr/shutil.sifr`

**Missing**:
- `copytree()` - recursive directory copy
- `copystat()` - copy file metadata
- `copy2()` - copy with metadata
- `move()` - intelligent move (handles both files and directories)
- `rmtree()` exists but may not handle all edge cases (symlinks, permissions)
- `make_archive()` - create zip/tar archives
- `unpack_archive()` - extract archives
- `chown()` - change ownership
- `get_archive_formats()` - list supported formats
- `get_unpack_formats()` - list supported unpack formats
- Error handling differences for permission errors

---

### 5. Missing CPython Parity: tempfile Module (MEDIUM)

**Location**: `lib/sifr/tempfile.sifr`

**Missing**:
- `NamedTemporaryFile` - file object with name (deleted on close)
- `SpooledTemporaryFile` - spool to disk after size threshold
- `TemporaryDirectory` - context manager for temp directory
- `tempfile.gettempdir()` - returns the default temp directory (different from `gettempdir` which may exist)
- `tempfile.gettempprefix()` - returns the default prefix
- `tempfile.gettempprefixb()` - bytes version
- `mkdtemp` only returns path string, not a context manager

---

### 6. Missing CPython Parity: gzip Module (LOW-MEDIUM)

**Location**: `lib/sifr/gzip.sifr`

**Missing**:
- `GzipFile` class (file-like interface)
- `open()` function (most common usage: `gzip.open(filename, 'rt')`)
- Compression level parameter support
- `compressobj()` for streaming
- `decompressobj()` for streaming

---

### 7. Missing CPython Parity: zipfile Module (MEDIUM)

**Location**: `lib/sifr/zipfile.sifr`

**Missing**:
- `is_zipfile()` - check if file is valid ZIP
- `ZipInfo` class for per-file metadata (timestamps, permissions, etc.)
- `ZipFile` constructor parameters: `mode`, `compression`, `compresslevel`
- Writing to existing archives (append mode)
- `extract()` / `extractall()` - extract files
- `testzip()` - test archive integrity
- `setpassword()` - encrypted archives
- `PyZipFile` class for Python packages
- Exception `BadZipFile`, `LargeZipFile`

---

### 8. Negative-Path Coverage Gaps (MEDIUM)

**Tests only cover basic error paths**:
- Missing file open: ✓ covered
- Invalid mode: ✓ covered
- Missing directory for glob: ✓ covered

**Not covered**:
- Permission denied errors (read-only filesystem, no permission)
- Symlink handling (circular symlinks, broken symlinks)
- Race conditions (file deleted between check and operation)
- Disk full errors
- Path too long errors
- Invalid UTF-8 in paths
- Concurrent access (file locked)
- Edge cases with special files (devices, sockets, FIFO)

---

### 9. pathlib Path Method Gaps (LOW-MEDIUM)

**Location**: `lib/sifr/pathlib.sifr`

**Missing methods**:
- `parts` - tuple of path components
- `anchor` - the drive or root
- `suffixes` - list of suffixes (for `file.tar.gz`)
- `is_relative_to()` - check if path is relative to another
- `is_mount()` - check if path is a mount point
- `is_symlink()` - check if path is a symlink
- `is_socket()` / `is_fifo()` / `is_block_device()` / `is_char_device()`
- `owner()` / `group()` - file ownership
- `stat()` / `lstat()` - file metadata
- `symlink_to()` - create symlink
- `hardlink_to()` - create hardlink
- `absolute()` - return absolute path (similar to resolve but may not resolve symlinks)
- `expanduser()` - expand ~ in path
- `expandvars()` - expand environment variables
- `match()` - glob-style matching

---

### 10. Test Coverage Gaps (MEDIUM)

**Missing traceability artifacts**:
- No `wave_psp_d1_cpython_traceability.md` created
- No `demos/wave_psp_d1_*_demo.sifr` created
- No `phase_psp_d1_*` pass/fail e2e tests

**Incomplete CPython subset tests**:
- No `cpython_gzip_subset.sifr` (only `stdlib_gzip.sifr`)
- No `cpython_zipfile_subset.sifr` (only `stdlib_zipfile.sifr`)

---

## Production-Grade Concerns

### 1. Error Handling Inconsistency

**Example in pathlib.sifr**:
```sifr
def read_text(self) -> Result[str, IOError]:
    return read_text(self._path)
```

But methods like `exists()`, `is_file()`, `is_dir()` return `bool` directly without error handling. This is inconsistent with Python's pathlib where `stat()` can raise but basic queries are infallible.

### 2. Type System Limitations

- No support for `bytes` path arguments (Python 3 on Unix allows bytes paths)
- No `os.PathLike` interface support
- No PathLike generic support for custom path types

### 3. Platform-Specific Behavior

The implementation assumes Unix-style paths (`/` separator). Windows paths are not handled:
- No backslash separator handling
- No drive letter support
- No UNC path support (`//server/share`)

### 4. Unicode Handling

No explicit handling for:
- Unicode normalization
- Non-ASCII path handling on different filesystems
- Surrogate escaping (Python 3's flexible string/bytes path representation)

---

## Recommendations

### Must Fix Before Production

1. **Path type consistency**: Fix pathlib to return `Path` objects for `parent()`, `joinpath()`, `with_name()`, `with_suffix()`

2. **Create traceability artifacts**:
   - `verification/stdlib/wave_psp_d1_cpython_traceability.md`
   - Demo files for each module
   - Phase test files

### Should Fix for Full Parity

3. Add missing modules: StringIO, BytesIO, iglob, glob.escape, NamedTemporaryFile, etc.

4. Expand negative-path testing for permission errors, symlinks, race conditions

5. Add Windows path handling or explicitly document platform limitations

### Consider for Future Waves

6. Full pathlib method parity (stat, symlink, owner, etc.)
7. Archive creation/extraction (shutil.make_archive, unpack_archive)
8. Context manager consistency across all file-handling modules

---

## Validation Evidence

All existing tests pass:
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_gzip.sifr` ✓
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_zipfile.sifr` ✓
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_pathlib_consolidated.sifr` ✓
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_shutil_consolidated.sifr` ✓
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_tempfile_consolidated.sifr` ✓
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_io_consolidated.sifr` ✓
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_glob_consolidated.sifr` ✓

---

## Conclusion

The wave_psp_d1 implementation provides a **functional but incomplete** CPython stdlib surface. The core file operations work correctly, but significant parity gaps exist that would cause friction for users expecting Python-compatible behavior.

**Primary blocker**: The pathlib `Path` type inconsistency (returning `str` instead of `Path`) is a root-cause issue that would affect code correctness for users relying on method chaining.

**Secondary blockers**: Missing traceability artifacts, incomplete negative-path testing, and missing high-usage APIs (iglob, StringIO, NamedTemporaryFile, etc.)
