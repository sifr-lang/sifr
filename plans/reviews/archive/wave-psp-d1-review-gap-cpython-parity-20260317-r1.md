# wave_psp_d1 Review: Implementation Gaps and CPython Parity

**Review Date**: 2026-03-17
**Reviewer**: agent
**Scope**: filesystem, paths, archives (wave_psp_d1)
**Status**: DRAFT

---

## Executive Summary

wave_psp_d1 covers filesystem operations, path manipulation, and archive handling. The implementation is largely functional with all core surfaces working, but several gaps exist between claimed parity and shipped behavior.

**Key Findings**:
- Core functionality works (demo passes, integration tests pass)
- 4 actionable gaps identified
- 2 known limitations (documented in waivers)

---

## Traceability Validation

### ✅ Verified Surfaces (Match Claims)

| Surface | Claimed | Implemented | Status |
|---------|---------|-------------|--------|
| `sifr.io.open` | text/binary reads/writes, context-managed | ✓ `open(path, mode)` with FileHandle class | OK |
| `sifr.pathlib.Path` | exists, is_file, is_dir, read_text, write_text, glob, rglob, iterdir, resolve, with_name, with_suffix | ✓ All methods present | OK |
| `sifr.glob.glob` | wildcard matching, hidden-entry handling, missing-root | ✓ Returns [] for missing root | OK |
| `sifr.shutil` | copy, move_file, rmtree, which, disk_usage | ✓ All present | OK |
| `sifr.tempfile` | mkstemp, mkdtemp | ✓ Implemented with collision safety | OK |
| `sifr.gzip` | compress/decompress | ✓ list[int] interface (waiver noted) | OK |
| `sifr.zipfile` | create/write/read/namelist | ✓ ZipFile class | OK |

---

## Actionable Findings

### 1. HIGH: pathlib.is_absolute() Incomplete for POSIX/Windows

**Location**: `lib/sifr/pathlib.sifr:57-64`

**Issue**: The `is_absolute()` function only checks for "/" prefix:

```sifr
def is_absolute(path: str) -> bool:
    if len(path) == 0:
        return False
    first: str | None = path[0]
    if first is not None:
        if first == "/":
            return True
    return False
```

**Problems**:
- Does NOT handle Windows absolute paths (e.g., `C:\Users\...`)
- Does NOT handle POSIX double-slash host paths (e.g., `//host/share`)
- Does NOT handle UNC paths

**CPython Behavior**:
```python
>>> from pathlib import Path
>>> Path("/tmp").is_absolute()     # True
>>> Path("C:\\Windows").is_absolute()  # True on Windows
>>> Path("//host/share").is_absolute()  # True on Unix
```

**Recommendation**: Expand to handle platform-specific absolute path patterns or add platform detection.

---

### 2. MEDIUM: Missing File Open Modes (r+, w+, a+)

**Location**: `crates/sifr_codegen/src/intrinsics/file_handles.rs:339-383`

**Issue**: The `open()` built-in only supports:
- `"r"`, `"rt"` - read text
- `"w"`, `"wt"` - write text (truncate)
- `"a"`, `"at"` - append text
- `"rb"` - read binary
- `"wb"` - write binary (truncate)
- `"ab"` - append binary

**Missing**: `"r+"`, `"w+"`, `"a+"` (read-write modes)

**CPython Behavior**:
```python
>>> f = open("/tmp/test.txt", "r+")  # Read and write
>>> f.write("hello")
>>> f.read()
```

**Recommendation**: Add support for read-write modes in file_handles.rs build_open_match function.

---

### 3. LOW: walk_dir Intrinsic Exists but Not Tested

**Location**: `crates/sifr_hir/src/stdlib/sys_fs.rs:260-267` (intrinsic defined)

**Issue**: The `_sifr.fs.walk_dir` intrinsic is defined in the HIR but there's no test or public API exposure in `sifr.os` or `sifr.pathlib`.

**Current Usage**: Not exposed in any public module.

**Recommendation**: Either expose via public API (e.g., `sifr.os.walk()`) or document as internal-only.

---

