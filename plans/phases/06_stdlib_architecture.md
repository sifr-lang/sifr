# Stdlib Architecture

This phase rewires the stdlib from compiler-emitted Rust code into a three-tier hybrid architecture: Rust intrinsics (`_sifr.*`) at the bottom, Sifr stdlib modules (`sifr.*`) as `.sifr` files in the middle, and user code on top. It then migrates all existing modules, adds ~24 new modules, closes API gaps with a parity audit, polishes naming and test coverage, and proves the class-in-stdlib pipeline end-to-end with `collections.Counter`.

---

## milestone_intrinsics: Intrinsics Layer and Stdlib Compilation Pipeline

status: completed

**Goal:** Rewire how stdlib works internally. Introduce the three-tier hybrid architecture: Rust intrinsics (`_sifr.*`) at the bottom, Sifr stdlib modules (`sifr.*`) as `.sifr` files in the middle, and user code on top. No new user-facing features, but establishes the architecture everything else builds on.

**Full plan:** historical local Cursor plan `hybrid_stdlib_architecture_67d3c0a1.md` (removed)

### Three-Tier Model

- **Tier 1: Rust Intrinsics (`_sifr.*`)** -- Compiler-provided primitives that map directly to Rust code. Intentionally minimal -- only operations that cannot be written in pure Sifr (OS access, unsafe code, Rust crate bindings). ~60 primitives across 10 modules (`_sifr.fs`, `_sifr.sys`, `_sifr.io`, `_sifr.time`, `_sifr.math`, `_sifr.crypto`, `_sifr.regex`, `_sifr.json`, `_sifr.toml`, `_sifr.datetime`).
- **Tier 2: Sifr Stdlib (`sifr.*`)** -- `.sifr` files that import from `_sifr.*` intrinsics and provide the user-facing API. Written in Sifr itself. Users can read the source to understand how things work.
- **Tier 3: User Code** -- Users import from `sifr.*` (Tier 2). They never need to touch `_sifr.*`.

### Compiler Changes

1. Rename current `sifr.*` registry to `_sifr.*` in the compiler-host stdlib contract (`sifr_stdlib`) -- mechanical rename of intrinsic module/member metadata and public stdlib module checks
2. Rename `emit_stdlib_call` to `emit_intrinsic_call` in [sifr_codegen/src/lib.rs](../../crates/sifr_codegen/src/lib.rs)
3. Split current 55 functions into initial intrinsic primitives across `_sifr.fs`, `_sifr.sys`, `_sifr.io`, `_sifr.time`, `_sifr.math`, `_sifr.crypto`, `_sifr.regex`, `_sifr.json`
4. Add `lib/sifr/` directory with `.sifr` files embedded via `include_str!`
5. Update driver ([sifr_driver/src/lib.rs](../../crates/sifr_driver/src/lib.rs)) to discover and compile embedded stdlib `.sifr` modules before user modules (two-phase compilation)
6. Update lowering import resolution to resolve stdlib `.sifr` files first, falling back to `_sifr.*` intrinsics
7. Update codegen to handle stdlib modules as regular Rust `mod`/`use` (not inline emit)
8. Block user imports of `_sifr.*` in lowering import resolution -- emit a compile error if user code tries to `from _sifr.X import Y` (only stdlib `.sifr` files may import intrinsics). Trust boundary: the compiler distinguishes stdlib from user code by checking whether the source originated from the embedded `lib/sifr/` module set (via `sifr_stdlib` source inventory), not by filename convention.
9. Proof-of-concept: `lib/sifr/test.sifr` (assert_eq, assert_ne, assert_true, assert_false are pure Sifr)

### Design Constraint: Safety Contract

All stdlib modules must uphold Sifr's safety guarantees:

