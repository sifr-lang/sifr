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

**Goal:** Replace flat, message-only error types with a structured error hierarchy where the **variant is the information**. Developers handle specific failure modes via `except` arms with compile-time exhaustiveness checking — no string matching, no `message` fields on errors where the type already tells you everything. Where Rust provides genuinely useful structured data (line/column on parse errors), surface it as typed fields.

**Depends on:** milestone_zero_panic_gate (all prior safety milestones must be complete; this builds on top of the stable, non-panicking foundation they established)

**Why now:** The prior milestones replaced all panics with `Result[T, IOError]`, but every I/O failure is a flat `IOError` with only a `message: str` field. That message is just Rust's `e.to_string()` — a verbose restatement of the `ErrorKind` that the developer must string-match at runtime. This is the exact kind of fragile check Sifr's type system is designed to eliminate. Furthermore, the codegen already generates `Display` trait implementations for all error types (used by `print(e)` and `f"{e}"`), so the infrastructure to surface human-readable error messages without a stored `message` field is already in place — it's just not being used. Current Sifr code overwhelmingly accesses `e.message` directly (~120+ occurrences across demos and tests), which means the `message` field is doing work that `Display` should be doing. This milestone removes the redundant field and makes `Display` the single source of human-readable error text.

### Design Principles

**1. The variant IS the information.** For most errors, the type name tells you everything you need to know. `FileNotFoundError` means the file wasn't found — the developer already knows which file (they passed the path). No `message` field needed.

**2. Structured fields only where Rust provides data the developer can't know in advance.** JSON/TOML parse errors have `line` and `column` — the developer can't predict where a parse error will occur in runtime input. Regex errors have a `detail` string describing the syntax problem. These are genuinely useful fields.

**3. `Display` replaces `message`.** Every error type implements `Display`, generating a human-readable string from the variant name (and fields, if any). The codegen already emits `Display` impls for all error types today — `print(e)` and `f"{e}"` already work via this trait. After this milestone, `Display` becomes the single source of human-readable error text: `print(e)` produces messages like `"file not found"`, `"JSON decode error at line 3, column 12"`, or `"regex error: unclosed group"`. No stored `message` field needed. Existing `e.message` usage across demos and tests migrates to `print(e)` or `f"{e}"`.

**4. No catch-all `Other` variants.** Every `ErrorKind` that Sifr's I/O intrinsics can produce maps to a named subclass. The parent type (`IOError`) serves as the catch-all for error kinds not yet mapped to a subclass — but this is a parent-level catch, not a variant-level escape hatch.

**5. Enum variants in Rust, subclasses in Sifr.** At the Sifr language level, these look like subclasses (matching CPython). At the Rust codegen level, they are enum variants of the parent type. The intrinsic return type stays `Result[T, IOError]` — no signature changes.

### Error Hierarchy (Complete)

#### Audit of Rust Error Sources

Every error type was audited against the actual Rust error types the Sifr codegen encounters. The `message: str` field is removed wherever the variant name or structured fields already carry the information.

| Sifr error type | Rust source | What Rust provides | Sifr fields | Rationale |
|---|---|---|---|---|
| **IOError** | `std::io::Error` | `kind()` → `ErrorKind` (41 variants), `to_string()` → OS message | **none** | Variant name IS the error; developer knows the path |
| **ParseError** | `ParseIntError`, `ParseFloatError`, `FromUtf8Error`, `base64::DecodeError` | `to_string()` → description | **none** | Developer knows the input; variant name is sufficient |
| **JSONDecodeError** | `serde_json::Error` | `line()`, `column()`, `classify()` → Category | **`line: int`, `column: int`** | Position in runtime input is genuinely unknown to developer |
| **TOMLDecodeError** | `toml::de::Error` | `line_col()` → `Option<(usize, usize)>` | **`line: int`, `column: int`** | Position in runtime input is genuinely unknown to developer |
| **RegexError** | `regex::Error` | `Syntax(String)` — syntax error description | **`detail: str`** | Regex syntax errors are complex; description aids debugging |
| **ValueError** | Manually constructed | Hardcoded strings | **none** | Always a validation failure; developer knows the input |
| **DivisionError** | Compiler-generated | Division by zero check | **none** | Always division by zero |
| **KeyError** | Compiler-generated | Missing key access | **none** | Developer knows the key |

