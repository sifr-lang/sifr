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

## milestone_error_subclasses: Error Subclass Hierarchy (Priority 6 -- Enhancement)

status: pending

**Goal:** Introduce CPython-aligned error subclasses (`FileNotFoundError`, `PermissionError`, etc.) so that developers can handle specific failure modes via `except` arms with compile-time exhaustiveness checking — not by inspecting message strings at runtime. This extends Sifr's safety guarantee deeper: the compiler enforces that every distinguishable error category is handled.

**Depends on:** milestone_zero_panic_gate (all prior safety milestones must be complete; this builds on top of the stable, non-panicking foundation they established)

**Why now:** The prior milestones replaced all panics with `Result[T, IOError]`, but every I/O failure is a flat `IOError` with only a `message: str` field. Distinguishing "file not found" from "permission denied" requires string matching — the exact kind of fragile runtime check that Sifr's type system is designed to eliminate. This milestone refines the error types that milestones 1-5 established, without changing any of their completed work.

**Backward compatibility:** All existing code that catches `except IOError as e` continues to work unchanged — `IOError` becomes a parent type that catches all its subclass variants. No existing E2E tests need modification. The change is purely additive: new subclass types become available for finer-grained handling, but are not required.

### Design Decision: Enum Variants, Not Separate Structs

Error subclasses are represented as **enum variants of their parent error type** in generated Rust. This is the cleanest mapping because:

- The intrinsic return type stays `Result[T, IOError]` — no signature changes in `stdlib.rs`
- The Rust `match` on enum variants maps directly to Sifr's `except` arms
- `except IOError as e` catches all variants (parent = catch-all for children)
- `except FileNotFoundError as e` catches one variant (triggers exhaustiveness checking for the rest)
- Rust's `std::io::ErrorKind` maps directly to enum variants in codegen

At the **Sifr language level**, these look like subclasses (matching CPython). At the **Rust codegen level**, they are enum variants of the parent type.

### Error Subclass Hierarchy

