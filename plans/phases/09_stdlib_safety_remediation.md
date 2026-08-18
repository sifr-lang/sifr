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

- `loads` -> `Result[JsonValue, JSONDecodeError]`
- `toml_parse` -> `Result[str, TOMLDecodeError]`
- `b64decode` / `urlsafe_b64decode` -> `Result[str, ParseError]` (generic is fine here — no module-specific error type needed for base64)
- `decode_utf8` / `bytes_from_hex` -> `Result[str, ParseError]`
- Canonical regex operations (`search`, `sub`, `findall`, `split`) -> `Result[T, RegexError]`
- Update all `.sifr` wrappers and E2E tests

### Definition of Done (milestone_parse_safety)

- All parse/decode intrinsics return `Result` with specific error types
- E2E tests for both valid and invalid input
- A `try` block containing both JSON `loads` and TOML `loads` requires handling both `JSONDecodeError` and `TOMLDecodeError` (exhaustiveness checking)

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
- Update stdlib-parity reports under `verification/areas/stdlib_parity/reports/` with post-remediation safety scores

### Definition of Done (milestone_zero_panic_gate)

- All gate criteria pass
- CI lint integrated and passing
- Safety audit report updated
- Phase 10 is unblocked

---

## milestone_error_subclasses: Error Subclass Hierarchy (Priority 6 -- Enhancement)

