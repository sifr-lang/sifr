---
name: Stdlib Architecture Phase
overview: "Implement the 4-milestone Stdlib Architecture Phase: rewire stdlib from Rust codegen to a three-tier hybrid architecture (_sifr.* intrinsics, .sifr stdlib files, user code). Each milestone gets a PRDS, GitHub tickets, implementation branch, demo, PR review, and merge -- following the project workflow."
todos:
  - id: m1-prds
    content: Create PRDS for milestone_intrinsics (issues/milestone_intrinsics.md)
    status: completed
  - id: m1-epic
    content: Create Epic ticket on GitHub board for milestone_intrinsics
    status: completed
  - id: m1-tasks
    content: Break milestone_intrinsics into task tickets (rename intrinsics, stdlib embedding, two-phase compilation, import blocking, proof-of-concept)
    status: completed
  - id: m1-implement
    content: Implement milestone_intrinsics on feat/milestone-intrinsics branch
    status: completed
  - id: m1-demo
    content: Create and verify demos/milestone_intrinsics_demo.sifr
    status: completed
  - id: m1-pr
    content: Create PR for milestone_intrinsics, review, and merge
    status: in_progress
  - id: m2-prds
    content: Create PRDS for milestone_stdlib_migration (issues/milestone_stdlib_migration.md)
    status: pending
  - id: m2-epic
    content: Create Epic ticket on GitHub board for milestone_stdlib_migration
    status: pending
  - id: m2-tasks
    content: Break milestone_stdlib_migration into task tickets (migrate 13 modules, delete emit_stdlib_call, rename hash/encoding)
    status: pending
  - id: m2-implement
    content: Implement milestone_stdlib_migration on feat/milestone-stdlib-migration branch
    status: pending
  - id: m2-demo
    content: Create and verify demos/milestone_stdlib_migration_demo.sifr
    status: pending
  - id: m2-pr
    content: Create PR(s) for milestone_stdlib_migration, review, and merge
    status: pending
  - id: m3-prds
    content: Create PRDS for milestone_stdlib_expansion (issues/milestone_stdlib_expansion.md)
    status: pending
  - id: m3-epic
    content: Create Epic ticket on GitHub board for milestone_stdlib_expansion
    status: pending
  - id: m3-tasks
    content: Break milestone_stdlib_expansion into task tickets (9 pure Sifr modules, 5 intrinsic-backed modules)
    status: pending
  - id: m3-implement
    content: Implement milestone_stdlib_expansion on feat/milestone-stdlib-expansion branch
    status: pending
  - id: m3-demo
    content: Create and verify demos/milestone_stdlib_expansion_demo.sifr
    status: pending
  - id: m3-pr
    content: Create PR(s) for milestone_stdlib_expansion, review, and merge
    status: pending
  - id: m4-prds
    content: Create PRDS for milestone_stdlib_parity (issues/milestone_stdlib_parity.md)
    status: pending
  - id: m4-epic
    content: Create Epic ticket on GitHub board for milestone_stdlib_parity
    status: pending
  - id: m4-tasks
    content: Break milestone_stdlib_parity into task tickets (expand existing, new modules, parity audit)
    status: pending
  - id: m4-implement
    content: Implement milestone_stdlib_parity on feat/milestone-stdlib-parity branch
    status: pending
  - id: m4-demo
    content: Create and verify demos/milestone_stdlib_parity_demo.sifr
    status: pending
  - id: m4-pr
    content: Create PR(s) for milestone_stdlib_parity, review, and merge
    status: pending
isProject: false
---

# Stdlib Architecture Phase -- Implementation Plan

## Current State

- **Branch:** `main` (clean, all tests pass -- 301 tests across 10 crates)
- **Stdlib:** 13 modules, 57 functions, all implemented as Rust codegen in [crates/sifr_hir/src/stdlib.rs](crates/sifr_hir/src/stdlib.rs) (type registry) and [crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs) (`emit_stdlib_call`, ~352 lines)
- **No `lib/sifr/` directory yet** -- stdlib .sifr files don't exist
- **Import resolution:** `lower.rs` calls `get_stdlib_module()` which matches on `"sifr.*"` strings; codegen tracks `used_stdlib_modules` for Cargo dependency injection
- **Multi-file compilation:** `sifr_driver` discovers `.sifr` files in project dir, lowers non-main modules first, collects `ExternalDefs`, then lowers main with externals

## Workflow Per Milestone

Each milestone follows this loop:

```
1. Create PRDS (issues/<milestone>.md)
2. Create Epic ticket on GitHub board (gh issue create + gh project item-add)
3. Break Epic into Task tickets
4. Create branch (feat/<milestone>)
5. Implement tasks
6. Create demo (demos/<milestone>_demo.sifr)
7. Verify demo works + all tests pass
8. Create PR
9. Review PR
10. Merge PR
```

---

## Milestone 1: `milestone_intrinsics` -- Intrinsics Layer and Stdlib Compilation Pipeline

