# Stdlib Parity

This phase closes the remaining gaps between Sifr's stdlib and CPython's stdlib. It fixes compiler correctness issues (import error reporting, `with` statement protocol, `Callable`-as-struct-field), adds lazy iterator support, adds test infrastructure, expands the function surface (including generic stdlib functions), aligns API names with CPython, rolls out 6 new class-based APIs — 7 concrete classes including `datetime`/`timedelta` with operator overloading, and ports ~500 CPython test assertions as behavioral validation.

**Predecessor:** Phase 6 (Stdlib Architecture) — all 37 modules exist, the three-tier hybrid architecture is in place, and the class-in-stdlib pipeline is proven via `collections.Counter`.

**Successor:** Phase 8 (Ecosystem) — the async runtime, web framework, and all ecosystem milestones depend on a mature stdlib with both function and class APIs, CPython-compatible naming, proper `with` statement support, and lazy iteration.

**Detailed plan:** [plans/issues/archive/stdlib_architecture_remaining_milestones.md](../issues/archive/stdlib_architecture_remaining_milestones.md)

---

## milestone_compiler_hardening: Import Errors, Context Managers, and Callable Fix

status: done

**Goal:** Fix three compiler correctness gaps: (1) importing from a nonexistent module silently fails instead of producing a clear error, (2) the `with` statement is incomplete — it's syntactic sugar for scoped blocks but doesn't implement the Python context manager protocol (`__enter__`/`__exit__`), and (3) `Callable` types emit `impl Fn(...)` which is invalid in Rust struct fields — needs `Box<dyn Fn(...)>`.

### Import Error Reporting

The import resolution in `lower.rs` silently falls through when a `sifr.*` module doesn't exist — no error is emitted, and the error only surfaces later as "undefined function" when the symbol is used. Fix: emit `"unknown stdlib module"` for nonexistent `sifr.*` modules and `"unknown module"` for nonexistent local modules, at the import site.

### `with` Statement Protocol

The `with` statement was implemented in `milestone_generators` but only as a minimal "scoped block" desugaring. Missing: `__enter__`/`__exit__` method calls, `ContextManager` protocol enforcement, multiple context managers (`with A() as a, B() as b:`), and E2E tests.

Fix:
- Define `ContextManager` as a built-in protocol with `__enter__(self) -> Self` and `__exit__(self) -> None`
- Lowering: check protocol compliance, bind variable to `__enter__()` result, handle multiple items
- Codegen: emit `__enter__()` call at scope start, `__exit__()` call on all exit paths (normal completion, early `return`, `break`, `continue`, error propagation) — maps to Rust `Drop` semantics

### `Callable`-as-Struct-Field Fix

`Callable` types currently emit `impl Fn(...)` via `rust_type()` in `types.rs` (line 293). This is valid for function parameters but **invalid in Rust struct fields** — Rust requires `Box<dyn Fn(...)>` for struct fields. Fix: when `Callable` appears in a struct field context, emit `Box<dyn Fn(...)>` instead of `impl Fn(...)`. This unblocks `argparse.ArgumentParser`, `collections.defaultdict`, and `timeit.Timer` — all of which need to store callbacks as struct fields.

### Definition of Done

- `from sifr.nonexistent import foo` produces `"unknown stdlib module 'sifr.nonexistent'"`
- `from mymodule import bar` (nonexistent) produces `"unknown module 'mymodule'"`
- `with X() as y:` calls `__enter__()` and `__exit__()` correctly
- `__exit__()` is called on all exit paths: normal completion, early `return`, `break`, `continue`, and error propagation (maps to Rust `Drop` semantics)
- `with A() as a, B() as b:` handles all context managers
- Non-`ContextManager` type in `with` produces a compile error
- A class with a `Callable` field compiles correctly (struct field emits `Box<dyn Fn(...)>`)
- E2E pass tests: `with_enter_exit`, `with_multiple`, `callable_struct_field`
- E2E fail tests: `with_non_context_manager`, `import_nonexistent_local`
- `cargo test` passes (zero regressions)

---

## milestone_lazy_iterators: Lazy Iterator Protocol

status: done

