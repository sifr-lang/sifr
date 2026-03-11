## Product Requirements & Solution Design

---

### 1. Product Requirements

#### **Title**

milestone_stdlib_polish: Stdlib API Alignment, Test Coverage, and Cleanup

---

#### **Objective / Problem Statement**

The Stdlib Architecture Phase delivered 37 modules with full compilation pipeline support, but a reviewer audit identified several remaining gaps:
- Several module APIs use different function names than the architecture plan specifies
- Three intrinsic-backed modules (glob, shutil, tempfile) lack E2E tests
- Negative/fail-side test coverage for stdlib is thin (only 2 fail tests)
- A stale comment in lower.rs references a fallback path that no longer applies

These are polish items that don't require new language features but are needed to fully satisfy the phase's Definition of Done.

Note: The safety contract (Result/Option for fallible ops) and class-based APIs (ArgumentParser, Logger, Path, File) are explicitly deferred -- they require compiler-level work and new language features respectively.

---

#### **Constraints**

| Constraint | Rationale |
| --- | --- |
| No new language features required | All changes use existing Sifr capabilities |
| No compiler architecture changes | Only stdlib .sifr files, tests, and minor codegen touch-ups |
| Safety contract deferred | Result/Option threading is a separate milestone |
| Class-based APIs deferred | Requires class support in stdlib modules (future milestone) |

---

#### **Scope**

##### Features In

1. Rename module functions to match CPython's API names where feasible without classes
2. Add `perf_counter` and `monotonic` intrinsics to `_sifr.time` (uses `std::time::Instant`, no external deps), re-export in `sifr.time`
3. Implement full `sifr.timeit` API: `default_timer` (backed by `perf_counter`), `timeit(stmt, number)` and `repeat(stmt, repeat, number)` using existing `Callable` type support
4. Add missing E2E pass tests for glob, shutil, tempfile
5. Add negative/fail E2E tests for stdlib error paths
6. Fix stale comment in lower.rs
7. Add missing `_sifr.fs.copy_file` and `_sifr.fs.walk_dir` intrinsics for shutil
8. Update parity report with final metrics

##### Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| Result/Option safety contract | Requires compiler-level intrinsic type changes (separate milestone) |
| open()/File context manager | Requires `with` statement + class support in stdlib |
| ArgumentParser class | Requires `Callable` as struct field (codegen emits `impl Fn` which Rust rejects in struct fields; needs `Box<dyn Fn>` fix) |
| Logger/getLogger class API | Same `Callable`-in-struct-field blocker; also needs class support in stdlib |
| Path class with operator overloading | Requires class + operator support in stdlib |
| timeit.Timer class | Functional API (`timeit`/`repeat`/`default_timer`) covers 100% of the functionality; Timer class adds no new capability, just OOP style. Also blocked by `Callable`-as-struct-field codegen issue. |
| time.process_time() / time.thread_time() | Requires `libc` crate for platform-specific CPU clocks; niche usage, defer to future milestone |

