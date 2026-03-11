# Stdlib Safety Remediation Plan

**Date:** 2026-02-17
**Goal:** Bring all 37 sifr stdlib modules into compliance with the safety philosophy: "if it compiles, it works."

---

## Current State

- **59 CPython exception paths** exist across implemented modules
- **2 (3.4%)** are correctly handled with `Result`/`Option`
- **~40** panic via `.unwrap()` in codegen
- **~12** have undefined behavior
- **~5** silently return wrong values (NaN, 0.0, -inf)

## Target State

- **100%** of CPython exception paths return `Result[T, E]` or `Option[T]`
- **0** panic paths in stdlib (except `sifr.test` assertions)
- All error types defined and documented
- All `.sifr` wrappers propagate errors with `?`

---

## Phase 1: Error Type Definitions

Define the error types needed by the stdlib. These should be classes in the sifr type system.

```python
# Proposed error types for sifr stdlib

class IOError:
    message: str
    path: str

class ParseError:
    message: str
    source: str

class ValueError:
    message: str

class OverflowError:
    message: str

class StatisticsError:
    message: str

class RegexError:
    message: str
    pattern: str

class CycleError:
    message: str
    cycle: list[int]
```

**Changes needed:**
- Add error type definitions to `sifr_type_system`
- Register them in `stdlib.rs` as known types
- Generate corresponding Rust error structs in codegen

---

## Phase 2: Intrinsic Signature Updates (`stdlib.rs`)

Update return types in `crates/sifr_hir/src/stdlib.rs`:

### File I/O Intrinsics (Priority 1)

| Intrinsic | Current Signature | New Signature |
| --- | --- | --- |
| `_sifr.io.read_text` | `(str) -> str` | `(str) -> Result[str, IOError]` |
| `_sifr.io.write_text` | `(str, str) -> None` | `(str, str) -> Result[None, IOError]` |
| `_sifr.io.read_lines` | `(str) -> list[str]` | `(str) -> Result[list[str], IOError]` |
| `_sifr.fs.append_text` | `(str, str) -> None` | `(str, str) -> Result[None, IOError]` |
| `_sifr.fs.mkdir` | `(str) -> None` | `(str) -> Result[None, IOError]` |
| `_sifr.fs.rmdir` | `(str) -> None` | `(str) -> Result[None, IOError]` |
| `_sifr.fs.remove_file` | `(str) -> None` | `(str) -> Result[None, IOError]` |
| `_sifr.fs.rename` | `(str, str) -> None` | `(str, str) -> Result[None, IOError]` |
| `_sifr.fs.copy_file` | `(str, str) -> None` | `(str, str) -> Result[None, IOError]` |
| `_sifr.fs.rmdir_all` | `(str) -> None` | `(str) -> Result[None, IOError]` |
| `_sifr.fs.listdir` | `(str) -> list[str]` | `(str) -> Result[list[str], IOError]` |
| `_sifr.fs.getcwd` | `() -> str` | `() -> Result[str, IOError]` |

### Parse/Decode Intrinsics (Priority 2)

| Intrinsic | Current Signature | New Signature |
| --- | --- | --- |
| `_sifr.json.json_loads` | `(str) -> str` | `(str) -> Result[str, ParseError]` |
| `_sifr.toml.toml_parse` | `(str) -> str` | `(str) -> Result[str, ParseError]` |
| `_sifr.crypto.base64_decode` | `(str) -> str` | `(str) -> Result[str, ParseError]` |
| `_sifr.crypto.urlsafe_b64decode` | `(str) -> str` | `(str) -> Result[str, ParseError]` |
| `_sifr.bytes.decode_utf8` | `(str) -> str` | `(str) -> Result[str, ParseError]` |
| `_sifr.bytes.bytes_from_hex` | `(str) -> str` | `(str) -> Result[str, ParseError]` |

### Regex Intrinsics (Priority 2)

| Intrinsic | Current Signature | New Signature |
| --- | --- | --- |
| `_sifr.regex.re_match` | `(str, str) -> bool` | `(str, str) -> Result[bool, RegexError]` |
| `_sifr.regex.re_replace` | `(str, str, str) -> str` | `(str, str, str) -> Result[str, RegexError]` |
| `_sifr.regex.re_findall` | `(str, str) -> list[str]` | `(str, str) -> Result[list[str], RegexError]` |
| `_sifr.regex.re_split` | `(str, str) -> list[str]` | `(str, str) -> Result[list[str], RegexError]` |

