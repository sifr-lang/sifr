# Review: `wave_psp_runtime_2` Production-Grade Assessment (Review Pass 2)

**Phase:** `ad-hoc-runtime-and-file-object-parity-expansion`
**Wave:** `wave_psp_runtime_2` (Tempfile and Archive Object Lifecycles)
**Reviewer:** External production-grade review
**Date:** 2026-03-20
**Status:** CONDITIONAL PASS - Issues identified requiring remediation

---

## Executive Summary

The `wave_psp_runtime_2` implementation delivers the core tempfile and zipfile functionality as specified in the wave scope. However, this production-grade review identifies **several correctness and safety issues** that require remediation before the wave can be considered production-ready:

1. **Discarded Result errors** in tempfile cleanup operations
2. **Negative size handling bug** in `ZipReadHandle.read_bytes()`
3. **Incomplete mode validation** in `ZipFile._writable_mode()`
4. **Missing Result checking** in constructors

These issues represent violations of the Sifr safety contract (Result/Option instead of exceptions) and could lead to silent failures in production.

---

## 1. Production-Grade Issues

### 1.1 Tempfile: Discarded Result Errors

**Location:** `lib/sifr/tempfile.sifr`

**Issue 1a: NamedTemporaryFile._cleanup_path (lines 98-104)**
```sifr
def _cleanup_path(self) -> Result[None, IOError]:
    if self._cleaned:
        return None
    if exists(self._path):
        _rm_result: Result[None, IOError] = remove_file(self._path)
    self._cleaned = True
    return None
```

**Problems:**
- The Result from `remove_file()` is discarded (`_rm_result` is unused)
- `_cleaned` is set to `True` regardless of whether removal succeeded
- Returns `Ok(None)` even when the file removal failed

**Risk:** Silent failure - users cannot detect cleanup errors. If `remove_file` fails, the caller has no way to know, violating the Sifr error handling contract.

**Issue 1b: NamedTemporaryFile.close() (lines 106-110)**
```sifr
def close(self) -> Result[None, IOError]:
    self._closed = True
    if self._delete:
        _cleaned_now: Result[None, IOError] = self._cleanup_path()
    return None
```

**Problem:** The Result from `_cleanup_path()` is discarded.

**Issue 1c: NamedTemporaryFile.cleanup() (lines 112-115)**
```sifr
def cleanup(self) -> Result[None, IOError]:
    self._closed = True
    _cleaned_now: Result[None, IOError] = self._cleanup_path()
    return None
```

**Problem:** Same as above - Result discarded.

**Issue 1d: TemporaryDirectory.cleanup() (lines 152-160)**
```sifr
def cleanup(self) -> Result[None, IOError]:
    if self._cleaned:
        self._closed = True
        return None
    if exists(self._path):
        _rm_tree_result: Result[None, IOError] = rmdir_all(self._path)
    self._cleaned = True
    self._closed = True
    return None
```

**Problem:** Result from `rmdir_all()` is discarded.

**Issue 1e: TemporaryDirectory.__init__ (lines 132-140)**
```sifr
def __init__(self, prefix: str = "sifr_tempdir_") -> None:
    candidate: str = mktemp_path(prefix)
    while exists(candidate):
        candidate = mktemp_path(prefix)
    _created_result: Result[None, IOError] = mkdir(candidate)
    # ... rest of initialization
```

**Problem:** Result from `mkdir()` is discarded. If directory creation fails, the object is created with an invalid path.

**Remediation Required:** All Result-returning operations must propagate errors to callers. The `cleanup()` and `close()` methods should return the error if cleanup fails.

---

### 1.2 Zipfile: Negative Size Bug in read_bytes

**Location:** `lib/sifr/zipfile.sifr`, lines 60-74

```sifr
def read_bytes(self, size: int | None = None) -> Result[bytes, IOError]:
    if self._closed:
        raise IOError(_closed_stream_error())

    end: int = len(self._data)
    if size is not None:
        requested_size: int = size
        if requested_size >= 0:
            requested_end: int = self._cursor + requested_size
            if requested_end < end:
                end = requested_end

    out: bytes = self._data[self._cursor:end]
    self._cursor = end
    return out
```

**Problem:** The negative size case is not handled according to CPython semantics. Per Python documentation, `read(-1)` or `read(None)` should read all remaining bytes. The current implementation:

1. When `size < 0`: The condition `requested_size >= 0` is false, so `end` remains `len(self._data)`, which is actually correct for negative sizes
2. However, this is incidental - the logic is confusing and could be misinterpreted

**Additional Issue:** The CPython convention is:
- `read()` or `read(None)` - read all
- `read(-1)` - read all (any negative is treated as "all")
- `read(n)` where n > 0 - read n bytes