**Goal:** Replace the eager generator codegen (`_yields.push()` → return `Vec<T>`) with a proper lazy `Iterator` implementation using state machines. Currently, `yield` in a function collects all values into a `Vec` and returns the full list. This milestone makes generators produce lazy iterators that yield values on demand via `next() -> Option<T>`.

### Current State

Generators work but are eager: the codegen creates a `Vec<T>`, pushes every `yield`ed value into it, and returns the full `Vec` at the end (lines 1578-1600 and 2373-2377 of `sifr_codegen/src/lib.rs`). There is no state machine, no `Iterator` trait implementation, no `next()` method.

### Changes

1. **HIR:** Add `HirType::Iterator(Box<Type>)` to represent lazy iterator types. A function containing `yield` returns `Iterator[T]` instead of `list[T]`.
2. **Codegen — state machine:** Instead of `_yields.push(val)`, emit a Rust struct that implements `Iterator<Item = T>`. Each `yield` becomes a state transition. The struct stores local variables as fields and tracks the current state via an enum.
3. **Codegen — `for` loop integration:** `for x in lazy_iter` emits `while let Some(x) = iter.next()` instead of `for x in &vec`.
4. **Codegen — eager collection:** `list(iter)` is the explicit materialization boundary (`.collect::<Vec<T>>()` in generated Rust).
5. **Type system:** `Iterator[T]` is a first-class type and is **not** implicitly assignable to `list[T]`; users must call `list(...)` explicitly.

### What This Unblocks

- `glob.iglob` — lazy directory traversal
- `csv.reader` — lazy line-by-line reading
- `itertools` functions (`chain`, `take`, `repeat`) — can be lazy instead of materializing full lists
- Foundation for async iterators (`async for`) in Phase 8

### Definition of Done

- A generator function returns a lazy iterator, not a `Vec`
- `for x in generator_fn()` works lazily (no full materialization)
- `list(generator_fn())` eagerly collects into a list
- Existing generator E2E tests pass with explicit `list(...)` materialization where eager values are required
- New E2E tests: `lazy_generator`, `lazy_for_loop`, `lazy_collect`
- `cargo test` passes (zero regressions)

---

## milestone_test_infra: Test Infrastructure for CPython Parity

status: done

**Goal:** Add test assertion primitives needed to port CPython tests, and fix the known `statistics.variance` bug.

### New `_sifr.test` Intrinsics

- `assert_almost_eq(actual: float, expected: float, tolerance: float) -> None` — unlocks ~400+ float test assertions
- `assert_gt(a: int, b: int) -> None` — range-based testing
- `assert_lt(a: int, b: int) -> None` — range-based testing

### Fix `statistics.variance` Bug

Current `variance` divides by N (population variance). CPython's `variance` divides by N-1 (sample variance). Fix `variance`/`stdev` and add `pvariance`/`pstdev`.

### Definition of Done

- `assert_almost_eq(3.14159, 3.14159, 0.001)` passes
- `variance([2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])` returns sample variance (N-1)
- `pvariance(...)` returns population variance (N)
- `cargo test` passes (zero regressions)

---

## milestone_stdlib_functions: Pure-Sifr and Intrinsic Function Additions

status: done

**Goal:** Close function-level gaps. Add ~25 pure-Sifr functions and ~12 new intrinsics across 12 modules. Also make `bisect`, `heapq`, and `itertools` generic (using `TypeVar`) — proving that generics work in the stdlib export pipeline.

### Pure-Sifr Functions

- `sifr.math`: `factorial`, `gcd`, `lcm`, `comb`, `perm`, `isclose`, `prod`
- `sifr.statistics`: `fmean`, `harmonic_mean`, `geometric_mean`, `median_low`, `median_high`
- `sifr.bisect`: `insort_right`
- `sifr.heapq`: `heapreplace`, `heappushpop`
- `sifr.string`: `printable`, `capwords`
- `sifr.textwrap`: `shorten`
- `sifr.pathlib`: `stem`, `is_absolute`
- `sifr.fnmatch`: `fnmatchcase`
- `sifr.ipaddress`: `int_to_ip`, `is_multicast`, `is_global`
- `sifr.itertools`: `pairwise`, `batched`, `islice`
- `sifr.tempfile`: `gettempdir`