#### IOError Subclasses

Every `std::io::ErrorKind` that Sifr's I/O intrinsics can produce is mapped to a named subclass. No `Other` catch-all variant.

| Sifr type | CPython equivalent | Rust `io::ErrorKind` | Raised by |
|---|---|---|---|
| `FileNotFoundError` | `FileNotFoundError` | `NotFound` | `read_text`, `read_lines`, `remove_file`, `rmdir`, `rename`, `copy_file`, `listdir`, `walk_dir`, `run_command` |
| `PermissionError` | `PermissionError` | `PermissionDenied` | `read_text`, `write_text`, `mkdir`, `rmdir`, `remove_file`, `rename`, `copy_file`, `listdir`, `run_command` |
| `FileExistsError` | `FileExistsError` | `AlreadyExists` | `mkdir` (when not using `create_dir_all`), `write_text` (dir conflict) |
| `IsADirectoryError` | `IsADirectoryError` | `IsADirectory` | `read_text`, `remove_file`, `write_text` |
| `NotADirectoryError` | `NotADirectoryError` | `NotADirectory` | `listdir`, `rmdir`, `read_dir` |
| `DirectoryNotEmptyError` | (no CPython equivalent) | `DirectoryNotEmpty` | `rmdir` |

`IOError` itself is the parent — `except IOError as e` catches all subclasses. For any `ErrorKind` not in the table above (which would be unusual for filesystem operations), the codegen maps to the parent `IOError` type directly.

**Future subclasses** (added when networking/async stdlib lands):
- `ConnectionRefusedError` — `ConnectionRefused`
- `ConnectionResetError` — `ConnectionReset`
- `TimedOutError` — `TimedOut`
- `BrokenPipeError` — `BrokenPipe`

### Work Items

#### 1. Type System: Add `parent_class` to `Type::Class` (`crates/sifr_type_system/src/types.rs`)

- Add `parent_class: Option<String>` field to the `Type::Class` variant (line ~60)
- Update `is_assignable_to` (line ~533): a child class is assignable to its parent class. Walk up the `parent_class` chain. `FileNotFoundError` is assignable to `IOError`, which is assignable to `Error`
- Update all `Type::Class { name, fields, methods }` construction sites across the codebase to include `parent_class: None` (or the appropriate parent). This is a mechanical but wide-reaching change — grep for `Type::Class {` across all crates

#### 2. HIR: Register Error Types with Correct Fields (`crates/sifr_hir/src/lower.rs`)

Rework the `builtin_error_classes` registration block (lines ~1376-1410). Instead of all error types sharing `message: str`, each type gets its own field list:

- **No fields:** `Error`, `IOError`, `FileNotFoundError`, `PermissionError`, `FileExistsError`, `IsADirectoryError`, `NotADirectoryError`, `DirectoryNotEmptyError`, `ParseError`, `ValueError`, `DivisionError`, `KeyError`
- **`line: int`, `column: int`:** `JSONDecodeError`, `TOMLDecodeError`
- **`detail: str`:** `RegexError`

Register parent relationships:
```
("FileNotFoundError", parent: "IOError")
("PermissionError", parent: "IOError")
("FileExistsError", parent: "IOError")
("IsADirectoryError", parent: "IOError")
("NotADirectoryError", parent: "IOError")
("DirectoryNotEmptyError", parent: "IOError")
```

Store the parent relationship in `Type::Class { parent_class: Some("IOError".to_string()), ... }`. Add all types to `ctx.error_types`.

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

