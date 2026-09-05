# Stdlib Architecture Phase — Remaining Milestones

Date: 2026-02-16
Status: Proposed
Phase: Stdlib Architecture (extends the existing chain after `milestone_stdlib_classes`)

---

## Context & Sources

This plan synthesizes findings from three audit documents:

1. **`stdlib_cpython_parity_audit_agent.md`** — Module-by-module detail with coverage %, blockers, quick-wins, and intrinsic gaps for all 37 stdlib modules.
2. **`stdlib_cpython_parity_audit_gpt.md`** — Cross-cutting findings: naming mismatches, class rollout status, safety contract gaps, function-signature parity.
3. **`cpython_test_suite_portability_research.md`** — Analysis of CPython's test suite portability: ~60-80% of assertions are directly portable, `assert_almost_eq` is the critical enabler, and Tier 1 modules (math, statistics, json, re) offer the highest ROI.

### Current State (post-`milestone_stdlib_classes`)

- **37 stdlib modules** exist and compile
- **46 E2E pass tests**, **7 fail tests**, **~160 total assertions**
- **1 class-based API** (`collections.Counter`) — pipeline proven end-to-end
- **Known bug:** `statistics.variance` computes population variance (÷N) instead of CPython's sample variance (÷N-1)
- **Known blockers:**
  - `Callable`-as-struct-field (`Box<dyn Fn>` fix) blocks `argparse.ArgumentParser`, `collections.defaultdict`, `timeit.Timer`
  - Generics blocks generic `bisect`, `heapq`, `itertools`
  - Iterator protocol blocks lazy `itertools`, `csv.reader`, `glob.iglob`
  - Exception types block `tomllib.TOMLDecodeError`, `graphlib.CycleError`
  - Context managers (`with`) block `io.open`, `tempfile.NamedTemporaryFile`
- **Known compiler gaps (addressed in m29):**
  - Importing from a nonexistent module silently fails — no "unknown module" error, only downstream "undefined function" when the symbol is used
  - `with` statement is syntactic sugar only — no `__enter__`/`__exit__` protocol, no multiple context managers, no compile-time enforcement
  - `Callable` types emit `impl Fn(...)` which is invalid in Rust struct fields — needs `Box<dyn Fn(...)>` (blocks `ArgumentParser`, `defaultdict`, `Timer`)

---

## Milestone Overview

```
milestone_stdlib_classes (done)
    │
    ├── m29:   milestone_compiler_hardening   (S-M — import errors + with protocol + Callable fix)
    │
    ├── m29.5: milestone_lazy_iterators       (M — lazy state machine codegen for generators)
    │
    ├── m30:   milestone_test_infra           (S — test infrastructure)
    │
    ├── m31:   milestone_stdlib_functions     (M — pure-Sifr + intrinsic additions + generic stdlib)
    │
    ├── m32:   milestone_stdlib_naming        (S — API naming alignment)
    │
    ├── m33:   milestone_stdlib_class_rollout (L — 6 new stdlib classes incl. datetime)
    │
    └── m34:   milestone_cpython_tests        (M — port CPython test assertions)
```

**CPython test suite reference:** `/Users/yaseralnajjar/work/sifr/cpython/` — the CPython source tree used as the authoritative behavioral reference. Test files live at `Lib/test/test_<module>.py`. External test data (e.g., `Lib/test/mathdata/math_testcases.txt`) can be mined for additional test vectors.

**Dependency chain:**
```
milestone_stdlib_classes → m29 (compiler_hardening) → m29.5 (lazy_iterators) → m30 → m31 → m32 → m33 → m34 → milestone_async
```

> **Note:** `milestone_lazy_iterators` was added after initial planning as a prerequisite milestone. It sits between m29 and m30 in the chain. See `07_stdlib_parity.md` for the canonical ordering.

**Rationale for ordering:**
- **m29 first:** Fixes compiler correctness issues (silent import failures, incomplete `with` statement, `Callable`-as-struct-field) that affect every subsequent milestone. New stdlib modules added in m31-m33 need proper import error reporting, and `with` support is a prerequisite for `io.open()` and `tempfile` class APIs in m33.
- **lazy_iterators after m29:** Lazy iteration is a core compiler feature that should be in place before adding new stdlib functions. This way, `itertools` functions in m31 can be written as lazy generators from the start, and `csv.reader`/`glob.iglob` can be implemented properly.
- **m30 before m31:** `assert_almost_eq` is needed to properly test the float-returning functions added in m31.
- **m31 before m32:** Add the missing functions first (including generic `bisect`/`heapq`/`itertools`), then rename everything in one pass. Renaming before adding would require naming new functions twice (once with old convention, once with new).
- **m32 before m33:** Classes should be written with the final CPython-aligned names from the start.
- **m33 before m34:** CPython test porting for class-based modules (re, graphlib, pathlib) requires the classes to exist first. `datetime`/`timedelta` with operator overloading validates the operator export pipeline.
- **m34 last:** Test porting is the validation layer — it should run against the final API surface.

**Total estimated effort:** ~5-6 sprints (S=1-2 days, M=3-5 days, L=5-8 days)

---

## m29: milestone_compiler_hardening — Import Errors, Context Managers, and Callable Fix

**Goal:** Fix three compiler correctness gaps: (1) importing from a nonexistent module silently fails instead of producing a clear error, (2) the `with` statement is incomplete — it's syntactic sugar for scoped blocks but doesn't implement the Python context manager protocol (`__enter__`/`__exit__`), and (3) `Callable` types emit `impl Fn(...)` which is invalid in Rust struct fields — needs `Box<dyn Fn(...)>`.

**Size:** Small-Medium (2-3 days)

### Issue 1: Silent Import Failures for Nonexistent Modules

**Current behavior (confirmed by testing):**

| Scenario | Current Error | Expected Error |
|---|---|---|
| `from sifr.math import sqrt` (valid) | None (works) | None (works) |
| `from sifr.math import nonexistent` (bad member) | `module 'sifr.math' has no member 'nonexistent'` | Same (already correct) |
| `from sifr.nonexistent import foo` (bad module) | `undefined function: 'foo'` (at usage site) | `unknown module 'sifr.nonexistent'` (at import) |
| `from mymodule import bar` (bad local module) | `undefined function: 'bar'` (at usage site) | `unknown module 'mymodule'` (at import) |

**Root cause in `crates/sifr_hir/src/lower.rs`:**

The import resolution at line 422-474 checks `has_module` for `sifr.*` imports. When the module doesn't exist, the code falls through to the local module resolution path (line 476-503), which also finds nothing — but no error is emitted. The import is silently pushed to the imports list (line 505-509), and the error only surfaces later as "undefined function" when the symbol is used.

**Fix:**

After the `sifr.*` check and the local module resolution, add a check: if the module name doesn't exist in any externals map (functions, classes, constants), emit `"unknown module '{module_name}'"`. Specifically:

1. For `sifr.*` modules: after the `has_module` check at line 425-427, if `has_module` is false, emit `"unknown stdlib module '{module_name}'"` and continue (don't fall through to local resolution).
2. For local modules: after the local resolution loop at line 476-503, check if any names were actually resolved. If none were found and the module key doesn't exist in any externals map, emit `"unknown module '{module_name}'"`.

**E2E tests:**

- Update `crates/sifr/tests/e2e/fail/stdlib_invalid_module.sifr` — change expected error from `"undefined function: 'foo'"` to `"unknown stdlib module 'sifr.nonexistent'"` (or similar)
- Add `crates/sifr/tests/e2e/fail/import_nonexistent_local.sifr` — `from mymodule import bar` should produce `"unknown module 'mymodule'"`

### Issue 2: Incomplete `with` Statement

**Current behavior:**

The `with` statement was implemented in `milestone_generators` but only as a minimal "scoped block" desugaring. The Definition of Done for that milestone included items that were never delivered:
- "`ContextManager` protocol enforced at compile time" — **not done**
- "E2E fail tests: `with_non_context_manager`" — **not done**
- "E2E pass tests: `with_file`, `with_multiple`" — **not done**

**What currently works:**
- `with X() as y:` parses and compiles — but emits `{ let y = X(); ... }` (just a scoped block)
- `with` without `as` works (defaults to `_with_val`)
- One passing test: `with_basic.sifr`

**What's missing:**

| Gap | Description | Impact |
|---|---|---|
| `__enter__` call | Python's `with X() as y` calls `X().__enter__()` and binds the result to `y`. Currently `y = X()` directly | Incorrect semantics |
| `__exit__` call | No cleanup call at scope end. Rust's `Drop` is not wired to `__exit__` | Resource leaks |
| `ContextManager` protocol | No protocol definition. Any type can be used in `with` — no compile-time check | No type safety |
| Multiple context managers | `with A() as a, B() as b:` silently drops everything after `items[0]` (line 1686 of `lower.rs`) | Silent data loss |
| E2E tests | `with_non_context_manager`, `with_file`, `with_multiple` were in DoD but never created | Missing coverage |

**Fix — Lowering (`crates/sifr_hir/src/lower.rs`):**

1. **Multiple items:** Replace `let item = &with_stmt.items[0]` with a loop over all items, nesting scopes for each.
2. **Protocol check:** After resolving the context expression type, check that it implements a `ContextManager` protocol (has `__enter__` and `__exit__` methods). Emit error if not.
3. **`__enter__` binding:** The `HirStmt::With` should record that the variable is bound to the result of `__enter__()`, not the context expression directly.

**Fix — Codegen (`crates/sifr_codegen/src/lib.rs`):**

1. **`__enter__` call:** Instead of `let y = X();`, emit:
   ```rust
   let __ctx = X();
   let y = __ctx.__enter__();
   ```
2. **`__exit__` call:** At scope end, emit `__ctx.__exit__();` before the closing brace.
3. **Multiple items:** Nest the blocks for each context manager.

**Fix — Protocol definition:**

Define `ContextManager` as a built-in protocol (similar to how `Iterator` or `Sized` would be defined):
```python
protocol ContextManager:
    def __enter__(self) -> Self
    def __exit__(self) -> None
```

Note: Python's `__exit__` takes `(exc_type, exc_val, exc_tb)` and can suppress exceptions. For Sifr's initial implementation, `__exit__(self) -> None` is sufficient — exception suppression is deferred.

**E2E tests:**

- `crates/sifr/tests/e2e/pass/with_enter_exit.sifr` — class with `__enter__`/`__exit__`, verify both are called
- `crates/sifr/tests/e2e/pass/with_multiple.sifr` — `with A() as a, B() as b:` — verify both context managers work
- `crates/sifr/tests/e2e/fail/with_non_context_manager.sifr` — using a type without `__enter__`/`__exit__` in `with` should produce a compile error

### Issue 3: `Callable`-as-Struct-Field Emits Invalid Rust

`Callable` types currently emit `impl Fn(...)` via `rust_type()` in `types.rs` (line 293). This is valid for function parameters but **invalid in Rust struct fields** — Rust requires a concrete or boxed type. Fix: when `Callable` appears in a struct field context, emit `Box<dyn Fn(...)>` instead of `impl Fn(...)`.

**What this unblocks:** `argparse.ArgumentParser`, `collections.defaultdict`, and `timeit.Timer` — all of which need to store callbacks as struct fields.

**Files to change:**
- `crates/sifr_type_system/src/types.rs` — `rust_type()` for `Type::Callable` (line 293): add context parameter or separate method for struct field emission

**E2E tests:**
- `crates/sifr/tests/e2e/pass/callable_struct_field.sifr` — class with a `Callable` field, verify it compiles and works

### Files to Change

- `crates/sifr_hir/src/lower.rs` — import error reporting (lines 422-509), `with` lowering (lines 1681-1706)
- `crates/sifr_codegen/src/lib.rs` — `with` codegen (lines 2379-2401)
- `crates/sifr_type_system/src/types.rs` — `Callable` `rust_type()` for struct field context
- Possibly `crates/sifr_hir/src/hir.rs` — update `HirStmt::With` to carry `__enter__`/`__exit__` method info
- 3-4 new E2E pass tests, 2-3 new E2E fail tests
- Update 1 existing fail test (`stdlib_invalid_module.sifr`)

### What This Unblocks

- **Proper import errors** improve developer experience for every subsequent milestone — when adding new stdlib functions/classes (m31-m33), typos in import statements will be caught immediately instead of producing confusing downstream errors.
- **`with` statement** unblocks `io.open()` as a context manager, `tempfile.NamedTemporaryFile`, and any future stdlib class that manages resources.
- **`Callable`-as-struct-field** unblocks `argparse.ArgumentParser`, `collections.defaultdict`, and `timeit.Timer`.

### Definition of Done

- `from sifr.nonexistent import foo` produces `"unknown stdlib module 'sifr.nonexistent'"` (not `"undefined function: 'foo'"`)
- `from mymodule import bar` (when `mymodule` doesn't exist) produces `"unknown module 'mymodule'"`
- Existing import error for bad members still works: `from sifr.math import nonexistent` → `"module 'sifr.math' has no member 'nonexistent'"`
- `with X() as y:` calls `X().__enter__()` and binds result to `y`, calls `__exit__()` at scope end
- `__exit__()` is called on all exit paths: normal completion, early `return`, `break`, `continue`, and error propagation (maps to Rust `Drop` semantics)
- `with A() as a, B() as b:` handles all context managers (not just the first)
- Using a non-`ContextManager` type in `with` produces a compile error
- A class with a `Callable` field compiles correctly (struct field emits `Box<dyn Fn(...)>`)
- All existing E2E tests pass (zero regressions)
- New E2E tests: `with_enter_exit`, `with_multiple`, `callable_struct_field`
- New E2E fail tests: `with_non_context_manager`, `import_nonexistent_local`

---

## m29.5: milestone_lazy_iterators — Lazy Iterator Protocol

**Goal:** Replace the eager generator codegen (`_yields.push()` → return `Vec<T>`) with a proper lazy `Iterator` implementation using state machines. Currently, `yield` in a function collects all values into a `Vec` and returns the full list. This milestone makes generators produce lazy iterators that yield values on demand via `next() -> Option<T>`.

**Size:** Medium (3-5 days)

### Current State

Generators work but are eager: the codegen creates a `Vec<T>`, pushes every `yield`ed value into it, and returns the full `Vec` at the end (lines 1578-1600 and 2373-2377 of `sifr_codegen/src/lib.rs`). There is no state machine, no `Iterator` trait implementation, no `next()` method.

### Changes

1. **HIR:** Add `HirType::Iterator(Box<Type>)` to represent lazy iterator types. A function containing `yield` returns `Iterator[T]` instead of `list[T]`.
2. **Codegen — state machine:** Instead of `_yields.push(val)`, emit a Rust struct that implements `Iterator<Item = T>`. Each `yield` becomes a state transition. The struct stores local variables as fields and tracks the current state via an enum.
3. **Codegen — `for` loop integration:** `for x in lazy_iter` emits `while let Some(x) = iter.next()` instead of `for x in &vec`.
4. **Codegen — eager collection:** `list(iter)` or assigning an iterator to `list[T]` calls `.collect::<Vec<T>>()` for backward compatibility.
5. **Type system:** `Iterator[T]` is a first-class type. It is assignable to `list[T]` via implicit `.collect()`.

### Files to Change

- `crates/sifr_hir/src/hir_nodes.rs` — add `HirType::Iterator(Box<Type>)`
- `crates/sifr_type_system/src/types.rs` — add `Type::Iterator` variant, `rust_type()` implementation
- `crates/sifr_codegen/src/lib.rs` — replace eager `_yields.push()` pattern (lines 1578-1600) with state machine struct emission; update `for` loop codegen; add `.collect()` for `list(iter)` conversion
- `crates/sifr_hir/src/lower.rs` — update generator function return type inference from `list[T]` to `Iterator[T]`
- 3 new E2E test files

### What This Unblocks

- `glob.iglob` — lazy directory traversal
- `csv.reader` — lazy line-by-line reading
- `itertools` functions (`chain`, `take`, `repeat`) — can be lazy instead of materializing full lists
- Foundation for async iterators (`async for`) in Phase 8

### Definition of Done

- A generator function returns a lazy iterator, not a `Vec`
- `for x in generator_fn()` works lazily (no full materialization)
- `list(generator_fn())` eagerly collects into a list
- Existing generator E2E tests pass (backward compatible via implicit collect)
- New E2E tests: `lazy_generator`, `lazy_for_loop`, `lazy_collect`
- `cargo test` passes (zero regressions)

---

## m30: milestone_test_infra — Test Infrastructure for CPython Parity

**Goal:** Add the test assertion primitives needed to port CPython tests, and fix the known `statistics.variance` bug. This is a small, focused milestone that unblocks all subsequent testing work.

**Size:** Small (1-2 days)

### Changes

#### 1. New `_sifr.test` intrinsics

| Intrinsic | Signature | Rust Codegen |
|---|---|---|
| `assert_almost_eq` | `(actual: float, expected: float, tolerance: float) -> None` | `assert!((actual - expected).abs() < tolerance, "assert_almost_eq failed: {} != {} (tolerance {})", actual, expected, tolerance);` |
| `assert_gt` | `(a: int, b: int) -> None` | `assert!(a > b, "assert_gt failed: {} is not > {}", a, b);` |
| `assert_lt` | `(a: int, b: int) -> None` | `assert!(a < b, "assert_lt failed: {} is not < {}", a, b);` |

**Why these three:** `assert_almost_eq` is the critical enabler (unlocks ~400+ float test assertions from CPython). `assert_gt`/`assert_lt` are needed for range-based testing (e.g., `random_int` returns value in range, `bisect_left` returns correct position). These are trivial to implement — 3 new intrinsic signatures + 3 codegen match arms.

**Not included:** `assert_raises` (needs `std::panic::catch_unwind`, complex codegen). ValueError tests can be converted to NAN/INF checks instead, which is what Rust's `f64` methods actually do for domain errors.

#### 2. Update `lib/sifr/test.sifr`

```python
from _sifr.test import assert_eq, assert_ne, assert_true, assert_false, assert_almost_eq, assert_gt, assert_lt
```

#### 3. Fix `statistics.variance` bug

The current `variance` divides by N (population variance). CPython's `variance` divides by N-1 (sample variance). Fix:

```python
# Current (wrong):
def variance(data: list[float]) -> float:
    ...
    return total / float(len(data))

# Fixed:
def variance(data: list[float]) -> float:
    ...
    return total / float(len(data) - 1)

# Add population variant:
def pvariance(data: list[float]) -> float:
    ...
    return total / float(len(data))
```

Also fix `stdev` (which calls `variance`) and add `pstdev`.

#### 4. E2E tests

- `stdlib_test_almost_eq.sifr` — test `assert_almost_eq` with known float values
- `stdlib_test_gt_lt.sifr` — test `assert_gt`, `assert_lt`
- `stdlib_statistics_variance_fix.sifr` — verify sample variance vs population variance
- Update existing `stdlib_statistics.sifr` if its expected output changes

### Files to Change

- `crates/sifr_hir/src/stdlib.rs` — 3 new intrinsic signatures in `intrinsic_test()`
- `crates/sifr_codegen/src/lib.rs` — 3 new codegen match arms for `_sifr.test`
- `lib/sifr/test.sifr` — add new imports
- `lib/sifr/statistics.sifr` — fix `variance`/`stdev`, add `pvariance`/`pstdev`
- New E2E test files (3-4 files)

### Definition of Done

- `assert_almost_eq(3.14159, 3.14159, 0.001)` passes
- `assert_almost_eq(1.0, 2.0, 0.001)` panics with clear message
- `variance([2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])` returns `4.571...` (sample, N-1), not `4.0` (population, N)
- `pvariance(...)` returns the population variance
- `cargo test` passes (zero regressions)

---

## m31: milestone_stdlib_functions — Pure-Sifr and Intrinsic Function Additions

**Goal:** Close the function-level gaps identified by both audits. Add ~25 pure-Sifr functions and ~12 new intrinsics across 12 modules. Also convert `bisect`, `heapq`, and `itertools` to generic `list[T]` using `TypeVar` — proving that generics work in the stdlib export pipeline. This is the largest single increase in stdlib API surface.

**Size:** Medium (3-5 days)

### Part A: Pure-Sifr Functions (no compiler changes)

These are functions that can be implemented entirely in `.sifr` files using existing language features.

#### `sifr.math` — 7 new functions

```python
def factorial(n: int) -> int:
    # Iterative loop, n >= 0

def gcd(a: int, b: int) -> int:
    # Euclidean algorithm

def lcm(a: int, b: int) -> int:
    # a * b // gcd(a, b)

def comb(n: int, k: int) -> int:
    # Combinations: n! / (k! * (n-k)!)

def perm(n: int, k: int) -> int:
    # Permutations: n! / (n-k)!

def isclose(a: float, b: float, rel_tol: float, abs_tol: float) -> bool:
    # abs(a-b) <= max(rel_tol * max(abs(a), abs(b)), abs_tol)

def prod(data: list[int]) -> int:
    # Product of all elements
```

#### `sifr.statistics` — 5 new functions

```python
def fmean(data: list[float]) -> float:
    # Same as mean (already float-based)

def harmonic_mean(data: list[float]) -> float:
    # n / sum(1/x for x in data)

def geometric_mean(data: list[float]) -> float:
    # exp(mean(log(x) for x in data))

def median_low(data: list[float]) -> float:
    # Low median (for even-length: lower of two middle values)

def median_high(data: list[float]) -> float:
    # High median (for even-length: higher of two middle values)
```

#### `sifr.bisect` — 1 new function + generic conversion

```python
from typing import TypeVar
T = TypeVar("T")

def bisect_left(a: list[T], x: T) -> int:
    # (existing, converted to generic)

def bisect_right(a: list[T], x: T) -> int:
    # (existing, converted to generic)

def insort_left(a: list[T], x: T) -> list[T]:
    # (existing, converted to generic)

def insort_right(a: list[T], x: T) -> list[T]:
    # Insert at bisect_right position (new)
```

#### `sifr.heapq` — 2 new functions + generic conversion

```python
from typing import TypeVar
T = TypeVar("T")

def heappush(heap: list[T], item: T) -> list[T]:
    # (existing, converted to generic)

def heappop(heap: list[T]) -> list[T]:
    # (existing, converted to generic)

def heapify(data: list[T]) -> list[T]:
    # (existing, converted to generic)

def nsmallest(n: int, data: list[T]) -> list[T]:
    # (existing, converted to generic)

def nlargest(n: int, data: list[T]) -> list[T]:
    # (existing, converted to generic)

def heapreplace(heap: list[T], item: T) -> list[T]:
    # Pop smallest, push item (new)

def heappushpop(heap: list[T], item: T) -> list[T]:
    # Push item, pop smallest (new)
```

#### `sifr.string` — 2 new items

```python
printable: str = ...  # All printable ASCII characters

def capwords(s: str) -> str:
    # Capitalize each word (split on whitespace, capitalize, rejoin)
```

#### `sifr.textwrap` — 1 new function

```python
def shorten(text: str, width: int) -> str:
    # Truncate to width with "..." placeholder
```

#### `sifr.pathlib` — 3 new functions

```python
def stem(path: str) -> str:
    # Filename without extension

def is_absolute(path: str) -> bool:
    # Starts with /

def splitext(path: str) -> str:
    # Returns "root|ext" (pipe-separated, since no tuples yet)
    # Or: two separate functions root(path) and ext(path)
```

Note: `splitext` returning a tuple is blocked by limited tuple support. Instead, provide `stem` + `extension` (already exists) which covers the same use case.

#### `sifr.fnmatch` — 1 new function

```python
def fnmatchcase(name: str, pat: str) -> bool:
    # Case-sensitive match (current fnmatch is case-insensitive on some platforms)
```

#### `sifr.ipaddress` — 3 new functions

```python
def int_to_ip(n: int) -> str:
    # Reverse of ip_to_int

def is_multicast(addr: str) -> bool:
    # 224.0.0.0 - 239.255.255.255

def is_global(addr: str) -> bool:
    # Not private, not loopback, not multicast, not reserved
```

#### `sifr.itertools` — 3 new functions + generic conversion

```python
from typing import TypeVar
T = TypeVar("T")

def chain(a: list[T], b: list[T]) -> list[T]:
    # (existing, converted to generic)

def take(n: int, data: list[T]) -> list[T]:
    # (existing, converted to generic)

def flatten(data: list[list[T]]) -> list[T]:
    # (existing, converted to generic)

def pairwise(data: list[T]) -> list[str]:
    # Consecutive pairs as "a,b" strings (new)

def batched(data: list[T], n: int) -> list[list[T]]:
    # Batch into groups (new)

def islice(data: list[T], start: int, stop: int) -> list[T]:
    # Slice from start to stop (new)
```

Note: Converting `bisect`, `heapq`, and `itertools` to generic `list[T]` using `TypeVar` is the first use of generics in stdlib `.sifr` files. The full pipeline (generic function in `.sifr` → `TypeVar` in exported signatures → user import → monomorphized codegen) must be validated here.

#### `sifr.tempfile` — 1 new function

```python
def gettempdir() -> str:
    # Return system temp directory path
```

This needs a new `_sifr.fs` intrinsic (see Part B).

### Part B: New Intrinsics (Rust codegen additions)

| Module | Intrinsic | Signature | Rust Implementation |
|---|---|---|---|
| `_sifr.math` | `exp` | `(x: float) -> float` | `f64::exp()` |
| `_sifr.math` | `expm1` | `(x: float) -> float` | `f64::exp_m1()` |
| `_sifr.math` | `log1p` | `(x: float) -> float` | `f64::ln_1p()` |
| `_sifr.math` | `fabs` | `(x: float) -> float` | `f64::abs()` |
| `_sifr.math` | `isfinite` | `(x: float) -> bool` | `f64::is_finite()` |
| `_sifr.crypto` | `sha1` | `(s: str) -> str` | `sha1` crate (hex digest) |
| `_sifr.crypto` | `sha512` | `(s: str) -> str` | `sha2` crate (hex digest) |
| `_sifr.crypto` | `urlsafe_b64encode` | `(s: str) -> str` | `base64::engine::general_purpose::URL_SAFE` |
| `_sifr.crypto` | `urlsafe_b64decode` | `(s: str) -> str` | `base64::engine::general_purpose::URL_SAFE` |
| `_sifr.fs` | `gettempdir` | `() -> str` | `std::env::temp_dir()` |
| `_sifr.fs` | `makedirs` | `(path: str) -> None` | `std::fs::create_dir_all()` |
| `_sifr.platform` | `platform_node` | `() -> str` | `gethostname` crate or `libc::gethostname` |

### Part C: Re-exports in `.sifr` modules

Update the following `.sifr` files to import and re-export the new intrinsics:
- `lib/sifr/math.sifr` — add `exp`, `expm1`, `log1p`, `fabs`, `isfinite`
- `lib/sifr/hashlib.sifr` — add `sha1`, `sha512`
- `lib/sifr/base64.sifr` — add `urlsafe_b64encode`, `urlsafe_b64decode`
- `lib/sifr/os.sifr` — add `makedirs`
- `lib/sifr/platform.sifr` — add `node` (re-export of `platform_node`)

### E2E Tests

Add or expand tests for every new function. Target: ~40 new assertions across ~10 test files.

- `stdlib_math_functions.sifr` — `factorial`, `gcd`, `lcm`, `comb`, `perm`, `isclose`, `prod`, `exp`, `expm1`, `log1p`, `fabs`, `isfinite`
- `stdlib_statistics_extended.sifr` — `fmean`, `harmonic_mean`, `geometric_mean`, `median_low`, `median_high`
- `stdlib_bisect_insort.sifr` — `insort_right`
- `stdlib_heapq_extended.sifr` — `heapreplace`, `heappushpop`
- `stdlib_string_extended.sifr` — `printable`, `capwords`
- `stdlib_pathlib_extended.sifr` — `stem`, `is_absolute`
- `stdlib_ipaddress_extended.sifr` — `int_to_ip`, `is_multicast`, `is_global`
- `stdlib_hashlib_extended.sifr` — `sha1`, `sha512`
- `stdlib_base64_extended.sifr` — `urlsafe_b64encode`, `urlsafe_b64decode`

### Files to Change

- `crates/sifr_hir/src/stdlib.rs` — 12 new intrinsic signatures
- `crates/sifr_codegen/src/lib.rs` — 12 new codegen match arms
- 12 `.sifr` stdlib files (new functions + re-exports)
- ~10 new E2E test files

### Definition of Done

- All ~25 pure-Sifr functions work correctly
- All ~12 new intrinsics compile and produce correct output
- `bisect`, `heapq`, `itertools` work with generic types (e.g., `bisect_left([1.0, 2.0, 3.0], 2.5)` works)
- `sifr.math` coverage goes from ~85% to ~95% (29 → 41 functions)
- `sifr.statistics` coverage goes from ~50% to ~70%
- ~40 new test assertions pass
- `cargo test` passes (zero regressions)
- Demo: `demos/milestone_stdlib_functions_demo.sifr`

---

## m32: milestone_stdlib_naming — API Naming Alignment with CPython

**Goal:** Rename Sifr stdlib functions to match CPython naming conventions. This is a deliberate pre-1.0 breaking change that should be done once, in one pass, rather than incrementally.

**Size:** Small (1-2 days)

### Rust Keyword Collisions

Two proposed CPython names collide with Rust strict keywords. Since Sifr compiles to Rust source code, any function name that is a Rust keyword would produce invalid Rust output unless handled. The codegen must emit `r#` raw identifiers for these names.

**Colliding names (Rust strict keywords):**

| CPython Name | Module | Rust Keyword | Resolution |
|---|---|---|---|
| `match` | `sifr.re` | `match` (strict) | Sifr name: `match` → codegen emits `r#match` in generated Rust |
| `move` | `sifr.shutil` | `move` (strict) | Sifr name: `move` → codegen emits `r#move` in generated Rust |

**Colliding names (Rust reserved keywords — currently unused but future-proof):**

| CPython Name | Module | Rust Keyword | Resolution |
|---|---|---|---|
| `type` | (future: `sifr.builtins`) | `type` (strict) | If ever added, codegen emits `r#type` |
| `yield` | (future: generators) | `yield` (reserved) | Already handled differently — `yield` is a Sifr keyword mapped to generator state machine, not a function name |

**Names that look like keywords but are NOT Rust keywords (safe as-is):**

`filter`, `search`, `split`, `random`, `time`, `match` (only in Sifr source — the `.sifr` layer is fine, only the generated Rust needs `r#`).

**Implementation note:** The codegen change is small — when emitting a function name, check if it's in a set of Rust keywords and prefix with `r#` if so. This is a one-time change in `sifr_codegen` that applies to all current and future keyword collisions. See the architecture file's Python Divergences table for the documented divergence.

### Renames

| Module | Current Name | CPython Name | Rust Keyword? | Notes |
|---|---|---|---|---|
| `sifr.math` | `abs_val` | `fabs` | No | Keep `abs_val` as alias initially |
| `sifr.math` | `pow_val` | `pow` | No | `pow` is a builtin in CPython, but `math.pow` exists |
| `sifr.math` | `min_val` | — | — | Remove from math (these are builtins, not `math` functions) |
| `sifr.math` | `max_val` | — | — | Remove from math (these are builtins, not `math` functions) |
| `sifr.math` | `round_val` | — | — | Remove from math (this is a builtin) |
| `sifr.re` | `re_match` | `match` | **Yes** (`match` is strict) | Codegen must emit `r#match` in generated Rust |
| `sifr.re` | `re_find` | `search` | No | |
| `sifr.re` | `re_replace` | `sub` | No | |
| `sifr.re` | `re_findall` | `findall` | No | |
| `sifr.re` | `re_split` | `split` | No | |
| `sifr.json` | `json_loads` | `loads` | No | |
| `sifr.json` | `json_dumps` | `dumps` | No | |
| `sifr.shutil` | `move_file` | `move` | **Yes** (`move` is strict) | Codegen must emit `r#move` in generated Rust |
| `sifr.base64` | `base64_encode` | `b64encode` | No | |
| `sifr.base64` | `base64_decode` | `b64decode` | No | |
| `sifr.random` | `random_int` | `randint` | No | |
| `sifr.random` | `random_float` | `random` | No | |
| `sifr.random` | `random_choice` | `choice` | No | |
| `sifr.random` | `random_uniform` | `uniform` | No | |
| `sifr.platform` | `platform_system` | `system` | No | |
| `sifr.platform` | `platform_arch` | `machine` | No | |
| `sifr.fnmatch` | `fnmatch_filter` | `filter` | No | |
| `sifr.time` | `time_now` | `time` | No | |
| `sifr.time` | `time_format` | `strftime` | No | |

### Strategy

The renames happen at the **Sifr stdlib layer only** — the `_sifr.*` intrinsic names stay unchanged. Each `.sifr` file re-exports with the new name:

```python
# lib/sifr/re.sifr — AFTER rename
from _sifr.regex import re_match, re_find, re_replace, re_findall, re_split

# CPython-compatible names (codegen emits r#match for the Rust function)
def match(pattern: str, text: str) -> bool:
    return re_match(pattern, text)

def search(pattern: str, text: str) -> str | None:
    return re_find(pattern, text)

def sub(pattern: str, replacement: str, text: str) -> str:
    return re_replace(pattern, replacement, text)

def findall(pattern: str, text: str) -> list[str]:
    return re_findall(pattern, text)

def split(pattern: str, text: str) -> list[str]:
    return re_split(pattern, text)
```

```python
# lib/sifr/shutil.sifr — AFTER rename
from _sifr.fs import copy_file, rename

# CPython-compatible names (codegen emits r#move for the Rust function)
def move(src: str, dst: str) -> None:
    rename(src, dst)
```

This approach:
- Keeps intrinsic names stable (no Rust codegen changes for intrinsics)
- Requires a one-time codegen change: emit `r#` prefix for Rust-keyword function names
- Allows old names to coexist as aliases during transition if desired
- All E2E tests must be updated to use new names

### E2E Test Updates

All existing stdlib test files that use renamed functions must be updated. This is mechanical but touches ~20 test files.

### Files to Change

- ~15 `.sifr` stdlib files (add wrapper functions with CPython names)
- ~20 E2E test files (update imports and function calls)
- Demo files (update to use new names)

### Definition of Done

- All renamed functions work with CPython-compatible names
- All E2E tests updated and passing
- Old names still work (re-exported as aliases). Deprecation schedule: aliases kept in Phase 7, compiler warnings in Phase 8, removal in Phase 9
- `cargo test` passes (zero regressions)

---

## m33: milestone_stdlib_class_rollout — Expand Class-Based APIs

**Goal:** Add 6 new class-based APIs to the stdlib, leveraging the pipeline proven by `milestone_stdlib_classes`. Includes `datetime`/`timedelta` with operator overloading (`__add__`/`__sub__`) — proving that operator methods export correctly from stdlib classes.

**Size:** Large (5-8 days)

### Classes to Implement

#### 1. `sifr.graphlib.TopologicalSorter` — Pure algorithmic class

```python
class TopologicalSorter:
    nodes: str      # JSON-encoded adjacency list
    num_nodes: int

    def __init__(self, nodes: str, num_nodes: int):
        self.nodes = nodes
        self.num_nodes = num_nodes

    def add(self, node: int, predecessor: int) -> None:
        # Add edge: predecessor → node
        self.nodes = sorter_add(self.nodes, node, predecessor)

    def static_order(self) -> list[int]:
        # Return topological ordering
        return sorter_static_order(self.nodes, self.num_nodes)
```

**New intrinsics:** `sorter_add`, `sorter_static_order` (2 intrinsics in `_sifr.collections` or a new `_sifr.graph` module)

**Why:** Replaces the awkward `topological_sort(num_nodes, from_nodes, to_nodes)` function API with the standard CPython class API. Pure algorithmic — no external dependencies.

#### 2. `sifr.pathlib.Path` — File path class wrapping existing intrinsics

```python
class Path:
    path: str

    def __init__(self, path: str):
        self.path = path

    def name(self) -> str:
        return basename(self.path)

    def parent(self) -> Path:
        return Path(dirname(self.path))

    def suffix(self) -> str:
        return extension(self.path)

    def stem(self) -> str:
        return stem(self.path)

    def exists(self) -> bool:
        return fs_exists(self.path)

    def is_file(self) -> bool:
        return fs_is_file(self.path)

    def is_dir(self) -> bool:
        return fs_is_dir(self.path)

    def read_text(self) -> str:
        return fs_read_text(self.path)

    def write_text(self, content: str) -> None:
        fs_write_text(self.path, content)

    def mkdir(self) -> None:
        fs_mkdir(self.path)

    def joinpath(self, child: str) -> Path:
        return Path(join_path(self.path, child))

    def resolve(self) -> Path:
        return Path(fs_resolve(self.path))
```

**New intrinsics:** `fs_resolve` (1 intrinsic — `std::fs::canonicalize`)

**Why:** `pathlib.Path` is one of the most-used CPython classes. The functional helpers already exist — this wraps them in the standard class interface. Most methods delegate to existing `_sifr.fs` intrinsics.

#### 3. `sifr.logging.Logger` — Logging class with levels

```python
class Logger:
    name: str
    level: int  # 0=DEBUG, 1=INFO, 2=WARNING, 3=ERROR, 4=CRITICAL

    def __init__(self, name: str, level: int):
        self.name = name
        self.level = level

    def debug(self, msg: str) -> None:
        if self.level <= 0:
            print("[DEBUG] " + self.name + ": " + msg)

    def info(self, msg: str) -> None:
        if self.level <= 1:
            print("[INFO] " + self.name + ": " + msg)

    def warning(self, msg: str) -> None:
        if self.level <= 2:
            print("[WARNING] " + self.name + ": " + msg)

    def error(self, msg: str) -> None:
        if self.level <= 3:
            print("[ERROR] " + self.name + ": " + msg)

    def critical(self, msg: str) -> None:
        if self.level <= 4:
            print("[CRITICAL] " + self.name + ": " + msg)

def getLogger(name: str) -> Logger:
    return Logger(name, 1)  # Default INFO level
```

**New intrinsics:** None — pure Sifr.

**Why:** `logging.getLogger` is the standard CPython pattern. This gives users named loggers with level filtering. No handlers/formatters yet (those need more advanced features), but this covers the 80% use case.

#### 4. `sifr.re.Match` — Regex match result class

```python
class Match:
    matched: str    # The matched text
    start_pos: int  # Start position in original string
    end_pos: int    # End position in original string

    def __init__(self, matched: str, start_pos: int, end_pos: int):
        self.matched = matched
        self.start_pos = start_pos
        self.end_pos = end_pos

    def group(self) -> str:
        return self.matched

    def start(self) -> int:
        return self.start_pos

    def end(self) -> int:
        return self.end_pos

    def span(self) -> str:
        # Returns "start,end" since no tuple type
        return str(self.start_pos) + "," + str(self.end_pos)
```

**New intrinsics:** `re_search_match` — returns JSON-encoded `{"matched": "...", "start": N, "end": N}` (1 intrinsic in `_sifr.regex`)

**Why:** CPython's `re.search()` returns a `Match` object, not a string. This brings Sifr closer to the real API. The existing `re_find` returns `str | None`; the new `search` function returns `Match | None` (or a wrapper that constructs `Match`).

**Note:** Full `Match` with groups (`.group(1)`, `.groups()`) is deferred — requires list-of-optional-string support. This initial version covers the most common use case: checking if a match exists and getting the matched text + position.

#### 5. `sifr.uuid.UUID` — UUID value class

```python
class UUID:
    hex_str: str  # 32-char hex string (no dashes)

    def __init__(self, hex_str: str):
        self.hex_str = hex_str

    def hex(self) -> str:
        return self.hex_str

    def urn(self) -> str:
        return "urn:uuid:" + self.to_str()

    def to_str(self) -> str:
        # Format as 8-4-4-4-12
        return self.hex_str[:8] + "-" + self.hex_str[8:12] + "-" + self.hex_str[12:16] + "-" + self.hex_str[16:20] + "-" + self.hex_str[20:]

    def version(self) -> int:
        return 4  # Only uuid4 for now

def uuid4() -> UUID:
    raw: str = _uuid4()
    # Strip dashes to get hex_str
    return UUID(uuid_strip_dashes(raw))
```

**New intrinsics:** `uuid_strip_dashes` (1 intrinsic — simple string manipulation, or done in pure Sifr)

**Why:** CPython's `uuid.uuid4()` returns a `UUID` object, not a string. This is a lightweight class that wraps the existing `uuid4` intrinsic.

#### 6. `sifr.datetime.datetime` / `timedelta` — Date/time classes with operator overloading

```python
class timedelta:
    days: int
    seconds: int
    microseconds: int

    def __init__(self, days: int, seconds: int, microseconds: int):
        self.days = days
        self.seconds = seconds
        self.microseconds = microseconds

    def total_seconds(self) -> float:
        return float(self.days * 86400 + self.seconds) + float(self.microseconds) / 1000000.0

    def __add__(self, other: timedelta) -> timedelta:
        return timedelta(self.days + other.days, self.seconds + other.seconds, self.microseconds + other.microseconds)

    def __sub__(self, other: timedelta) -> timedelta:
        return timedelta(self.days - other.days, self.seconds - other.seconds, self.microseconds - other.microseconds)
```

```python
class datetime:
    year: int
    month: int
    day: int
    hour: int
    minute: int
    second: int

    def __init__(self, year: int, month: int, day: int, hour: int, minute: int, second: int):
        # ...fields

    def __add__(self, delta: timedelta) -> datetime:
        # Add timedelta to datetime via intrinsic
        return datetime_add(self, delta)

    def __sub__(self, delta: timedelta) -> datetime:
        return datetime_sub(self, delta)

    def isoformat(self) -> str:
        return datetime_isoformat(self)

    def timestamp(self) -> float:
        return datetime_timestamp(self)

def now() -> datetime:
    return datetime_now()
```

**New intrinsics:** `datetime_add`, `datetime_sub`, `datetime_isoformat`, `datetime_timestamp`, `datetime_now` (5 intrinsics in `_sifr.datetime`, wrapping `chrono`)

**Why:** `datetime` is one of the most-used CPython modules. `timedelta` is pure Sifr (no intrinsics needed). This is the first stdlib class with `__add__`/`__sub__` operator overloading — validates that operator methods export correctly from stdlib `.sifr` files.

### E2E Tests

- `stdlib_graphlib_sorter.sifr` — `TopologicalSorter` construction, `add`, `static_order`
- `stdlib_pathlib_path.sifr` — `Path` construction, `name`, `parent`, `suffix`, `exists`, `is_file`, `read_text`, `write_text`, `joinpath`
- `stdlib_logging_logger.sifr` — `Logger` construction, level filtering, `getLogger`
- `stdlib_re_match.sifr` — `Match` object from `search`, `.group()`, `.start()`, `.end()`
- `stdlib_uuid_class.sifr` — `UUID` object, `.hex()`, `.urn()`, `.to_str()`
- `stdlib_datetime_class.sifr` — `datetime.now()`, `timedelta` construction, `datetime + timedelta`, `timedelta + timedelta`, `isoformat`
- Fail tests: wrong types to constructors (3-5 fail tests)

### Files to Change

- `crates/sifr_hir/src/stdlib.rs` — ~10 new intrinsic signatures
- `crates/sifr_codegen/src/lib.rs` — ~10 new codegen match arms
- `lib/sifr/graphlib.sifr` — add `TopologicalSorter` class
- `lib/sifr/pathlib.sifr` — add `Path` class
- `lib/sifr/logging.sifr` — add `Logger` class + `getLogger`
- `lib/sifr/re.sifr` — add `Match` class + updated `search` function
- `lib/sifr/uuid.sifr` — add `UUID` class + updated `uuid4`
- `lib/sifr/datetime.sifr` — add `datetime` + `timedelta` classes with `__add__`/`__sub__`, `now` factory
- ~9 new E2E test files

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Class method signature export fails for multi-class modules | Low | Medium | Pipeline proven by Counter; same pattern |
| `Path` methods that call `_sifr.fs` intrinsics fail with borrow issues | Medium | Medium | Same pattern as Counter calling `_sifr.collections` — proven |
| `Match` construction from JSON-encoded intrinsic output is fragile | Medium | Low | Can parse JSON in pure Sifr or add a dedicated intrinsic |
| Operator overloading (`__add__`/`__sub__`) fails in stdlib class export | Medium | Medium | Operator overloading works in user code (`operator_overload.sifr`); first test in stdlib pipeline |
| 6 classes in one milestone is too large | Low | Medium | Classes are independent — can be split if needed |

### Definition of Done

- All 6 classes compile, export, and are importable by user code
- `from sifr.pathlib import Path` works; `p = Path("/tmp/test.txt"); p.exists()` works
- `from sifr.logging import getLogger` works; `logger.info("hello")` prints with level filtering
- `from sifr.graphlib import TopologicalSorter` works with `add`/`static_order`
- `from sifr.re import Match` works; `search` returns `Match` objects
- `from sifr.uuid import UUID` works; `uuid4()` returns `UUID` object
- `from sifr.datetime import datetime, timedelta` works; `datetime.now() + timedelta(days=1)` works with operator overloading
- All E2E tests pass
- `cargo test` passes (zero regressions)
- Demo: `demos/milestone_stdlib_class_rollout_demo.sifr`

---

## m34: milestone_cpython_tests — Port CPython Test Assertions

**Goal:** Port ~500 test assertions from CPython's test suite to Sifr, focusing on the highest-ROI modules. This is the validation layer that proves Sifr's stdlib behaves correctly against CPython's behavioral specification.

**Size:** Medium (3-5 days)

### CPython Source Reference

All test porting uses the CPython source tree at **`/Users/yaseralnajjar/work/sifr/cpython/`** as the authoritative reference:

| What | Path |
|---|---|
| Test files | `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_<module>.py` |
| Math test data | `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/mathdata/math_testcases.txt` |
| JSON test suite | `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_json/` (19 files) |
| Stdlib source | `/Users/yaseralnajjar/work/sifr/cpython/Lib/<module>.py` |

### Porting Strategy

Based on the CPython test portability research:
- **No automated translator needed** — the mapping is mechanical (`assertEqual` → `assert_eq`, etc.)
- **Skip:** TypeError tests (compiler handles), dunder protocol tests, pickling, platform-specific tests
- **Convert:** ValueError tests to NAN/INF checks where applicable
- **Focus:** Pure value assertions and float tolerance assertions (these are ~60% of CPython tests and are directly portable)

### Assertion Mapping (CPython → Sifr)

| CPython | Sifr | Notes |
|---|---|---|
| `self.assertEqual(a, b)` | `assert_eq(a, b)` | Direct |
| `self.assertTrue(x)` | `assert_true(x)` | Direct |
| `self.assertFalse(x)` | `assert_false(x)` | Direct |
| `self.assertNotEqual(a, b)` | `assert_ne(a, b)` | Direct |
| `self.ftest(name, got, expected)` | `assert_almost_eq(got, expected, 0.00001)` | Float tolerance |
| `self.assertAlmostEqual(a, b)` | `assert_almost_eq(a, b, 0.0000001)` | Float tolerance |
| `self.assertGreater(a, b)` | `assert_gt(a, b)` | Added in m30 |
| `self.assertLess(a, b)` | `assert_lt(a, b)` | Added in m30 |
| `self.assertRaises(TypeError, f, x)` | *skip — compiler catches this* | Compile-time |
| `self.assertRaises(ValueError, f, x)` | `assert_true(isnan(f(x)))` | NAN/INF check |
| `self.assertIs(a, b)` | *skip — identity not meaningful* | Compiled lang |
| `self.assertIn(x, container)` | *express as boolean check* | Manual |

### Tier 1: Highest ROI (port first)

#### `sifr.math` — Target: ~200 assertions (from CPython's ~1,075)

Port from `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_math.py`:

| CPython Test Method | Assertions to Port | Notes |
|---|---|---|
| `testConstants` | ~5 | pi, e, tau values |
| `testAcos` | ~10 | Domain [-1,1], NAN propagation, edge values |
| `testAsin` | ~10 | Domain [-1,1], NAN propagation |
| `testAtan` | ~10 | Full range, INF handling |
| `testAtan2` | ~25 | All quadrants, INF, NAN, zero signs |
| `testCos` | ~10 | Key values, NAN propagation |
| `testSin` | ~10 | Key values, NAN propagation |
| `testTan` | ~8 | Basic values, NAN |
| `testCosh`, `testSinh`, `testTanh` | ~15 | Hyperbolic functions |
| `testDegrees`, `testRadians` | ~8 | Angle conversion |
| `testFloor`, `testCeil` | ~10 | Integer rounding, negative values |
| `testTrunc` | ~5 | Truncation |
| `testCopysign` | ~15 | Sign copying with INF, NAN, zero |
| `testFmod` | ~8 | Float modulo |
| `testHypot` | ~10 | Pythagorean distance, edge cases |
| `testIsnan`, `testIsinf` | ~8 | Special value detection |
| `testLog`, `testLog2`, `testLog10` | ~15 | Logarithms, edge cases |
| `testSqrt` | ~8 | Square root, edge cases |
| `testExp` | ~8 | Exponential (new in m31) |
| `testFabs` | ~5 | Absolute value (new in m31) |
| `testIsfinite` | ~5 | Finite check (new in m31) |
| `testFactorial` | ~8 | Factorial (new in m31) |
| `testGcd`, `testLcm` | ~10 | GCD/LCM (new in m31) |

**File structure:** Split into multiple test files by category:
- `stdlib_math_cpython_trig.sifr` — trig functions (~50 assertions)
- `stdlib_math_cpython_exp_log.sifr` — exp/log functions (~30 assertions)
- `stdlib_math_cpython_special.sifr` — INF/NAN/copysign/fmod (~40 assertions)
- `stdlib_math_cpython_integer.sifr` — floor/ceil/trunc/factorial/gcd/lcm (~40 assertions)
- `stdlib_math_cpython_misc.sifr` — hypot, degrees, radians, isclose, prod (~40 assertions)

#### `sifr.statistics` — Target: ~80 assertions (from CPython's ~527)

Port from `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_statistics.py`:

- `mean` — basic, single element, large values, negative values (~15)
- `median` — odd/even length, single element, sorted/unsorted (~15)
- `variance` / `stdev` — basic, single element (should raise?), known values (~15)
- `pvariance` / `pstdev` — same pattern (~10)
- `mode` — basic, ties, single element (~10)
- `fmean`, `harmonic_mean`, `geometric_mean` — basic correctness (~15)

**File:** `stdlib_statistics_cpython.sifr`

#### `sifr.json` — Target: ~40 assertions

Port from `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_json/`:

- Basic decode: strings, numbers, booleans, null, arrays, objects (~15)
- Basic encode: same types (~10)
- Edge cases: empty objects, nested structures, unicode (~10)
- Whitespace handling (~5)

**File:** `stdlib_json_cpython.sifr`

#### `sifr.re` — Target: ~50 assertions

Port from `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/test_re.py`:

- Basic matching: literal, `.`, `*`, `+`, `?`, `^`, `$` (~15)
- Character classes: `[abc]`, `[a-z]`, `\d`, `\w`, `\s` (~10)
- `findall` with various patterns (~10)
- `split` with various patterns (~10)
- `sub` with various patterns (~5)

**File:** `stdlib_re_cpython.sifr`

### Tier 2: Medium ROI

#### `sifr.collections` — Target: ~40 assertions

- Set operations: union, intersection, add, remove, contains, len (~20)
- Counter: get, total, most_common, increment, values, keys (~20)

**File:** `stdlib_collections_cpython.sifr`

#### `sifr.bisect` — Target: ~20 assertions

- `bisect_left`, `bisect_right` with sorted lists, boundary cases (~10)
- `insort_left`, `insort_right` correctness (~10)

**File:** `stdlib_bisect_cpython.sifr`

#### `sifr.heapq` — Target: ~20 assertions

- Heap invariant after push/pop sequences (~10)
- `nsmallest`, `nlargest` correctness (~10)

**File:** `stdlib_heapq_cpython.sifr`

#### `sifr.textwrap` — Target: ~25 assertions

- `wrap` at various widths (~8)
- `fill` behavior (~5)
- `dedent` with various indentation (~7)
- `indent` with prefix (~5)

**File:** `stdlib_textwrap_cpython.sifr`

#### `sifr.fnmatch` — Target: ~20 assertions

- Wildcard patterns: `*`, `?`, `[abc]` (~10)
- Edge cases: empty pattern, no match, case sensitivity (~10)

**File:** `stdlib_fnmatch_cpython.sifr`

### What NOT to Port (and Why)

Per the CPython test portability research:

1. **TypeError tests** — Sifr's compiler catches these at compile time
2. **Dunder protocol tests** (`__float__`, `__index__`, `__hash__`) — Sifr doesn't have dynamic dispatch
3. **Subclassing builtin types** — Sifr's type system is different
4. **Pickling/unpickling** — Not applicable
5. **`eval()`/`repr()` roundtrips** — Sifr doesn't have `eval()`
6. **Locale-dependent tests** — Rust has fixed locale behavior
7. **`Decimal`/`Fraction` type tests** — Sifr doesn't have these types
8. **CPython-internal tests** (`@cpython_only`) — Implementation-specific

### Files to Change

- ~15 new E2E test files
- No stdlib code changes (this milestone only adds tests)

### Definition of Done

- ~500 new test assertions across ~15 test files
- All assertions pass
- Total stdlib test assertion count goes from ~200 (after m29-m33) to ~700+
- Math function coverage: every exported function has at least 5 assertions including edge cases (NAN, INF, boundary values)
- `cargo test` passes (zero regressions)

---

## Summary Table

| Milestone | ID | Size | New Functions | New Intrinsics | New Tests | Key Deliverable |
|---|---|---|---|---|---|---|
| Compiler Hardening | m29 | S-M | 0 | 0 | ~8 assertions | Import errors, `with` protocol, `Callable` struct field fix |
| Lazy Iterators | m29.5 | M | 0 | 0 | ~6 assertions | Lazy state machine codegen for generators |
| Test Infrastructure | m30 | S | 2 (`pvariance`, `pstdev`) | 3 | ~10 assertions | `assert_almost_eq`, variance bug fix |
| Stdlib Functions | m31 | M | ~25 | ~12 | ~40 assertions | Function gaps + generic `bisect`/`heapq`/`itertools` |
| Naming Alignment | m32 | S | 0 (renames) | 0 | ~0 (updates) | CPython-compatible names |
| Class Rollout | m33 | L | 6 class APIs (7 classes) | ~10 | ~30 assertions | Path, Logger, Match, TopologicalSorter, UUID, datetime/timedelta |
| CPython Tests | m34 | M | 0 | 0 | ~500 assertions | Behavioral validation against CPython |

**Cumulative impact:**

| Metric | Current | After All Milestones |
|---|---|---|
| Stdlib test assertions | ~160 | ~750+ |
| Stdlib functions (across all modules) | ~120 | ~170+ |
| Class-based APIs | 1 (Counter) | 7 concrete classes across 6 APIs (+ Path, Logger, Match, TopologicalSorter, UUID, datetime/timedelta) |
| Generic stdlib functions | 0 | `bisect`, `heapq`, `itertools` use `TypeVar` |
| Modules with CPython-compatible names | ~5 | ~30+ |
| Math function coverage | ~85% | ~95% |
| Average module coverage | ~35% | ~55% |
| Import error quality | Silent failures for nonexistent modules | Clear "unknown module" errors |
| `with` statement | Scoped block only (no protocol) | Full `__enter__`/`__exit__` protocol with cleanup on all exits |
| `Callable` in struct fields | Compile error (`impl Fn`) | Works via `Box<dyn Fn(...)>` |
| Generators | Eager (`Vec<T>`) | Lazy iterators via state machine |

---

## What's Explicitly Deferred (and Why)

| Item | Blocker | When to Address |
|---|---|---|
| Exception types (`TOMLDecodeError`, `CycleError`) | Error types in stdlib `.sifr` files are untested — custom error classes work in user code but the export pipeline from stdlib is unproven | Next phase — requires validating error type export from stdlib |
| `assert_raises` | `std::panic::catch_unwind` codegen | Lower priority — NAN/INF checks suffice for most cases |

---

## Validation Against Codebase

Cross-checked against actual codebase state (2026-02-16):

- **37 `.sifr` files** confirmed in `lib/sifr/` — all module references in this plan are valid
- **`_sifr.test` intrinsic module** exists with `assert_eq`, `assert_ne`, `assert_true`, `assert_false` — confirmed `assert_almost_eq` is missing
- **`statistics.variance`** confirmed to divide by `float(len(data))` (population, not sample) — bug is real
- **`_sifr.math` intrinsics** confirmed: 29 functions + 5 constants — `exp`, `expm1`, `log1p`, `fabs`, `isfinite` are genuinely missing
- **`_sifr.crypto` intrinsics** confirmed: `sha256`, `md5`, `base64_encode`, `base64_decode` — `sha1`, `sha512`, `urlsafe_*` are genuinely missing
- **`_sifr.fs` intrinsics** confirmed: no `gettempdir` or `makedirs` — these are genuinely missing
- **Class pipeline** confirmed working via `collections.Counter` — `ExternalDefs.classes` export, method signature loading, borrow convention emission all proven
- **46 pass tests, 7 fail tests, ~160 assertions** — confirmed baseline
