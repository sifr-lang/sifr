# wave_psp_d2 Process/Runtime/Platform - Review Pass 1: Implementation Gaps and CPython Parity

## Executive Summary

wave_psp_d2 covers process execution, runtime introspection, and platform information: `os`, `subprocess`, `sys`, `logging`, `platform`, `time`, `timeit`, `env`, `tempfile`, `io`, and `test`. The implementation is functional with all tests passing, but several correctness gaps, traceability inconsistencies, and production risks were identified.

**Status**: Implementation largely complete, tests passing. Review identifies issues requiring remediation.

---

## Test Validation

All test files pass:

```
$ cargo run -q -p sifr -- run demos/wave_psp_d2_process_runtime_platform_demo.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_d2_process_runtime_platform.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_os_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_subprocess_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_sys_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_logging_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_platform_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_time_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_timeit_subset.sifr
Exit: 0

$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_env_subset.sifr
Exit: 0
```

---

## Traceability Gaps

### 1. Traceability Document Missing Implemented APIs

**Severity**: Medium

**Location**: `verification/stdlib/wave_psp_d2_cpython_traceability.md`

**Issue**: The traceability document omits several implemented APIs that are shipped but not documented as part of the wave scope.

| Module | Implemented but NOT in Traceability |
|--------|-------------------------------------|
| `subprocess` | `run_raw()` - exists in `cpython_subprocess_subset.sifr` and `lib/sifr/subprocess.sifr` |
| `sys` | `exit()` - exists in `lib/sifr/sys.sifr` |
| `logging` | `basicConfig()`, `FileHandler`, `Formatter`, `log_info()`, `log_warn()`, `log_error()`, `log_debug()` - all exist in `lib/sifr/logging.sifr` and tested in `cpython_logging_subset.sifr` |
| `platform` | All `platform_*` alias functions (`platform_system`, `platform_arch`, etc.) - exist in `lib/sifr/platform.sifr` and tested in `cpython_platform_subset.sifr` |
| `timeit` | `default_timer()` - exists in `lib/sifr/timeit.sifr` and tested in `cpython_timeit_subset.sifr` |

**Recommendation**: Update the traceability document to include all implemented APIs, or confirm these are intentionally out-of-scope. The current document is misleading as it suggests these APIs are not part of the wave when they are shipped.

---

### 2. Test File Name Inconsistency

**Severity**: Low

**Location**: `crates/sifr/tests/e2e/pass/phase_psp_d2_process_runtime_platform.sifr`

**Issue**: The test file is named `phase_psp_d2_process_runtime_platform.sifr` but the wave is named `wave_psp_d2`. This inconsistency makes it harder to correlate tests with wave documentation.

**Recommendation**: Consider renaming to match wave naming convention, or document the naming rationale.

---

## Correctness Gaps

### 3. subprocess.run() Redundant Error Handling

**Severity**: Low

**Location**: `lib/sifr/subprocess.sifr` lines 32-33

```sifr
except IOError as e:
    raise IOError(e.message)
```

**Issue**: The except block catches an IOError and immediately re-raises a new IOError with the same message. This is redundant and adds no value.

**Impact**: Unnecessary code, slightly increased runtime overhead.

**Recommendation**: Remove the try-except block or propagate the original error correctly:

```sifr
except IOError as e:
    raise e  # or just remove the try-except
```

---

### 4. Silent Error Swallowing in Logging FileHandler

**Severity**: Medium

**Location**: `lib/sifr/logging.sifr` lines 54-62

```sifr
def emit(self, level: str, name: str, msg: str) -> None:
    line: str = self._formatter.format(level, name, msg) + "\n"
    try:
        fh: FileHandle = open(self._path, "a")
        try:
            _: None = fh.write(line)
        except IOError as e2:
            pass  # SILENTLY SWALLOWS ERROR
        fh.close()
    except IOError as e:
        pass  # SILENTLY SWALLOWS ERROR
```

**Issue**: Write errors and file open errors are silently swallowed with `pass`. This means:
- Disk full errors are ignored
- Permission errors are ignored
- Missing directory errors are ignored
- Users have no way to know logging failed

**Traceability claim**: The traceability document claims "missing-path safety" as part of the logging surface. This is implemented, but "safety" here means "doesn't panic" rather than "reports errors to user."

**Impact**: Production users cannot detect logging failures. Silent failures can mask underlying system problems.

**Recommendation**: Either:
1. Return a `Result` type to propagate errors
2. Document this as intentional "fire-and-forget" adapted behavior in the traceability
3. At minimum, log to stderr as a fallback

---

### 5. Silent Error Swallowing in Logger._emit()

**Severity**: Medium

**Location**: `lib/sifr/logging.sifr` lines 81-94

Similar to issue #4, the Logger._emit() method silently swallows all IOErrors when writing to the log file.

---

### 6. time.strftime Epoch 0 Format Difference

**Severity**: Low (may be intentional)

**Location**: `lib/sifr/time.sifr` / `crates/sifr_codegen/src/intrinsics/time.rs`