1. **Fallible operations return `Result[T, E]` or `Option[T]`** -- file I/O, parsing, network calls, and any operation that can fail must return a `Result` or `Option`. No panics, no `.unwrap()` in user-facing APIs.
2. **`open()` returns a `File` context manager** -- `sifr.io.open()` returns `Result[File, IOError]`. The `File` object implements the context manager protocol (`with` statement) to guarantee resource cleanup.
3. **No raw pointers or unsafe code in Tier 2** -- all `unsafe` is confined to Tier 1 intrinsics. Tier 2 `.sifr` files are pure safe Sifr.
4. **Borrow-by-default applies uniformly** -- stdlib functions accept `&T` by default, `&mut T` when mutation is needed, and `T` (owned) only when the function must consume the value.
5. **No silent data loss** -- operations like `write_text` return `Result[None, IOError]`, not `None`. The caller must handle the error or propagate with `?`.

This contract is an **acceptance criterion for every milestone** in this phase.

### Definition of Done (milestone_intrinsics)

- `from sifr.test import assert_eq` resolves to the `.sifr` file, compiles, and works
- All existing E2E tests still pass (old modules still use intrinsics path during transition)
- `_sifr.*` imports are blocked for user code with a clear compile error
- Two-phase compilation pipeline works (stdlib compiled before user code)
- E2E pass tests: stdlib_import_test, intrinsics_block_test
- E2E fail tests: user_imports_intrinsics_rejected

---

## milestone_stdlib_migration: Migrate Existing 13 Modules to Sifr

status: completed

**Goal:** Port all 13 existing stdlib modules from Rust codegen to `.sifr` files. Each module becomes a thin wrapper importing from `_sifr.*` intrinsics. At the end, `emit_stdlib_call` is deleted.

**Full plan:** historical local Cursor plan `hybrid_stdlib_architecture_67d3c0a1.md` (removed)

### Modules to Migrate (in dependency order)

1. `lib/sifr/env.sifr` -- wraps `_sifr.sys` (env_get, env_set) -- simplest, good first migration
2. `lib/sifr/bytes.sifr` -- wraps `_sifr.io` (encode_utf8, decode_utf8, to_hex, from_hex)
3. `lib/sifr/base64.sifr` -- wraps `_sifr.crypto` or pure Sifr (b64encode, b64decode)
4. `lib/sifr/math.sifr` -- wraps `_sifr.math` (12 functions + pi, e constants)
5. `lib/sifr/hashlib.sifr` -- wraps `_sifr.crypto` (sha256, md5)
6. `lib/sifr/io.sifr` -- wraps `_sifr.fs` + `_sifr.io` (read_text, write_text, exists, read_lines, `open()` / `File` context manager). Needs new intrinsics: `_sifr.fs.open_file`, `read_fd`, `write_fd`, `close_fd`
7. `lib/sifr/os.sifr` -- wraps `_sifr.sys` + `_sifr.fs` (run_command, get_args)
8. `lib/sifr/json.sifr` -- wraps `_sifr.json` (json_loads, json_dumps)
9. `lib/sifr/time.sifr` -- wraps `_sifr.time` (time_now, sleep, time_format)
10. `lib/sifr/random.sifr` -- wraps `_sifr.crypto` (random_int, random_float, random_choice)
11. `lib/sifr/re.sifr` -- wraps `_sifr.regex` (re_match, re_find, re_replace)
12. `lib/sifr/collections.sifr` -- wraps existing set/counter/defaultdict intrinsics
13. `lib/sifr/test.sifr` -- already done in milestone_intrinsics (verify still works)

**Note:** During migration, two modules are renamed to match Python conventions: `sifr.hash` -> `sifr.hashlib`, `sifr.encoding` -> `sifr.base64`. This is a deliberate pre-1.0 breaking change; existing tests and code must be updated as part of this milestone.

### Final Cleanup

- Delete the ~430-line `emit_stdlib_call` function in codegen
- Delete the old `sifr.*` entries in `get_stdlib_module()`
- Update Cargo dependency injection to trace through `_sifr.*` intrinsics

### Definition of Done (milestone_stdlib_migration)

- `emit_stdlib_call` is deleted
- Every `from sifr.X import Y` resolves to a `.sifr` file
- All fallible functions return `Result` or `Option` (safety contract)
- All existing E2E tests, audit tests, and stdlib tests pass with zero regressions
- `sifr.hash` and `sifr.encoding` references updated to `sifr.hashlib` and `sifr.base64`

