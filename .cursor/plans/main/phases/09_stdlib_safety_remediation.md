# Stdlib Safety Remediation

**Why before borrow-by-default:** The safety audit found ~45+ `.unwrap()` panic paths in intrinsics. These must be fixed before changing the parameter passing convention, because borrow-by-default will touch the same codegen paths. Fixing safety first means borrow-by-default works on a stable, non-panicking foundation.

---

## milestone_io_safety: File I/O Safety (Priority 1 -- Critical)

status: completed (PR #158)

**Goal:** Eliminate all panic paths in file I/O intrinsics. The most critical safety violation — 5 modules, ~15 intrinsics.

**Depends on:** milestone_error_safety (IOError must be a compiler built-in), milestone_error_safety_stdlib_types (export pipeline validated)

### Work Items

- Change intrinsic type signatures in `stdlib.rs` to return `Result[T, IOError]`: `read_text`, `write_text`, `read_lines`, `append_text`, `mkdir`, `rmdir`, `remove_file`, `rename`, `copy_file`, `rmdir_all`, `listdir`, `getcwd`
- Update codegen to emit `Result::Ok(...)` / `Result::Err(IOError {...})` instead of `.unwrap()`
- Update stdlib wrappers (`io.sifr`, `os.sifr`, `shutil.sifr`, `pathlib.sifr`, `tempfile.sifr`, `tomllib.sifr`) to propagate `Result`
- E2E tests for both success and error paths

### Definition of Done (milestone_io_safety)

- All file I/O intrinsics return `Result[T, IOError]` instead of panicking
- Stdlib wrappers propagate `Result` correctly
- E2E pass tests: file read/write success, directory operations success
- E2E fail tests: file not found, permission denied, directory not empty — all handled via `try`/`except`, no panics

---

## milestone_parse_safety: Parse/Decode Safety (Priority 2 -- Critical)

status: completed (PR #159)

**Goal:** Eliminate all panic paths in parse/decode intrinsics. Uses the specific error types defined in Phase 08 — not generic `ParseError` — so that exhaustiveness checking can distinguish between different parse failure sources in the same `try` block.

**Depends on:** milestone_error_safety (JSONDecodeError, TOMLDecodeError, RegexError must be compiler built-ins)

### Work Items

- `json_loads` -> `Result[str, JSONDecodeError]`
- `toml_parse` -> `Result[str, TOMLDecodeError]`
- `base64_decode` / `urlsafe_b64decode` -> `Result[str, ParseError]` (generic is fine here — no module-specific error type needed for base64)
- `decode_utf8` / `bytes_from_hex` -> `Result[str, ParseError]`
- Regex intrinsics (`re_match`, `re_replace`, `re_findall`, `re_split`) -> `Result[T, RegexError]`
- Update all `.sifr` wrappers and E2E tests

### Definition of Done (milestone_parse_safety)

- All parse/decode intrinsics return `Result` with specific error types
- E2E tests for both valid and invalid input
- A `try` block containing both `json_loads` and `toml_parse` requires handling both `JSONDecodeError` and `TOMLDecodeError` (exhaustiveness checking)

---

## milestone_collection_safety: Collection, Math, and Built-in Safety (Priority 3 -- High)

status: completed (PR #160)

**Goal:** Eliminate panic paths in collection operations, math functions, and built-in functions. Also make an explicit design decision on math domain errors.

**Depends on:** milestone_error_safety_stdlib_types (StatisticsError must be exportable)

### Work Items

- `statistics.mean/median/variance/stdev/mode/harmonic_mean` -> `Result[float, StatisticsError]` on empty input
- `heapq.heappop` -> `Option[T]` on empty heap
- `heapq.heapreplace` -> `Result[T, ValueError]` on empty heap
- `collections.set_pop` -> `Option[str]` on empty set
- `math.factorial(-1)` -> `Result[int, ValueError]`; `factorial(large_n)` -> overflow check
- `list.remove()` / `list.index()` -> `Option`/`Result` instead of panic
- `min()` / `max()` on empty -> `Option[T]` (CPython raises `ValueError`, which would be Rule 1 / `Result[T, ValueError]`, but the semantics here are "no value exists" — absence, not failure — consistent with the spirit of Rules 2-3. This is a deliberate divergence from Rule 1; document in `architecture.md` Python Divergences.)
- `sorted()` with floats -> use `total_cmp` instead of `partial_cmp().unwrap()`

### Math Domain Errors (Design Decision Required)

`sqrt(-1)`, `log(0)`, `log(-1)`, `asin(2)`, `acos(2)` currently return NaN/inf silently (matching Rust's `f64` behavior). CPython raises `ValueError`. The architecture's Rule 1 says "Where CPython raises an exception, Sifr returns `Result[T, E]`."

Decide one of:
- **(a)** Return `Result[float, ValueError]` for domain errors (matching architecture/CPython)
- **(b)** Document as an explicit divergence ("Sifr follows Rust's IEEE 754 behavior for math domain errors") and add to the Python Divergences table in `architecture.md`

Either choice is valid — document whichever is chosen. This decision affects `math`'s safety score and the zero-panic gate threshold.

### Definition of Done (milestone_collection_safety)

- All collection/math/built-in panic paths eliminated
- Math domain error decision made and documented
- E2E tests for empty collections, invalid math inputs, built-in edge cases

---

## milestone_edge_case_safety: Edge Case Validation (Priority 4 -- Moderate)

status: completed (PR #161)

**Goal:** Validate inputs for edge cases that currently panic or produce undefined behavior.

### Work Items

- `random.randint(5, 3)` -> validate a <= b
- `secrets.randbelow(0)` -> validate n > 0
- `textwrap.wrap(text, 0)` -> validate width > 0
- `itertools.batched(data, 0)` -> validate n > 0
- `graphlib.topological_sort(cyclic)` -> detect cycles, return `Result[list[int], CycleError]`
- `uuid.UUID(invalid_hex)` -> return `Result`
- `ipaddress.ip_to_int(invalid)` -> return `Result`
- `datetime.from_timestamp(invalid)` -> return `Result`
- `SubscriptAssign` (`x[i] = val`) -> bounds check instead of panic

### Definition of Done (milestone_edge_case_safety)

- All edge case inputs validated with proper `Result`/`Option` returns
- E2E tests for each edge case (both valid and invalid inputs)
- `SubscriptAssign` bounds-checked in codegen

---

## milestone_zero_panic_gate: Safety Verification Gate

status: completed (PR #162)

**Goal:** Systematic verification that the safety remediation is complete. This is a hard quality gate — Phase 10 cannot start until this passes.

### Gate Criteria

- Audit all codegen intrinsic emission paths in `crates/sifr_codegen/src/lib.rs` — assert zero panic-inducing patterns on user-facing operations. This includes `.unwrap()`, `.expect(`, `panic!(`, `unreachable!(`, and unchecked indexing (`[i as usize]` without bounds check). The only acceptable panic-path code is on compiler-internal invariants that cannot fail.
- Add a CI lint that scans intrinsic codegen blocks for all panic-inducing patterns (`.unwrap()`, `.expect(`, `panic!(`, `unreachable!(`, raw indexing on user data) and fails if any are found on user-facing operations (file I/O, parsing, collection access, subscript assignment)
- Run the full stdlib safety audit script against all 37 modules and produce an updated safety score — every module must score 7/10 or higher (up from the current state where 12 modules score below 5/10). Modules with documented divergences (e.g., math domain errors if option (b) is chosen in `milestone_collection_safety`) are scored against the documented behavior, not the CPython behavior — a deliberate, documented design choice does not count as a safety violation.
- E2E test: a program that calls every fallible stdlib function with invalid input must compile and run without panicking, handling all errors via `try`/`except`
- Update `audit/STDLIB_PARITY_MASTER_REPORT.md` with post-remediation safety scores

### Definition of Done (milestone_zero_panic_gate)

- All gate criteria pass
- CI lint integrated and passing
- Safety audit report updated
- Phase 10 is unblocked

---

## Milestone Ordering

- **milestone_io_safety first:** File I/O is the most critical safety violation (5 modules, ~15 intrinsics).
- **milestone_parse_safety second:** Parse/decode is the second most critical (5 modules, ~8 intrinsics).
- **milestone_collection_safety third:** Collection/math/built-in safety is high priority but less critical than I/O and parsing.
- **milestone_edge_case_safety fourth:** Edge cases are moderate priority — important but not blocking.
- **milestone_zero_panic_gate last:** The gate verifies all prior milestones are complete. It is a hard quality gate — Phase 10 cannot start until this passes.