**Observation**: The demo shows `time.strftime("%Y-%m-%d %H:%M:%S", 0.0)` returns `"1970-01-01 00:00:00"`.

**Potential gap**: This works for epoch 0 (UTC), but the behavior for other epochs depends on local timezone. CPython's `strftime` uses the local timezone by default. Sifr's implementation may behave differently.

**Recommendation**: Verify timezone handling matches CPython across edge cases (DST transitions, negative epochs, etc.) and document any differences in the traceability.

---

## Production Risks

### 7. No Process Termination APIs

**Severity**: Medium

**Location**: `lib/sifr/os.sifr`, `lib/sifr/subprocess.sifr`

**Issue**: The wave provides process creation (`run`, `run_with_input`) but no process termination APIs. CPython's `os` module includes:
- `kill(pid, sig)`
- `terminate()`
- `wait()`

**Current state**: Classified as "unsupported" in the traceability, but there's no explicit waiver explanation.

**Impact**: Users cannot terminate processes they spawn, which is a significant gap for long-running process management.

**Recommendation**: Either:
1. Add process termination APIs in a future wave
2. Document the waiver more explicitly with rationale

---

### 8. subprocess.run_raw Not Fully Tested

**Severity**: Low

**Location**: `cpython_subprocess_subset.sifr`

**Issue**: `run_raw()` is implemented and tested in the subset, but not included in:
- The main integration test `phase_psp_d2_process_runtime_platform.sifr`
- The demo file
- The traceability document

**Impact**: Lower confidence in this API's production readiness.

**Recommendation**: Add `run_raw` to the integration test and demo.

---

## Test Coverage Assessment

### Coverage by Module

| Module | Subset Tests | Integration Test | Demo | Coverage |
|--------|-------------|------------------|------|----------|
| os | `cpython_os_subset.sifr` (15 asserts) | ✓ | ✓ | Good |
| subprocess | `cpython_subprocess_subset.sifr` (4 asserts) | ✓ | ✓ | Good |
| sys | `cpython_sys_subset.sifr` (4 asserts) | ✓ | ✓ | Good |
| logging | `cpython_logging_subset.sifr` (8 asserts) | ✓ | ✓ | Good |
| platform | `cpython_platform_subset.sifr` (8 asserts) | ✓ | ✓ | Good |
| time | `cpython_time_subset.sifr` (9 asserts) | ✓ | ✓ | Good |
| timeit | `cpython_timeit_subset.sifr` (8 asserts) | ✓ | ✓ | Good |
| env | `cpython_env_subset.sifr` | ✓ | ✓ | Good |

**Assessment**: Test coverage is comprehensive. Each module has:
- A dedicated subset test file with bool-vector assertions
- Coverage in the main integration test
- Coverage in the demo file

---

## Ownership and Borrow Safety

**Assessment**: No ownership or borrow safety issues identified.

All modules use proper patterns:
- String cloning with `value + ""` pattern where needed
- Result types for error propagation
- No inappropriate mutability

---

## Regression Assessment

No regressions identified. All existing tests continue to pass. The implementation extends the surface without breaking existing functionality.

---

## Recommendations

### High Priority

1. **Update traceability document** - Add all implemented APIs that are currently missing (run_raw, exit, basicConfig, FileHandler, Formatter, log_*, platform_*, default_timer)

2. **Document logging error behavior** - The silent error swallowing should be explicitly documented as intentional "fire-and-forget" adapted behavior, or changed to propagate errors

### Medium Priority

3. **Fix subprocess.run() error handling** - Remove redundant try-except or propagate correctly

4. **Add process termination to waiver** - Document why process termination is not supported and plans for future

5. **Test run_raw more comprehensively** - Add to integration test and demo

### Low Priority

6. **Consider renaming test file** - Match wave naming convention

7. **Verify timezone handling** - Confirm time.strftime timezone behavior matches CPython

---

## Files Reviewed

- `lib/sifr/os.sifr`
- `lib/sifr/subprocess.sifr`
- `lib/sifr/sys.sifr`
- `lib/sifr/logging.sifr`
- `lib/sifr/platform.sifr`
- `lib/sifr/time.sifr`
- `lib/sifr/timeit.sifr`
- `lib/sifr/env.sifr`
- `lib/sifr/tempfile.sifr`
- `lib/sifr/io.sifr`
- `lib/sifr/test.sifr`
- `verification/stdlib/wave_psp_d2_cpython_traceability.md`
- `demos/wave_psp_d2_process_runtime_platform_demo.sifr`
- `crates/sifr/tests/e2e/pass/phase_psp_d2_process_runtime_platform.sifr`
- `crates/sifr/tests/e2e/pass/cpython_*_subset.sifr` (all d2-related)

---

## Conclusion

wave_psp_d2 provides solid process, runtime, and platform support with comprehensive test coverage. The main issues are:

1. **Traceability inconsistency** - Document doesn't match shipped code
2. **Silent error handling** - Logging silently swallows errors
3. **Minor code quality** - Redundant error handling in subprocess

These should be addressed to ensure the wave is production-ready and the documentation accurately reflects the implementation.
