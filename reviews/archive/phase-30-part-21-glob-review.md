# Phase 30 Part 21: Glob Module Parity Review

## Overview

This review examines the implementation of the glob module in wave_30_1e, focusing on correctness, root-cause quality, panic-safety, CPython parity classification, and production readiness for the approved glob subset.

## Implementation Architecture

The glob functionality is implemented in two parallel implementations:

1. **`sifr.glob` module** (`lib/sifr/glob.sifr`)
   - Uses `sifr.fnmatch` for pattern matching (pure Sifr)
   - Exposes `glob(directory, pattern) -> list[str]`
   - Returns empty list for missing directories

2. **`pathlib.Path.glob/rglob`** (`crates/sifr_codegen/src/intrinsics/pathlib.rs`)
   - Uses regex intrinsics (`glob_pattern`, `rglob_pattern`)
   - Exposed as `Path.glob(pattern)` and `Path.rglob(pattern)` methods
   - Intrinsics defined in `crates/sifr_hir/src/stdlib/sys_fs.rs`

## Approved Scope (from phase30_parity_matrix.md)

| Feature | Status | Classification |
|---------|--------|-----------------|
| `glob(directory, pattern)` with `*`/`?` wildcards | done | parity |
| CPython-style hidden-file filtering | done | parity |
| Missing directory returns empty list | done | parity |
| `recursive=True` / `**` | out of scope | intentional-diff |
| `iglob` (lazy iterator) | out of scope | intentional-diff |
| Character classes `[seq]`, `[!seq]` | out of scope | intentional-diff |

## Issues Found

### Critical Issue 1: `?` Wildcard Not Supported in pathlib.glob

**Location**: `crates/sifr_codegen/src/intrinsics/pathlib.rs:80-96`

The regex-based implementation in `regex_source_expr()` only handles `*` wildcard by replacing `\\*` with `.*`. The `?` wildcard is not handled.

```rust
fn regex_source_expr(pattern_ident: &str) -> RustExpr {
    RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "^{}$".to_string(),
        args: vec![RustExpr::MethodCall {
            // ... regex::escape(pattern) ...
            method: "replace".to_string(),
            args: vec![str_ref_lit("\\*"), str_ref_lit(".*")],  // Only handles *
            // Missing: replace "\\?" with "."
        }],
    }
}
```

**Impact**: `Path.glob("?.txt")` returns 0 results instead of matching single-character filenames.

**Verified**:
- `Path.glob("?.txt")` on directory with `a.txt`, `b.txt`, `ab.txt` → returns `[]` (should return `["a.txt", "b.txt"]`)
- `glob("/path", "?.txt")` via `sifr.glob.glob` → works correctly

### Critical Issue 2: Hidden File Filtering Missing in pathlib.glob

**Location**: `crates/sifr_codegen/src/intrinsics/pathlib.rs:256-349`

The intrinsic implementation doesn't filter out hidden files (files starting with `.`) when using `*` pattern.

**Impact**: `Path.glob("*")` includes hidden files when it shouldn't.

**Verified**:
- `Path.glob("*")` returns `.hidden` files alongside regular files
- `glob("/path", "*")` via `sifr.glob.glob` correctly filters out hidden files

### Critical Issue 3: Missing Directory Behavior Inconsistent

**Location**: `crates/sifr_codegen/src/intrinsics/pathlib.rs:290-297`

The intrinsic uses `std::fs::read_dir` directly which returns an error for missing directories. The `sifr.glob` implementation catches the exception and returns an empty list.

**Impact**: `Path.glob()` raises `IOError` for missing directories, but CPython pathlib returns an empty list.

**Verified**:
- `Path.glob("*.txt")` on missing directory → raises `IOError: No such file or directory`
- `glob("/missing", "*.txt")` via `sifr.glob.glob` → returns `[]` (matches CPython)

## What Works Correctly

1. **sifr.glob.glob()** - Correctly implements the approved subset:
   - `*` wildcard works
   - `?` wildcard works
   - Hidden file filtering works
   - Missing directory returns empty list
   - Results are sorted

2. **Path.rglob()** - Recursive glob works correctly:
   - Recursively finds files in subdirectories
   - Uses stack-based traversal

3. **Path.glob()** - Basic functionality:
   - `*` wildcard matching works (though includes hidden files)
   - Results are sorted

## Panic-Safety Analysis

All implementations are panic-safe:
- `sifr.glob`: Uses try-catch for IOError, returns empty list on failure
- pathlib intrinsics: Use `RustExpr::Try` to handle IO errors, returning Result types
- No `.unwrap()` or `.expect()` in user-facing code paths

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

### For pathlib.Path.glob/rglob: ⚠️ Not Production Ready
The pathlib glob implementation has three critical bugs that affect CPython parity:
1. `?` wildcard not working
2. Hidden file filtering not implemented
3. Missing directory behavior inconsistent

## Recommendations

### Option A: Fix pathlib intrinsics (recommended)
Fix the regex conversion in `regex_source_expr` to:
1. Add `?` → `.` replacement after `*` → `.*`
2. Add hidden file filtering logic
3. Handle missing directories gracefully (return empty list)

### Option B: Redirect pathlib to use sifr.glob
Modify pathlib.sifr to call `sifr.glob.glob` instead of using the intrinsic, which already works correctly. This would require:
1. Modifying `lib/sifr/pathlib.sifr` to use `sifr.glob.glob` internally
2. Ensuring the return type conversion works correctly

## Test Evidence

```bash
# Test ? wildcard
$ cargo run -q -p sifr -- run test_qmark_pathlib.sifr
found=0  # BUG: should be 2

$ cargo run -q -p sifr -- run test_qmark_glob.sifr
found=2,a.txt,b.txt  # CORRECT

# Test hidden files
$ cargo run -q -p sifr -- run test_hidden_pathlib.sifr
glob *: found=6,..., .hidden,...  # BUG: should not include .hidden

$ cargo run -q -p sifr -- run test_hidden_glob.sifr
glob *: found=5,a.txt,ab.txt,...  # CORRECT

# Test missing directory
$ cargo run -q -p sifr -- run test_missing_pathlib.sifr
error: No such file or directory  # BUG: should return []

$ cargo run -q -p sifr -- run test_missing_glob.sifr
missing dir result=0  # CORRECT
```

## Conclusion

The `sifr.glob` module correctly implements the approved glob subset and is production ready. However, the `pathlib.Path.glob` and `Path.rglob` intrinsics have three critical bugs that prevent CPython parity:

1. `?` wildcard not supported
2. Hidden file filtering not implemented
3. Missing directory handling inconsistent

These issues need to be addressed before the pathlib glob can be considered production ready for the approved subset.