---

## milestone_stdlib_expansion: New Modules (Algorithms, CLI, File Utilities)

status: completed

**Goal:** Add ~14 new modules. These are the most commonly needed modules that Python developers reach for daily. Ordered by dependency and implementation complexity (pure Sifr first, then intrinsic-backed).

**Full plan:** historical local Cursor plan `hybrid_stdlib_architecture_67d3c0a1.md` (removed)

### Pure Sifr Modules (no new intrinsics needed)

1. `lib/sifr/string.sifr` -- `ascii_letters`, `digits`, `punctuation`, `whitespace` constants
2. `lib/sifr/statistics.sifr` -- `mean`, `median`, `stdev`, `variance`
3. `lib/sifr/bisect.sifr` -- `bisect_left`, `bisect_right`, `insort`
4. `lib/sifr/heapq.sifr` -- `heappush`, `heappop`, `heapify`, `nlargest`, `nsmallest`
5. `lib/sifr/functools.sifr` -- `reduce`
6. `lib/sifr/itertools.sifr` -- `chain`, `zip_longest`, `groupby`
7. `lib/sifr/textwrap.sifr` -- `wrap`, `fill`, `dedent`, `indent`
8. `lib/sifr/csv.sifr` -- `reader`, `writer`
9. `lib/sifr/argparse.sifr` -- `ArgumentParser` class with `add_argument`, `parse_args`

### Intrinsic-backed Modules (need new `_sifr.*` primitives)

10. `lib/sifr/fnmatch.sifr` -- `fnmatch`, `filter`, `translate` (wraps `_sifr.regex`)
11. `lib/sifr/glob.sifr` -- `glob`, `iglob` (wraps `_sifr.fs.list_dir` + fnmatch)
12. `lib/sifr/shutil.sifr` -- `copy`, `copytree`, `rmtree`, `move` (wraps `_sifr.fs` -- needs new intrinsics: `copy_file`, `walk_dir`)
13. `lib/sifr/tempfile.sifr` -- `mkstemp`, `mkdtemp` (wraps `_sifr.fs` + `_sifr.crypto.random_bytes`)
14. `lib/sifr/secrets.sifr` -- `token_hex`, `token_urlsafe`, `token_bytes`, `choice` (wraps `_sifr.crypto`)

**New intrinsics needed:** `_sifr.fs.copy_file`, `_sifr.fs.walk_dir` (2 new primitives added to existing `_sifr.fs`)

### Definition of Done (milestone_stdlib_expansion)

- Each new module compiles, imports work, functions produce correct output
- All fallible functions return `Result` or `Option` (safety contract)
- No panic paths in stdlib code
- E2E tests for each module, including negative tests (bad input)
- Language gaps discovered during dogfooding are filed as issues

---

## milestone_stdlib_parity: Gap Closing, Remaining Modules, and Audit

status: completed

**Goal:** Three parts: (A) close gaps in existing modules by adding missing functions, (B) add remaining Tier 1+2 modules, (C) run the comprehensive parity audit.

**Full plan:** historical local Cursor plan `hybrid_stdlib_architecture_67d3c0a1.md` (removed)

### Part A -- Expand Existing Modules

