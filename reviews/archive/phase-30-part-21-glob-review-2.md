# Phase 30 Part 21: Glob Module Parity Review (Round 2)

## Overview

This is a follow-up review for the glob module (wave_30_1e) focusing on production-quality assessment for the approved subset. This review examines correctness, safety-contract alignment, panic paths, and production blockers.

## Implementation Architecture

The glob functionality has two parallel implementations:

1. **`sifr.glob` module** (`lib/sifr/glob.sifr`)
   - Uses `sifr.fnmatch` for pattern matching (pure Sifr)
   - Exposes `glob(directory, pattern) -> list[str]`
   - Returns empty list for missing directories

2. **`pathlib.Path.glob/rglob`** (`crates/sifr_codegen/src/intrinsics/pathlib.rs`)
   - Uses regex intrinsics (`glob_pattern`, `rglob_pattern`)
   - Exposed as `Path.glob(pattern)` and `Path.rglob(pattern)` methods
   - Returns `Result[list[str], IOError]`

## Approved Scope (from phase30_parity_matrix.md)

| Feature | Status | Classification |
|---------|--------|-----------------|
| `glob(directory, pattern)` with `*`/`?` wildcards | done | parity |
| CPython-style hidden-file filtering | done | parity |
| Missing directory returns empty list | done | parity |
| `recursive=True` / `**` | out of scope | intentional-diff |
| `iglob` (lazy iterator) | out of scope | intentional-diff |
| Character classes `[seq]`, `[!seq]` | out of scope | intentional-diff |

## Critical Issues Found

### Issue 1: `?` Wildcard Not Supported in pathlib.glob ✅ VERIFIED

**Location**: `crates/sifr_codegen/src/intrinsics/pathlib.rs:80-96`

The regex-based implementation in `regex_source_expr()` only handles `*` wildcard by replacing `\\*` with `.*`. The `?` wildcard is not handled.

```rust
fn regex_source_expr(pattern_ident: &str) -> RustExpr {
    RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "^{}$".to_string(),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "regex".to_string(),
                    "escape".to_string(),
                ])),
                args: vec![RustExpr::Ident(pattern_ident.to_string())],
            }),
            method: "replace".to_string(),
            args: vec![str_ref_lit("\\*"), str_ref_lit(".*")],  // Only handles *
            // Missing: replace "\\?" with "."
        }],
    }
}
```

**Verified Behavior**:
- `Path.glob("?.txt")` on directory with `a.txt`, `b.txt`, `ab.txt` → returns `Ok([])` (should return `Ok(["a.txt", "b.txt"])`)
- `glob("/path", "?.txt")` via `sifr.glob.glob` → returns `["a.txt", "b.txt"]` ✅ CORRECT

**Impact**: CPython parity broken for `?` wildcard in pathlib.

---

### Issue 2: Hidden File Filtering Missing in pathlib.glob ✅ VERIFIED

**Location**: `crates/sifr_codegen/src/intrinsics/pathlib.rs:256-349`

The intrinsic implementation doesn't filter out hidden files (files starting with `.`) when using `*` pattern.

**Verified Behavior**:
- `Path.glob("*")` returns `Ok([".hidden.txt", "a.txt", "b.txt"])` - includes hidden files
- `glob("/path", "*")` via `sifr.glob.glob` → returns `["a.txt", "b.txt"]` ✅ CORRECT (filters hidden)

**Impact**: CPython pathlib behavior is to NOT include hidden files in `*` glob results unless the pattern explicitly starts with `.`.

---

### Issue 3: Missing Directory Behavior Inconsistent ✅ VERIFIED

**Location**: `crates/sifr_codegen/src/intrinsics/pathlib.rs:290-297`

The intrinsic uses `std::fs::read_dir` directly which returns an error for missing directories. The `sifr.glob` implementation catches the exception and returns an empty list.

**Verified Behavior**:
- `Path.glob("*.txt")` on missing directory → returns `Err(IOError { message: "No such file or directory (os error 2)", kind: "FileNotFound" })`
- `glob("/missing", "*.txt")` via `sifr.glob.glob` → returns `[]` ✅ CORRECT (matches CPython)

**Impact**: CPython pathlib returns an empty list for missing directories, but Sifr pathlib raises an error.

---

## What Works Correctly

### sifr.glob module ✅ Production Ready

1. **Basic glob functionality**:
   - `*` wildcard works correctly
   - `?` wildcard works correctly
   - Hidden file filtering works correctly (filters `.` files unless pattern starts with `.`)
   - Missing directory returns empty list (matches CPython)
   - Results are sorted

2. **Test coverage**:
   - `crates/sifr/tests/e2e/pass/cpython_glob_subset.sifr` - 7 test cases covering:
     - `*.txt` pattern matching
     - `?.txt` single character matching
     - `a*` prefix matching
     - `sub*` directory matching
     - `.*.txt` hidden file matching
     - No match returns empty
     - Missing directory returns empty

