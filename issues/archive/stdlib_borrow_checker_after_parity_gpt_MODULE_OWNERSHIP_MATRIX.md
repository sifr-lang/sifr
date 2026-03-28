# Module Ownership Matrix (37 Modules)

Generated: 2026-02-17

Legend:
- **Style**: `IntrinsicWrapper`, `PureSifr`, `ClassAPI`, `Hybrid`.
- **Ownership Pattern**:
  - `RO params` = read-only by default.
  - `CopyReturn` = returns new collections/values instead of mutating input.
  - `SelfMut` = class method mutates internal fields.
- **Explicit own/mut** refers to exported stdlib function signatures in `lib/sifr`.
- **Risk** reflects borrow/ownership + contract alignment risk, not feature completeness.

| Module | Style | Ownership Pattern | Explicit own/mut | Primary Risk | Notes |
| --- | --- | --- | --- | --- | --- |
| `argparse` | PureSifr | RO params, CopyReturn | No | Low | Pure parsing helpers; no ownership transfer paths. |
| `base64` | IntrinsicWrapper | RO params | No | High | Decode path can panic in intrinsic runtime on bad input. |
| `bisect` | PureSifr | RO params, CopyReturn | No | Medium | Generic over `T`; insertion returns new list. |
| `bytes` | IntrinsicWrapper | RO params | No | High | UTF-8/hex decode paths use panic-style runtime unwraps. |
| `collections` | Hybrid | SelfMut + RO params | No | Medium | `Counter.increment` mutates `self.data`; no explicit ownership-transfer APIs. |
| `csv` | PureSifr | RO params, CopyReturn | No | Low | Stateless conversions to/from rows. |
| `datetime` | Hybrid | RO params + class value ops | No | Medium | `timedelta` is value-style; intrinsic backing still not `Result`-typed. |
| `difflib` | PureSifr | RO params, CopyReturn | No | Low | Local accumulation only. |
| `env` | IntrinsicWrapper | RO params | No | Medium | `env_get` is optional-safe; `env_set` side-effectful but not ownership-heavy. |
| `fnmatch` | PureSifr | RO params | No | Low | Recursive matcher, no aliasing complexities. |
| `functools` | PureSifr | RO params | No | Low | Small utility surface. |
| `glob` | Hybrid | RO params, CopyReturn | No | High | Depends on filesystem intrinsic behavior that panics on IO failures. |
| `graphlib` | Hybrid | SelfMut + CopyReturn | No | Medium | `TopologicalSorter.add` mutates receiver; algorithmic state remains local/owned. |
| `hashlib` | IntrinsicWrapper | RO params | No | Medium | Mostly deterministic hashing, lower borrow risk. |
| `heapq` | PureSifr | CopyReturn | No | Medium | Functionalized heap API avoids mut-borrow inputs; diverges from in-place style. |
| `io` | IntrinsicWrapper | RO params | No | High | File read/write wrappers use panic-style runtime behavior on failure. |
| `ipaddress` | PureSifr | RO params, CopyReturn | No | Low | Pure transforms/parsing with sentinel-style error handling. |
| `itertools` | PureSifr | CopyReturn + limited generator usage | No | Medium | Mostly eager copy-return paths; minimal ownership stress. |
| `json` | IntrinsicWrapper | RO params | No | High | Parse/serialize uses panic-style runtime unwrap, not `Result`. |
| `logging` | ClassAPI | SelfMut + RO params | No | Low | `set_level` mutates receiver; otherwise read-only formatting behavior. |
| `math` | Hybrid | RO params | No | Low-Med | Mostly numeric operations; borrow semantics straightforward. |
| `os` | IntrinsicWrapper | RO params | No | High | Command/filesystem wrappers can panic on runtime failures. |
| `pathlib` | ClassAPI | RO params + side-effect methods | No | High | Methods delegate to filesystem wrappers with panic-style intrinsic behavior. |
| `platform` | IntrinsicWrapper | RO params | No | Low-Med | Lightweight platform value wrappers. |
| `random` | IntrinsicWrapper | RO params | No | High | Invalid range conditions can panic in runtime intrinsic paths. |
| `re` | Hybrid | RO params + class value type | No | High | Regex compile/find wrappers currently panic on invalid regex in runtime path. |
| `secrets` | Hybrid | RO params | No | High | Uses random range wrappers; edge cases like invalid bounds can panic. |
| `shutil` | IntrinsicWrapper | RO params | No | High | File operations currently emitted through panic-style runtime wrappers. |
| `statistics` | PureSifr | RO params, CopyReturn | No | Medium | Pure math style, but fallback/sentinel semantics instead of typed errors. |
| `string` | PureSifr | RO params, constants | No | Low | Constants + capwords helper only. |
| `tempfile` | Hybrid | RO params, CopyReturn | No | High | Delegates to filesystem wrappers with panic-style behavior on failures. |
| `test` | IntrinsicWrapper | Assertion-only | No | Low | Panic is intentional for test assertions; not general runtime stdlib path. |
| `time` | IntrinsicWrapper | RO params | No | Medium-High | Formatting/runtime conversions still use panic-prone paths in places. |
| `timeit` | PureSifr | RO params + callable invocation | No | Medium | Ownership-light surface; callable capture semantics mostly external. |
| `tomllib` | Hybrid | RO params | No | High | TOML parse wrapper path currently panics on invalid input. |
| `uuid` | ClassAPI | RO params, value object | No | Low-Med | Value wrapper around generated UUID string. |

Notes:
- `lib/sifr` currently contains **no exported function signatures with explicit `own` or `mut` parameters**.
- This means stdlib API surface primarily exercises default borrow behavior and copy-return design, not explicit ownership transfer ergonomics.
- Many high-risk modules are high due to **contract alignment** (fallibility handling), not due to borrow checker crashes.