**IOError subclasses** (matching CPython's `OSError` subclasses):

| Sifr type | CPython equivalent | Rust `io::ErrorKind` | When raised |
|---|---|---|---|
| `FileNotFoundError` | `FileNotFoundError` | `NotFound` | File/directory does not exist |
| `PermissionError` | `PermissionError` | `PermissionDenied` | Insufficient permissions |
| `FileExistsError` | `FileExistsError` | `AlreadyExists` | File/directory already exists |
| `IsADirectoryError` | `IsADirectoryError` | `IsADirectory` | Operation expected file, got directory |
| `NotADirectoryError` | `NotADirectoryError` | `NotADirectory` | Operation expected directory, got file |

`IOError` itself remains the catch-all for any I/O error not matching a specific subclass (maps to the `Other` variant).

**No subclasses needed for other error types (yet):**

- `ParseError`, `JSONDecodeError`, `TOMLDecodeError`, `RegexError` — the failure mode is already distinguished by having separate types per domain. Sub-categorizing "invalid JSON syntax" vs "unexpected EOF in JSON" adds little value for error handling.
- `ValueError`, `DivisionError`, `KeyError` — single failure mode per type.
- Future milestones may add subclasses to other error types if the need arises.

### Work Items

#### 1. Type System: Add `parent_class` to `Type::Class` (`crates/sifr_type_system/src/types.rs`)

- Add `parent_class: Option<String>` field to the `Type::Class` variant (line ~60)
- Update `is_assignable_to` (line ~533): a child class is assignable to its parent class. Walk up the `parent_class` chain. `FileNotFoundError` is assignable to `IOError`, which is assignable to `Error`
- Update all `Type::Class { name, fields, methods }` construction sites across the codebase to include `parent_class: None` (or the appropriate parent). This is a mechanical but wide-reaching change — grep for `Type::Class {` across all crates

#### 2. HIR: Register Error Subclasses as Built-ins (`crates/sifr_hir/src/lower.rs`)

- Extend the `builtin_error_classes` registration block (lines ~1376-1410) to register subclass types with their parent relationship:
  ```
  ("FileNotFoundError", parent: "IOError")
  ("PermissionError", parent: "IOError")
  ("FileExistsError", parent: "IOError")
  ("IsADirectoryError", parent: "IOError")
  ("NotADirectoryError", parent: "IOError")
  ```
- Each subclass gets the same `message: str` field and constructor as its parent
- Store the parent relationship in `Type::Class { parent_class: Some("IOError".to_string()), ... }`
- Add all subclasses to `ctx.error_types` so they are recognized as valid error types

#### 3. HIR: Expand `is_error_class` for Transitive Inheritance (`crates/sifr_hir/src/lower.rs`)

- Currently `is_error_class` (line ~712) only checks `(Error)` as the base class
- Expand to recognize any class whose base is itself in `ctx.error_types` — e.g., `class MyIOError(IOError)` should be recognized as an error class
- This enables user-defined error subclasses in future milestones (not just built-in ones)

#### 4. HIR: Exhaustiveness Checking with Inheritance (`crates/sifr_hir/src/lower.rs`)

- Build an error hierarchy map during lowering: `error_hierarchy: HashMap<String, Vec<String>>` mapping parent -> list of known children
- Update exhaustiveness checking (lines ~2091-2105):
  - `except IOError as e` covers `IOError` AND all its subclasses (`FileNotFoundError`, `PermissionError`, etc.)
  - `except FileNotFoundError as e` covers only `FileNotFoundError` — the compiler requires remaining `IOError` subtypes to be covered (or a catch-all `except IOError as e` / `except Error as e`)
  - When a `try` body produces `IOError` errors (from intrinsics), the exhaustiveness checker knows the full set of possible subclasses
- The existing `except Error as e` catch-all continues to work unchanged

#### 5. Codegen: Generate `IOError` as Enum with Subclass Variants (`crates/sifr_codegen/src/lib.rs`)

- Change the built-in `IOError` struct generation (lines ~475-499) from a flat struct to a Rust enum:
  ```rust
  #[derive(Debug, Clone)]
  enum IOError {
      FileNotFound { message: String },
      PermissionDenied { message: String },
      FileExists { message: String },
      IsADirectory { message: String },
      NotADirectory { message: String },
      Other { message: String },
  }
  ```
- Generate `impl IOError` with a `message(&self) -> &str` method that returns the message from whichever variant
- Generate `Display` impl that delegates to the message
- Generate `std::error::Error` impl
- Other built-in error types (`ParseError`, `ValueError`, etc.) remain as flat structs — no enum needed since they have no subclasses

#### 6. Codegen: Map `io::ErrorKind` to Enum Variants (~16 Intrinsic Sites) (`crates/sifr_codegen/src/lib.rs`)

- Replace all `IOError { message: e.to_string() }` constructions (lines ~5278-5397) with a helper that maps `std::io::ErrorKind`:
  ```rust
  .map_err(|e| match e.kind() {
      std::io::ErrorKind::NotFound => IOError::FileNotFound { message: e.to_string() },
      std::io::ErrorKind::PermissionDenied => IOError::PermissionDenied { message: e.to_string() },
      std::io::ErrorKind::AlreadyExists => IOError::FileExists { message: e.to_string() },
      std::io::ErrorKind::IsADirectory => IOError::IsADirectory { message: e.to_string() },
      std::io::ErrorKind::NotADirectory => IOError::NotADirectory { message: e.to_string() },
      _ => IOError::Other { message: e.to_string() },
  })
  ```
- Emit a shared helper function `fn __io_err(e: std::io::Error) -> IOError` in the generated Rust preamble to avoid repeating the match at every call site
- Affected intrinsics: `read_text`, `write_text`, `read_lines`, `append_text`, `getcwd`, `listdir`, `mkdir`, `rmdir`, `remove_file`, `rename`, `copy_file`, `walk_dir`, `rmdir_all`, `makedirs`, `run_command`

#### 7. Codegen: Try/Except Match Arms for Subclass Dispatch (`crates/sifr_codegen/src/lib.rs`)

- Update the try/except codegen (lines ~2761-2921) to handle subclass matching:
  - `except FileNotFoundError as e` generates: `Err(IOError::FileNotFound { message }) => { let e = FileNotFoundError { message }; ... }` (or direct field access)
  - `except IOError as e` generates a catch-all arm: `Err(e @ IOError::...) => { ... }` matching all variants
  - When mixed with other error types (e.g., `IOError` + `ParseError`), the existing `_TryErr` enum pattern wraps the parent types as before — subclass dispatch happens inside the parent's match arm
- For the single-error-type case (only `IOError` in the try body), the codegen generates a `match` on `IOError` variants directly

#### 8. Stdlib Signatures: No Changes Required (`crates/sifr_hir/src/stdlib.rs`)

- Intrinsic return types stay as `Result[T, IOError]` — the subclass information is a runtime property of the enum variant, not a type-level change
- Stdlib `.sifr` wrappers (`io.sifr`, `os.sifr`, `shutil.sifr`, etc.) propagate `Result[T, IOError]` unchanged

#### 9. Architecture Documentation (`architecture.md`)

- Update the Built-in Error Classes section to document the subclass hierarchy
- Add `FileNotFoundError`, `PermissionError`, `FileExistsError`, `IsADirectoryError`, `NotADirectoryError` to the error type table
- Update the `except` exhaustiveness examples to show subclass handling
- Document the design decision: subclasses at Sifr level = enum variants at Rust level

#### 10. E2E Tests

- **Pass test: specific subclass handling** — `read_text` on missing file caught by `except FileNotFoundError`
- **Pass test: parent catch-all** — `except IOError as e` catches `FileNotFoundError`
- **Pass test: mixed subclass + parent** — `except FileNotFoundError` + `except IOError` covers all cases
- **Pass test: mixed error families** — `try` block with `read_text` (IOError family) + `json_loads` (JSONDecodeError), catching `FileNotFoundError` + `IOError` + `JSONDecodeError`
- **Fail test: incomplete subclass coverage** — `except FileNotFoundError` without covering remaining IOError subtypes (missing catch-all) is a compile error
- **Fail test: wrong subclass family** — `except FileNotFoundError` on a `try` block that only calls `json_loads` (no IOError source) is a compile error or warning
- **Pass test: user-defined error subclass** — `class MyAppError(IOError)` works as an error type with exhaustiveness checking
- **Demo file:** `demos/milestone_error_subclasses_demo.sifr` showing all patterns

### Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| `Type::Class` change is pervasive — adding `parent_class` field touches dozens of construction sites | High churn, potential for missed sites | Mechanical change; compiler errors will catch every missed site since the struct pattern becomes non-exhaustive |
| `IOError` changing from struct to enum breaks existing codegen that accesses `.message` directly | Breaks all existing `e.message` access in user code | Generate a `.message` accessor method on the enum; update `Display` impl; existing `e.message` in `.sifr` code compiles to `e.message()` method call |
| Exhaustiveness checking complexity increases | Harder to reason about coverage | Start with IOError only; keep other error types as flat structs; expand later if needed |
| `IsADirectory` / `NotADirectory` ErrorKind variants may not be stable on all Rust versions | Codegen may not compile on older rustc | Use `#[allow(unreachable_patterns)]` and fall through to `Other` for unrecognized kinds; check minimum Rust version |

### Definition of Done (milestone_error_subclasses)

- `IOError` is generated as a Rust enum with variants: `FileNotFound`, `PermissionDenied`, `FileExists`, `IsADirectory`, `NotADirectory`, `Other`
- All ~16 I/O intrinsic codegen sites map `std::io::ErrorKind` to the correct variant
- `Type::Class` has `parent_class` field; `is_assignable_to` walks the inheritance chain
- `except FileNotFoundError as e` compiles and catches only file-not-found errors
- `except IOError as e` catches all IOError variants (parent = catch-all)
- Exhaustiveness checking enforces coverage of subclasses when specific handlers are used
- `e.message` continues to work on all error types (accessor method on enum)
- All existing E2E tests pass (no regressions from IOError struct-to-enum change)
- New E2E pass/fail tests for subclass handling and exhaustiveness
- `architecture.md` updated with error subclass hierarchy documentation
- Demo file: `demos/milestone_error_subclasses_demo.sifr`

---

## Milestone Ordering

- **milestone_io_safety first:** File I/O is the most critical safety violation (5 modules, ~15 intrinsics).
- **milestone_parse_safety second:** Parse/decode is the second most critical (5 modules, ~8 intrinsics).
- **milestone_collection_safety third:** Collection/math/built-in safety is high priority but less critical than I/O and parsing.
- **milestone_edge_case_safety fourth:** Edge cases are moderate priority — important but not blocking.
- **milestone_zero_panic_gate fifth:** The gate verifies all prior milestones (1-4) are complete. It is a hard quality gate.
- **milestone_error_subclasses last:** Refines the flat error types into a CPython-aligned subclass hierarchy, enabling compile-time checked fine-grained error handling. Builds on top of the stable, non-panicking foundation established by milestones 1-5. Fully backward compatible — all existing `except IOError` code continues to work. Phase 10 (Borrow-by-Default) depends on this milestone.
