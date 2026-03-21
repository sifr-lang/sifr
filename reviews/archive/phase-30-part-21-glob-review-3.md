# Phase 30 Part 21: Glob Module Parity Review (Round 3 - Post-Remediation)

## Overview

This is a follow-up review for the glob module (wave_30_1e) examining the remediation applied to address the three critical issues identified in Round 2. This review validates that the pathlib glob/rglob fixes are production-grade quality.

## Prior Issues (from Round 2)

The Round 2 review identified three critical bugs in `pathlib.Path.glob/rglob`:

1. **`?` wildcard not supported** - regex conversion only handled `*`, missing `?` → `.`
2. **Hidden file filtering missing** - included `.hidden` files in `*` glob results
3. **Missing directory behavior inconsistent** - returned error instead of empty list

## Remediation Applied

### Fix 1: `?` Wildcard Support ✅ FIXED

**Location**: `crates/sifr_codegen/src/intrinsics/pathlib.rs:80-100`

The `regex_source_expr()` function now chains two `.replace()` calls:

```rust
fn regex_source_expr(pattern_ident: &str) -> RustExpr {
    RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "^{}$".to_string(),
        args: vec![RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "regex".to_string(),
                        "escape".to_string(),
                    ])),
                    args: vec![RustExpr::Ident(pattern_ident.to_string())],
                }),
                method: "replace".to_string(),
                args: vec![str_ref_lit("\\*"), str_ref_lit(".*")],  // * → .*
            }),
            method: "replace".to_string(),
            args: vec![str_ref_lit("\\?"), str_ref_lit(".")],       // ? → . (NEW)
        }],
    }
}
```

**Verification**: `Path.glob("?.txt")` on directory with `a.txt`, `b.txt`, `ab.txt` → returns `Ok(["a.txt", "b.txt"])`

---

### Fix 2: Hidden File Filtering ✅ FIXED

**Location**: `crates/sifr_codegen/src/intrinsics/pathlib.rs:293-358` (glob), `pathlib.rs:416-508` (rglob)

Added `__include_hidden` variable that checks if the pattern starts with `.`:

```rust
RustStmt::Let {
    mutable: false,
    name: "__include_hidden".to_string(),
    ty: None,
    value: starts_with_dot_expr(RustExpr::Ident("__pat".to_string())),
},
```

Then filtering logic skips hidden files when the pattern doesn't start with `.`:

```rust
RustStmt::If {
    cond: RustExpr::BinOp {
        left: Box::new(RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(RustExpr::Ident("__include_hidden".to_string())),
        }),
        op: "&&".to_string(),
        right: Box::new(starts_with_dot_expr(RustExpr::Ident("__name".to_string()))),
    },
    then_body: vec![RustStmt::Continue],
    else_body: None,
},
```

**Verification**: `Path.glob("*")` returns only visible files, `Path.glob(".*")` includes hidden files.

---

### Fix 3: Missing Directory Handling ✅ FIXED

**Location**: `crates/sifr_codegen/src/intrinsics/pathlib.rs:380-385` (glob), `pathlib.rs:548-551` (rglob)

Added match arm for `Err(_)` that returns an empty vector:

```rust
RustMatchArm {
    pattern: "Err(_)".to_string(),
    bindings: vec![],
    guard: None,
    body: vec![RustStmt::Return(Some(ok_expr(empty_string_vec_expr())))],
},
```

**Verification**: `Path.glob("*.txt")` on missing directory → returns `Ok([])` matching CPython behavior.

---

## Test Coverage

### New Test Fixture: `pathlib_glob_semantics.sifr`

Created comprehensive test covering all three fixes:

```sifr
# Test ? wildcard
q: list[str] = d.glob("?.txt")
q_ok: bool = len(q) == 2 and q[0] == base + "/a.txt" and q[1] == base + "/b.txt"

# Test hidden file filtering
star: list[str] = d.glob("*")
hidden_filtered: bool = len(star) == 3 and star[0] == base + "/a.txt" ...

# Test missing directory
missing_matches: list[str] = missing.glob("*.txt")
len(missing_matches) == 0
```

### Existing Tests Verified

All existing glob tests continue to pass:

| Test | Status |
|------|--------|
| `pathlib_glob_semantics.sifr` | ✅ Pass |
| `cpython_glob_subset.sifr` | ✅ Pass |
| `stdlib_glob.sifr` | ✅ Pass |
| `path_glob.sifr` | ✅ Pass |
| `cpython_pathlib_subset.sifr` | ✅ Pass |
| `demos/m30_1e_glob_parity_demo/main.sifr` | ✅ Pass |

## CPython Parity Matrix (Post-Remediation)

| Behavior | sifr.glob | pathlib.Path.glob | pathlib.Path.rglob |
|----------|------------|-------------------|-------------------|
| `*` wildcard | ✅ parity | ✅ parity | ✅ parity |
| `?` wildcard | ✅ parity | ✅ parity | ✅ parity |
| Hidden file filter | ✅ parity | ✅ parity | ✅ parity |
| Missing dir → [] | ✅ parity | ✅ parity | ✅ parity |
| Sorted results | ✅ parity | ✅ parity | ✅ parity |
| Recursive | N/A | N/A | ✅ parity |

## Panic-Safety Analysis

All implementations remain panic-free:

- Uses `RustExpr::Try` for regex compilation errors
- Uses `RustStmt::Match` with `Ok`/`Err` arms for IO operations
- Returns `Result[list[str], IOError]` type for user-facing APIs
- No `.unwrap()` or `.expect()` in user-facing code paths

## Code Quality Assessment

### Strengths

1. **Consistent implementation**: Both `glob` and `rglob` share identical filtering logic
2. **Proper error handling**: Missing directories return empty list (CPython-compatible)
3. **Correct regex escaping**: Uses `regex::escape` before pattern substitution
4. **Hidden file semantics**: Matches CPython behavior (hidden files only included when pattern starts with `.`)

### Minor Observations

- The implementation uses generated Rust AST expressions, which is the correct approach for codegen
- No clippy warnings specific to `pathlib.rs` (pre-existing warnings in other files are unrelated)

## Production Readiness Assessment

### For pathlib.Path.glob/rglob: ✅ Production Ready

All three critical issues from Round 2 have been resolved:

1. ✅ `?` wildcard now works correctly
2. ✅ Hidden file filtering implemented per CPython semantics
3. ✅ Missing directory returns empty list (matches CPython)

### Sign-Off Checklist

- [x] Parity scope is clear and evidenced by CPython-derived tests
- [x] Remaining gaps are classified correctly
- [x] Every intentional divergence is justified by Sifr's safety contract
- [x] No unresolved mismatch lacks an owner and tracking issue
- [x] No user-facing runtime panic path remains
- [x] Implementation quality is production-grade
- [x] Module is CPython-parity aligned for approved scope

## Conclusion

The pathlib glob/rglob remediation successfully addresses all three critical issues identified in the Round 2 review. The implementation is now production-ready and achieves CPython parity for the approved glob subset:

- ✅ `?` wildcard supported
- ✅ Hidden file filtering matches CPython
- ✅ Missing directory returns empty list
- ✅ All tests pass
- ✅ No panic paths

**Status**: ✅ Approved for production use

---

## Review Metadata

- **Review Round**: 3 (Post-Remediation)
- **Reviewer**: Claude Code
- **Date**: 2026-03-09
- **Files Reviewed**:
  - `crates/sifr_codegen/src/intrinsics/pathlib.rs`
  - `crates/sifr/tests/e2e/pass/pathlib_glob_semantics.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_glob_subset.sifr`
