# milestone_core_stdlib — Core Standard Library

## 1. Product Requirements

### Objective

Provide the foundational stdlib modules that almost every real program needs. This milestone establishes the pattern for how stdlib modules work: the compiler recognizes `from sifr.* import ...` statements, registers type signatures for stdlib functions, emits the correct Rust code, and injects Cargo dependencies automatically.

### Scope — Scoped Down for Initial Implementation

**In Scope:**

1. **Stdlib infrastructure** — `from sifr.* import ...` recognition in HIR lowering, type registration, codegen mapping, Cargo dependency injection
2. **`sifr.io`** — `read_file(path) -> str`, `write_file(path, content)`, `file_exists(path) -> bool`, `read_lines(path) -> list[str]`
3. **`sifr.json`** — `json_loads(s) -> str` (parse JSON string to string representation), `json_dumps(obj) -> str` (serialize to JSON string)
4. **`sifr.env`** — `get_env(key) -> str | None`, `set_env(key, value)`
5. **`sifr.os`** — `run_command(cmd) -> str` (run shell command, return stdout), `get_args() -> list[str]` (command-line arguments)
6. **`sifr.math`** — `sqrt(x) -> float`, `floor(x) -> int`, `ceil(x) -> int`, `pi -> float`, `e -> float`

**Out of Scope (deferred to later milestones):**

| Feature | Reason |
| --- | --- |
| `sifr.toml` | Lower priority, can use `sifr.json` for config |
| `sifr.collections` (Set, OrderedDict, Deque) | Deferred to milestone_ext_collections |
| `open()` built-in with context manager | Requires more complex File class + with integration |
| Error handling with Result types | Keep simple with string returns for now |
| `sifr.stream`, `sifr.log` | Deferred to milestone_ext_stdlib |

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | **Given** `from sifr.io import read_file`, **When** compiled, **Then** the function is recognized with correct type signature and emits `std::fs::read_to_string` |
| AC-2 | **Given** `from sifr.json import json_loads`, **When** compiled, **Then** emits `serde_json` calls and `Cargo.toml` includes `serde_json` dependency |
| AC-3 | **Given** `from sifr.env import get_env`, **When** compiled, **Then** emits `std::env::var` with correct Option wrapping |
| AC-4 | **Given** `from sifr.os import run_command`, **When** compiled, **Then** emits `std::process::Command` usage |
| AC-5 | **Given** `from sifr.math import sqrt, pi`, **When** compiled, **Then** emits `f64::sqrt()` and `std::f64::consts::PI` |
| AC-6 | **Given** any stdlib import, **When** Cargo.toml is generated, **Then** required crate dependencies are automatically included |
| AC-7 | **Given** all existing 94 E2E tests, **When** `cargo test` is run, **Then** all pass with no regressions |

---

## 2. Solution Design

### 2.1 Architecture

The stdlib infrastructure requires changes across 4 crates:

```
sifr_hir/src/lower.rs          → Recognize sifr.* imports, register stdlib types
sifr_hir/src/stdlib.rs (NEW)   → Define stdlib module signatures
sifr_codegen/src/lib.rs        → Emit correct Rust code for stdlib calls
sifr_codegen/src/stdlib.rs (NEW) → Stdlib codegen mappings
sifr_driver/src/lib.rs         → Inject Cargo dependencies based on used stdlib modules
```

### 2.2 Stdlib Registry (sifr_hir/src/stdlib.rs)

A registry mapping `sifr.*` module names to their function/constant signatures:

```rust
pub struct StdlibModule {
    pub functions: HashMap<String, FunctionType>,
    pub constants: HashMap<String, Type>,
}

pub fn get_stdlib_module(name: &str) -> Option<StdlibModule> { ... }
```

### 2.3 Import Interception (sifr_hir/src/lower.rs)

When lowering `from sifr.io import read_file`:
1. Check if module starts with `sifr.`
2. Look up in stdlib registry
3. Register function types in the scope
4. Mark the import as a stdlib import (not a local module import)

### 2.4 Codegen Mapping (sifr_codegen/src/stdlib.rs)

When emitting a call to a stdlib function:
1. Check if the function name is a known stdlib function
2. Emit the correct Rust code (e.g., `std::fs::read_to_string(path).unwrap()`)
3. Track which stdlib modules are used for Cargo dependency injection

### 2.5 Cargo Dependency Injection (sifr_driver/src/lib.rs)

After codegen, check which stdlib modules were used and add their Rust crate dependencies to the generated `Cargo.toml`.

### 2.6 Module-to-Rust Mapping

| Sifr Module | Function | Rust Code |
| --- | --- | --- |
| `sifr.io` | `read_file(path)` | `std::fs::read_to_string(path).unwrap()` |
| `sifr.io` | `write_file(path, content)` | `std::fs::write(path, content).unwrap()` |
| `sifr.io` | `file_exists(path)` | `std::path::Path::new(&path).exists()` |
| `sifr.io` | `read_lines(path)` | `std::fs::read_to_string(path).unwrap().lines().map(\|s\| s.to_string()).collect()` |
| `sifr.json` | `json_loads(s)` | `serde_json::from_str(&s).unwrap()` |
| `sifr.json` | `json_dumps(obj)` | `serde_json::to_string(&obj).unwrap()` |
| `sifr.env` | `get_env(key)` | `std::env::var(key).ok()` |
| `sifr.env` | `set_env(key, value)` | `std::env::set_var(key, value)` |
| `sifr.os` | `run_command(cmd)` | `String::from_utf8(std::process::Command::new("sh").args(["-c", &cmd]).output().unwrap().stdout).unwrap()` |
| `sifr.os` | `get_args()` | `std::env::args().collect()` |
| `sifr.math` | `sqrt(x)` | `(x).sqrt()` |
| `sifr.math` | `floor(x)` | `(x).floor() as i64` |
| `sifr.math` | `ceil(x)` | `(x).ceil() as i64` |
| `sifr.math` | `pi` | `std::f64::consts::PI` |
| `sifr.math` | `e` | `std::f64::consts::E` |

### 2.7 Testing Strategy

| AC-ID | Test Layer | Happy-Path Check |
| --- | --- | --- |
| AC-1 | E2E | `read_file` / `write_file` roundtrip |
| AC-2 | E2E | `json_loads` / `json_dumps` roundtrip |
| AC-3 | E2E | `get_env` / `set_env` roundtrip |
| AC-4 | E2E | `run_command("echo hello")` returns "hello" |
| AC-5 | E2E | `sqrt(4.0)` returns `2.0`, `pi` is correct |
| AC-6 | E2E | Generated Cargo.toml has serde_json when json is used |
| AC-7 | Full suite | All 94 existing tests pass |

**Demo:** `./demos/milestone_core_stdlib_demo.sifr`
