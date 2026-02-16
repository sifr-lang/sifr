# Stdlib Parity

This phase closes the remaining gaps between Sifr's stdlib and CPython's stdlib. It fixes two compiler correctness issues (import error reporting, `with` statement protocol), adds test infrastructure, expands the function surface, aligns API names with CPython, rolls out 5 new class-based APIs, and ports ~500 CPython test assertions as behavioral validation.

**Predecessor:** Phase 6 (Stdlib Architecture) — all 37 modules exist, the three-tier hybrid architecture is in place, and the class-in-stdlib pipeline is proven via `collections.Counter`.

**Successor:** Phase 8 (Ecosystem) — the async runtime, web framework, and all ecosystem milestones depend on a mature stdlib with both function and class APIs, CPython-compatible naming, and proper `with` statement support.

**Detailed plan:** [issues/stdlib_architecture_remaining_milestones.md](../../issues/stdlib_architecture_remaining_milestones.md)

---

## milestone_compiler_hardening: Import Errors and Context Managers

status: pending

**Goal:** Fix two compiler correctness gaps: (1) importing from a nonexistent module silently fails instead of producing a clear error, and (2) the `with` statement is incomplete — it's syntactic sugar for scoped blocks but doesn't implement the Python context manager protocol (`__enter__`/`__exit__`).

### Import Error Reporting

The import resolution in `lower.rs` silently falls through when a `sifr.*` module doesn't exist — no error is emitted, and the error only surfaces later as "undefined function" when the symbol is used. Fix: emit `"unknown stdlib module"` for nonexistent `sifr.*` modules and `"unknown module"` for nonexistent local modules, at the import site.

### `with` Statement Protocol

The `with` statement was implemented in `milestone_generators` but only as a minimal "scoped block" desugaring. Missing: `__enter__`/`__exit__` method calls, `ContextManager` protocol enforcement, multiple context managers (`with A() as a, B() as b:`), and E2E tests.

Fix:
- Define `ContextManager` as a built-in protocol with `__enter__(self) -> Self` and `__exit__(self) -> None`
- Lowering: check protocol compliance, bind variable to `__enter__()` result, handle multiple items
- Codegen: emit `__enter__()` call at scope start, `__exit__()` call at scope end

### Definition of Done

- `from sifr.nonexistent import foo` produces `"unknown stdlib module 'sifr.nonexistent'"`
- `from mymodule import bar` (nonexistent) produces `"unknown module 'mymodule'"`
- `with X() as y:` calls `__enter__()` and `__exit__()` correctly
- `with A() as a, B() as b:` handles all context managers
- Non-`ContextManager` type in `with` produces a compile error
- E2E pass tests: `with_enter_exit`, `with_multiple`
- E2E fail tests: `with_non_context_manager`, `import_nonexistent_local`
- `cargo test` passes (zero regressions)

---

## milestone_test_infra: Test Infrastructure for CPython Parity

status: pending

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

status: pending

**Goal:** Close function-level gaps. Add ~25 pure-Sifr functions and ~12 new intrinsics across 12 modules.

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

### New Intrinsics

- `_sifr.math`: `exp`, `expm1`, `log1p`, `fabs`, `isfinite`
- `_sifr.crypto`: `sha1`, `sha512`, `urlsafe_b64encode`, `urlsafe_b64decode`
- `_sifr.fs`: `gettempdir`, `makedirs`
- `_sifr.platform`: `platform_node`

### Definition of Done

- All ~25 pure-Sifr functions and ~12 intrinsics work correctly
- `sifr.math` coverage: ~85% → ~95%
- ~40 new test assertions pass
- `cargo test` passes (zero regressions)
- Demo: `demos/milestone_stdlib_functions_demo.sifr`

---

## milestone_stdlib_naming: API Naming Alignment with CPython

status: pending

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

Renames happen at the Sifr stdlib layer only — `_sifr.*` intrinsic names stay unchanged. Each `.sifr` file re-exports with the new name via wrapper functions. Old names kept as aliases initially.

### Definition of Done

- All renamed functions work with CPython-compatible names
- All E2E tests updated and passing
- Old names still work as aliases
- `cargo test` passes (zero regressions)

---

## milestone_stdlib_class_rollout: Expand Class-Based APIs

status: pending

**Goal:** Add 5 new class-based APIs leveraging the pipeline proven by `milestone_stdlib_classes` (Counter). These classes don't need `Callable`-as-struct-field.

### Classes

1. **`sifr.graphlib.TopologicalSorter`** — pure algorithmic class with `add`/`static_order`
2. **`sifr.pathlib.Path`** — file path class wrapping existing `_sifr.fs` intrinsics (`name`, `parent`, `suffix`, `stem`, `exists`, `is_file`, `is_dir`, `read_text`, `write_text`, `mkdir`, `joinpath`, `resolve`)
3. **`sifr.logging.Logger`** — named loggers with level filtering (`debug`, `info`, `warning`, `error`, `critical`) + `getLogger` factory
4. **`sifr.re.Match`** — regex match result with `group`, `start`, `end`, `span`
5. **`sifr.uuid.UUID`** — UUID value class with `hex`, `urn`, `to_str`, `version`

### Definition of Done

- All 5 classes compile, export, and are importable by user code
- `from sifr.pathlib import Path` works end-to-end
- `from sifr.logging import getLogger` works with level filtering
- `from sifr.graphlib import TopologicalSorter` works with `add`/`static_order`
- `from sifr.re import Match` works; `search` returns `Match` objects
- `from sifr.uuid import UUID` works; `uuid4()` returns `UUID` object
- `cargo test` passes (zero regressions)
- Demo: `demos/milestone_stdlib_class_rollout_demo.sifr`

---

## milestone_cpython_tests: Port CPython Test Assertions

status: pending

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

## Milestone Ordering

```
milestone_stdlib_classes (done, Phase 6) → milestone_compiler_hardening → milestone_test_infra → milestone_stdlib_functions → milestone_stdlib_naming → milestone_stdlib_class_rollout → milestone_cpython_tests → milestone_async (Phase 8)
```

Why this order:

- **compiler_hardening first:** Fixes compiler correctness issues that affect every subsequent milestone. Import errors catch typos immediately; `with` protocol unblocks `io.open()` and `tempfile` class APIs.
- **test_infra before stdlib_functions:** `assert_almost_eq` is needed to test float-returning functions.
- **stdlib_functions before stdlib_naming:** Add functions first, then rename in one pass.
- **stdlib_naming before stdlib_class_rollout:** Classes should use final CPython-aligned names from the start.
- **stdlib_class_rollout before cpython_tests:** CPython test porting for class-based modules requires the classes to exist.
- **cpython_tests last:** Validation layer against the final API surface.

---

## What's Explicitly Deferred

| Item | Blocker | When to Address |
|---|---|---|
| `argparse.ArgumentParser` | `Callable`-as-struct-field (`Box<dyn Fn>`) | After codegen fix milestone |
| `collections.defaultdict` class | `Callable`-as-struct-field | After codegen fix milestone |
| `timeit.Timer` class | `Callable`-as-struct-field | After codegen fix milestone |
| Generic `bisect`/`heapq`/`itertools` | Generics milestone | After `milestone_generics_impl` |
| Lazy iterators (`iglob`, `csv.reader`) | Iterator protocol | After generators/iterators milestone |
| `datetime`/`timedelta` class arithmetic | Operator overloading for stdlib classes | After operator overloading export is proven |
| Exception types (`TOMLDecodeError`, `CycleError`) | Exception/error type support | After error handling milestone |