- `sifr/math.sifr` -- add ~20 missing functions: `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `exp`, `log2`, `log10`, `log1p`, `factorial`, `gcd`, `lcm`, `isnan`, `isinf`, `isfinite`, `fmod`, `hypot`, `tau`, `inf` (needs new `_sifr.math` intrinsics for inverse trig and hyperbolic)
- `sifr/os.sifr` -- add `getcwd`, `listdir`, `mkdir`, `makedirs`, `rename`, `remove`, `walk`
- `sifr/re.sifr` -- add `findall`, `split`
- `sifr/random.sifr` -- add `shuffle`, `sample`, `seed`, `uniform`, `randrange`
- `sifr/io.sifr` -- add `append_text`, binary I/O
- `sifr/collections.sifr` -- add `deque`, `OrderedDict`
- `sifr/time.sifr` -- add `monotonic`, `perf_counter`
- `sifr/hashlib.sifr` -- add `sha1`, `sha512`, `hmac`
- `sifr/base64.sifr` -- add `urlsafe_b64encode`, `urlsafe_b64decode`, `b32encode`, `b32decode`
- `sifr/itertools.sifr` -- add `combinations`, `permutations`, `product`, `accumulate`
- `sifr/functools.sifr` -- add `partial` (**stretch goal** -- requires closure capture support; skip if not available by M4, revisit when closures mature)

### Part B -- New Modules (remaining Tier 1+2)

1. `lib/sifr/difflib.sifr` -- `unified_diff`, `get_close_matches`, `SequenceMatcher` (pure Sifr, algorithmic)
2. `lib/sifr/graphlib.sifr` -- `TopologicalSorter` (pure Sifr, algorithmic)
3. `lib/sifr/ipaddress.sifr` -- `ip_address`, `ip_network` (pure Sifr, parsing + math)
4. `lib/sifr/timeit.sifr` -- `timeit`, `repeat` (wraps `_sifr.time.perf_counter_ns`)
5. `lib/sifr/platform.sifr` -- `system`, `machine`, `architecture` (wraps `_sifr.sys.platform_os`, `platform_arch`)
6. `lib/sifr/tomllib.sifr` -- `loads`, `load` (wraps new `_sifr.toml` intrinsic)
7. `lib/sifr/datetime.sifr` -- `date`, `datetime`, `timedelta`, `timezone` (wraps new `_sifr.datetime` intrinsic)
8. `lib/sifr/pathlib.sifr` -- `Path` class with `/` operator, `exists`, `read_text`, `write_text`, `stem`, `suffix`, `parent` (wraps `_sifr.fs`)
9. `lib/sifr/uuid.sifr` -- `uuid4` (wraps `_sifr.crypto.random_bytes`)
10. `lib/sifr/logging.sifr` -- `Logger`, `getLogger`, `info`, `warning`, `error`, `debug` (wraps `_sifr.io` + `_sifr.time`)

**New intrinsics needed:** `_sifr.toml.toml_parse`, `_sifr.datetime.*` (4 primitives), `_sifr.sys.platform_os`, `_sifr.sys.platform_arch`, `_sifr.math` inverse trig/hyperbolic (~8 primitives)

### Part C -- Parity Audit

- Run the comprehensive stdlib parity audit from historical local Cursor plan `stdlib_parity_audit_2c354444.md` (removed) (~200 test files across 30 directories)
- Produce stdlib-parity coverage evidence under `verification/areas/stdlib_parity/reports/` with coverage percentages per module
- Target: 60%+ coverage across the top 20 CPython modules
- **Reference:** CPython stdlib source is available at `/Users/yaseralnajjar/work/sifr/cpython` for comparing implementations and verifying API surfaces

### Definition of Done (milestone_stdlib_parity)

- All expanded modules pass their tests
- All new modules compile and work
- All fallible functions return `Result` or `Option` (safety contract)
- No panic paths in stdlib code
- Negative tests (bad input) for each module
- Parity audit report generated with coverage metrics
- `cargo test` passes
- 37 total stdlib modules available (13 pure Sifr + 24 intrinsic-backed)

---

## milestone_stdlib_polish: Stdlib API Alignment, Test Coverage, and Cleanup

status: completed

**Goal:** Polish the stdlib to align API names with the architecture plan, fill test coverage gaps, and clean up stale code. This milestone addresses reviewer findings that don't require new language features or compiler-level changes.

**Full plan:** [plans/issues/archive/milestone_stdlib_polish.md](../issues/archive/milestone_stdlib_polish.md)

**Context:** The Stdlib Architecture Phase delivered 37 modules with full compilation pipeline support. However, a reviewer audit identified: (1) function names that don't match the plan, (2) missing E2E tests for 3 modules, (3) thin negative/fail test coverage, and (4) a stale comment in lower.rs. The safety contract (Result/Option) and class-based APIs are deferred to future milestones.

### API Alignment (renames to match CPython)

- `glob.sifr`: `glob_match` → `glob` (matches `glob.glob()`)
- `shutil.sifr`: `copy_file` → `copy`, `move_file` → `move`, add `rmtree` (matches `shutil.copy/move/rmtree`)
- `timeit.sifr`: full CPython API -- `default_timer()` backed by `perf_counter`, plus `timeit(stmt, number)` and `repeat(stmt, repeat, number)` using existing `Callable` type support (no new language features needed)
- `tomllib.sifr`: add `load(path)` (pragmatic adaptation of `tomllib.load(fp)` since Sifr lacks file objects)

### New Intrinsics

**`_sifr.time` (monotonic clocks via `std::time::Instant`):**
- `perf_counter() -> float` -- high-resolution monotonic clock for benchmarking (matches `time.perf_counter()`)
- `monotonic() -> float` -- guaranteed non-decreasing clock for timeouts (matches `time.monotonic()`)

**`_sifr.fs` (file operations):**
- `copy_file(src, dst)` -- wraps `std::fs::copy`
- `walk_dir(path)` -- wraps recursive `std::fs::read_dir`
- `rmdir_all(path)` -- wraps `std::fs::remove_dir_all`

### Stdlib Re-exports and New Functions

- `sifr.time` adds `perf_counter`, `monotonic` (from `_sifr.time`)
- `sifr.timeit` rewritten with full CPython API:
  - `default_timer()` → `perf_counter()`
  - `timeit(stmt: Callable[[], None], number: int)` → run stmt N times, return total seconds
  - `repeat(stmt: Callable[[], None], repeat: int, number: int)` → run timeit() M times, return list[float]
  - Old `timer`/`elapsed` removed

### Missing E2E Pass Tests

- `stdlib_glob.sifr` -- test glob with directory listing
- `stdlib_shutil.sifr` -- test copy/move
- `stdlib_tempfile.sifr` -- test mkstemp/mkdtemp

### New E2E Fail Tests (negative coverage)

- `stdlib_invalid_module.sifr` -- import nonexistent `sifr.nonexistent`
- `stdlib_wrong_type.sifr` -- pass wrong type to stdlib function
- `stdlib_missing_function.sifr` -- import nonexistent function from valid module
- `stdlib_intrinsic_direct_import.sifr` -- another `_sifr.*` direct import attempt
- `stdlib_readonly_param.sifr` -- attempt to mutate a borrowed stdlib parameter

### Cleanup

- Fix stale fallback comment in `lower.rs`
- Fix `has_pure_sifr_code` check in `sifr_driver` to include classes (future-proofing)
- Update stdlib-parity reports under `verification/areas/stdlib_parity/reports/` with final metrics

### Not included (and why)

- **`timeit.Timer` class:** Functional API covers 100% of the functionality. Also blocked by a codegen issue: `Callable` emits `impl Fn(...)` which Rust rejects in struct fields (needs `Box<dyn Fn(...)>` -- a small fix, but not needed since the functional API suffices).
- **Class-based stdlib APIs** (ArgumentParser, Logger, Path, File): Same `Callable`-in-struct-field codegen issue applies to any class storing callbacks. Infrastructure for classes in stdlib `.sifr` files otherwise exists (parsing, lowering, export, import resolution, codegen all wired up).

### Definition of Done (milestone_stdlib_polish)

- `perf_counter` and `monotonic` intrinsics work (backed by `std::time::Instant`)
- `sifr.time` re-exports `perf_counter` and `monotonic`
- `sifr.timeit` has full CPython API: `default_timer` (uses `perf_counter`), `timeit(stmt, number)`, `repeat(stmt, repeat, number)` using `Callable` type
- All renamed functions work and existing tests updated
- E2E pass tests for glob, shutil, tempfile
- At least 5 new stdlib fail tests
- Stale comment fixed
- `has_pure_sifr_code` check includes classes
- `cargo test` passes (zero regressions)
- Parity report updated
- Demo: `demos/stdlib_tools/main.sifr`

---

## milestone_stdlib_classes: Class-Based APIs in Sifr Standard Library

status: completed

**Goal:** Prove the stdlib class pipeline end-to-end by implementing `collections.Counter` as the first class defined in a stdlib `.sifr` file. This unblocks 12+ modules that need class-based APIs to reach CPython parity.

**Full plan:** [plans/issues/archive/milestone_stdlib_classes.md](../issues/archive/milestone_stdlib_classes.md)

**Context:** The CPython parity audit identified class-based APIs as the single biggest blocker — 12+ modules (argparse, csv, logging, pathlib, graphlib, uuid, collections, datetime, re, tempfile, difflib) need classes. The compiler already supports user-defined classes (constructors, methods, `&self`/`&mut self` inference, inheritance, protocols, `isinstance`), and the driver already exports classes via `ExternalDefs.classes`. However, **no stdlib `.sifr` module has ever defined a class** — the pipeline is wired but unproven.

### Why `Counter` as the Proof-of-Concept

1. **Source-owned behavior:** `Counter[T: Hashable]` stores a checked
   `dict[T, int]`; it has no compiler intrinsic or JSON-serialized backing.
2. **Full pipeline exercise:** class in `.sifr` → HIR lowering →
   `ExternalDefs.classes` export → user import → normal class codegen.
3. **Both receiver types:** read methods and mutating methods exercise the
   complete stdlib class receiver path.
4. **Well-known API:** CPython's `Counter` is familiar and easily testable.

### Counter Class API

```sifr
class Counter[T: Hashable]:
    counts: dict[T, int]

    def __init__(self, source: dict[T, int] | None = None,
                 iterable: list[T] | None = None) -> None:
        # Checked Sifr source constructs and updates counts.
        ...

    def get(self, key: T, default: int = 0) -> int:
        ...

    def increment(self, key: T) -> None:
        ...

    def most_common(self, n: int | None = None) -> list[tuple[T, int]]:
        ...