---

## Phase 3: Codegen Updates (`codegen/lib.rs`)

Replace `.unwrap()` with proper error handling in generated Rust code.

### Pattern: Before (panics)

```rust
// Current codegen for read_text
let result = std::fs::read_to_string(path).unwrap();
```

### Pattern: After (safe)

```rust
// New codegen for read_text
let result = std::fs::read_to_string(path)
    .map_err(|e| IOError { message: e.to_string(), path: path.to_string() })?;
```

Or for intrinsics that return `Result`:

```rust
// New codegen for read_text intrinsic
match std::fs::read_to_string(path) {
    Ok(content) => Ok(content),
    Err(e) => Err(IOError { message: e.to_string(), path: path.to_string() }),
}
```

---

## Phase 4: Sifr Wrapper Updates (`lib/sifr/*.sifr`)

Update `.sifr` wrappers to propagate errors.

### Pattern: Before

```python
# lib/sifr/io.sifr
from _sifr.io import read_text

def read(path: str) -> str:
    return read_text(path)
```

### Pattern: After

```python
# lib/sifr/io.sifr
from _sifr.io import read_text

def read(path: str) -> Result[str, IOError]:
    return read_text(path)?
```

---

## Phase 5: Pure Sifr Module Safety (`statistics`, `heapq`, `math`, etc.)

Update pure sifr implementations to check preconditions.

### Pattern: Before

```python
# lib/sifr/statistics.sifr
def mean(data: list[float]) -> float:
    total: float = 0.0
    i: int = 0
    n: int = len(data)
    while i < n:
        total = total + data[i]
        i = i + 1
    return total / float(n)  # Division by zero if empty!
```

### Pattern: After

```python
# lib/sifr/statistics.sifr
def mean(data: list[float]) -> Result[float, StatisticsError]:
    n: int = len(data)
    if n == 0:
        raise StatisticsError("mean requires at least one data point")
    total: float = 0.0
    i: int = 0
    while i < n:
        total = total + data[i]
        i = i + 1
    return total / float(n)
```

---

## Phase 6: E2E Test Updates

Add safety-specific E2E tests for every remediated path.

### Test Categories

1. **Happy path** — operation succeeds, returns `Ok(value)`
2. **Error path** — operation fails, returns `Err(error)` with correct error type
3. **Error handling** — user handles error with `try`/`except` or `match`
4. **Error propagation** — user propagates error with `?`
5. **Must-use enforcement** — ignoring `Result` is a compile-time error

### Example Test

```python
# tests/e2e/pass/stdlib_io_safe_read.sifr
# expect-stdout: File not found

def main() -> Result[None, IOError]:
    try:
        content: str = read_text("/nonexistent/file")?
    except IOError as e:
        print("File not found")
    return Ok(None)
```

```python
# tests/e2e/fail/stdlib_io_must_use.sifr
# expect-error: [unused-result]

def main():
    read_text("/some/file")  # Must handle the Result!
```

---

## Effort Estimate

| Phase | Scope | Estimated Effort |
| --- | --- | --- |
| Phase 1: Error types | 7 error types | Small |
| Phase 2: Intrinsic signatures | ~25 intrinsics | Medium |
| Phase 3: Codegen updates | ~25 codegen paths | Medium |
| Phase 4: Sifr wrapper updates | ~15 wrapper files | Small |
| Phase 5: Pure sifr safety | ~15 functions | Small |
| Phase 6: E2E tests | ~30 new tests | Medium |
| **Total** | | **Medium-Large** |

---

## Dependencies

- Error type system must support class-based errors (already works via `milestone_classes`)
- `Result[T, E]` codegen must work for intrinsic return values (needs verification)
- `?` operator must work in sifr wrapper functions (already works for user code)
- `#[must_use]` enforcement must apply to intrinsic-returned Results (needs verification)

---

## Success Criteria

1. `cargo test` passes with all new safety tests
2. Zero `.unwrap()` calls in stdlib codegen paths
3. Every CPython exception path has a corresponding `Result`/`Option` return
4. E2E tests verify both success and error paths for every fallible operation
5. E2E fail tests verify `#[must_use]` enforcement on stdlib Results