status: completed (PR #163)

**Goal:** Introduce a structured error subclass hierarchy so developers can handle specific failure modes via `except` arms with compile-time exhaustiveness checking — no string matching needed. All errors retain `message: str` populated from Rust's `Display` (for built-ins) or the constructor (for user-defined errors). Where Rust provides genuinely useful structured data (line/column on parse errors), surface it as additional typed fields alongside `message`. `print(e)` becomes the idiomatic way to display error messages (via `Display` which formats `self.message`).

**Depends on:** milestone_zero_panic_gate (all prior safety milestones must be complete; this builds on top of the stable, non-panicking foundation they established)

**Why now:** The prior milestones replaced all panics with `Result[T, IOError]`, but every I/O failure is a flat `IOError` — the developer must string-match `e.message` at runtime to distinguish "file not found" from "permission denied". This is the exact kind of fragile check Sifr's type system is designed to eliminate. By introducing subclasses (`FileNotFoundError`, `PermissionError`, etc.), the developer catches specific errors by type, and the compiler enforces exhaustive coverage. The `message` field stays — it carries Rust's human-readable error text — but the developer no longer needs to inspect it to determine the error kind.

### Design Principles

**1. The type tells you the error kind; the message tells you the details.** `FileNotFoundError` means the file wasn't found — the developer catches it by type, not by parsing a string. `e.message` still carries Rust's full error text (e.g., `"No such file or directory (os error 2)"`) for logging or display purposes.

**2. All errors have `message: str`.** Every error type — built-in and user-defined — has a `message` field. For built-in errors, `message` is auto-populated from Rust's error `Display` (`e.to_string()`). For user-defined errors, `message` is whatever the developer passes to the constructor (`raise AppError("connection failed")`). This is inherited from the base `Error` class.

**3. `print(e)` is the idiomatic way to display errors.** The codegen already generates `Display` impls that format `self.message`. `print(e)`, `f"{e}"`, and `e.message` all produce the same human-readable string. `print(e)` is preferred over `print(e.message)` as the idiomatic form.

**4. Additional structured fields where Rust provides data the developer can't know in advance.** `JSONDecodeError` and `TOMLDecodeError` get `line: int` and `column: int` alongside `message` — the developer can't predict where a parse error will occur in runtime input. `RegexError` gets `detail: str` alongside `message`. These are additional fields, not replacements for `message`.

**5. No catch-all `Other` variants.** Every `ErrorKind` that Sifr's I/O intrinsics can produce maps to a named subclass. The parent type (`IOError`) serves as the catch-all for error kinds not yet mapped to a subclass — but this is a parent-level catch, not a variant-level escape hatch.

**6. Enum variants in Rust, subclasses in Sifr.** At the Sifr language level, these look like subclasses (matching CPython). At the Rust codegen level, they are enum variants of the parent type. The intrinsic return type stays `Result[T, IOError]` — no signature changes.

### Error Hierarchy (Complete)

#### Audit of Rust Error Sources

Every error type was audited against the actual Rust error types the Sifr codegen encounters. All errors have `message: str` populated from Rust's `e.to_string()`. Some errors get additional structured fields where Rust provides data the developer can't know in advance.

| Sifr error type | Rust source | What Rust provides | Sifr fields | `message` content |
|---|---|---|---|---|
| **IOError** | `std::io::Error` | `kind()` → `ErrorKind`, `to_string()` → OS message | **`message: str`** | `"No such file or directory (os error 2)"` |
| **ParseError** | `ParseIntError`, `ParseFloatError`, `FromUtf8Error`, `base64::DecodeError` | `to_string()` → description | **`message: str`** | `"invalid digit found in string"` |
| **JSONDecodeError** | `serde_json::Error` | `line()`, `column()`, `classify()`, `to_string()` | **`message: str`, `line: int`, `column: int`** | `"expected value at line 3 column 12"` |
| **TOMLDecodeError** | `toml::de::Error` | `line_col()` → `Option<(usize, usize)>`, `to_string()` | **`message: str`, `line: int`, `column: int`** | `"expected a table key at line 5 column 1"` |
| **RegexError** | `regex::Error` | `Syntax(String)`, `to_string()` | **`message: str`, `detail: str`** | `"regex parse error: unclosed group"` |
| **ValueError** | Manually constructed | Hardcoded strings | **`message: str`** | `"empty sequence"`, `"factorial of negative"` |
| **DivisionError** | Compiler-generated | Division by zero check | **`message: str`** | `"division by zero"` |
| **KeyError** | Compiler-generated | Missing key access | **`message: str`** | `"key not found"` |

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

#### 2. HIR: Register Error Subclasses and Additional Fields (`crates/sifr_lowering/src/lower/`)

Extend the `builtin_error_classes` registration block (lines ~1376-1410). All error types keep `message: str` (inherited from `Error`). Some get additional fields:

- **`message: str` only:** `Error`, `IOError`, `FileNotFoundError`, `PermissionError`, `FileExistsError`, `IsADirectoryError`, `NotADirectoryError`, `DirectoryNotEmptyError`, `ParseError`, `ValueError`, `DivisionError`, `KeyError`
- **`message: str` + `line: int` + `column: int`:** `JSONDecodeError`, `TOMLDecodeError`
- **`message: str` + `detail: str`:** `RegexError`

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

#### 3. HIR: Expand `is_error_class` for Transitive Inheritance (`crates/sifr_lowering/src/lower/`)

- Currently `is_error_class` (line ~712) only checks `(Error)` as the base class
- Expand to recognize any class whose base is itself in `ctx.error_types` — e.g., `class MyIOError(IOError)` should be recognized as an error class
- This enables user-defined error subclasses in future milestones (not just built-in ones)

#### 4. HIR: Exhaustiveness Checking with Inheritance (`crates/sifr_lowering/src/lower/`)

- Build an error hierarchy map during lowering: `error_hierarchy: HashMap<String, Vec<String>>` mapping parent -> list of known children
- Update exhaustiveness checking (lines ~2091-2105):
  - `except IOError as e` covers `IOError` AND all its subclasses (`FileNotFoundError`, `PermissionError`, etc.)
  - `except FileNotFoundError as e` covers only `FileNotFoundError` — the compiler requires remaining `IOError` subtypes to be covered (or a catch-all `except IOError as e` / `except Error as e`)
  - When a `try` body produces `IOError` errors (from intrinsics), the exhaustiveness checker knows the full set of possible subclasses
- The existing `except Error as e` catch-all continues to work unchanged

#### 5. Codegen: Generate Error Types with Subclass Support (`crates/sifr_codegen/src/lib.rs`)

Extend the current error type generation (lines ~475-499). All error types keep `message: String`. `IOError` becomes an enum with each variant carrying `message`. Other error types with additional fields get those fields alongside `message`.

**IOError — enum with message-carrying variants:**
```rust
#[derive(Debug, Clone)]
enum IOError {
    FileNotFound { message: String },
    PermissionDenied { message: String },
    FileExists { message: String },
    IsADirectory { message: String },
    NotADirectory { message: String },
    DirectoryNotEmpty { message: String },
}

impl IOError {
    fn message(&self) -> &str {
        match self {
            IOError::FileNotFound { message } => message,
            IOError::PermissionDenied { message } => message,
            IOError::FileExists { message } => message,
            IOError::IsADirectory { message } => message,
            IOError::NotADirectory { message } => message,
            IOError::DirectoryNotEmpty { message } => message,
        }
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}
```

**JSONDecodeError — struct with message + line/column:**
```rust
#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}
```

**TOMLDecodeError — struct with message + line/column:**
```rust
#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}
```

**RegexError — struct with message + detail:**
```rust
#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}
```

**Other error types** (`ParseError`, `ValueError`, `DivisionError`, `KeyError`) — structs with `message` only (same as today):
```rust
#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}
```

#### 6. Codegen: Map Rust Errors to Correct Sifr Types (~All Intrinsic Sites) (`crates/sifr_codegen/src/lib.rs`)

**I/O intrinsics (~16 sites):** Replace all `IOError { message: e.to_string() }` with a shared helper `fn __io_err(e: std::io::Error) -> IOError` that maps to the correct subclass variant while preserving the message:
```rust
fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    match e.kind() {
        std::io::ErrorKind::NotFound => IOError::FileNotFound { message: msg },
        std::io::ErrorKind::PermissionDenied => IOError::PermissionDenied { message: msg },
        std::io::ErrorKind::AlreadyExists => IOError::FileExists { message: msg },
        std::io::ErrorKind::IsADirectory => IOError::IsADirectory { message: msg },
        std::io::ErrorKind::NotADirectory => IOError::NotADirectory { message: msg },
        std::io::ErrorKind::DirectoryNotEmpty => IOError::DirectoryNotEmpty { message: msg },
        _ => IOError::FileNotFound { message: msg }, // unreachable for filesystem ops
    }
}
```

**JSON intrinsic:** Extract `line()`, `column()`, and `to_string()` from `serde_json::Error`:
```rust
.map_err(|e| JSONDecodeError {
    message: e.to_string(),
    line: e.line() as i64,
    column: e.column() as i64,
})
```

**TOML intrinsic:** Extract `line_col()` and `to_string()` from `toml::de::Error`:
```rust
.map_err(|e| {
    let (line, column) = e.line_col().unwrap_or((0, 0));
    TOMLDecodeError {
        message: e.to_string(),
        line: (line + 1) as i64,
        column: (column + 1) as i64,
    }
})
```

**Regex intrinsics (~7 sites):** Extract both message and detail:
```rust
.map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })
```

**Parse intrinsics (int/float/base64/utf8):** Preserve the Rust error message:
```rust
.map_err(|e| ParseError { message: e.to_string() })
```

**ValueError construction:** Include descriptive message:
```rust
Err(ValueError { message: "empty sequence".to_string() })
```

Affected intrinsics: `read_text`, `write_text`, `read_lines`, `append_text`, `getcwd`, `listdir`, `mkdir`, `rmdir`, `remove_file`, `rename`, `copy_file`, `walk_dir`, `rmdir_all`, `makedirs`, `run_command`, `json_loads`, `toml_parse`, `re_match`, `re_find`, `re_replace`, `re_findall`, `re_split`, `re_find_start`, `re_find_end`, `base64_decode`, `urlsafe_b64decode`, `decode_utf8`, `bytes_from_hex`, `int()`, `float()`, `datetime_from_timestamp`

#### 7. Codegen: Try/Except Match Arms for Subclass Dispatch (`crates/sifr_codegen/src/lib.rs`)

- Update the try/except codegen (lines ~2761-2921) to handle subclass matching:
  - `except FileNotFoundError` generates: `Err(IOError::FileNotFound) => { ... }`
  - `except IOError as e` generates a catch-all arm matching all variants
  - When mixed with other error types (e.g., `IOError` + `JSONDecodeError`), the existing `_TryErr` enum pattern wraps the parent types as before — subclass dispatch happens inside the parent's match arm
- For the single-error-type case (only `IOError` in the try body), the codegen generates a `match` on `IOError` variants directly

#### 8. Stdlib Signatures: No Changes Required (`crates/sifr_stdlib`)

- Intrinsic return types stay as `Result[T, IOError]`, `Result[T, JSONDecodeError]`, etc. — no signature changes
- Stdlib `.sifr` wrappers propagate `Result` unchanged

#### 9. Architecture Documentation (`architecture.md`)

- Update the Built-in Error Classes section to document the full error hierarchy with fields
- Document the "type tells you the kind, message tells you the details" design principle
- Update the `except` exhaustiveness examples to show subclass handling
- Document the design decision: subclasses at Sifr level = enum variants at Rust level
- Document that all errors have `message: str`; some have additional fields (`line`, `column`, `detail`)
- Document `print(e)` as the idiomatic way to display errors

#### 10. E2E Tests

- **Pass test: specific subclass handling** — `read_text` on missing file caught by `except FileNotFoundError`; `e.message` contains Rust's error text
- **Pass test: parent catch-all** — `except IOError as e` catches `FileNotFoundError`
- **Pass test: mixed subclass + parent** — `except FileNotFoundError` + `except IOError` covers all cases
- **Pass test: mixed error families** — `try` block with `read_text` (IOError family) + JSON `loads` (`JSONDecodeError`), catching `FileNotFoundError` + `IOError` + `JSONDecodeError`
- **Pass test: JSONDecodeError fields** — `except JSONDecodeError as e` with access to `e.message`, `e.line`, and `e.column`
- **Pass test: TOMLDecodeError fields** — `except TOMLDecodeError as e` with access to `e.message`, `e.line`, and `e.column`
- **Pass test: RegexError field** — `except RegexError as e` with access to `e.message` and `e.detail`
- **Pass test: message on all errors** — `e.message` works for every error type; `print(e)` produces the same output via `Display`
- **Pass test: Display on all errors** — `print(e)` works for every error type via `Display`, equivalent to `print(e.message)`
- **Fail test: incomplete subclass coverage** — `except FileNotFoundError` without covering remaining IOError subtypes is a compile error
- **Pass test: user-defined error** — `class AppError(Error): pass`; `raise AppError("connection failed")`; `e.message` is `"connection failed"`; `print(e)` prints the same
- **Demo file:** `demos/error_subclasses/main.sifr` showing all patterns

### Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| `Type::Class` change is pervasive — adding `parent_class` field touches dozens of construction sites | High churn, potential for missed sites | Mechanical change; compiler errors will catch every missed site since the struct pattern becomes non-exhaustive |
| Exhaustiveness checking complexity increases | Harder to reason about coverage | Start with IOError subclasses only; other error types get additional fields but no subclasses |
| `IsADirectory` / `NotADirectory` / `DirectoryNotEmpty` ErrorKind variants may not be stable on all Rust versions | Codegen may not compile on older rustc | Check minimum Rust version; use `#[allow(unreachable_patterns)]` fallback |
| `toml::de::Error::line_col()` returns `None` for some errors | `line`/`column` may be 0 | Default to `(0, 0)` when position unavailable; document that 0 means "position unknown" |

### Definition of Done (milestone_error_subclasses)

**Type system and exhaustiveness:**
- `Type::Class` has `parent_class` field; `is_assignable_to` walks the inheritance chain
- `except FileNotFoundError` compiles and catches only file-not-found errors
- `except IOError as e` catches all IOError variants (parent = catch-all)
- Exhaustiveness checking enforces coverage of subclasses when specific handlers are used

**Codegen — error type generation:**
- `IOError` is generated as a Rust enum with message-carrying variants: `FileNotFound { message }`, `PermissionDenied { message }`, etc.
- `JSONDecodeError` and `TOMLDecodeError` are structs with `message: String`, `line: i64`, and `column: i64` fields
- `RegexError` is a struct with `message: String` and `detail: String` fields
- `ParseError`, `ValueError`, `DivisionError`, `KeyError` are structs with `message: String` (same as today)
- All error types have `message: str`; `Display` formats `self.message`
- User-defined errors inherit `message: str` from `Error` — no changes needed

**Codegen — subclass dispatch (critical):**
- All ~16 I/O intrinsic sites use the shared `__io_err` helper to map `std::io::ErrorKind` to the most specific `IOError` variant, preserving `e.to_string()` as `message`
- No intrinsic raises a generic parent error when a more specific subclass applies
- JSON `loads` extracts `line()`, `column()`, and `to_string()` from `serde_json::Error` into `JSONDecodeError` fields
- `toml_parse` extracts `line_col()` and `to_string()` from `toml::de::Error` into `TOMLDecodeError` fields
- Regex intrinsics extract `to_string()` into both `message` and `detail` on `RegexError`
- Parse operations (`int()`, `float()`, `b64decode`, etc.) preserve `e.to_string()` as `message` on `ParseError`

**Existing code compatibility:**
- All existing E2E tests pass — `e.message` continues to work unchanged
- `print(e)` works via `Display` (formats `self.message`) — encouraged as the idiomatic form

**New tests:**
- E2E pass tests for every IOError subclass with `e.message` access
- E2E pass tests for JSONDecodeError/TOMLDecodeError additional field access (`e.line`, `e.column`)
- E2E pass tests for RegexError additional field access (`e.detail`)
- E2E pass test for `print(e)` producing the same output as `e.message` via Display
- E2E fail test for incomplete subclass coverage
- E2E pass tests for mixed error families (IOError subclasses + JSONDecodeError in same try block)
- E2E pass test for user-defined error with inherited `message`

**Documentation:**
- `architecture.md` updated with full error hierarchy, field reference, and design principles
- Demo file: `demos/error_subclasses/main.sifr`

---

## Milestone Ordering

- **milestone_io_safety first:** File I/O is the most critical safety violation (5 modules, ~15 intrinsics).
- **milestone_parse_safety second:** Parse/decode is the second most critical (5 modules, ~8 intrinsics).
- **milestone_collection_safety third:** Collection/math/built-in safety is high priority but less critical than I/O and parsing.
- **milestone_edge_case_safety fourth:** Edge cases are moderate priority — important but not blocking.
- **milestone_zero_panic_gate fifth:** The gate verifies all prior milestones (1-4) are complete. It is a hard quality gate.
- **milestone_error_subclasses last:** Refines the flat error types into a CPython-aligned subclass hierarchy, enabling compile-time checked fine-grained error handling. Builds on top of the stable, non-panicking foundation established by milestones 1-5. All errors keep `message: str`; some gain additional structured fields (`line`, `column`, `detail`). `print(e)` is the idiomatic way to display errors. Phase 10 (Borrow-by-Default) depends on this milestone.