The current code handles this correctly by accident (negative sizes skip the size adjustment), but the logic is not explicit and could be fragile.

**Remediation Required:** Add explicit negative size handling with clear comments matching CPython semantics.

---

### 1.3 Zipfile: Mode Validation Issues

**Location:** `lib/sifr/zipfile.sifr`, line 88

```sifr
def _writable_mode(self) -> bool:
    return ("w" in self.mode) or ("a" in self.mode)
```

**Problem:** This simplistic check is overly permissive. For example:
- Mode "rw" would be considered writable (contains "w")
- Mode "rwb" would be considered writable

While this may not affect current usage (modes are controlled by the API), it's a defensive programming concern.

**Remediation Required:** Use explicit mode matching:
```sifr
def _writable_mode(self) -> bool:
    return self.mode == "w" or self.mode == "a" or self.mode == "wb" or self.mode == "ab"
```

---

### 1.4 Zipfile: create() Missing Validation

**Location:** `lib/sifr/zipfile.sifr`, lines 90-91

```sifr
def create(self) -> Result[None, IOError]:
    return zip_create(self.path)
```

**Problem:** The intrinsic `zip_create` creates an empty zip file, but there's no validation that:
- The path is valid
- Parent directory exists
- No race condition between check and create

**Risk:** Low in current usage since paths are controlled, but defensive validation is recommended.

---

## 2. Boundary Safety Analysis

### 2.1 Path Handling

| Surface | Input Validation | Assessment |
|---------|-----------------|------------|
| `NamedTemporaryFile.__init__` | None | ⚠️ Accepts any prefix |
| `TemporaryDirectory.__init__` | None | ⚠️ Accepts any prefix |
| `ZipFile.__init__` | None | ⚠️ Accepts any path |
| `ZipFile.write` | Mode check | ✅ Partial |
| `ZipFile.read` | None | ⚠️ No validation |

**Assessment:** Path validation is minimal. While this matches CPython behavior, the lack of validation could lead to confusing errors in production.

### 2.2 State Machine Correctness

| State | close() | cleanup() | closed property |
|-------|---------|-----------|-----------------|
| Fresh | Sets _closed | Sets _closed, cleans | Returns _closed |
| After close | No-op | No-op (already closed) | Returns true |
| After cleanup | No-op | No-op | Returns true |
| After close, then cleanup | No-op | No-op (already cleaned) | Returns true |

**Assessment:** ✅ State transitions are handled consistently.

### 2.3 Resource Leak Analysis

**Tempfile:**
- `NamedTemporaryFile`: File handle not stored, relies on filesystem - ✅ Safe
- `TemporaryDirectory`: Directory handle not stored - ✅ Safe

**Zipfile:**
- `ZipFile`: No file handles stored in instance - ✅ Safe
- `ZipReadHandle`: Data stored in memory - ✅ Safe

**Assessment:** No resource leak vectors identified.

---

## 3. Correctness Verification

### 3.1 CPython Behavioral Parity

| CPython Behavior | Sifr Implementation | Status |
|-----------------|---------------------|--------|
| `read()` returns all bytes | Returns all bytes | ✅ Correct |
| `read(n)` returns n bytes | Returns n bytes | ✅ Correct |
| `read(-1)` returns all | Works (incidental) | ⚠️ Needs explicit handling |
| `NamedTemporaryFile(delete=True)` deletes on close | Deletes on close | ✅ Correct |
| `NamedTemporaryFile(delete=False)` persists | Persists | ✅ Correct |
| `TemporaryDirectory` cleanup on exit | Cleanup on exit | ✅ Correct |

### 3.2 Architecture Lock Compliance

| Requirement | Implementation | Status |
|------------|----------------|--------|
| First-class `bytes` type | Uses `bytes` | ✅ Compliant |
| RAII scope-exit cleanup | `__enter__`/`__exit__` | ✅ Compliant |
| Result error handling | Returns `Result[T, IOError]` | ⚠️ Partially violated (discarded Results) |
| No user panics | No `panic!` in user path | ✅ Compliant |

---

## 4. Security Considerations

### 4.1 Race Conditions

**Issue:** `NamedTemporaryFile.__init__` (lines 80-83)
```sifr
candidate: str = mktemp_path(prefix)
while exists(candidate):
    candidate = mktemp_path(prefix)
_created_result: Result[None, IOError] = write_text(candidate, "")
```

**Problem:** TOCTOU (time-of-check to time-of-use) race condition:
1. Check if path exists
2. Create path

Between check and creation, another process could create the same path. However, the collision retry provides reasonable protection.

**Mitigation:** The retry loop (up to collision) provides adequate protection for most use cases.

