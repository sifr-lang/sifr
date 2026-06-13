# Codegen Evidence: Line-by-Line References for Each Contradiction

Generated: 2026-02-17

This document provides exact file paths and line numbers for every contradiction identified in the audit. All line numbers are approximate (±5 lines) due to ongoing development.

---

## Contradiction 1: I/O Operations Use `.unwrap()` Instead of `Result`

### Intrinsic Type Signatures (no Result in return type)

**File:** `crates/sifr_hir/src/stdlib.rs`

```
// Lines 54-57: read_text returns Str, not Result[Str, IOError]
functions.insert("read_text".to_string(), FunctionType::all_borrow(
    vec![("path".to_string(), Type::Str)],
    Type::Str,
));

// Lines 60-63: write_text returns None, not Result[None, IOError]
functions.insert("write_text".to_string(), FunctionType::all_borrow(vec![
    ("path".to_string(), Type::Str),
    ("content".to_string(), Type::Str),
], Type::None));
```

### Codegen Emission (unwrap calls)

**File:** `crates/sifr_codegen/src/lib.rs`

| Operation | Approximate Line | Generated Rust |
| --- | --- | --- |
| `read_text` | 5090-5093 | `std::fs::read_to_string(...).unwrap()` |
| `write_text` | 5095-5100 | `std::fs::write(...).unwrap()` |
| `read_lines` | 5107-5110 | `std::fs::read_to_string(...).unwrap().lines()...` |
| `append_text` | 5112-5117 | `OpenOptions::new()...open(...).unwrap(); write!(...).unwrap()` |
| `getcwd` | 5119-5120 | `std::env::current_dir().unwrap()...` |
| `listdir` | 5122-5125 | `std::fs::read_dir(...).unwrap()...` |
| `mkdir` | 5127-5130 | `std::fs::create_dir_all(...).unwrap()` |
| `rmdir` | 5132-5135 | `std::fs::remove_dir(...).unwrap()` |
| `remove_file` | 5137-5140 | `std::fs::remove_file(...).unwrap()` |
| `rename` | 5142-5146 | `std::fs::rename(...).unwrap()` |

### Architecture Requirement

**File:** `internal_docs/architecture.md`

- Line 15: "No panics in user code."
- Line 21: "File I/O, network, and all stdlib operations that can fail" must return Result/Option
- Lines 56-57: "Where CPython raises an exception, Sifr returns `Result[T, E]`"

---

## Contradiction 2: `list.remove()` / `list.index()` Panic

### Codegen

**File:** `crates/sifr_codegen/src/lib.rs`

```
// Lines 3543-3553: list.remove(val)
(Type::List(_), "remove") => {
    // list.remove(val) -> { let pos = list.iter().position(|x| *x == val).unwrap(); list.remove(pos); }
    self.write("{ let __pos = ");
    self.emit_expr(object);
    self.write(".iter().position(|__x| *__x == ");
    // ...
    self.write(").unwrap(); ");
    // ...
}

// Lines 3555-3562: list.index(val)
(Type::List(_), "index") => {
    // list.index(val) -> list.iter().position(|x| *x == val).unwrap() as i64
    // ...
    self.write(").unwrap() as i64");
}
```

### Architecture Requirement

- Line 57: "Where CPython raises `IndexError`, Sifr returns `Option[T]`"
- CPython `list.remove()` raises `ValueError`; `list.index()` raises `ValueError`
- Both should return `Result` or `Option` per architecture rules

---

## Contradiction 3: `min()` / `max()` Panic on Empty Lists

### Codegen

**File:** `crates/sifr_codegen/src/lib.rs`

```
// Line ~4314: min() for float lists
self.write(".iter().cloned().reduce(f64::min).unwrap()");

// Line ~4319: min() for int lists
self.write(".iter().min().unwrap()");

// Line ~4338: max() for float lists
self.write(".iter().cloned().reduce(f64::max).unwrap()");

// Line ~4343: max() for int lists
self.write(".iter().max().unwrap()");
```

### Architecture Requirement

- Line 56: "Where CPython raises an exception, Sifr returns `Result[T, E]`"
- CPython `min([])` raises `ValueError`

---

## Contradiction 4: `SubscriptAssign` Bypasses Safe Indexing

### Codegen

**File:** `crates/sifr_codegen/src/lib.rs`

```
// Lines 2787-2797: list[i] = val
HirStmt::SubscriptAssign { object, index, value, object_ty } => {
    self.write_indent();
    match object_ty {
        Type::List(_) => {
            // list[i] = val -> list[i as usize] = val
            self.write(object);
            self.write("[");
            self.emit_expr(index);
            self.write(" as usize] = ");
            self.emit_expr(value);
            self.write(";\n");
        }
        // ...
    }
}
```

### Stdlib Usage