**Goal:** Rewire stdlib plumbing. Rename `sifr.*` to `_sifr.*` in the registry, add `lib/sifr/` directory with embedded `.sifr` files, implement two-phase compilation, block user `_sifr.*` imports, proof-of-concept with `sifr.test`.

### Key Files to Change

- [crates/sifr_hir/src/stdlib.rs](crates/sifr_hir/src/stdlib.rs) -- Rename all `"sifr.*"` match arms to `"_sifr.*"`, rename `get_stdlib_module` to `get_intrinsic_module`, rename `is_stdlib_module` to `is_intrinsic_module`
- [crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs) -- Rename `emit_stdlib_call` to `emit_intrinsic_call`, update `used_stdlib_modules` to `used_intrinsic_modules`, update Cargo dep injection match arms from `"sifr.*"` to `"_sifr.*"`
- [crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs) -- Add `.sifr` file resolution before intrinsic fallback, add `_sifr.*` import blocking for user code
- [crates/sifr_driver/src/lib.rs](crates/sifr_driver/src/lib.rs) -- Add stdlib `.sifr` file discovery from embedded sources, implement two-phase compilation (stdlib first, then user code)
- **NEW:** `lib/sifr/test.sifr` -- Proof-of-concept stdlib file (pure Sifr, no intrinsics needed)

### Technical Approach

1. **Mechanical rename** (low risk): All `"sifr.io"` becomes `"_sifr.io"` etc. in stdlib.rs and codegen. This is a find-and-replace across ~100 match arms.
2. **Stdlib embedding**: Create `lib/sifr/` directory. In `sifr_driver`, use `include_str!` to embed `.sifr` source files at compile time. Create a `STDLIB_FILES` constant mapping module names to source.
3. **Two-phase compilation**: In `build_project()` and `build()`, before processing user files:
  - Parse embedded stdlib `.sifr` files
  - Lower them (they can import `_sifr.*` intrinsics)
  - Collect their exports into `ExternalDefs`
  - Then lower user files with both stdlib and local module exports available
4. **Import resolution change** in `lower.rs`:
  - When encountering `from sifr.X import Y`, first check if there's a stdlib `.sifr` module for `sifr.X`
  - If yes, resolve from the pre-compiled stdlib module's exports
  - If no, fall back to `get_intrinsic_module("_sifr.X")` (for backward compat during transition)
  - If module starts with `_sifr.` and source is user code, emit compile error
5. **Proof-of-concept**: `lib/sifr/test.sifr` implements `assert_eq`, `assert_ne`, `assert_true`, `assert_false` as pure Sifr functions (they just use `print()` and conditionals -- no intrinsics needed).

### Acceptance Criteria

- `from sifr.test import assert_eq` resolves to the `.sifr` file
- All 173 E2E pass tests still pass
- All 17 E2E fail tests still pass
- `_sifr.*` imports blocked for user code with clear error
- Demo: `demos/milestone_intrinsics_demo.sifr`

### PR Structure: 1 PR

- Single PR covering the full milestone (rename + embedding + two-phase + proof-of-concept)
- This is tightly coupled work that can't be split without breaking the build

---

## Milestone 2: `milestone_stdlib_migration` -- Migrate 13 Modules to .sifr

**Goal:** Port all 13 existing stdlib modules from Rust codegen to `.sifr` files. Delete `emit_stdlib_call`. Zero regressions.

### Key Challenge

The `.sifr` files must import from `_sifr.*` intrinsics and re-export user-facing functions. This means each `.sifr` file is a thin wrapper:

```python
# lib/sifr/env.sifr
from _sifr.sys import env_get, env_set
```

The Sifr compiler must be able to:

- Parse `from _sifr.sys import env_get` in a stdlib `.sifr` file
- Resolve `_sifr.sys.env_get` to the intrinsic
- Generate Rust code that calls the intrinsic implementation
- When user code does `from sifr.env import env_get`, resolve it through the `.sifr` file

### Migration Order (simplest to most complex)

1. `env.sifr` (2 functions, wraps `_sifr.sys`)
2. `bytes.sifr` (4 functions, wraps `_sifr.io`)
3. `base64.sifr` (2 functions, rename from `encoding`)
4. `math.sifr` (12 functions + 2 constants, wraps `_sifr.math`)
5. `hashlib.sifr` (2 functions, rename from `hash`)
6. `io.sifr` (4 functions, wraps `_sifr.fs` + `_sifr.io`)
7. `os.sifr` (2 functions, wraps `_sifr.sys` + `_sifr.fs`)
8. `json.sifr` (2 functions, wraps `_sifr.json`)
9. `time.sifr` (3 functions, wraps `_sifr.time`)
10. `random.sifr` (3 functions, wraps `_sifr.crypto`)
11. `re.sifr` (3 functions, wraps `_sifr.regex`)
12. `collections.sifr` (14 functions, wraps intrinsics)
13. `test.sifr` (already done in M1, verify)

