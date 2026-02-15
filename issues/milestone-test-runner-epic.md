# milestone_test_runner — Built-in Test Runner

## 1. Product Requirements

### Objective

Ship a built-in test runner (`sifr test`) so that all subsequent stdlib work can be tested using Sifr's own test runner. Tests are first-class citizens of the language.

### Scope — Scoped Down for Initial Implementation

**In Scope:**

1. **`sifr test` CLI command** — discovers and runs test files
2. **Test discovery** — finds `test_*` functions in `test_*.sifr` / `*_test.sifr` files
3. **Assertions** — `assert_eq`, `assert_ne`, `assert_true`, `assert_false`
4. **`sifr.test` stdlib module** — provides assertion functions
5. **Test output** — clear pass/fail reporting with test names
6. **Exit code** — non-zero on failure (CI-friendly)

**Out of Scope (deferred):**

| Feature | Reason |
| --- | --- |
| Test filtering (`-k`) | Nice-to-have, not essential for initial version |
| Parallel execution | Complexity, can add later |
| Setup/teardown | Can add later |
| `assert_err`, `assert_ok`, `assert_none`, `assert_contains` | Can add later, basic assertions first |

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | `sifr test` discovers test files matching `test_*.sifr` or `*_test.sifr` |
| AC-2 | Test functions named `test_*` are automatically discovered and run |
| AC-3 | `assert_eq(a, b)` passes when a == b, fails with message when a != b |
| AC-4 | `assert_ne(a, b)` passes when a != b, fails with message when a == b |
| AC-5 | `assert_true(x)` passes when x is true, fails otherwise |
| AC-6 | `assert_false(x)` passes when x is false, fails otherwise |
| AC-7 | Test output shows pass/fail for each test with test name |
| AC-8 | Exit code is 0 on all pass, non-zero on any failure |
| AC-9 | All existing E2E tests pass (no regressions) |

---

## 2. Solution Design

### 2.1 Architecture

The test runner is implemented as:
1. A new `sifr test` CLI command in `crates/sifr/src/main.rs`
2. A `sifr.test` stdlib module providing assertion functions
3. Codegen that maps `test_*` functions to Rust `#[test]` functions
4. A test runner in the driver that discovers files, compiles them, and runs `cargo test`

### 2.2 `sifr.test` Stdlib Module

```python
from sifr.test import assert_eq, assert_ne, assert_true, assert_false
```

Type signatures:
- `assert_eq(actual: int, expected: int)` — also for str, float, bool
- `assert_ne(actual: int, expected: int)` — also for str, float, bool
- `assert_true(value: bool)`
- `assert_false(value: bool)`

Codegen mapping:
- `assert_eq(a, b)` → `assert_eq!(a, b)`
- `assert_ne(a, b)` → `assert_ne!(a, b)`
- `assert_true(x)` → `assert!(x)`
- `assert_false(x)` → `assert!(!x)`

### 2.3 Test Discovery

`sifr test [dir]` (default: current directory):
1. Find all `.sifr` files matching `test_*.sifr` or `*_test.sifr`
2. For each file, compile to Rust
3. Mark `test_*` functions with `#[test]` attribute
4. Build and run with `cargo test`

### 2.4 Codegen Changes

When compiling in test mode:
- Functions named `test_*` get `#[test]` attribute
- No `fn main()` is generated (Rust test harness provides its own)
- Assertion functions map to Rust macros

### 2.5 Testing Strategy

- E2E test: create a test sifr file, run `sifr test`, verify output
- Demo: `demos/milestone_test_runner_demo/` directory with test files