### 4. LOW: Gzip Invalid Data Rejection Not Fully Tested

**Location**: `crates/sifr/tests/e2e/pass/phase_psp_d1_filesystem_paths_archives.sifr:140-145`

**Issue**: The test only attempts decompression of `[1, 2, 3, 4]` which is clearly invalid. CPython's gzip module rejects:
- Truncated gzip streams
- Invalid CRC checksums
- Wrong compression method

**Current Implementation**: The flate2 crate handles some invalid data internally but error messages may differ from CPython.

**Recommendation**: Add test cases for:
- Truncated but parseable gzip headers
- Invalid compression method (should reject with specific error)

---

## Known Limitations (Waivers)

These are documented in the traceability and are intentional:

### 1. Gzip list[int] Interface
- **Status**: Documented waiver
- **Impact**: No file-object API (GzipFile), only string→bytes→string
- **File**: `verification/stdlib/wave_psp_d1_cpython_traceability.md:12`

### 2. Limited pathlib Class Family
- **Status**: Documented waiver
- **Impact**: Single Path class, no PurePath/PosixPath/WindowsPath specialization
- **File**: `verification/stdlib/wave_psp_d1_cpython_traceability.md:20`

---

## Test Coverage Analysis

### E2E Tests Present
- ✅ `phase_psp_d1_filesystem_paths_archives.sifr` - integration test
- ✅ `cpython_pathlib_subset.sifr` - pathlib parity
- ✅ `cpython_glob_subset.sifr` - glob parity
- ✅ `cpython_shutil_subset.sifr` - shutil parity
- ✅ `cpython_tempfile_subset.sifr` - tempfile parity
- ✅ `cpython_gzip_subset.sifr` - gzip parity
- ✅ `cpython_zipfile_subset.sifr` - zipfile parity
- ✅ `cpython_io_subset.sifr` - io parity
- ✅ 4 fail tests for type errors

### Missing Test Coverage
1. Binary file handle operations (read_bytes, write_bytes)
2. walk_dir functionality
3. tempfile missing-parent failure
4. Platform-specific path behavior (is_absolute on Windows)

---

## Recommendations

### Immediate Actions
1. **Fix is_absolute()** - Add Windows/POSIX path handling
2. **Add r+/w/a+ modes** - Complete file mode coverage

### Future Work
1. Expose walk_dir via public API or document as internal
2. Expand gzip invalid-data test coverage
3. Add Windows-specific path tests in CI

---

## Appendix: Files Reviewed

### Implementation
- `crates/sifr_hir/src/stdlib/sys_fs.rs` - HIR intrinsics
- `crates/sifr_codegen/src/intrinsics/file_handles.rs` - File handle lowering
- `crates/sifr_codegen/src/intrinsics/io.rs` - IO lowering
- `crates/sifr_codegen/src/intrinsics/pathlib.rs` - Pathlib lowering
- `crates/sifr_codegen/src/intrinsics/os.rs` - OS lowering
- `crates/sifr_codegen/src/intrinsics/gzip.rs` - Gzip lowering
- `crates/sifr_codegen/src/intrinsics/zipfile.rs` - Zipfile lowering
- `lib/sifr/io.sifr` - Public IO API
- `lib/sifr/pathlib.sifr` - Public pathlib API
- `lib/sifr/glob.sifr` - Public glob API
- `lib/sifr/shutil.sifr` - Public shutil API
- `lib/sifr/tempfile.sifr` - Public tempfile API
- `lib/sifr/gzip.sifr` - Public gzip API
- `lib/sifr/zipfile.sifr` - Public zipfile API

### Tests
- `crates/sifr/tests/e2e/pass/phase_psp_d1_filesystem_paths_archives.sifr`
- `crates/sifr/tests/e2e/pass/cpython_*_subset.sifr` (8 files)
- `crates/sifr/tests/e2e/fail/phase_psp_d1_*.sifr` (4 files)
- `demos/wave_psp_d1_filesystem_paths_archives_demo.sifr`

### Documentation
- `verification/stdlib/wave_psp_d1_cpython_traceability.md`
