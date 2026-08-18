# Standard Library

This phase delivers the standard library that makes Sifr a practical, batteries-included language. It starts with the core modules every real program needs (file I/O, JSON, config, env vars), ships a built-in test runner so all subsequent work can be dogfooded, then fills out extended collection types, binary data handling, and utility modules. A final codegen polish pass ensures all stdlib function calls emit clean, idiomatic Rust.

## milestone_core_stdlib: Core Standard Library

status: completed

**Goal:** Provide the foundational stdlib modules that almost every real program needs. This milestone establishes the pattern for how stdlib modules work: thin Sifr wrappers over battle-tested Rust crates, with auto-generated Cargo dependencies. No async dependency -- these are synchronous building blocks.

### Stdlib Modules

- `**sifr.io`:** file read/write, stdin/stdout, path operations -> wraps `std::fs` + `std::io` + `std::path`. Includes the `open()` built-in function:
  - `open(path)` -> `Result[File, IOError]` -- open file for reading (default mode)
  - `open(path, mode="w")` -> `Result[File, IOError]` -- open for writing
  - `open(path, mode="a")` -> `Result[File, IOError]` -- open for appending
  - `File` implements `ContextManager` protocol for use with `with` statement
  - `File.read()` -> `Result[str, IOError]`, `File.write(s)` -> `Result[int, IOError]`
  - `File.readlines()` -> `Result[list[str], IOError]`, `File.readline()` -> `Result[str, IOError]`
- `**sifr.json`:** JSON serialization/deserialization -> wraps `serde` + `serde_json`
- `**sifr.toml`:** TOML config parsing -> wraps `toml` crate
- `**sifr.env`:** environment variables, dotenv loading -> wraps `std::env` + `dotenvy`
- `**sifr.os`:** process spawning, signals, exit codes, argv, shell commands -> wraps `std::process` + `std::env`
- `**sifr.collections`:** `Set`, `OrderedDict`, `Deque` -> wraps `std::collections`

**Why these first:** File I/O, JSON, config, and env vars are needed by virtually every non-trivial program. `sifr.os` enables process spawning (needed by the test runner in milestone_test_runner). `sifr.collections` extends the built-in types.

### Implementation Strategy

Each stdlib module is a thin Sifr wrapper around battle-tested Rust crates. The codegen emits `use` statements and function calls to the underlying Rust crate. The sifr compiler bundles these as Cargo dependencies in the generated project.

```python
# Sifr code
from sifr.json import loads, dumps
from sifr.io import read_file, write_file

def main():
    data: str = read_file("config.json")
    config: dict[str, str] = loads(data)
    print(config["name"])
```

### Definition of Done (milestone_core_stdlib)

- Each stdlib module has a working Sifr API that compiles to the underlying Rust crate
- `sifr.io`: file read/write, path operations work end-to-end
- `sifr.json`: serialize/deserialize dicts and lists
- `sifr.toml`: parse TOML config files
- `sifr.env`: read environment variables, dotenv loading
- `sifr.os`: process spawning, argv, exit codes
- `sifr.collections`: Set, OrderedDict, Deque operations
- Each module has integration tests verifying the Sifr API against the Rust crate behavior
- Generated Cargo.toml includes correct dependencies for used stdlib modules
- E2E pass tests: file_io, json_roundtrip, env_vars, os_process, collections_basic
- CPython parity tests pass with safe error handling (no panics, `Result`/`Option` where CPython raises). Reference: `Lib/json/`, `Lib/os.py`, `Lib/test/test_json/`, `Lib/test/test_os.py`, `Objects/setobject.c`, `Objects/odictobject.c`
- Milestone demo in `./demos/core_stdlib/main.sifr`

---

## milestone_test_runner: Built-in Test Runner

status: completed

**Goal:** Ship a built-in test runner early so that all subsequent stdlib work (milestone_ext_collections, milestone_ext_stdlib) can be tested using Sifr's own test runner, dogfooding the language. Every modern language (Go, Rust, Bun, Deno) ships with a test runner -- Sifr does too. Tests are first-class citizens of the language.

### Test Syntax

```python
from sifr.test import test, assert_eq, assert_true, assert_err

def test_addition():
    assert_eq(1 + 1, 2)

def test_string_upper():
    assert_eq("hello".upper(), "HELLO")

def test_division_by_zero():
    result = 1 / 0
    assert_err(DivisionError, result)
```

### Features

- **Test discovery:** `sifr test` finds all functions named `test_*` in files named `test_*.sifr` or `*_test.sifr`
- **Assertions:** `assert_eq`, `assert_ne`, `assert_true`, `assert_false`, `assert_err`, `assert_ok`, `assert_none`, `assert_contains`
- **Test filtering:** `sifr test -k "test_string"` runs only matching tests
- **Parallel execution:** tests run in parallel by default (each test is independent)
- **Setup/teardown:** `setup()` and `teardown()` functions in test files run before/after each test
- **Test output:** clear pass/fail reporting with source locations for failures
- **Exit code:** non-zero exit on any failure (CI-friendly)

### Codegen