### Generic Stdlib Functions

Convert `bisect`, `heapq`, and `itertools` from concrete `list[int]` to generic `list[T]` using `TypeVar`. This is the first use of generics in stdlib `.sifr` files — the full pipeline (generic function in `.sifr` → `TypeVar` in exported signatures → user import → monomorphized codegen) is unproven and must be validated here.

- `sifr.bisect`: `bisect_left(a: list[T], x: T)`, `bisect_right(a: list[T], x: T)`, `insort_left`, `insort_right`
- `sifr.heapq`: `heappush(heap: list[T], item: T)`, `heappop`, `heapify`, `nsmallest`, `nlargest`
- `sifr.itertools`: `chain(a: list[T], b: list[T])`, `take`, `flatten`, `pairwise`, `batched`, `islice`

### New Intrinsics

- `_sifr.math`: `exp`, `expm1`, `log1p`, `fabs`, `isfinite`
- `_sifr.crypto`: `sha1`, `sha512`, `urlsafe_b64encode`, `urlsafe_b64decode`
- `_sifr.fs`: `gettempdir`, `makedirs`
- `_sifr.platform`: `platform_node`

### Definition of Done

- All ~25 pure-Sifr functions and ~12 intrinsics work correctly
- `bisect`, `heapq`, `itertools` work with generic types (e.g., `bisect_left([1.0, 2.0, 3.0], 2.5)` works)
- `sifr.math` coverage: ~85% → ~95%
- ~40 new test assertions pass
- `cargo test` passes (zero regressions)
- Demo: `demos/stdlib_functions/main.sifr`

---

## milestone_stdlib_naming: API Naming Alignment with CPython

status: done

**Goal:** Rename Sifr stdlib functions to match CPython naming conventions. Deliberate pre-1.0 breaking change done in one pass.

### Key Renames

- `sifr.math`: `abs_val` → `fabs`, `pow_val` → `pow`
- `sifr.re`: `re_match` → `match`, `re_find` → `search`, `re_replace` → `sub`, `re_findall` → `findall`, `re_split` → `split`
- `sifr.json`: `json_loads` → `loads`, `json_dumps` → `dumps`
- `sifr.shutil`: `move_file` → `move`
- `sifr.base64`: `base64_encode` → `b64encode`, `base64_decode` → `b64decode`
- `sifr.random`: `random_int` → `randint`, `random_float` → `random`, `random_choice` → `choice`, `random_uniform` → `uniform`
- `sifr.platform`: `platform_system` → `system`, `platform_arch` → `machine`
- `sifr.time`: `time_now` → `time`, `time_format` → `strftime`
- `sifr.fnmatch`: `fnmatch_filter` → `filter`

### Rust Keyword Handling

Two CPython names collide with Rust strict keywords: `match` (`sifr.re`) and `move` (`sifr.shutil`). Codegen emits `r#match` / `r#move` in generated Rust. One-time codegen change: check function names against Rust keyword set and prefix with `r#` if needed.

### Strategy

Renames happen at the Sifr stdlib layer only — `_sifr.*` intrinsic names stay unchanged. Each `.sifr` file re-exports with the new name via wrapper functions. Old names kept as aliases. Deprecation schedule: aliases kept in Phase 7, compiler warnings in Phase 8, removal in Phase 9.

### Definition of Done

- All renamed functions work with CPython-compatible names
- All E2E tests updated and passing
- Old names still work as aliases
- `cargo test` passes (zero regressions)

---

## milestone_stdlib_class_rollout: Expand Class-Based APIs

status: done

**Goal:** Add 6 new class-based APIs leveraging the pipeline proven by `milestone_stdlib_classes` (Counter). Includes `datetime`/`timedelta` with operator overloading (`__add__`/`__sub__`) — proving that operator methods export correctly from stdlib classes.

### Classes