---

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | glob.sifr exports `glob` (not `glob_match`) and has an E2E pass test |
| AC-2 | shutil.sifr exports `copy`, `move`, `rmtree` with `_sifr.fs` intrinsics and has an E2E pass test |
| AC-3 | tempfile.sifr has an E2E pass test |
| AC-4 | At least 5 new stdlib fail tests covering bad imports, type mismatches, and invalid usage |
| AC-5 | `_sifr.time` has `perf_counter` and `monotonic` intrinsics backed by `std::time::Instant` |
| AC-6 | `sifr.time` re-exports `perf_counter` and `monotonic` (matching CPython's `time.perf_counter()` / `time.monotonic()`) |
| AC-7 | `sifr.timeit.default_timer()` uses `perf_counter` (not wall clock) |
| AC-8 | `sifr.timeit.timeit(stmt, number)` accepts a `Callable[[], None]` and returns elapsed seconds |
| AC-9 | `sifr.timeit.repeat(stmt, repeat, number)` returns `list[float]` of timing results |
| AC-10 | tomllib.sifr exports `loads` and `load` |
| AC-11 | lower.rs fallback comment is updated to reflect current behavior |
| AC-12 | All existing tests pass (zero regressions) |
| AC-13 | Parity report updated |

---

## 2. Solution Design

### 2.1 Functional Requirements

**API renames (matching CPython's function names):**
- `glob.sifr`: rename `glob_match` → `glob` (matches `glob.glob()`)
- `shutil.sifr`: rename `copy_file` → `copy`, `move_file` → `move`, add `rmtree` (matches `shutil.copy()`, `shutil.move()`, `shutil.rmtree()`)
- `timeit.sifr`: full CPython-matching API using existing `Callable` type support:
  - `default_timer()` → calls `perf_counter()` (matches `timeit.default_timer()`)
  - `timeit(stmt: Callable[[], None], number: int)` → runs `stmt` `number` times, returns total seconds (matches `timeit.timeit()`)
  - `repeat(stmt: Callable[[], None], repeat: int, number: int)` → runs `timeit()` `repeat` times, returns `list[float]` (matches `timeit.repeat()`)
  - Remove old `timer`/`elapsed` (replaced by the above)
- `tomllib.sifr`: add `load` function that reads a file path then parses (pragmatic adaptation of `tomllib.load(fp)` since Sifr lacks file objects)

**Note on timeit:** Sifr already supports `Callable` type parameters (proven by `callable_type.sifr` and `callable_apply_twice.sifr` E2E tests). The `Callable` type emits `impl Fn(...)` in Rust codegen, which is exactly what's needed for `timeit(stmt, number)`. The `Timer` class is deferred (needs class support in stdlib).

**Note:** `time.process_time()` and `time.thread_time()` require `libc` for platform-specific CPU clocks. Deferred to a future milestone.

**New `_sifr.time` intrinsics (monotonic clocks):**
- `_sifr.time.perf_counter() -> float` -- wraps `std::time::Instant` via `OnceLock` baseline; high-resolution monotonic clock for benchmarking (matches `time.perf_counter()`)
- `_sifr.time.monotonic() -> float` -- same Rust implementation as `perf_counter`; guaranteed non-decreasing, for timeouts/scheduling (matches `time.monotonic()`)

**Codegen pattern (~10 lines each):**
```rust
"perf_counter" | "monotonic" => {
    self.write("{ static __START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new(); let __s = __START.get_or_init(std::time::Instant::now); __s.elapsed().as_secs_f64() }");
}
```

**New `_sifr.fs` intrinsics:**
- `_sifr.fs.copy_file(src: str, dst: str) -> None` -- wraps `std::fs::copy`
- `_sifr.fs.walk_dir(path: str) -> list[str]` -- wraps `std::fs::read_dir` recursively
- `_sifr.fs.rmdir_all(path: str) -> None` -- wraps `std::fs::remove_dir_all`

**Stdlib re-exports and new functions:**
- `sifr.time` adds re-exports: `perf_counter`, `monotonic`
- `sifr.timeit` rewritten with full CPython API:
  - `default_timer()` → `perf_counter()`
  - `timeit(stmt: Callable[[], None], number: int = 1000000)` → run stmt N times, return total seconds
  - `repeat(stmt: Callable[[], None], repeat: int = 5, number: int = 1000000)` → run timeit() M times, return list[float]
  - Old `timer`/`elapsed` removed

**New E2E pass tests:**
- `stdlib_glob.sifr` -- test glob with a real directory listing
- `stdlib_shutil.sifr` -- test copy/move with temp files
- `stdlib_tempfile.sifr` -- test mkstemp/mkdtemp

**New E2E fail tests:**
- `stdlib_invalid_module.sifr` -- import from nonexistent `sifr.nonexistent`
- `stdlib_wrong_type.sifr` -- pass wrong type to stdlib function
- `stdlib_missing_function.sifr` -- import nonexistent function from valid module
- `stdlib_intrinsic_direct.sifr` -- another `_sifr.*` import attempt (different module)
- `stdlib_readonly_param.sifr` -- attempt to mutate a borrowed stdlib parameter

### 2.2 Files to Change

- `crates/sifr_hir/src/stdlib.rs` -- add `perf_counter`, `monotonic` to `_sifr.time`; add `copy_file`, `walk_dir`, `rmdir_all` to `_sifr.fs`
- `crates/sifr_codegen/src/lib.rs` -- add codegen for `perf_counter`/`monotonic` (Instant + OnceLock) and `copy_file`/`walk_dir`/`rmdir_all`
- `lib/sifr/time.sifr` -- add re-exports for `perf_counter`, `monotonic`
- `lib/sifr/timeit.sifr` -- rewrite with full API: `default_timer`, `timeit(stmt, number)`, `repeat(stmt, repeat, number)` using `Callable` + `perf_counter`
- `lib/sifr/glob.sifr` -- rename `glob_match` → `glob`
- `lib/sifr/shutil.sifr` -- rename `copy_file` → `copy`, `move_file` → `move`, add `rmtree`
- `lib/sifr/tomllib.sifr` -- add `load` function
- `crates/sifr_driver/src/lib.rs` -- fix `has_pure_sifr_code` check to include `!result.module.classes.is_empty()` (future-proofing for class-containing stdlib modules)
- `crates/sifr_hir/src/lower.rs` -- fix stale comment
- `crates/sifr/tests/e2e/pass/` -- 3 new pass tests + update `stdlib_timeit.sifr` for new API
- `crates/sifr/tests/e2e/fail/` -- 5 new fail tests
- `audits/STDLIB_PARITY_MASTER_REPORT.md` -- update metrics
- `demos/milestone_stdlib_polish_demo.sifr` -- milestone demo

### 2.3 Testing Strategy

| AC-ID | Test Layer | Happy-Path Check | Non-Happy / Edge Check |
| --- | --- | --- | --- |
| AC-1 | E2E pass | glob lists directory entries matching pattern | N/A |
| AC-2 | E2E pass | copy creates duplicate, move removes source | N/A |
| AC-3 | E2E pass | mkstemp creates file, mkdtemp creates dir | N/A |
| AC-4 | E2E fail | N/A | Bad imports, wrong types, missing functions |
| AC-5 | E2E pass | perf_counter returns monotonic float >= 0.0 | Two calls return increasing values |
| AC-6 | E2E pass | sifr.time.perf_counter and monotonic importable | N/A |
| AC-7 | E2E pass | default_timer returns monotonic time | N/A |
| AC-8 | E2E pass | timeit(stmt, number) returns positive float | Callable param works with named function |
| AC-9 | E2E pass | repeat(stmt, repeat, number) returns list[float] of correct length | N/A |
| AC-10 | E2E pass | loads parses string, load reads file | N/A |
| AC-11 | Code review | Comment matches behavior | N/A |
| AC-12 | cargo test | All 340+ tests pass | N/A |