`sifr test` compiles test files into a Rust test binary using `#[test]` attributes. Assertions map to Rust's `assert_eq!`, `assert!`, etc. The test binary is built and run via `cargo test`.

### Dependencies

Depends on milestone_core_stdlib: needs `sifr.io` for test file discovery and `sifr.os` for process management. Does NOT depend on milestone_ext_collections or milestone_ext_stdlib.

### Definition of Done (milestone_test_runner)

- `sifr test` discovers and runs `test_*` functions in `test_*.sifr` / `*_test.sifr` files
- Assertions (`assert_eq`, `assert_ne`, `assert_true`, `assert_false`, `assert_err`, `assert_ok`, `assert_none`, `assert_contains`) work correctly
- Test filtering (`-k`) works
- Parallel execution works (tests run independently)
- Setup/teardown functions execute before/after each test
- Clear pass/fail reporting with source locations for failures
- Non-zero exit code on any failure (CI-friendly)
- Codegen emits `#[test]` attributes and maps assertions to Rust equivalents
- E2E pass tests: test_runner_basic, test_filtering, test_assertions, test_setup_teardown
- Milestone demo in `./demos/test_runner`

---

## milestone_ext_collections: Extended Collections and Binary Data

status: completed

**Goal:** Provide Python's extended collection types and the `bytes` type for binary data handling. These types are commonly needed in real programs but were not part of the core `list`/`dict`/`tuple` foundation in milestone_control_flow or the basic `Set`/`OrderedDict`/`Deque` in milestone_core_stdlib.

### Extended Collection Types