1. **`sifr.graphlib.TopologicalSorter`** — pure algorithmic class with `add`/`static_order`
2. **`sifr.pathlib.Path`** — file path class wrapping existing `_sifr.fs` intrinsics (`name`, `parent`, `suffix`, `stem`, `exists`, `is_file`, `is_dir`, `read_text`, `write_text`, `mkdir`, `joinpath`, `resolve`)
3. **`sifr.logging.Logger`** — named loggers with level filtering (`debug`, `info`, `warning`, `error`, `critical`) + `getLogger` factory
4. **`sifr.re.Match`** — regex match result with `group`, `start`, `end`, `span`
5. **`sifr.uuid.UUID`** — UUID value class with `hex`, `urn`, `to_str`, `version`
6. **`sifr.datetime.datetime` / `timedelta`** — date/time classes with `__add__`/`__sub__` operator overloading. `datetime` wraps existing `_sifr.datetime` intrinsics; `timedelta` is pure Sifr (stores days/seconds/microseconds). This is the first stdlib class with operator overloading — validates that `__add__`/`__sub__` methods export correctly from stdlib `.sifr` files.

### Definition of Done

- All 6 classes compile, export, and are importable by user code
- `from sifr.pathlib import Path` works end-to-end
- `from sifr.logging import getLogger` works with level filtering
- `from sifr.graphlib import TopologicalSorter` works with `add`/`static_order`
- `from sifr.re import Match` works; `search` returns `Match` objects
- `from sifr.uuid import UUID` works; `uuid4()` returns `UUID` object
- `from sifr.datetime import datetime, timedelta` works; `datetime.now() + timedelta(days=1)` works with operator overloading
- `cargo test` passes (zero regressions)
- Demo: `demos/class_libraries/main.sifr`

---

## milestone_cpython_tests: Port CPython Test Assertions

status: done

**Goal:** Port ~500 test assertions from CPython's test suite to Sifr, focusing on the highest-ROI modules.

**CPython source reference:** `/Users/yaseralnajjar/work/sifr/cpython/` — test files at `Lib/test/test_<module>.py`.

### Tier 1 (port first)

- `sifr.math` — ~200 assertions from `test_math.py` (trig, exp/log, special values, integer math)
- `sifr.statistics` — ~80 assertions from `test_statistics.py`
- `sifr.json` — ~40 assertions from `test_json/`
- `sifr.re` — ~50 assertions from `test_re.py`

### Tier 2

- `sifr.collections` — ~40 assertions (set + Counter)
- `sifr.bisect` — ~20 assertions
- `sifr.heapq` — ~20 assertions
- `sifr.textwrap` — ~25 assertions
- `sifr.fnmatch` — ~20 assertions

### What NOT to Port

TypeError tests (compiler handles), dunder protocol tests, subclassing builtins, pickling, `eval()`/`repr()` roundtrips, locale-dependent tests, `Decimal`/`Fraction` types, CPython-internal tests.

### Stretch Goal: Mine `mathdata/math_testcases.txt`

CPython ships external test data at `/Users/yaseralnajjar/work/sifr/cpython/Lib/test/mathdata/math_testcases.txt` containing hundreds of pre-computed math test vectors (input → expected output pairs for trig, exp, log, and special-value functions). Parsing and converting these into Sifr E2E test assertions would significantly increase math coverage beyond the hand-picked assertions in Tier 1. Requires building a small parser/converter to translate the data file format into `.sifr` test files.

### Definition of Done

- ~500 new test assertions across ~15 test files
- Total stdlib assertions: ~200 → ~700+
- Every exported math function has at least 5 assertions including edge cases
- `cargo test` passes (zero regressions)
- Stretch: `mathdata/math_testcases.txt` mined for additional math test vectors

---

## milestone_ordering_remediation: Fix Gaps from Out-of-Order Execution

status: done

**Goal:** The milestones in this phase were executed out of order. The plan specified `compiler_hardening → lazy_iterators → test_infra → stdlib_functions → stdlib_naming → stdlib_class_rollout → cpython_tests`, but the actual execution was `stdlib_class_rollout → cpython_tests → compiler_hardening → lazy_iterators`. This caused three categories of gaps that this ad-hoc milestone fixes:

### Gap 1: Eager itertools (lazy_iterators was done AFTER stdlib_functions)

The plan says: *"lazy_iterators after compiler_hardening: Lazy iteration is a compiler feature that should be in place before adding new stdlib functions. This way, itertools functions in milestone_stdlib_functions can be written as lazy generators from the start."*