def from_list[T: Hashable](items: list[T]) -> Counter[T]:
    ...
```

### Final Counter Ownership

| Layer | Responsibility |
| --- | --- | --- |
| `stdlib/sifr/collections.sifr` | Generic storage, construction, queries, mutation, arithmetic, and ordering policy. |
| HIR/type system | Generic bounds, dict-key hashability, class export, receiver and operator typing. |
| Codegen | Normal checked class/dict lowering; no Counter-specific dispatch. |
| Retained manifest | No Counter row or dependency feature. |

### Pipeline Verification Points

1. **HIR lowering:** `Counter` class in `collections.sifr` lowered to `HirClass` with fields, methods, receiver inference
2. **Driver export:** `compile_stdlib()` populates `ExternalDefs.classes["sifr.collections"]["Counter"]` with `Type::Class`
3. **User import:** `from sifr.collections import Counter` resolves via `externals.classes` lookup, registers constructor
4. **Codegen:** `pub struct Counter`, `pub fn new(...)`, methods emitted with correct `&self`/`&mut self`

### Final Implementation and Evidence

- `stdlib/sifr/collections.sifr` — generic source-owned `Counter` and
  `from_list`.
- `crates/sifr/tests/e2e/pass/stdlib_collections_counter.sifr` — basic Counter construction + method calls
- `crates/sifr/tests/e2e/pass/stdlib_collections_counter_mutate.sifr` — Counter mutation via `increment`
- `crates/sifr/tests/e2e/fail/stdlib_counter_wrong_type.sifr` — wrong argument type to Counter
- `demos/stdlib_classes/main.sifr` — demo showcasing Counter class usage
- `scripts/check_stdlib_native_intrinsic_allowlist.py` — permanently rejects
  restoration of the removed Counter compiler path.

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Class export pipeline has untested edge cases | Medium | High | This milestone specifically exercises and validates this path |
| Method receiver inference wrong for stdlib classes | Low | Medium | Already proven in user-defined class E2E tests |
| `pub_mode` doesn't apply correctly to class methods | Low | High | Codegen already handles `pub_mode` for struct, impl, and methods |
| Generic dict operations regress | Low | Medium | Counter E2E and type-system coverage exercises construction, mutation, queries, and arithmetic. |

### What This Unblocks

| Next Class | Module | Additional Compiler Work Needed |
| --- | --- | --- |
| `Path` | `sifr.pathlib` | Operator overloading export from stdlib |
| `Match` | `sifr.re` | None — wraps existing regex intrinsics |
| `Logger` | `sifr.logging` | None — wraps existing logging intrinsics |
| `TopologicalSorter` | `sifr.graphlib` | None — pure algorithmic class |
| `ArgumentParser` | `sifr.argparse` | `Callable`-as-struct-field fix (`Box<dyn Fn>`) |
| `defaultdict` | `sifr.collections` | `Callable`-as-struct-field fix (`Box<dyn Fn>`) |
| `datetime`/`timedelta` | `sifr.datetime` | Operator overloading for arithmetic |
| `DictReader`/`DictWriter` | `sifr.csv` | Iterator protocol |

### Definition of Done (milestone_stdlib_classes)

- `Counter` class defined in `lib/sifr/collections.sifr` with `__init__`, `get`, `most_common`, `total`, `values`, `keys`, `items`, `increment` methods
- `from_list` factory function works
- 5 new `_sifr.collections` intrinsics implemented and tested
- User code can `from sifr.collections import Counter` and use it
- E2E pass tests for construction + methods, and mutation
- E2E fail test for wrong argument type
- `cargo test` passes (zero regressions)
- Parity report updated
- Demo: `demos/stdlib_classes/main.sifr`

---

## Rust Crate Mapping for Intrinsics

Each `_sifr.*` intrinsic module maps to specific Rust crates or std modules. When a Sifr stdlib module is used, the compiler traces through to the intrinsics it depends on and injects the appropriate Cargo dependencies.

| Intrinsic Module | Rust Backing | External Crate? |
|---|---|---|
| `_sifr.fs` | `std::fs`, `std::path` | No |
| `_sifr.sys` | `std::env`, `std::process` | No |
| `_sifr.io` | `std::io` | No |
| `_sifr.time` | `std::time`, `std::thread` | No |
| `_sifr.math` | `f64` methods (all transcendental functions are on `f64` in Rust std) | No |
| `_sifr.crypto` | `sha2`, `sha1`, `md5`, `rand` | Yes |
| `_sifr.regex` | `regex` | Yes |
| `_sifr.json` | `serde_json`, `serde` | Yes |
| `_sifr.toml` | `toml` | Yes |
| `_sifr.datetime` | `chrono` | Yes |
| `_sifr.platform` | `std::env::consts`, `gethostname` | Partial (hostname needs `gethostname` crate) |

**Key insight:** 6 of 11 intrinsic modules use only Rust std -- no external dependencies. Only `_sifr.crypto`, `_sifr.regex`, `_sifr.json`, `_sifr.toml`, `_sifr.datetime`, and `_sifr.platform` (for hostname) need external crates.

---

## Module Triage: What to Port, Defer, or Skip

CPython has ~289 stdlib modules. Sifr targets 37 in this phase. The rest are triaged as follows.

### Modules Deferred to Ecosystem Phase

These depend on async, networking, or threading -- features that come after this phase:

- `socket`, `ssl`, `http`, `urllib` -- networking (needs `_sifr.net` + async runtime)
- `asyncio` -- IS the async milestone itself
- `threading`, `queue`, `multiprocessing`, `concurrent` -- concurrency primitives
- `selectors` -- I/O multiplexing (async runtime internal)
- `subprocess` -- full Popen API needs async; partially covered by `os.run_command`
- `sqlite3` -- in `milestone_database` roadmap
- `xml`, `html` -- parsing libraries (add during web milestone)
- `email` -- in `milestone_email` roadmap
- `gzip`, `bz2`, `lzma`, `zipfile`, `tarfile` -- compression (needs Rust crate bindings, add on demand)
- `decimal`, `fractions` -- arbitrary precision (needs `rust_decimal` crate, add on demand)

### Modules Never Ported

These exist because of Python's specific nature (interpreted, dynamic, REPL-oriented) and have no meaningful equivalent in a compiled, statically-typed language:

- **Python compiler internals:** `ast`, `dis`, `symtable`, `tokenize`, `token`, `keyword`, `code`, `codeop`, `compileall`, `py_compile`
- **Python import machinery:** `importlib`, `pkgutil`, `modulefinder`, `runpy`, `zipimport`
- **Runtime introspection:** `inspect`, `types`, `typing`, `abc`, `numbers`, `operator`
- **Python debugger/profiler:** `pdb`, `bdb`, `profile`, `cProfile`, `pstats`, `trace`, `traceback`, `tracemalloc`
- **Python serialization:** `pickle`, `pickletools`, `shelve`, `copyreg`
- **Python environment:** `warnings`, `__future__`, `annotationlib`, `site`, `_sitebuiltins`, `ensurepip`, `venv`
- **GUI/terminal:** `idlelib`, `tkinter`, `turtle`, `turtledemo`, `curses`
- **C FFI:** `ctypes`, `struct` (Sifr will have its own FFI)
- **Not needed in Sifr:** `dataclasses` (compiler auto-derives), `enum` (union types + classes), `copy` (compiler-derived `.clone()`), `pprint` (auto-derived `Debug`), `contextlib` (Result-based errors), `weakref` (ownership model eliminates most use cases)
- **Niche/deprecated:** `antigravity`, `this`, `gettext`, `locale`, `optparse`, `getopt`, `wave`, `pty`, `tty`, `webbrowser`, `netrc`, `mailbox`, `mimetypes`, `quopri`, `stringprep`, `reprlib`, `sched`, `filecmp`, `fileinput`, `linecache`, `tabnanny`, `stat`, `signal`, `contextvars`, `sysconfig`, `rlcompleter`, `codecs`, `encodings`

### Modules to Revisit Later

- `signal` -- OS signal handling (revisit if users need graceful shutdown without async)
- `weakref` -- if ownership model needs weak references for specific patterns
- `decimal` / `fractions` -- if financial/scientific computing becomes a priority
- `gzip` / `zipfile` / `tarfile` -- if compression is commonly requested
- `xml` / `html` -- if web scraping becomes a use case before the web milestone
- `configparser` -- INI file parsing; TOML covers the config use case for a new language
- `colorsys` -- color space conversions; very niche, better as a third-party package

---

## Why NOT Full Rust Interop for Stdlib

Full Rust interop solves a different problem: letting package authors expose Rust-backed Sifr declarations through checked Cargo integration and bridge contracts. For stdlib, the intrinsics approach is:

- **Simpler:** No `unsafe` keyword, no extern blocks, no type marshaling
- **Safer:** Intrinsics are compiler-controlled, always correct
- **Faster to ship:** Reuses the existing `emit_stdlib_call` mechanism
- **Forward-compatible:** Rust interop can later back selected stdlib internals without changing the stdlib `.sifr` files

---

## Milestone ordering

Why the milestones within this phase are in this order:

- **milestone_intrinsics before milestone_stdlib_migration:** The intrinsics layer (`_sifr.*`) and two-phase compilation pipeline must exist before any stdlib module can be ported to `.sifr` files. This milestone establishes the architecture; migration uses it.
- **milestone_stdlib_migration before milestone_stdlib_expansion:** All 13 existing stdlib modules must be ported to `.sifr` files (and `emit_stdlib_call` deleted) before adding new modules. This ensures new modules are written against the final architecture, not the legacy codegen path.
- **milestone_stdlib_expansion before milestone_stdlib_parity:** New pure-Sifr and intrinsic-backed modules (~14) are added before the gap-closing and parity audit. Expansion adds the modules; parity fills in missing functions and validates coverage.
- **milestone_stdlib_parity before milestone_stdlib_polish:** Parity adds all remaining modules and fills API gaps. Polish then aligns the API names with CPython, adds missing intrinsics (perf_counter, monotonic), and fills test coverage gaps identified by the parity audit.
- **milestone_stdlib_polish before milestone_stdlib_classes:** Polish ensures all function-level stdlib APIs are correct and tested before adding the first class-based stdlib module. Classes introduce a new pipeline (class parsing, lowering, export, import) that should build on a stable function-level foundation.