- `**frozenset[T]`:** immutable set. Codegen: `HashSet<T>` with compile-time mutation rejection. Useful as dict keys and set elements (since it's hashable). Supports all set operations (union, intersection, difference) but no `.add()` or `.remove()`.
- `**Counter[T]`:** counting collection. Thin wrapper over `HashMap<T, int>` with counting operations:
  - `Counter(iterable)` -> count occurrences of each element
  - `.most_common(n)` -> `list[tuple[T, int]]` -- top N elements by count
  - Counter arithmetic: `+` (combine counts), `-` (subtract counts), `&` (min counts), `|` (max counts)
  - `.total()` -> `int` -- sum of all counts
  - `.elements()` -> iterator repeating elements by count
- `**defaultdict[K, V]`:** dict with default factory. Codegen: `HashMap` with `.entry().or_insert_with(factory)`:
  - `defaultdict(int)` -> default value is `0`
  - `defaultdict(list)` -> default value is `[]`
  - `defaultdict(factory_fn)` -> custom default factory
  - Indexing `d[key]` auto-creates the default if key is missing (unlike regular `dict` which returns `Option`)

### Set Operations (for `Set` from milestone_core_stdlib and `frozenset`)

- `.add(item)` -> add item (Set only, compile error on frozenset)
- `.remove(item)` -> `Result[None, KeyError]` -- remove item, error if not found
- `.discard(item)` -> remove if present, no error if missing
- `.union(other)` / `|` operator -> new set with elements from both
- `.intersection(other)` / `&` operator -> new set with common elements
- `.difference(other)` / `-` operator -> new set with elements not in other
- `.symmetric_difference(other)` / `^` operator -> new set with elements in either but not both
- `.issubset(other)` -> `bool`, `.issuperset(other)` -> `bool`
- `len(s)` -> `int`, `in` operator for membership

### Binary Data Types

- `**bytes`:** immutable byte sequence. Codegen: `Vec<u8>` (with compile-time mutation rejection).
  - `b"hello"` literal syntax
  - `bytes(n)` -> zero-filled bytes of length n
  - `bytes(iterable)` -> from iterable of ints (0-255)
  - `.decode(encoding)` -> `Result[str, DecodeError]` -- decode to string (default UTF-8)
  - `str.encode(encoding)` -> `bytes` -- encode string to bytes (default UTF-8)
  - Indexing `b[i]` returns `Option[int]` (0-255)
  - Slicing `b[a:b]` returns `bytes`
  - `.hex()` -> `str` -- hexadecimal representation
  - `bytes.fromhex(s)` -> `Result[bytes, ParseError]`
- `**bytearray`:** mutable byte sequence. Codegen: `Vec<u8>`.
  - Same API as `bytes` plus mutation methods: `.append()`, `.extend()`, `.pop()`, `.clear()`
  - Converts to/from `bytes`: `bytes(ba)`, `bytearray(b)`

### Definition of Done (milestone_ext_collections)

- `frozenset` works as immutable set; mutation is a compile-time error
- `frozenset` is hashable and usable as dict key / set element
- `Counter` counts elements and supports arithmetic operations
- `defaultdict` auto-creates default values on missing key access
- Set operations (`|`, `&`, `-`, `^`) work for both `Set` and `frozenset`
- `bytes` and `bytearray` handle binary data with encode/decode
- `b"..."` literal syntax works
- `.decode()` / `.encode()` convert between `str` and `bytes`
- E2E pass tests: frozenset_basic, frozenset_as_key, counter_basic, counter_arithmetic, defaultdict_basic, set_operations, bytes_literal, bytes_decode_encode, bytearray_mutate
- E2E fail tests: frozenset_mutation_rejected, bytes_mutation_rejected, decode_invalid_utf8
- CPython parity tests pass with safe error handling (no panics, `Result`/`Option` where CPython raises). Reference: `Objects/setobject.c`, `Objects/bytesobject.c`, `Objects/bytearrayobject.c`, `Lib/collections/__init__.py` (Counter, defaultdict), `Lib/test/test_set.py`, `Lib/test/test_bytes.py`, `Lib/test/test_collections.py`
- Milestone demo in `./demos/extended_collections/main.sifr`

---

## milestone_ext_stdlib: Extended Standard Library

status: completed

**Goal:** Fill out the remaining stdlib modules -- utilities that are commonly needed but don't block other milestones. Uses the same stdlib infrastructure pattern established in milestone_core_stdlib.

### Stdlib Modules

- `**sifr.math`:** math functions (sqrt, pow, abs, min, max, floor, ceil, etc.) -> wraps `std::f64` + `num` traits
- `**sifr.time`:** timestamps, durations, sleep, formatting -> wraps `std::time` + `chrono`
- `**sifr.random`:** random number generation -> wraps `rand` crate
- `**sifr.re`:** regular expressions -> wraps `regex` crate
- `**sifr.hashlib`:** hashing (sha256, md5, etc.) -> wraps `sha2` + `md5` crates
- `**sifr.base64`:** base64, hex, url encoding -> wraps `base64` + `hex` + `percent-encoding`
- `**sifr.stream`:** streaming read/write for large data -> wraps Rust's `Read`/`Write` traits with buffered readers/writers, line-by-line iteration, and pipe-style chaining
- `**sifr.logging`:** structured logging -> wraps `tracing` crate

### Definition of Done (milestone_ext_stdlib)

- `sifr.math`: basic math functions work (sqrt, pow, abs, min, max, floor, ceil)
- `sifr.time`: timestamps, durations, sleep, formatting work
- `sifr.random`: random number generation works
- `sifr.re`: regex match, search, replace work
- `sifr.hashlib`: sha256, md5 hashing works
- `sifr.base64`: base64, hex, url encoding/decoding works
- `sifr.stream`: streaming read/write with line iteration and chaining
- `sifr.logging`: structured logging with levels (debug, info, warn, error)
- Each module has integration tests verifying the Sifr API against the Rust crate behavior
- Generated Cargo.toml includes correct dependencies for used stdlib modules
- E2E pass tests: math_ops, time_basic, random_gen, regex_match, hashlib_sha256, b64encode, stream_lines, logging_basic
- CPython parity tests pass with safe error handling (no panics, `Result`/`Option` where CPython raises). Reference: `Lib/test/test_math.py`, `Lib/test/test_time.py`, `Lib/test/test_random.py`, `Lib/test/test_re/`
- Milestone demo in `./demos/extended_stdlib/main.sifr`

---

## milestone_codegen_quality_v3: Phase 3 Codegen Polish

status: completed

**Goal:** Clean up the emitted Rust code from Phase 3 stdlib modules. Eliminate redundant allocations, unnecessary clones, and improve the idiomatic quality of generated code for all stdlib function calls.

### Quality Issues

1. **Redundant `.to_string()` on string literal args** — stdlib functions that accept `&str` receive `"literal".to_string()` instead of `"literal"` directly
2. **Redundant `.clone()` on `vec![...]` literals** — set operations clone freshly-created vecs
3. `**dumps` emits `.clone()` instead of `serde_json::to_string**` — incorrect serialization
4. `**set_intersection` re-creates second set inside filter closure** — O(n*m) allocation instead of O(n+m)
5. `**sub` uses `.to_string().as_str()**` — unnecessary String allocation
6. **Hash/encoding functions use `.to_string().as_bytes()**` — should use `.as_bytes()` directly on literals

### Implementation

Add `emit_expr_as_str_ref` helper to `RustEmitter` that emits bare `"literal"` for string literals and `&expr` for variables. Update all stdlib codegen call sites.

### Definition of Done (milestone_codegen_quality_v3)

- All stdlib function calls emit clean, idiomatic Rust without redundant allocations
- String literals passed directly to Rust APIs that accept `&str` / `AsRef<str>`
- Vec literals not cloned unnecessarily in set operations
- `dumps` uses `serde_json::to_string`
- `set_intersection` hoists second arg before filter
- All existing E2E tests pass (no regressions)
- All Phase 3 demos produce identical output with cleaner Rust

---

## Milestone ordering

The milestones within this phase are ordered as follows:

- **milestone_core_stdlib after milestone_decorators:** Core stdlib benefits from decorators for API design patterns (e.g., `@contextmanager`)
- **milestone_test_runner after milestone_core_stdlib:** Test runner lands early so subsequent stdlib work can be tested using Sifr's own test runner (dogfooding)
- **milestone_ext_collections and milestone_ext_stdlib after milestone_test_runner:** Both depend on core stdlib; in flat order ext_collections comes first since extended stdlib modules may use extended collection types