All 9 functions in `lib/sifr/itertools.sifr` (`chain`, `chain_str`, `repeat_val`, `take`, `flatten`, `enumerate_list`, `pairwise`, `batched`, `islice`) are implemented eagerly — they build a `list` with `.append()` and return it. They should use `yield` to be lazy generators, now that the lazy iterator codegen (`std::iter::from_fn`) is in place.

### Gap 2: CPython tests use old names (cpython_tests was done BEFORE stdlib_naming was verified)

The CPython test files were written using old pre-rename function names:
- `cpython_json.sifr`: uses `json_loads`/`json_dumps` instead of `loads`/`dumps`
- `cpython_fnmatch.sifr`: uses `fnmatch_filter` instead of `filter`
- `cpython_re.sifr`: uses `re_match` instead of the CPython-compatible alias

These should be updated to use the new CPython-compatible names to validate that the naming alignment actually works end-to-end.

### Gap 3: Callable struct fields can't be called (compiler_hardening was done AFTER stdlib_class_rollout)

The plan says Callable-as-struct-field should unblock `argparse.ArgumentParser`, `collections.defaultdict`, and `timeit.Timer` — all of which need to store AND call callbacks. The current implementation only supports *storing* a Callable in a struct field (`Box<dyn Fn(...)>`), but *calling* it (`obj.callback(args)`) fails because the lowering treats it as a method call and errors with `"class has no method 'callback'"`. The E2E test `callable_struct_field.sifr` only tests storage, not invocation.

### Definition of Done

- All `sifr.itertools` functions use `yield` (lazy generators) instead of eager list building
- Existing `cpython_itertools.sifr` tests still pass with lazy itertools
- `cpython_json.sifr` uses `loads`/`dumps` (new names)
- `cpython_fnmatch.sifr` uses `filter` (new name)
- `cpython_re.sifr` uses CPython-compatible names consistently
- `obj.callable_field(args)` works — lowering detects Callable fields and emits a field-call
- `callable_struct_field.sifr` tests both storing AND calling the callback
- `cargo test` passes (zero regressions)
- Demo: `demos/ordering_rules/main.sifr`

---

## Milestone Ordering

```
milestone_stdlib_classes (done, Phase 6) → milestone_compiler_hardening → milestone_lazy_iterators → milestone_test_infra → milestone_stdlib_functions → milestone_stdlib_naming → milestone_stdlib_class_rollout → milestone_cpython_tests → milestone_error_safety (Phase 8)
```

Why this order:

- **compiler_hardening first:** Fixes compiler correctness issues that affect every subsequent milestone. Import errors catch typos immediately; `with` protocol unblocks `io.open()` and `tempfile` class APIs; `Callable`-as-struct-field fix unblocks `ArgumentParser`, `defaultdict`, and `timeit.Timer`.
- **lazy_iterators after compiler_hardening:** Lazy iteration is a compiler feature that should be in place before adding new stdlib functions. This way, `itertools` functions in `milestone_stdlib_functions` can be written as lazy generators from the start, and `csv.reader`/`glob.iglob` can be implemented properly in `milestone_stdlib_class_rollout`.
- **test_infra before stdlib_functions:** `assert_almost_eq` is needed to test float-returning functions.
- **stdlib_functions before stdlib_naming:** Add functions first (including generic `bisect`/`heapq`/`itertools`), then rename in one pass.
- **stdlib_naming before stdlib_class_rollout:** Classes should use final CPython-aligned names from the start.
- **stdlib_class_rollout before cpython_tests:** CPython test porting for class-based modules requires the classes to exist. `datetime`/`timedelta` with operator overloading validates the operator export pipeline.
- **cpython_tests last:** Validation layer against the final API surface.

---

## What's Explicitly Deferred

| Item | Blocker | When to Address |
|---|---|---|
| Exception types (`TOMLDecodeError`, `CycleError`) | Error types in stdlib `.sifr` files are untested — custom error classes work in user code (`custom_error.sifr`) but the export pipeline from stdlib is unproven | Next phase — requires validating error type export from stdlib, then defining module-specific error types |