#### 5. Codegen: Generate Error Types as Enums or Fieldless Structs (`crates/sifr_codegen/src/lib.rs`)

Replace the current flat-struct generation (lines ~475-499) with type-appropriate codegen:

**IOError — enum with fieldless variants:**
```rust
#[derive(Debug, Clone)]
enum IOError {
    FileNotFound,
    PermissionDenied,
    FileExists,
    IsADirectory,
    NotADirectory,
    DirectoryNotEmpty,
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IOError::FileNotFound => write!(f, "file not found"),
            IOError::PermissionDenied => write!(f, "permission denied"),
            IOError::FileExists => write!(f, "file already exists"),
            IOError::IsADirectory => write!(f, "is a directory"),
            IOError::NotADirectory => write!(f, "not a directory"),
            IOError::DirectoryNotEmpty => write!(f, "directory not empty"),
        }
    }
}
```

**JSONDecodeError — struct with line/column:**
```rust
#[derive(Debug, Clone)]
struct JSONDecodeError {
    line: i64,
    column: i64,
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSON decode error at line {}, column {}", self.line, self.column)
    }
}
```

**TOMLDecodeError — struct with line/column:**
```rust
#[derive(Debug, Clone)]
struct TOMLDecodeError {
    line: i64,
    column: i64,
}
```

**RegexError — struct with detail:**
```rust
#[derive(Debug, Clone)]
struct RegexError {
    detail: String,
}
```

**Fieldless error types** (`ParseError`, `ValueError`, `DivisionError`, `KeyError`) — empty structs with `Display`:
```rust
#[derive(Debug, Clone)]
struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error")
    }
}
```

#### 6. Codegen: Map Rust Errors to Correct Sifr Types (~All Intrinsic Sites) (`crates/sifr_codegen/src/lib.rs`)

**I/O intrinsics (~16 sites):** Replace all `IOError { message: e.to_string() }` with a shared helper `fn __io_err(e: std::io::Error) -> IOError`:
```rust
fn __io_err(e: std::io::Error) -> IOError {
    match e.kind() {
        std::io::ErrorKind::NotFound => IOError::FileNotFound,
        std::io::ErrorKind::PermissionDenied => IOError::PermissionDenied,
        std::io::ErrorKind::AlreadyExists => IOError::FileExists,
        std::io::ErrorKind::IsADirectory => IOError::IsADirectory,
        std::io::ErrorKind::NotADirectory => IOError::NotADirectory,
        std::io::ErrorKind::DirectoryNotEmpty => IOError::DirectoryNotEmpty,
        _ => IOError::FileNotFound, // unreachable for filesystem ops; compile-time assert
    }
}
```

**JSON intrinsic:** Extract `line()` and `column()` from `serde_json::Error`:
```rust
.map_err(|e| JSONDecodeError { line: e.line() as i64, column: e.column() as i64 })
```

**TOML intrinsic:** Extract `line_col()` from `toml::de::Error`:
```rust
.map_err(|e| {
    let (line, column) = e.line_col().unwrap_or((0, 0));
    TOMLDecodeError { line: (line + 1) as i64, column: (column + 1) as i64 }
})
```

**Regex intrinsics (~7 sites):** Extract the syntax error description:
```rust
.map_err(|e| RegexError { detail: e.to_string() })
```

**Parse intrinsics (int/float/base64/utf8):** Fieldless construction:
```rust
.map_err(|_| ParseError)
```

**ValueError construction:** Fieldless:
```rust
Err(ValueError)
```

Affected intrinsics: `read_text`, `write_text`, `read_lines`, `append_text`, `getcwd`, `listdir`, `mkdir`, `rmdir`, `remove_file`, `rename`, `copy_file`, `walk_dir`, `rmdir_all`, `makedirs`, `run_command`, `json_loads`, `toml_parse`, `re_match`, `re_find`, `re_replace`, `re_findall`, `re_split`, `re_find_start`, `re_find_end`, `base64_decode`, `urlsafe_b64decode`, `decode_utf8`, `bytes_from_hex`, `int()`, `float()`, `datetime_from_timestamp`