**File:** `lib/sifr/graphlib.sifr`, line 32:
```
visited[node] = 1
```

### Architecture Requirement

**File:** `internal_docs/architecture.md`

- Line 307: "Global indexing contract: all indexable types (`str`, `list`, `dict`) use safe indexing. `x[i]` returns `Option[T]`, never panics. This is enforced uniformly across the language."

The word "uniformly" implies both read and write paths. Only read is safe.

---

## Contradiction 5: `set.pop()` Panics

### Codegen

**File:** `crates/sifr_codegen/src/lib.rs`

```
// Lines 3732-3737: set.pop()
self.write("{ let __v = ");
self.emit_expr(object);
self.write(".iter().next().cloned().unwrap(); ");
self.emit_expr(object);
self.write(".remove(&__v); __v }");
```

### Architecture Requirement

- CPython `set.pop()` raises `KeyError` on empty set
- Should return `Option[T]` per architecture rules

---

## Contradiction 6: Statistics Sentinel Returns

### Stdlib Code

**File:** `lib/sifr/statistics.sifr`

```
// Lines 75-90: median_low returns 0.0 for empty data
def median_low(data: list[float]) -> float:
    n: int = len(data)
    if n == 0:
        return 0.0
    // ...
        return 0.0

// Lines 103-124: mode returns 0 for empty data
def mode(data: list[int]) -> int:
    if len(data) == 0:
        return 0
```

Similar pattern in: `median` (line 11-26), `harmonic_mean` (line 53-62), `geometric_mean` (line 64-73), `median_high` (line 92-101).

### Architecture Requirement

- Line 56: "Where CPython raises an exception, Sifr returns `Result[T, E]`"
- CPython raises `statistics.StatisticsError` for empty data

---

## Contradiction 7: `heappop()` Sentinel Return

### Stdlib Code

**File:** `lib/sifr/heapq.sifr`

```
// Lines 90-96: returns 0 for empty heap
def heappop(heap: list[int]) -> int:
    if len(heap) == 0:
        return 0
    top: int | None = heap[0]
    if top is not None:
        return top
    return 0
```

### Architecture Requirement

- CPython raises `IndexError` for empty heap
- Should return `Option[int]` per architecture rules

---

## Contradiction 8: No `mut` Parameters in Stdlib

### Evidence

**Files:** All 37 files in `lib/sifr/*.sifr`

A grep for `def.*\bmut\b` across all stdlib files returns zero matches. Every function signature uses the default borrow convention.

### Explicit Workaround Comment

**File:** `lib/sifr/heapq.sifr`, line 3:
```
# Implements a min-heap. Functions return new lists (functional style)
# to work with Sifr's borrow-by-default parameter convention.
```

### Architecture Definition

**File:** `internal_docs/architecture.md`, line 175:
```
Use `mut` keyword for mutable borrowing (`mut x: list[int]` generates `x: &mut Vec<i64>`).
```

This feature is defined but never used by the stdlib.

---

## Contradiction 9: Generator Codegen vs. Borrowed Parameters

### Evidence

**File:** `lib/sifr/itertools.sifr`, lines 56-64:
```
def enumerate_list(data: list[str]) -> list[int]:
    # Eager: references borrowed parameter inside loop condition,
    # which conflicts with move closure in generator codegen
    result: list[int] = []
    i: int = 0
    while i < len(data):
        result.append(i)
        i = i + 1
    return result
```

Also lines 6-8:
```
# Functions that can use lazy generators (yield) do so. Functions that require
# list indexing (which returns Option<T>) or for-loop iteration remain eager
# because the generator codegen doesn't yet handle Option unwrapping in yield
```

### Architecture Definition

**File:** `internal_docs/architecture.md`, Contract #12 (line 398):
The `Iterator` protocol should enable lazy iteration. But the generator codegen cannot handle borrowed parameters in loop conditions, forcing eager collection.

---

## Contradiction 10: Counter JSON Workaround

### Evidence

**File:** `lib/sifr/collections.sifr`, lines 4-9:
```
class Counter:
    data: str  # This is a JSON-encoded dict, not dict[str, int]

    def __init__(self, data: str):
        self.data = data
```

**File:** `lib/sifr/collections.sifr`, lines 28-29:
```
    def increment(self, key: str) -> None:
        self.data = counter_increment(self.data, key)
```

The intrinsic `counter_increment` (in `crates/sifr_hir/src/stdlib.rs`) takes `str` and returns `str` — it deserializes JSON, modifies the map, and re-serializes. This avoids having a `dict[str, int]` field that would require more complex `&mut self` codegen.

### Architecture Expectation

Contract #2 states method receivers use auto-borrow based on body analysis. A `dict` field with `.insert()` calls should trigger `&mut self`. The workaround suggests this path may not work reliably for dict fields.