3. **Demo coverage**:
   - `demos/m30_1e_glob_parity_demo/main.sifr` - validates all approved behaviors

### pathlib.Path.rglob

- Recursive glob works correctly for basic `*` wildcard
- Returns `Result[list[str], IOError]` type
- Has the same three issues as `.glob()` since it uses the same `regex_source_expr` function

## Panic-Safety Analysis

### sifr.glob ✅ Panic-Free
- Uses try-catch for IOError, returns empty list on failure
- No `.unwrap()` or `.expect()` in user-facing code paths

### pathlib.glob ⚠️ Returns Error Not Panic
- Uses `RustExpr::Try` to handle IO errors, returning Result types
- No panics in user-facing code paths
- BUT: Returns error instead of empty list for missing directories (inconsistent with CPython)

## CPython Parity Classification

| Behavior | sifr.glob | pathlib.Path.glob | pathlib.Path.rglob |
|----------|------------|-------------------|-------------------|
| `*` wildcard | ✅ parity | ✅ parity | ✅ parity |
| `?` wildcard | ✅ parity | ❌ broken | ❌ broken |
| Hidden file filter | ✅ parity | ❌ broken | ❌ broken |
| Missing dir → [] | ✅ parity | ❌ broken | ❌ broken |
| Sorted results | ✅ parity | ✅ parity | ✅ parity |
| Recursive | N/A | N/A | ✅ parity |

## Root Cause Analysis

The issues stem from a fundamental architectural decision to use regex-based matching for pathlib glob instead of reusing the already-correct fnmatch implementation:

1. **regex_source_expr bug**: The function was implemented with only `*` replacement, missing `?` conversion to `.`
2. **Missing hidden file logic**: The intrinsic doesn't implement the CPython behavior of filtering `.` prefixed files unless pattern starts with `.`
3. **Error propagation**: The intrinsic propagates IO errors rather than handling them gracefully like sifr.glob does

## Production Readiness Assessment

### For sifr.glob module: ✅ Production Ready
- Correctly implements approved subset
- Hidden file filtering works
- Missing directory handling correct
- All tests pass
- No panic paths

### For pathlib.Path.glob/rglob: ❌ Not Production Ready
The pathlib glob implementation has three critical bugs that affect CPython parity:
1. `?` wildcard not working
2. Hidden file filtering not implemented
3. Missing directory behavior inconsistent (returns error instead of empty list)

## Recommendations

### Option A: Fix pathlib intrinsics (recommended)
Fix the regex conversion in `regex_source_expr` to:
1. Add `?` → `.` replacement after `*` → `.*`
2. Add hidden file filtering logic
3. Handle missing directories gracefully (return empty list)

### Option B: Redirect pathlib to use sifr.glob
Modify pathlib.sifr to call `sifr.glob.glob` instead of using the intrinsic, which already works correctly. This would require:
1. Modifying `lib/sifr/pathlib.sifr` to use `sifr.glob.glob` internally
2. Ensuring the return type conversion works correctly (Result → unwrap or match)

## Test Evidence

```bash
# Test ? wildcard
$ cargo run -q -p sifr -- run test_pathlib_glob_issues.sifr
pathlib glob ? wildcard: result=Ok([])  # BUG: should be Ok(["a.txt", "b.txt"])

# Test hidden files
$ cargo run -q -p sifr -- run test_pathlib_glob_issues.sifr
pathlib glob *: result=Ok(["/tmp/sifr_pathlib_test4/.hidden.txt", "a.txt", "b.txt"])  # BUG: should not include .hidden.txt

# Test missing directory
$ cargo run -q -p sifr -- run test_pathlib_missing.sifr
pathlib glob on missing dir: result=Err(IOError { message: "No such file or directory (os error 2)", kind: "FileNotFound" })  # BUG: should return Ok([])

# sifr.glob works correctly
$ cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_glob_subset.sifr
# All tests pass

$ cargo run -q -p sifr -- run demos/m30_1e_glob_parity_demo/main.sifr
m30_1e glob parity demo: pass
```

## Conclusion

The `sifr.glob` module correctly implements the approved glob subset and is production ready. However, the `pathlib.Path.glob` and `Path.rglob` intrinsics have three critical bugs that prevent CPython parity:

1. `?` wildcard not supported
2. Hidden file filtering not implemented
3. Missing directory handling inconsistent

**These issues need to be addressed before the pathlib glob can be considered production ready for the approved subset.**

## Reviewer Sign-Off Checklist

- [ ] Parity scope is clear and evidenced by CPython-derived tests
- [ ] Remaining gaps are classified correctly
- [ ] Every intentional divergence is justified by Sifr's safety contract
- [ ] No unresolved mismatch lacks an owner and tracking issue
- [ ] No user-facing runtime panic path remains
- [ ] Implementation quality is production-grade
- [ ] Module is CPython-parity aligned for approved scope

**Status**: ⚠️ Requires fixes before production sign-off