### 4.2 Path Traversal

**Assessment:** No path traversal vulnerabilities identified. All paths are user-provided and used directly without manipulation.

### 4.3 Sensitive Data

**Assessment:** Temp files are created in system temp directory (`/tmp`). No sensitive data exposure vectors identified beyond standard OS tempfile semantics.

---

## 5. Test Coverage Assessment

### 5.1 Positive Path Coverage

| Scenario | Test Coverage |
|----------|---------------|
| NamedTemporaryFile delete on close | ✅ Covered |
| NamedTemporaryFile persist with delete=False | ✅ Covered |
| TemporaryDirectory cleanup | ✅ Covered |
| ZipFile write/read bytes | ✅ Covered |
| ZipFile write/read text | ✅ Covered |
| ZipReadHandle read | ✅ Deferred (feature not implemented) |

### 5.2 Negative Path Coverage

| Scenario | Test Coverage |
|----------|---------------|
| ZipFile.open() unsupported | ✅ Covered |
| ZipFile.extract() unsupported | ✅ Covered |
| ZipFile.extractall() unsupported | ✅ Covered |
| ZipFile mode validation | ⚠️ Not explicitly tested |

### 5.3 Edge Cases Not Covered

1. ❌ Cleanup failure propagation (current behavior discards error)
2. ❌ Negative size in read_bytes (works but not explicitly tested)
3. ❌ Concurrent access to same tempfile
4. ❌ Disk full scenarios during write

---

## 6. Regression Analysis

### 6.1 Prior Wave Regression

| Wave | Test | Result |
|------|------|--------|
| wave_psp_runtime_0 | Architecture lock test | ✅ Pass |
| wave_psp_runtime_0 | Stream hierarchy demo | ✅ Pass |
| wave_psp_runtime_1 | IO in-memory hierarchy | ✅ Pass |

### 6.2 Known Limitations (Documented)

| Limitation | Status |
|------------|--------|
| `ZipFile.open()` read handle | ✅ Explicit deferred |
| `ZipFile.extract()` | ✅ Explicit deferred |
| `ZipFile.extractall()` | ✅ Explicit deferred |
| `ZipFile.infolist()` | ✅ Explicit deferred |
| `ZipFile.getinfo()` | ✅ Explicit deferred |
| `ZIP_BZIP2` constant | ✅ Not exported |
| `ZipExtFile` | ✅ Not exported |

---

## 7. Required Remediation

### Priority 1: Result Error Propagation (Blocking)

| File | Line | Issue | Remediation |
|------|------|-------|-------------|
| `tempfile.sifr` | 102 | `_rm_result` discarded | Return error from cleanup |
| `tempfile.sifr` | 109 | `_cleaned_now` discarded | Propagate Result |
| `tempfile.sifr` | 114 | `_cleaned_now` discarded | Propagate Result |
| `tempfile.sifr` | 157 | `_rm_tree_result` discarded | Propagate Result |
| `tempfile.sifr` | 84 | `_created_result` discarded | Check and propagate |

### Priority 2: Correctness Fixes (High)

| File | Line | Issue | Remediation |
|------|------|-------|-------------|
| `zipfile.sifr` | 65-70 | Negative size implicit handling | Add explicit negative size handling with comment |
| `zipfile.sifr` | 88 | Overly permissive mode check | Use explicit mode matching |

### Priority 3: Enhancements (Medium)

| File | Line | Issue | Remediation |
|------|------|-------|-------------|
| `tempfile.sifr` | N/A | No path validation | Consider adding basic validation |
| `zipfile.sifr` | N/A | No read validation | Consider adding existence check |

---

## 8. Conclusion

**Assessment:** CONDITIONAL PASS - Production-grade issues identified

The wave_psp_runtime_2 implementation delivers core functionality but contains **5 critical Result-handling violations** that must be fixed before production deployment. These violations represent a departure from Sifr's core safety contract (explicit error handling via Result types).

**Blocking Issues:**
1. Tempfile cleanup errors are silently discarded
2. Constructor creation errors are silently ignored

**Recommended Action:** Fix Priority 1 and Priority 2 issues, then re-validate with the full test suite before declaring production-ready.

---

## 9. Sign-off

- **Review type:** Production-grade assessment
- **Artifacts reviewed:** Implementation (`lib/sifr/tempfile.sifr`, `lib/sifr/zipfile.sifr`), intrinsics (`crates/sifr_codegen/src/intrinsics/zipfile.rs`), fixtures, demos
- **Result:** CONDITIONAL PASS (requires remediation)
- **Remediation required:** Yes - 5 priority 1 issues, 2 priority 2 issues
- **Next step:** Implement remediation, re-run validation, update review status