### Final Cleanup

- Delete `emit_stdlib_call` (~352 lines) from codegen
- Delete old `sifr.*` entries from `get_stdlib_module()` (replaced by `_sifr.*`)
- Update Cargo dep injection to use `_sifr.*` module names
- Rename `sifr.hash` to `sifr.hashlib`, `sifr.encoding` to `sifr.base64` in all tests

### Acceptance Criteria

- `emit_stdlib_call` deleted
- Every `from sifr.X import Y` resolves to a `.sifr` file
- All 173+ E2E tests pass with zero regressions
- Demo: `demos/milestone_stdlib_migration_demo.sifr`

### PR Structure: 1-2 PRs

- PR 1: Migrate first 6 modules (env, bytes, base64, math, hashlib, io) + verify
- PR 2: Migrate remaining 7 modules + delete `emit_stdlib_call` + cleanup

---

## Milestone 3: `milestone_stdlib_expansion` -- New Modules

**Goal:** Add ~14 new modules (9 pure Sifr + 5 intrinsic-backed).

### Pure Sifr Modules (no new intrinsics)

1. `string.sifr` -- constants only (ascii_letters, digits, etc.)
2. `statistics.sifr` -- mean, median, stdev, variance
3. `bisect.sifr` -- bisect_left, bisect_right, insort
4. `heapq.sifr` -- heappush, heappop, heapify, nlargest, nsmallest
5. `functools.sifr` -- reduce
6. `itertools.sifr` -- chain, zip_longest, groupby
7. `textwrap.sifr` -- wrap, fill, dedent, indent
8. `csv.sifr` -- reader, writer
9. `argparse.sifr` -- ArgumentParser class

### Intrinsic-backed Modules (need new `_sifr.fs` primitives)

1. `fnmatch.sifr` -- wraps `_sifr.regex`
2. `glob.sifr` -- wraps `_sifr.fs.list_dir` + fnmatch
3. `shutil.sifr` -- wraps `_sifr.fs` (needs `copy_file`, `walk_dir`)
4. `tempfile.sifr` -- wraps `_sifr.fs` + `_sifr.crypto`
5. `secrets.sifr` -- wraps `_sifr.crypto`

### Acceptance Criteria

- Each module compiles and imports work
- E2E tests for each module
- Demo: `demos/milestone_stdlib_expansion_demo.sifr`

### PR Structure: 2-3 PRs

- PR 1: Pure Sifr modules (string, statistics, bisect, heapq, functools)
- PR 2: Pure Sifr modules (itertools, textwrap, csv, argparse)
- PR 3: Intrinsic-backed modules (fnmatch, glob, shutil, tempfile, secrets) + new intrinsics

---

## Milestone 4: `milestone_stdlib_parity` -- Gap Closing and Audit

**Goal:** Expand existing modules with missing functions, add 10 remaining modules, run parity audit.

### Part A: Expand Existing Modules

- `math.sifr` -- add ~20 functions (asin, acos, atan, etc.)
- `os.sifr` -- add getcwd, listdir, mkdir, etc.
- `re.sifr` -- add findall, split
- `random.sifr` -- add shuffle, sample, seed
- `io.sifr` -- add append_text, binary I/O
- `collections.sifr` -- add deque, OrderedDict
- Plus: time, hashlib, base64, itertools expansions

### Part B: New Modules

1. difflib, graphlib, ipaddress (pure Sifr)
2. timeit, platform (wraps `_sifr.time`/`_sifr.sys`)
3. tomllib (wraps new `_sifr.toml`)
4. datetime (wraps new `_sifr.datetime`)
5. pathlib (wraps `_sifr.fs`)
6. uuid (wraps `_sifr.crypto`)
7. logging (wraps `_sifr.io` + `_sifr.time`)

### Part C: Parity Audit

- Run audit, produce `audit/STDLIB_PARITY_MASTER_REPORT.md`
- Target: 60%+ coverage across top 20 CPython modules

### Acceptance Criteria

- 37 total stdlib modules available
- All tests pass, parity audit report generated
- Demo: `demos/milestone_stdlib_parity_demo.sifr`

### PR Structure: 3-4 PRs

- PR 1: Part A (expand existing modules + new intrinsics)
- PR 2: Part B pure Sifr modules (difflib, graphlib, ipaddress)
- PR 3: Part B intrinsic-backed modules (timeit, platform, tomllib, datetime, pathlib, uuid, logging)
- PR 4: Part C parity audit

---

## Risk Assessment

- **Highest risk:** Milestone 1 (two-phase compilation is a deep architectural change)
- **Medium risk:** Milestone 2 (each module migration could surface codegen issues)
- **Lower risk:** Milestones 3-4 (adding new `.sifr` files on established architecture)
- **Mitigation:** Each milestone has a demo that must work before PR creation. All 173+ E2E tests must pass at every step.

