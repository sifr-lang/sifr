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

1. Rename module functions to align with architecture plan where feasible without classes
2. Add missing E2E pass tests for glob, shutil, tempfile
3. Add negative/fail E2E tests for stdlib error paths
4. Fix stale comment in lower.rs
5. Add missing `_sifr.fs.copy_file` and `_sifr.fs.walk_dir` intrinsics for shutil
6. Update parity report with final metrics

##### Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| Result/Option safety contract | Requires compiler-level intrinsic type changes (separate milestone) |
| open()/File context manager | Requires `with` statement + class support in stdlib |
| ArgumentParser class | Requires class definitions in stdlib .sifr files |
| Logger/getLogger class API | Requires class definitions in stdlib .sifr files |
| Path class with operator overloading | Requires class + operator support in stdlib |

---

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | glob.sifr exports `glob` (not `glob_match`) and has an E2E pass test |
| AC-2 | shutil.sifr exports `copy`, `move`, `rmtree` with `_sifr.fs` intrinsics and has an E2E pass test |
| AC-3 | tempfile.sifr has an E2E pass test |
| AC-4 | At least 5 new stdlib fail tests covering bad imports, type mismatches, and invalid usage |
| AC-5 | timeit.sifr exports `timeit` and `repeat` (matching plan) |
| AC-6 | tomllib.sifr exports `loads` and `load` |
| AC-7 | lower.rs fallback comment is updated to reflect current behavior |
| AC-8 | All existing tests pass (zero regressions) |
| AC-9 | Parity report updated |

---

## 2. Solution Design

### 2.1 Functional Requirements

**API renames (no behavior change, just function name alignment):**
- `glob.sifr`: rename `glob_match` to `glob`
- `shutil.sifr`: rename `copy_file` to `copy`, `move_file` to `move`, add `rmtree`
- `timeit.sifr`: rename `timer` to `timeit`, `elapsed` to `repeat` (or add aliases)
- `tomllib.sifr`: add `load` function (reads file then parses)

**New intrinsics:**
- `_sifr.fs.copy_file(src: str, dst: str) -> None` -- wraps `std::fs::copy`
- `_sifr.fs.walk_dir(path: str) -> list[str]` -- wraps `std::fs::read_dir` recursively
- `_sifr.fs.rmdir_all(path: str) -> None` -- wraps `std::fs::remove_dir_all`

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

- `lib/sifr/glob.sifr` -- rename function
- `lib/sifr/shutil.sifr` -- rename functions, add rmtree
- `lib/sifr/timeit.sifr` -- rename functions
- `lib/sifr/tomllib.sifr` -- add load function
- `crates/sifr_hir/src/stdlib.rs` -- add copy_file, walk_dir, rmdir_all intrinsics
- `crates/sifr_codegen/src/lib.rs` -- add codegen for new intrinsics
- `crates/sifr_hir/src/lower.rs` -- fix stale comment
- `crates/sifr/tests/e2e/pass/` -- 3 new pass tests
- `crates/sifr/tests/e2e/fail/` -- 5 new fail tests
- `audit/STDLIB_PARITY_MASTER_REPORT.md` -- update metrics
- `demos/milestone_stdlib_polish_demo.sifr` -- milestone demo

### 2.3 Testing Strategy

| AC-ID | Test Layer | Happy-Path Check | Non-Happy / Edge Check |
| --- | --- | --- | --- |
| AC-1 | E2E pass | glob lists directory entries matching pattern | N/A |
| AC-2 | E2E pass | copy creates duplicate, move removes source | N/A |
| AC-3 | E2E pass | mkstemp creates file, mkdtemp creates dir | N/A |
| AC-4 | E2E fail | N/A | Bad imports, wrong types, missing functions |
| AC-5 | E2E pass | timeit returns timing, repeat runs N times | N/A |
| AC-6 | E2E pass | loads parses string, load reads file | N/A |
| AC-7 | Code review | Comment matches behavior | N/A |
| AC-8 | cargo test | All 340+ tests pass | N/A |