#### 7. Codegen: Try/Except Match Arms for Subclass Dispatch (`crates/sifr_codegen/src/lib.rs`)

- Update the try/except codegen (lines ~2761-2921) to handle subclass matching:
  - `except FileNotFoundError` generates: `Err(IOError::FileNotFound) => { ... }`
  - `except IOError as e` generates a catch-all arm matching all variants
  - When mixed with other error types (e.g., `IOError` + `JSONDecodeError`), the existing `_TryErr` enum pattern wraps the parent types as before — subclass dispatch happens inside the parent's match arm
- For the single-error-type case (only `IOError` in the try body), the codegen generates a `match` on `IOError` variants directly

#### 8. Codegen: Migrate `e.message` to `Display` (`crates/sifr_codegen/src/lib.rs`)

- All existing `.sifr` code that accesses `e.message` must be migrated to use `print(e)` or f-string interpolation `f"{e}"` (which uses `Display`)
- For `JSONDecodeError` and `TOMLDecodeError`, field access changes from `e.message` to `e.line` and `e.column`
- For `RegexError`, field access changes from `e.message` to `e.detail`
- Update all stdlib `.sifr` wrappers and demo files that reference `e.message`

#### 9. Stdlib Signatures: No Changes Required (`crates/sifr_hir/src/stdlib.rs`)

- Intrinsic return types stay as `Result[T, IOError]`, `Result[T, JSONDecodeError]`, etc. — no signature changes
- Stdlib `.sifr` wrappers propagate `Result` unchanged

#### 10. Architecture Documentation (`architecture.md`)

- Update the Built-in Error Classes section to document the full error hierarchy with fields
- Document the "variant IS the information" design principle
- Update the `except` exhaustiveness examples to show subclass handling
- Document the design decision: subclasses at Sifr level = enum variants at Rust level
- Add `JSONDecodeError.line`, `JSONDecodeError.column`, `TOMLDecodeError.line`, `TOMLDecodeError.column`, `RegexError.detail` to the error type reference

#### 11. E2E Tests

- **Pass test: specific subclass handling** — `read_text` on missing file caught by `except FileNotFoundError`
- **Pass test: parent catch-all** — `except IOError as e` catches `FileNotFoundError`
- **Pass test: mixed subclass + parent** — `except FileNotFoundError` + `except IOError` covers all cases
- **Pass test: mixed error families** — `try` block with `read_text` (IOError family) + `json_loads` (JSONDecodeError), catching `FileNotFoundError` + `IOError` + `JSONDecodeError`
- **Pass test: JSONDecodeError fields** — `except JSONDecodeError as e` with access to `e.line` and `e.column`
- **Pass test: TOMLDecodeError fields** — `except TOMLDecodeError as e` with access to `e.line` and `e.column`
- **Pass test: RegexError field** — `except RegexError as e` with access to `e.detail`
- **Pass test: fieldless errors** — `except ParseError`, `except ValueError`, `except DivisionError` work without field access
- **Pass test: Display on all errors** — `print(e)` works for every error type via `Display`
- **Fail test: incomplete subclass coverage** — `except FileNotFoundError` without covering remaining IOError subtypes is a compile error
- **Fail test: `e.message` access** — accessing `.message` on any error type is a compile error (field no longer exists)
- **Pass test: user-defined error subclass** — `class MyAppError(IOError)` works as an error type with exhaustiveness checking
- **Demo file:** `demos/milestone_error_subclasses_demo.sifr` showing all patterns

### Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| `Type::Class` change is pervasive — adding `parent_class` field touches dozens of construction sites | High churn, potential for missed sites | Mechanical change; compiler errors will catch every missed site since the struct pattern becomes non-exhaustive |
| Removing `message` field breaks all existing `e.message` access in `.sifr` code | Breaking change for user code and demos | Migrate all `.sifr` files to use `print(e)` / `f"{e}"` via Display; update demos; provide clear compiler diagnostic: "error type X has no field 'message'; use print(e) or f\"{e}\" instead" |
| Exhaustiveness checking complexity increases | Harder to reason about coverage | Start with IOError subclasses only; other error types get field changes but no subclasses |
| `IsADirectory` / `NotADirectory` / `DirectoryNotEmpty` ErrorKind variants may not be stable on all Rust versions | Codegen may not compile on older rustc | Check minimum Rust version; use `#[allow(unreachable_patterns)]` fallback |
| `toml::de::Error::line_col()` returns `None` for some errors | `line`/`column` may be 0 | Default to `(0, 0)` when position unavailable; document that 0 means "position unknown" |

### Definition of Done (milestone_error_subclasses)

**Type system and exhaustiveness:**
- `Type::Class` has `parent_class` field; `is_assignable_to` walks the inheritance chain
- `except FileNotFoundError` compiles and catches only file-not-found errors
- `except IOError as e` catches all IOError variants (parent = catch-all)
- Exhaustiveness checking enforces coverage of subclasses when specific handlers are used

**Codegen — error type generation:**
- `IOError` is generated as a Rust enum with fieldless variants: `FileNotFound`, `PermissionDenied`, `FileExists`, `IsADirectory`, `NotADirectory`, `DirectoryNotEmpty`
- `JSONDecodeError` and `TOMLDecodeError` are structs with `line: i64` and `column: i64` fields
- `RegexError` is a struct with `detail: String` field
- `ParseError`, `ValueError`, `DivisionError`, `KeyError` are fieldless structs
- No error type has a `message` field — `Display` impl replaces it everywhere

**Codegen — subclass dispatch (critical):**
- All ~16 I/O intrinsic sites use the shared `__io_err` helper to map `std::io::ErrorKind` to the most specific `IOError` variant
- No intrinsic raises a generic parent error when a more specific subclass applies
- `json_loads` extracts `line()` and `column()` from `serde_json::Error` into `JSONDecodeError` fields
- `toml_parse` extracts `line_col()` from `toml::de::Error` into `TOMLDecodeError` fields
- Regex intrinsics extract the syntax description into `RegexError.detail`
- Parse intrinsics (`int()`, `float()`, `base64_decode`, etc.) construct fieldless `ParseError`
- Audit: zero remaining instances of `{ message: e.to_string() }` pattern in codegen

**Backward compatibility:**
- All existing E2E tests pass (migrated from `e.message` to `Display`-based output)

**New tests:**
- E2E pass tests for every IOError subclass, JSONDecodeError/TOMLDecodeError field access, RegexError detail access, fieldless error handling, Display output, mixed error families
- E2E fail test for incomplete subclass coverage
- E2E fail test for accessing nonexistent `.message` field

**Documentation:**
- `architecture.md` updated with full error hierarchy, field reference, and design principles
- Demo file: `demos/milestone_error_subclasses_demo.sifr`

---

## Milestone Ordering

- **milestone_io_safety first:** File I/O is the most critical safety violation (5 modules, ~15 intrinsics).
- **milestone_parse_safety second:** Parse/decode is the second most critical (5 modules, ~8 intrinsics).
- **milestone_collection_safety third:** Collection/math/built-in safety is high priority but less critical than I/O and parsing.
- **milestone_edge_case_safety fourth:** Edge cases are moderate priority — important but not blocking.
- **milestone_zero_panic_gate fifth:** The gate verifies all prior milestones (1-4) are complete. It is a hard quality gate.
- **milestone_error_subclasses last:** Refines the flat error types into a CPython-aligned subclass hierarchy, enabling compile-time checked fine-grained error handling. Builds on top of the stable, non-panicking foundation established by milestones 1-5. Fully backward compatible — all existing `except IOError` code continues to work. Phase 10 (Borrow-by-Default) depends on this milestone.
