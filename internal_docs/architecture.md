# Sifr Compiler -- Architecture

## Architecture Scope

- Iterator lowering uses one canonical path from the type system through HIR and code generation, with explicit single-pass, multi-pass, and reversible/double-ended capability tracking.
- RNG and cryptographic behavior follow the production semantic rules.
- Ownership-aware collection lowering uses these planner rules:
  - value category: `Place | Temporary`
  - source access mode: `Preserve | Consume`
  - yield mode: `Copy | Clone | Move | Borrow`
  - generic `TypeVar`, `Any`, and move unions use conservative ownership handling.
  - sources are classified as `ValueCategory::Place` or `ValueCategory::Temporary`, then mapped to `SourceAccessMode::Preserve` or `SourceAccessMode::Consume`.
  - `Preserve + Some(Copy)` yields `Copy`; `Preserve + Some(Move)` yields `Clone`; `Preserve + None` yields `Borrow`; and `Consume` or iterator sources yield `Move`.
  - Rust lowering follows this plan and does not use one-off clone heuristics.
- The integer model is defined by `internal_docs/integer_model.md`:
  - Target source semantics: `int` is an exact signed arbitrary-precision value-semantic scalar backed by inline-small `SifrInt`; explicit fixed-width `int8`/`int16`/`int32`/`int64` and `uint8`/`uint16`/`uint32`/`uint64` are for storage, dtypes, binary formats, and FFI.
  - Ordinary fixed-width scalar arithmetic promotes to exact `int`; fixed-width array/tensor/dataframe arithmetic preserves dtype and exposes checked/wrapping/saturating/overflowing policies explicitly.
  - The compiler now uses `SifrInt` as the single emitted representation for source `int` values across storage, calls, aggregates, unions, Rust/Python bridges, and lazy `SifrRange` iteration. Exact floor/modulo, bounded power/shift, integer-to-float conversion, and true division lower through checked runtime operations with typed source errors when static proof is unavailable.
- Network/TLS/URL/HTTP substrate architecture is tracked in [`network_http_architecture.md`](./network_http_architecture.md). The public boundary is `sifr.net`, `sifr.tls`, `sifr.url`, and `sifr.http`; CPython-shaped networking modules remain unsupported diagnostics or rejected surfaces.
- Embedded CPython interop is production-grade complete as a separate lane from Rust-backed packages, raw C ABI interop, and CPython source-parity adaptation. The declaration-first continuation now includes synchronous declarations, opaque lifecycle, synchronous and asynchronous contexts, hermetic package-local bridges, one application-owned asyncio loop, typed coroutine declarations with terminally ordered native cancellation, consuming async close, typed current-thread, foreign-thread, and asyncio callbacks with retained-owner shutdown, the compiler-known affine non-send `python.Buffer[T]` contract with exclusive writable borrowing and exact-once release, five certified affine Arrow C Data Interface resources with structural capsule validation and owned transfer, and affine `python.DlpackTensor[T]`/`python.DlpackStream` resources with versioned one-shot acquisition, exact device/stream matching, no-copy enforcement, and exact-once deleter cleanup. Symbol-selective `sifr python bind` authors checked-in typed declarations from deterministic override/stub/inline/introspection precedence and rejects untyped boundaries; frozen `bind --check` and ordinary package operations validate environment, typing-source, and generated-source fingerprints. General `sifr python certify` authoring and read-only rechecks cover both Arrow and DLPack executable evidence. The analysis host exposes the compiler-owned Python interop plan to the language server, whose completion, hover, navigation, diagnostics, target status, protocol help, cancellation, and cache invalidation reuse compiler/driver queries and package-selected inputs. Editor attribution uses exact declaration file/symbol identity; diagnostics retain declaration spans and package-wide failures have one deterministic document owner. LSP environment status goes through the same Cargo package graph, root trust, final-application deferral, live environment digest, binding/certification, and protocol-certification decisions as package checking. Source edits retain cached raw inspections for unchanged targets, while watcher/external-input invalidation clears environment and target caches; cancellation is checked between target probes. The raw API exposes typed `from_value`, `to_value`, and `kwarg` conversion through the same declaration conversion authority, plus compiler-known checked `Object.get_attr`, `get_item`, `call`, and `call_method` operations on the canonical sealed object. Raw objects use ordinary automatic release, and raw coroutine execution submits to the same application-owned loop rather than creating a per-call loop. The raw Python handle is selected by its canonical stdlib declaration identity rather than the `Object` basename, and every declaration family validates the exact five-field `PythonError` runtime contract before code generation. Package metadata records Python environment selection, inferred and manual import roots, and root-owned Python trust; the package layer resolves one environment and validates canonical CPython probe JSON before runtime embedding consumes it. Ordinary package checking and `sifr python check` consume that same typed environment decision, while `sifr python doctor` reports deterministic manifest patches without mutating source, trust, locks, or environments; build and run remain strict at final application resolution. Runtime lifecycle, GIL/refcount ownership, blocking/offload, callbacks, resources, and zero-copy rules are documented in [`python_interop_architecture.md`](./python_interop_architecture.md), with verification under [`verification/areas/python_interop/`](../verification/areas/python_interop/).
- Rust interop is designed as declaration-level Cargo integration, not a runtime `dlopen` layer or Rust ABI FFI surface. Rust-backed Sifr packages expose normal Sifr declarations annotated with `@rust(...)`, direct Cargo bindings are allowed only for checked bridge-compatible signatures, and package-local/shared bridge crates own adaptation. The source of truth is [`rust_interop_architecture.md`](./rust_interop_architecture.md). <!-- rust-interop-rejected -->
- The sysroot and stdlib toolchain migration is tracked in [`sifr_sysroot_and_stdlib_architecture.md`](./sifr_sysroot_and_stdlib_architecture.md). The final stdlib boundary is checked Sifr source plus trusted sysroot Rust interop: public APIs live in `stdlib/sifr`, sysroot-private declaration source lives in `stdlib/_sifr`, stdlib behavior lives in `crates/sifr_stdlib`, and reusable runtime substrate lives in `crates/sifr_runtime`. The compiler may emit language scaffolding, Rust interop bridge glue, panic wrappers, exact-int conversions, entrypoint machinery, and runtime call glue; it must not implement stdlib behavior through intrinsic dispatch, pasted preambles, or handwritten Rust literals. Existing compiler-native stdlib glue survives only as exhaustive retained-by-design exceptions in `stdlib_retained_compiler_intrinsics.toml`.
- Schema-first SQL architecture is defined in [`sql_architecture.md`](./sql_architecture.md).
  It uses template strings, nominal schema profiles, immutable structural records, and deterministic compiler components. Compiler-owned `sifr.sql` declaration files define only the shared source-language types that the frontend recognizes. They contain no parser, driver, pool, migration engine, or host-tool behavior and add no application dependency. SQL implementations ship as first-party Cargo-backed packages, not as core standard-library implementations. Dialect packages own exact PostgreSQL, MySQL, and SQLite semantics and select qualified parsers and raw drivers. The [MySQL provider](./sql_mysql_provider.md) and [SQLite provider](./sql_sqlite_provider.md) own their compiler, runtime, tool, migration, and editor stacks. `sifr_sql_runtime` owns common verified pool behavior above those drivers. Query templates own bound values. Callers select explicit fetch methods. Runtime execution requires verified schema evidence and strictness. Connections cache statements automatically. Cargo resolves host-only tool packages. These packages expose direct namespaces such as `sifr sql`. The canonical schema remains compile authority. Affine migration plans prove every supported path to that schema.
  `sifr_compiler_component` owns the package-neutral embedded-language boundary. It hosts WebAssembly components with Wasmtime, an empty linker, deterministic fuel and memory limits, a closed versioned protocol, exact package identities, diagnostic registries, and atomic cache transport. `sifr_package` resolves component metadata and artifacts. `sifr_frontend` owns the full semantic cache context. Official and third-party providers use this same path.
  `sifr_sql_contract` owns canonical provider-neutral `SchemaIR`, schema and profile fingerprints, semantic diffs, minimum dependency slices, generated profile-module metadata, runtime verification manifests, and the fingerprinted schema context sent through that component boundary. `sifr_package` parses credential-free named profiles and resolves each provider through the exact locked dependency graph. [`sql_schema_profiles.md`](./sql_schema_profiles.md) records this boundary.
  The same contract crate owns the closed database type, bind, codec, cardinality, effect, and provider-analysis data. `sifr_sql_runtime` owns verified handle states, owned parameters, redacted runtime errors, explicit execution shapes, and panic-safe provider futures. It depends on the package-neutral `sifr_runtime` substrate and no database driver. [`sql_common_contracts.md`](./sql_common_contracts.md) records this boundary.
  The query substrate adds stable reusable-template identities, top-level `RowOf`, typed fragments, static hygienic aliases, explicit cardinality adapters, and audited unsafe syntax. The frontend lowers accepted provider analysis to closed SQL HIR with normal Sifr parameter and structural-row types. The driver supplies one production profile-module registry. Generated schema names use one injective and reversible codec. [`sql_query_fragments.md`](./sql_query_fragments.md) records these contracts.
  The PostgreSQL compiler embeds exact `libpg_query` sources for PostgreSQL 13 through 18. Its owned adapter, catalog, analyzer, diagnostics, and server matrix are defined in [`sql_postgresql_compiler.md`](./sql_postgresql_compiler.md).
  The PostgreSQL runtime uses a verified and bounded raw-driver bridge. It owns typed session reset, explicit fetch contracts, consuming transactions, streams, cancellation cleanup, and live PostgreSQL 13 through 18 qualification. [`sql_postgresql_runtime.md`](./sql_postgresql_runtime.md) records the implemented boundary.
  Incremental SQL analysis uses frontend-owned semantic keys and bounded values, plus an analysis-owned dependency index. The language server uses lossless virtual SQL documents, exact bidirectional maps, schema-aware editor facts, structured fixes, cancellation checkpoints, and named latency budgets. [`sql_incremental_editor.md`](./sql_incremental_editor.md) records this boundary. Provider-neutral schema lifecycle rules and the PostgreSQL live catalog adapter are host-only crates. They write one deterministic, atomic artifact generation and never enter the standard library or application graph. [`sql_schema_tools.md`](./sql_schema_tools.md) records this boundary.
  The migration compiler checks every baseline path and creates one nominal state
  for each step. The runtime records exact recovery evidence and never creates a
  rollback plan. [`sql_migrations.md`](./sql_migrations.md) records this boundary.
  The PostgreSQL host tool reflects provider DDL, validates transaction classes,
  owns the advisory lock and ledger, imports truthful baselines, and qualifies
  recovery against PostgreSQL 13 through 18.
  [`sql_postgresql_migrations.md`](./sql_postgresql_migrations.md) records this
  provider boundary.
  Host tools use one Cargo-locked workspace member that stays outside every application dependency closure. `sifr_package` owns exact entry-point identity, the committed `sifr-tools.lock.json`, capability grants, lock fingerprints, and host-target build plans. The CLI builds then directly executes the binary in a fail-closed native sandbox, exposes direct namespaces, and validates the common SQL test connection manifest. [`sql_host_tools.md`](./sql_host_tools.md) records this boundary.
- The core type system owns immutable structural records independently of SQL.
  Field names and canonical field types define identity. Declaration order affects diagnostics only.
  Shared-borrow parameters use generated record-view traits for width subtyping. These views do not clone fields and cannot escape the call.
  Owned `.project[Target]()` conversions consume the source and move selected fields.
  Code generation maps each `ShapeIdentity` to one exact Rust layout instantiation. Layout families use field names and take the exact field types as Rust parameters. This rule lets generic and concrete records use the same representation.
  Rust derives and bounded implementations expose equality, ordering, hashing, display, and cloning only when all field types support them.
  IPC schema extraction provides the explicit serialization hook. The general Rust bridge rejects these compiler-owned layouts.
- SQL dependency qualification uses `sifr_sql_dependency_lock` as a lockfile root.
  This package has no runtime API and never enters an application artifact. The
  SQL verification area owns its dependency baseline, artifact topology,
  ownership map, capability matrix, and verification inventory.

- The core compiler preserves PEP 750 template strings as typed HIR. The
  frontend owns their U+FFFC virtual-document projection and bidirectional range
  maps. Code generation emits a private, move-only carrier for inert runtime
  values. Compiler components consume the typed structure before code generation.

- Python ecosystem integration uses declarations or hermetic typed bridges as the ordinary example path. Live service clients execute from built Sifr binaries, and the schema-2 capability ledger binds current-run compiled callback, async HTTP, buffer, Arrow, and DLPack evidence to exact source, marker, trust, certification, resource-zero, and report-digest provenance.

## Vision

Sifr is a compiled programming language that uses Python syntax with enforced static typing. It compiles Python-like source code to Rust source code, which is then compiled by `rustc` into native binaries. Assignment uses move semantics (like Rust), while function parameters are borrow-by-default with opt-in `mut` (mutable borrow) and `own` (ownership transfer). Types are strict with an opt-in `Any` escape hatch (like TypeScript's strict mode).

The type system draws heavily from TypeScript's design: union and intersection types, literal types, and full control-flow-based type narrowing are first-class citizens. Unlike TypeScript (which erases types at runtime), sifr uses types to generate efficient Rust code -- union types become Rust enums, narrowing becomes `match` expressions, and literal types enable compile-time value checking.

The end goal is a language capable of building web applications and general-purpose programs -- anywhere Python is used today, but with native performance and compile-time safety.

## Safety Philosophy

Sifr's core guarantee: **if it compiles, it works.** The language is designed so that a successfully compiled program will not crash at runtime under normal conditions. This guarantee is **fully enforced from safe indexing onward** -- earlier bootstrap work uses panic-based indexing as a bootstrap mechanism until `Option`/`Result` types are available. The principles are:

- **No panics in user code.** Sifr programs never panic during normal execution. Every operation that can fail returns `Result[T, E]` or `Option[T]`, forcing the caller to handle the failure case at compile time.
- **Mandatory error handling.** `Result` and `Option` values are `#[must_use]`. Ignoring a `Result` returned by a function is a **compile-time error**. The programmer must either handle the error (`try`/`except`) or explicitly discard it (`_ = ...`). There is no user-facing `?` operator -- the compiler handles error propagation internally via `try`/`except` auto-unwrap (see rules #3).
- **All fallible operations return `Result` or `Option`.** This includes:
  - Indexing (`x[i]` returns `Option[T]`)
  - Division (`a / b` returns `Result[T, DivisionError]` when the divisor is not provably non-zero)
  - Type conversions (`int(s)` where `s: str` returns `Result[int, ParseError]`)
  - File I/O, network, and all stdlib operations that can fail
  - Fixed-width integer narrowing and representation-preserving fixed-width arithmetic (`int` itself is exact; overflow policy is explicit at fixed-width/storage boundaries)
- `**assert` is the only panic.** The `assert` statement is a programmer invariant check -- it generates `panic!()` and is intentionally unrecoverable. It exists to catch programmer bugs (violated assumptions), not to handle runtime errors. It is the one escape hatch from the no-panic guarantee.
- **Panic = unrecoverable system failure.** Beyond `assert`, panics only occur from truly unrecoverable situations: stack overflow, double panic, or hardware failure. These are never part of normal control flow.
- **Generated runtime panic-shape gate is enforced.** diagnostic architecture requires an emitted-code sweep across pass fixtures to ensure generated Rust contains no `.unwrap(` or `.expect(` in user-facing runtime paths.
- **Exceptions are not errors.** Sifr does not use Python's exception model. There is no stack unwinding, no exception propagation. The `try`/`except` syntax is reinterpreted as pattern matching on `Result` values with **compiler-enforced exhaustiveness checking** on error types. `raise` is syntax sugar for returning `Err(...)`. `return value` in a `Result`-returning function auto-wraps in `Ok(...)`.

This philosophy means that a Sifr programmer who handles all `Result` and `Option` values (which the compiler enforces) can be confident their program will not crash at runtime.

## CPython Reference

Sifr uses the CPython source code (`/Users/yaseralnajjar/work/sifr/cpython`) as the **authoritative reference** for Python behavior. The goal is to match CPython's semantics for built-in functions, data structure methods, and standard library behavior -- but always through Sifr's safety lens.

### Reference Directory Mapping


| Sifr feature area                                                                   | CPython reference location                                              | Notes                                                                        |
| ----------------------------------------------------------------------------------- | ----------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Built-in functions (`len`, `abs`, `min`, `max`, `sorted`, `zip`, `enumerate`, etc.) | `Python/bltinmodule.c`                                                  | Match behavior, but return `Result`/`Option` where CPython would raise/panic |
| `list` methods (`.append`, `.pop`, `.sort`, `.index`, etc.)                         | `Objects/listobject.c`                                                  | Match semantics, safe indexing returns `Option`                              |
| `dict` methods (`.keys`, `.values`, `.get`, `.pop`, etc.)                           | `Objects/dictobject.c`                                                  | Match semantics, safe lookup returns `Option`                                |
| `str` methods (`.replace`, `.find`, `.split`, `.join`, etc.)                        | `Objects/unicodeobject.c`                                               | Match behavior, UTF-8 safe, character-based indexing                         |
| `tuple`                                                                             | `Objects/tupleobject.c`                                                 | Immutable, compile-time enforced                                             |
| `set` / `frozenset`                                                                 | `Objects/setobject.c`                                                   | Match operations, `frozenset` immutability enforced at compile time          |
| `int` / `float` / `bool`                                                            | `Objects/longobject.c`, `Objects/floatobject.c`, `Objects/boolobject.c` | Checked arithmetic, safe conversions                                         |
| `bytes` / `bytearray`                                                               | `Objects/bytesobject.c`, `Objects/bytearrayobject.c`                    | Match API, safe encode/decode                                                |
| `range` / `slice`                                                                   | `Objects/rangeobject.c`, `Objects/sliceobject.c`                        | Match iteration and slicing behavior                                         |
| Iterators / generators                                                              | `Objects/iterobject.c`, `Objects/genobject.c`                           | Match protocol, `Option`-based `__next__`                                    |
| Standard library modules                                                            | `Lib/<module>.py`, `Modules/<module>module.c`                           | Match API surface, wrap Rust crates                                          |
| Test suite (behavioral reference)                                                   | `Lib/test/test_<module>.py`                                             | Use as specification for expected behavior                                   |


### Bytes Representation Note

- The first-class `bytes` surface is now a distinct compiler type (`Type::Bytes`) across type checking, lowering, and codegen.
- Current Rust codegen representation is raw-byte storage (`Vec<u8>`) for typed bytes-native paths.
- Target integer-model amendment: indexing and iteration yield `uint8`; callers use `int(b)` when they want exact scalar integer arithmetic.
- Explicit construction boundaries (`bytes.from_ints`, `bytes.from_hex`, UTF-8 decode) remain responsible for runtime validation and typed error propagation.

### Safety Adaptation Rules

When adapting CPython behavior to Sifr, apply these rules:

1. **Where CPython raises an exception, Sifr returns `Result[T, E]`.** Example: `int("abc")` raises `ValueError` in CPython; in Sifr it returns `Result[int, ParseError]`.
2. **Where CPython raises `IndexError`, Sifr returns `Option[T]`.** Example: `list[99]` raises `IndexError` in CPython; in Sifr it returns `None`.
3. **Where CPython raises `KeyError`, Sifr returns `Option[V]`.** Example: `dict["missing"]` raises `KeyError` in CPython; in Sifr it returns `None`.
4. **Where CPython uses arbitrary-precision integers, Sifr uses exact `int`.** Source-level `int` is signed, arbitrary precision, and value-semantic. Fixed-width integer families are explicit for storage, binary protocols, dataframes/tensors, and FFI. Narrowing from `int` to a fixed-width type is explicit and fallible unless the compiler proves a constant fits.
5. **Where CPython allows mutation on immutable types at runtime, Sifr rejects at compile time.** Example: `tuple[0] = 1` raises `TypeError` at runtime in CPython; in Sifr it is a compile-time error.
6. **Where CPython behavior is undefined or platform-dependent, Sifr defines explicit behavior.** Document any deviations from CPython in the codebase work notes.

### Safety Testing Rules

Every change that implements built-in functions, data structure methods, or stdlib modules must include a **safety test layer** that verifies:

1. **Behavioral parity with CPython:** for each function/method, write tests that match CPython's expected output for valid inputs. Use `Lib/test/test_<module>.py` as the specification.
2. **Safe error handling:** for each CPython operation that raises an exception, verify that Sifr returns the correct `Result::Err` or `Option::None` instead.
3. **No panics on any input:** fuzz or property-test each function/method to ensure it never panics, regardless of input. The only acceptable panic is from `assert` statements.
4. **Compile-time rejection of unsafe patterns:** verify that operations CPython rejects at runtime (e.g., mutating a tuple, unhashable dict key) are caught at compile time in Sifr.

This safety test layer is tracked in each codebase implementation record as: **"CPython parity tests pass with safe error handling (no panics, Result/Option where CPython raises)"**.

### Python source parity Governance Artifact

For the Python source parity governance track, the canonical parity governance source is:

- `verification/areas/stdlib_parity/reports/stdlib_parity_governance_inventory.md`

It is the single consolidated inventory for builtin parity status, core object-model parity status, shipped-module terminal classification, CPython adopt/adapt/waive traceability links, and waiver-index governance rules.

## Python Divergences

Sifr intentionally diverges from CPython in several areas to achieve compile-time safety. This table documents each divergence, its rationale, and the implementation record where it is introduced.

### Standard Library Namespace Rules

Sifr is Python-syntax and CPython-behavior-informed, but it is not Python-source-compatible. The standard library import rules is explicit:

| Import root | Owner | Resolution |
| --- | --- | --- |
| `_sifr.*` | Sysroot-private stdlib declaration source | Naming convention for modules loaded with `SysrootPrivateDeclaration` origin. Importability is source-origin based: only `SysrootPublicStdlib` sources may import private declarations. |
| `sifr.*` | Sifr standard library | Resolved from the active sysroot public stdlib source inventory; never package-manager resolution. |
| top-level | User code and third-party packages | Workspace/package resolution. |

The stdlib manifest loader classifies sysroot sources as either
`SysrootPublicStdlib` or `SysrootPrivateDeclaration`; the driver passes that
origin into HIR lowering. Lowering derives private declaration importability
from the source origin, not from whether an import path starts with `_sifr.*`.
The `_sifr.*` prefix remains the on-disk/private namespace convention and the
diagnostic target for rejected imports, but it is not a trust signal by itself.

Bare CPython stdlib roots such as `math`, `json`, `os`, `heapq`, and `collections` are not aliases for `sifr.*`. A real top-level user or package module named `math`, `json`, or similar wins normal resolution. If no real top-level module resolves and the written import root matches an embedded Sifr stdlib module tail, the compiler emits `SIFR-IMPORT-0008` with a suggestion to use `sifr.*`.

Examples:

```python
from sifr.math import sqrt
from sifr.collections import deque
```

Unsupported bare forms:

```python
from math import sqrt
import collections
```


| Python Behavior | Sifr Behavior | Rationale |
| --- | --- | --- |
| Exceptions for error handling (`try`/`except`/`raise`) | `Result[T, E]` and `Option[T]` with mandatory handling; `try`/`except` reinterpreted as pattern matching on `Result` with compiler-enforced exhaustiveness checking on error types; no `?` operator in user code; `raise` maps to `Err(...)`, `return` auto-wraps in `Ok(...)` | Compile-time error handling eliminates unhandled exceptions at runtime; exhaustiveness checking ensures all error types are covered |
| `IndexError` on out-of-bounds access | `x[i]` returns `Option[T]` (no panic) | Safe indexing: no runtime crashes from bad indices |
| `KeyError` on missing dict key | `d[key]` returns `Option[V]` (no panic) | Safe access: caller must handle missing keys |
| Arbitrary-precision integers | `int` is exact and arbitrary precision; fixed-width integer families are explicit for storage, dtypes, binary formats, and FFI | Python-simple default arithmetic without overflow; widths are visible only where representation matters |
| Import-time side effects (`__init__.py` runs code) | `__init__.sifr` defines exported API only; no side effects on import | Deterministic, safe module loading |
| Mutable default arguments (`def f(x=[])`) | Default values are evaluated fresh each call (no shared mutable state) | Eliminates a common Python footgun |
| Parameter reassignment is implicit (`def f(x): x = ...`) | Rebinding or mutating a parameter requires explicit `mut` / `own mut`; bare parameters are immutable by default | Keeps ownership and mutability explicit; avoids hidden local mutation that conflicts with borrow-by-default semantics |
| Augmented assignment on immutables | Augmented assignment (`+=`) on immutable types (tuple, frozenset) is a compile-time error | Compile-time enforcement of immutability |
| `global` / `nonlocal` keywords | Not supported; use closures or pass values explicitly | Encourages explicit data flow; avoids hidden state mutation |
| Metaclasses (`type()`, `__metaclass__`) | Not supported; use decorators and protocols instead | Simplification: metaclasses add complexity with limited benefit in a compiled language |
| `__slots__` | Not needed; all classes compile to Rust structs | Rust structs are fixed-layout by default |
| Runtime duck typing | Structural typing via protocols (compile-time checked) | Same flexibility as duck typing but errors are caught at compile time |
| `finally` for cleanup | Supported for error handling; prefer `with` statements, which map to Rust `Drop` | Scope-based cleanup is more idiomatic and less error-prone |
| `del x` (name unbinding) | Not supported; variables are dropped at scope end (Rust RAII) | Explicit lifetime management is handled by the compiler; manual unbinding adds complexity |
| `getattr`/`setattr`/`hasattr`/`delattr` (reflection) | Not supported; use protocols for dynamic dispatch and pattern matching for type inspection | Compile-time type safety; runtime reflection undermines static guarantees |
| `type()` for runtime type creation | Not supported; use class definitions | All types must be known at compile time for Rust codegen |
| Positional-only parameters (`def f(x, /, y)`) | Deferred to metaprogramming; not commonly needed in user code | Low priority; most APIs use keyword arguments |
| Math domain errors (`sqrt(-1)`, `log(0)`, etc.) raise `ValueError` | Sifr follows Rust's IEEE 754 behavior: returns `NaN` / `inf` silently | Consistent with Rust semantics; avoids panic; user can check with `isnan()`/`isinf()` |
| `list.remove(x)` raises `ValueError` if x not in list | `list.remove(x)` is a no-op if x is not found | Safe by default; callers do not need to pre-check membership |
| `list.index(x)` raises `ValueError` if x not in list | `list.index(x)` returns `int \| None`; `None` if not found | Safe by default; callers handle absence via pattern matching |
| `min([])`/`max([])` raise `ValueError` on empty | `min(list)`/`max(list)` return `T \| None`; `None` on empty list | Safe by default; absence is a value, not an error |
| `set.pop()` raises `KeyError` on empty set | `set.pop()` returns `T \| None`; `None` on empty set | Consistent with safe collection semantics |
| Error subclass fields | Typed fields such as `message`, `line`, `column`, and `detail` | Structured error data avoids string parsing |
| `@dataclass` for auto-generated methods | Constructor and common methods are generated from typed field declarations; `@dataclass` remains reserved for advanced options | Eliminates boilerplate while keeping field shapes explicit |
| `match`/`case` with soft keywords | `match`/`case` are hard keywords | Avoids parser ambiguity; `match` is already reserved as a Rust keyword |
| `enum.Enum` class-based syntax | Dedicated `enum Color: RED, GREEN, BLUE` syntax, no class inheritance | Cleaner syntax; direct mapping to Rust enums; no metaclass machinery |
| No enum associated data | Union types plus classes model data-carrying variants; enums are simple constants only | One obvious model: classes and unions for data, enums for constants |
| Dict insertion order guaranteed (Python 3.7+) | Dict order is unspecified (`HashMap`); use `collections.OrderedDict` if order matters | `HashMap` is faster than ordered alternatives; `IndexMap` may be considered later |


**Migration note:** code that relies heavily on exception propagation, import-time side effects, arbitrary-precision integers, or runtime reflection will require redesign when porting to Sifr. The compiler provides clear diagnostics for each divergence.

### Pre-v1 canonical API contracts

`verification/areas/developer_tooling/check_no_pre_v1_compatibility.py` is the
executable authority for removed Sifr-owned syntax, names, schemas, wrappers,
diagnostics, and layouts. The guard scans source, stdlib, verification,
workflows, documentation, demos, fixtures, scripts, and editor integrations.
It explicitly excludes generated, vendored, and historical files.

`verification/compatibility/retained_compatibility_contracts.json` is the sole
retained-contract registry. Every row identifies an external protocol,
external format, dependency contract, or current product behavior. Public
stdlib modules expose only documented names. Imports from `_sifr.*` use private
aliases when they are implementation details.

The canonical numeric spellings are `sifr.math.fabs` and `sifr.math.pow`;
`abs_val`, `pow_val`, `min_val`, `max_val`, and `round_val` are private
intrinsic spellings, not public aliases. The canonical module-state random
operations are `randint`, `random`, `uniform`, and `choice`. Runtime-information
modules expose their wrapper names (`system`, `time`, `argv`, `getenv`, and the
other inventory entries) rather than intrinsic-shaped imports. `UTC` is the
single public zero-offset timezone constructor.

The canonical text/data spellings are the unprefixed public operations:
`re.search`/`sub`/`findall`/`split`, `json.loads`, the `json.dumps*` family,
`tomllib.loads`, the `base64.b64*` family, `fnmatch.filter`, `html.escape` and
`unescape`, `calendar.isleap`/`weekday`/`monthrange`, and `url.parse`/`build`.
Typed text and bytes Base64 operations remain distinct because their input and
output types differ. `Pattern.is_match` is the approved spelling for the Rust
and Sifr keyword conflict.

First-class `bytes` owns binary storage and construction. Hashing is
bytes-native; text callers encode explicitly. `HashObject.digest()` and
`hexdigest()` remain distinct because one returns bytes and the other returns
hexadecimal text. First-class generic `set[T]` replaces list-backed set
helpers. `heapq` and `bisect` use explicit mutating operations only.

Receiver syntax has no compatibility interpretation: `self` is a shared
borrow, `mut self` is a mutable borrow, `own self` is an immutable owned
receiver, and `own mut self` is a mutable owned receiver. Method-body analysis
does not change the declared convention.

Packages use one optional `[source].root`, with `src` as the only default.
Source exports come from `__init__.sifr`; application targets come from
`main.sifr` or `bin/*.sifr`. Compiler services receive a `SourceProvider` or a
captured `WorkspaceSession`, and package import resolution preserves every
structured result variant.

Managed standalone installs use `<sysroot>/bin/sifr` with
`<sysroot>/install.json`. Verification profiles use schema version 2,
structured area results, and assertion-based runtime expectations. Codegen
maps supported source types to structured `RustType` nodes and reports an
error before emission for unsupported types.

The retained-contract registry locks the external and current product behavior
excluded from broad compatibility scans. It includes DLPack capsule
requirements, the LSP UTF-16 default, release bootstrap `legacy-index`
evidence, and `RuleStatus::Deprecated` lint lifecycle metadata.

### API Naming Divergences

Several stdlib functions intentionally diverge from CPython names due to Rust keyword conflicts or Sifr type-system constraints. This table is the authoritative reference — do not "fix" these names or introduce inconsistent workarounds.

| sifr name | CPython name | reason |
|---|---|---|
| `sifr.shutil.move_file` | `shutil.move` | `move` is a Rust keyword |
| `sifr.math.fabs` | `math.fabs` | Canonical public float-absolute operation; `_sifr.math.abs_val` is private implementation detail. |
| `sifr.math.pow` | `math.pow` | Canonical public float-power operation; `_sifr.math.pow_val` is private implementation detail. |
| `sifr.itertools.repeat` | `itertools.repeat` | CPython-compatible name; `repeat_val` was the old non-CPython name (removed) |
| `sifr.itertools.count` | `itertools.count` | CPython-compatible lazy counter iterator |
| `sifr.itertools.count_from` | — (bounded helper over `count`) | Sifr extension; finite convenience helper equivalent to `islice(count(start, step), n)` |
| `sifr.os.remove_file` | `os.remove` | `remove` is used as a method name on collections; `remove_file` avoids ambiguity |
| `sifr.random.shuffle` | `random.shuffle` | CPython-compatible name; returns a new shuffled list (Sifr is immutable-by-default) instead of mutating in place |
| `sifr.operator.mod_val` | `operator.mod` | `mod` is a Rust keyword |
| `sifr.re.Pattern.is_match` | `re.Pattern.match` | `match` is a Rust keyword (also a Sifr keyword from pattern-matching work) |
| `sifr.itertools.take` | — (no CPython equivalent) | Sifr extension; returns first N elements from an `Iterable[T]`. Kept for ergonomics. |
| `sifr.itertools.flatten` | `itertools.chain.from_iterable` | Sifr extension; flattens `Iterable[Iterable[T]]`. Simpler API than CPython's `chain.from_iterable`. |

Iterator-taking `sifr.itertools` APIs own their `Iterator[T]` inputs. Callers
that start with an iterable use an explicit `iter(...)` conversion, making
single-pass consumption visible at the call site. `islice(data, stop)` treats
the second argument as the stop bound, while `islice(data, start, None)` is
unbounded and an explicit `None` step has the canonical unit-step meaning.

**Removed type-specific duplicates (type-system architecture — stdlib generic rewrite):** `chain_str`, `chain_float`, `accumulate_float`, `accumulate_str`, `counter_add`, `counter_sub`, and other monomorphic variants have been deleted. All stdlib functions are now generic — e.g., `chain[T]`, `accumulate[T: Addable]`, `Counter[T: Hashable]`, `deque[T]`, `heapq` functions with `[T: Comparable]` bounds, `reduce[T, U]`, `shuffle[T]`, `sample[T]`.

## Compiler Pipeline

```mermaid
flowchart LR
    Source["Source (.sifr)"] --> Lexer
    Lexer --> Parser
    Parser --> AST["Sifr AST"]
    AST --> Binder["Binder / Name Resolution"]
    Binder --> Checker["Type Checker"]
    Checker --> HIR["Sifr HIR"]
    HIR --> RustCodegen["Rust Codegen"]
    RustCodegen --> RustIR["Rust IR"]
    RustIR --> Passes["IR Passes\n(imports, DCE, clone opt)"]
    Passes --> Renderer["Pretty-Printer"]
    Renderer --> RustSource[".rs files"]
    RustSource --> Rustc["rustc"]
    Rustc --> Binary["Native Binary"]
```

## Crate Structure (Rust Workspace)

**Hybrid dependency approach:** Infrastructure crates, parser, AST crates, and
the formatter are referenced from the Ruff fork submodule, currently based on
Ruff 0.16.4. Parser and AST crates include Sifr-specific syntax extensions and
are imported through Cargo aliases as `sifr_python_ast` and
`sifr_python_parser`. The Ruff fork formatter is Sifr-aware for parameter
conventions, Sifr type syntax, generics, match/case, ownership-aware
collections, formatter pragmas, and Sifr-tagged docstring snippets. The root
workspace pins Sifr's direct and generated-runtime support crates to the latest
stable releases independently from the excluded Ruff fork, which keeps its own
sub-workspace dependency pins. Sifr is latest-stable-only: the root
`rust-toolchain.toml` and CI select Rust 1.98.0 exactly, while workspace
manifests declare Rust 1.98 as the required compiler line.

```
sifr/
  Cargo.toml                (workspace root)
  crates/
    sifr_source/            (canonical source text, line-map, file metadata, and UTF-8/UTF-16/UTF-32 position conversion primitives)
    sifr_frontend/          (canonical parse/lower/type-check/diagnostics query facade shared by CLI and tooling)
    sifr_diagnostics/       (canonical diagnostic codes, source-map spans, model, render schema, and sink)
    sifr_ir/                (High-level IR data rules, public lowered views, CFG/flow graph data)
    sifr_lowering/          (AST-to-IR lowering, name/type/ownership/async analysis, lowering diagnostics)
    sifr_stdlib_manifest/   (compiler-host stdlib source inventory, private declaration inventory, feature planning, and sysroot validation)
    sifr_stdlib_imports/    (legacy and bare stdlib import suggestion policy over manifest inventory)
    sifr_ipc/               (shared IPC protocol/frame/schema/request-tracking crate)
    sifr_type_system/       (type definitions, inference, checking, subtyping)
    sifr_codegen/           (Rust source code generation from HIR via structured Rust IR)
    sifr_driver/            (CLI/project orchestration, split into diagnostics.rs + stdlib/ frontend/ project/ build/ test_runner/)
    sifr_format/            (Sifr-facing Ruff-backed formatter API, config conversion, diagnostics, and text edits)
    sifr_analysis/          (editor query host; routes formatting through sifr_format)
    sifr_lsp/               (LSP protocol shell; serves document/range formatting over sifr_analysis)
    sifr/                   (CLI binary: sifr build, sifr check, sifr run, sifr fmt)

  # Path dependencies from the Ruff fork submodule:
  #   ruff_text_size          -- text span/range utilities
  #   ruff_source_file        -- source file representation, line indexing
  #   ruff_python_trivia      -- whitespace/comment handling
  #   ruff_python_literal     -- literal parsing (string escapes, number formats)
  #   ruff_python_formatter   -- Sifr-aware Ruff formatter rules and range formatting

  third_party/
    ruff/                    (sifr-lang/ruff submodule, branch sifr/0.16.4-maintenance)
      crates/
        ruff_python_ast/      (imported as Cargo dependency alias sifr_python_ast)
        ruff_python_parser/   (imported as Cargo dependency alias sifr_python_parser)
        ruff_python_formatter/ (Sifr-aware formatter wrapper consumed by sifr_format)
```

New crates added as compiler and runtime needs grow:

- core_stdlib/ext_collections: `sifr_std` (standard library wrappers, extended collections)
- semantic diagnostic taxonomy: `sifr_diagnostics` (shared diagnostic model and schema, introduced before migrating existing emission paths)
- `sifr_source`: bottom-of-graph source text, line maps, source-file metadata, and editor position conversion authority.
- Architecture-transfer guardrails: `internal_docs/typescript_go_architecture_transfer_guardrails.md` records the direct-read inventory, current LSP single-file rebuild caveat, aggregate LSP budget caveat, and guardrail automation for behavior migration.
- `sifr_frontend::SourceProvider`: typed semantic filesystem boundary. `internal_docs/typescript_go_architecture_transfer_source_provider.md` records how `DiskSourceProvider`, `OverlaySourceProvider`, and `TrackingSourceProvider` cover disk reads, unsaved editor buffers, read/probe/canonicalization dependency records, failed lookup records, package import ambiguity, and overlay lifecycle requirements.
- `sifr_frontend::WorkspaceSession`: serialized compiler-service workspace state and inspectable `WorkspaceSnapshot` creation. `internal_docs/typescript_go_architecture_transfer_workspace_session.md` records the session/snapshot state.
- `sifr_analysis::AnalysisSnapshot`: analysis-facing handle to a captured `WorkspaceSnapshot`. LSP request handling routes editor queries through snapshot methods while staying serialized. `internal_docs/typescript_go_architecture_transfer_analysis_snapshot.md` records query metadata snapshot ids, stale-result identity, and the conservative dirty-scope report slot.
- `sifr_lsp::Session`: persistent analysis handles that wrap `WorkspaceSession`; `DocumentStore` keeps protocol document state. `internal_docs/typescript_go_architecture_transfer_lsp_persistent_session.md` records overlay-fed open/change/save handling and document-version stale-result checks.
- LSP event compaction and dirty-scope reporting: `sifr_lsp` compacts batched editor edits and watcher notifications before updating analysis, while `sifr_frontend::WorkspaceSession` records dirty-scope reports with explicit reasons, merge priority, and conservative degradation. `internal_docs/typescript_go_architecture_transfer_event_compaction_dirty_scope.md` records the precise invalidation vocabulary.
- Dependency-sensitive invalidation: `sifr_frontend::FrontendContext` records import/export/module signatures and reverse dependency edges so private body edits invalidate only the changed module when public/import signatures are unchanged, while public API or import graph changes invalidate reverse dependents. `internal_docs/typescript_go_architecture_transfer_module_signatures_dependency_invalidation.md` records the invalidation policy.
- `sifr_lowering::flow_graph`: first-class data-flow nodes, edges, and effects for definitions, assignments, conditions, branches, loops, calls, mutations, moves, borrows, joins, unreachable statements, and exits. `LoweringResult` carries a snapshot-scoped `FlowGraph`, and `FlowFacts` exposes graph fingerprints and debug traces. `internal_docs/typescript_go_architecture_transfer_first_class_flow_graph.md` records graph-backed narrowing and ownership-effect behavior.
- `sifr_frontend::cache_keys`: deterministic `CompilerFingerprint`, `CacheKeyFingerprint`, common workspace/package/query-policy context fingerprints, and typed cache-key identities for parse, source-map, HIR/lowering, diagnostics, lint, format, package graph, symbol bucket, and flow graph cache families. `internal_docs/typescript_go_architecture_transfer_fingerprints_cache_keys.md` records cache-key identity requirements.
- Snapshot reuse: `sifr_frontend` adds ref-counted, cache-key identity-keyed reuse storage for parse trees, source-map file views, lowered HIR, module diagnostics, and module symbol indexes. `WorkspaceSnapshot` stores immutable snapshot payloads behind `Arc`, and `FrontendContext::can_replace_module_in_project` gates safe one-module replacement on unchanged import/export signatures. `internal_docs/typescript_go_architecture_transfer_snapshot_reuse.md` records reuse requirements.
- `sifr_lsp::RequestQueue`: latency-sensitive, formatting, workspace, and background requests route through explicit priority lanes with bounded fairness, while diagnostic jobs preserve captured document versions. `internal_docs/typescript_go_architecture_transfer_lsp_scheduler.md` records scheduler behavior.
- LSP latency budgets: protocol-level LSP performance coverage is split into per-request `perf.lsp.*` budget ids, leaving `perf.lsp.request_families` as aggregate smoke only. `internal_docs/typescript_go_architecture_transfer_lsp_latency_budgets.md` records the request-family budget taxonomy and frontend query architecture relationship.
- LSP cancellation, progress, and watchdog: LSP cancellation state, delayed workspace-diagnostic progress, and parent-process watchdog behavior are explicit server behaviors. `internal_docs/typescript_go_architecture_transfer_lsp_cancellation_progress_watchdog.md` records cancellation/progress/watchdog semantics.
- Bucketed symbol index: `sifr_analysis::SymbolIndex` exposes workspace/package/stdlib bucket readiness states, import-entry counts, and dirty-module refresh for editor symbol/import queries. Package and stdlib buckets are explicit unavailable states until frontend graph views carry those identities. `sifr_analysis::ApprovedWorkerLane` records compiler operations eligible for future parallel execution while `SingleOwnerCompilerPhase` keeps type identity, ownership mutation, package graph mutation, and codegen state serialized. `internal_docs/typescript_go_architecture_transfer_bucketed_indexes.md` records bucket and worker-lane behavior.
- Project residency: `sifr_frontend::WorkspaceSession` snapshots include `WorkspaceResidencySnapshot` for project residency, config registry entries, deduped watch registrations, and verified non-authoritative `.sifrbuildinfo` metadata. Build info is retained only after current compiler, package/config, and source hashes match. `internal_docs/typescript_go_architecture_transfer_project_residency.md` records residency behavior.
- Trace and status: `WorkspaceSnapshot` carries bounded `WorkspaceDebugSnapshot` trace/status output with normalized `WorkspaceTracePhase` events for source update, parse, lower, type check, ownership, flow, cache, invalidation, stale rejection, and LSP timing. LSP scheduler/cancellation/stale/timing events are exposed through `sifr/debugTrace`; `AnalysisHost::debug_snapshot` adds side-effect-free index readiness, and `sifr trace` prints representative CLI snapshots. `internal_docs/typescript_go_architecture_transfer_trace_status.md` records trace/status behavior.
- Editor corpus and snapshot handles: marker-based multi-file editor corpus fixtures cover the analysis query families and stale-snapshot behavior, while internal-only `SnapshotHandleKind` handles for symbols, types, signatures, diagnostics, and source spans reject wrong-snapshot resolution. Runtime package fixtures also lock `SIFR-IMPORT-0005` ambiguity and fatal `SIFR-PACKAGE-*` non-duplication. `internal_docs/typescript_go_architecture_transfer_editor_corpus_snapshot_handles.md` records corpus/handle behavior.
- ffi: FFI codegen extensions in `sifr_codegen`
- frontend query architecture shared analysis/query architecture: `sifr_frontend` (canonical frontend API and query/database ownership)
- developer-tooling work: `sifr_lsp` (language server), `sifr_format` (formatter), `sifr_lint` (linter)
- ecosystem: `sifr_registry` (package registry client)

## Formatter Architecture

Generated Rust field cleanup runs before identifier and layout cleanup. One
registry resolves module-qualified nominal declarations, imports and re-exports,
and typed field receivers across the complete generated project. Field spelling
and collisions belong to that nominal owner; shorthand pattern/value bindings
retain their separate local identities. Binary bridge sources are generated and
registered alongside main and support modules before this pass; their finalized
sources also participate in the artifact cache identity. Binary and test-project
materialization use the same registry and do not repeat field canonicalization
while formatting individual files. External fields are not renamed, and an unresolved
generated-field receiver is a code-generation diagnostic.

The production Sifr formatter is Ruff-backed and in-process. `.sifr` source
flows through `sifr_syntax` into the Sifr Ruff fork parser, AST, comments,
trivia, and formatter rules, then through the Sifr-owned `sifr_format` wrapper.
`sifr_format` owns Sifr-facing options, config conversion, deterministic
diagnostics, and text edits; it does not lower to HIR, type-check, or run
ownership analysis. Standalone file and config reads use short-lived
`sifr_frontend::SourceProvider` instances so formatting does not create a
separate source text or line-map authority.

The single formatter core is shared by:

- `sifr fmt` for CLI write, check, diff, stdin, range, path-selection, and cache behavior
- `sifr_analysis` document and range formatting queries
- `sifr_lsp` `textDocument/formatting` and `textDocument/rangeFormatting`
- checked-in editor integrations through `sifr lsp --stdio`

Formatter validation is part of local validation. `verification/areas/developer_tooling/check_formatter_ast_coverage.py`
fails when a Sifr parser or AST extension lacks both Ruff fork formatter fixture
coverage and Sifr wrapper corpus coverage. Formatter performance budgets cover a
large-file check and a representative project check.

## Driver Build Model

`sifr_driver` uses one rooted-entrypoint compilation model for native binary builds.

- `RootedEntrypointPlan` is the canonical driver abstraction for build planning.
- `RootedEntrypointShape::SingleFile` models the one-module case.
- `RootedEntrypointShape::Project` models the reachable user import closure.
  The nearest ancestor `sifr.toml` selects this shape for every valid entry
  filename below the manifest directory. Outside a workspace, every explicit
  file uses `RootedEntrypointShape::SingleFile`; mode selection never parses
  imports or probes sibling modules. Workspace `[source].root` configures
  user-module lookup after this selection; it does not select the shape.
- Native `sifr.toml` workspace discovery lives in `sifr_driver::workspace`. `[source].root` defines one workspace user-module search root and defaults to `src`; malformed workspace config is a hard build diagnostic rather than a single-file fallback.
- User module resolution keeps embedded `sifr.*` / `_sifr.*` stdlib registry precedence separate from filesystem lookup. It then searches the entry parent first and the configured workspace source root second. Dotted module IDs such as `helpers.nodes` map to `helpers/nodes.sifr`.
- Generated Rust preserves canonical dotted module IDs through HIR/codegen and materializes them as nested Rust files, for example `helpers.nodes` -> `src/helpers/nodes.rs` plus `src/helpers/mod.rs`.
- Code generation records one structured runtime-and-stdlib support demand per
  HIR module. Single-file emission renders that demand inline once. Project and
  test-project emission defer every module demand, merge the demands, and render
  one private crate-root `__sifr_generated_support` owner whose items are visible
  only inside the generated crate. Canonical item fingerprints reject conflicting
  support bodies instead of silently selecting one, and Rust interop bridge types
  use the same exact-deduplication/fail-closed ownership rule.
  Compiler-owned `tokio::task_local!` statics share one declaration parser for
  visibility, symbol discovery, and dependency pruning. Each declaration is a
  separate demand owner; project relocation grants crate visibility while
  preserving its name, type, attributes, and Tokio cancellation behavior.
- Generated-code simplification is structural at both boundaries. Typed
  `RustItem`/`RustStmt`/`RustExpr` optimization runs before rendering. After
  project metadata, inline stdlib, and bridge fragments have been assembled,
  the complete file is parsed as a `syn` Rust syntax tree for demand pruning,
  identifier canonicalization, and syntax/API cleanup before its final render.
  No regex or unchecked text-pattern rewrite decides Rust semantics; the only
  source-range substitutions are identifiers whose spans and spellings were
  validated by that parsed tree, which preserves comments and literals. At
  each public emit boundary and after final project metadata or bridge assembly,
  `sifr_driver` renders every generated `.rs` file through the repository-pinned
  toolchain's `rustfmt` with an empty configuration. The `RUSTFMT` environment
  variable can select the executable; failure to start or complete formatting
  is a structured build diagnostic, never an unformatted fallback.
  Materialization repeats this fail-closed check for synthetic namespace and
  bridge files that do not exist at the earlier emit boundary.
- Both shapes materialize through the same generated-binary-project path. Native
  build state uses local sysroot and package paths only while Cargo resolves and
  builds it. The source-only materialization boundary then replaces that local
  manifest with exact Git or registry dependencies and rewrites the generated
  lock to the same portable sources; local Rust interop dependencies that have
  no publishable source fail closed instead of leaking a host path.
- Single-file and non-package projects resolve from the immutable sysroot vendor
  source into a cached lock before compilation. The vendor inventory, rather
  than a potentially newer development-workspace lock entry, selects available
  versions; every result must still belong to an authoritative lock or trusted
  vendor source and remains byte-stable during the Cargo invocation.
- Rematerialization replaces the generated `src` tree as one owned output set so
  removed support or bridge modules cannot survive from an earlier build. The
  Cargo `target` directory remains outside that replacement boundary and retains
  incremental build artifacts.
- Native binary builds return a `BuildReport` at the driver boundary. The
  report records entrypoint path, compilation mode, target profile, binary
  path, optional binary size, total elapsed time, cache-hit status where
  applicable, and measured stage timings for stdlib loading, parsing, semantic
  analysis, Rust project generation, Cargo project materialization, and release
  native Cargo build.
- Dependency metadata for both shapes comes from codegen outputs (`used_stdlib_modules` and `required_crates`), never from emitted Rust text scans.
- Workspace design details and deferred package-management semantics are tracked in [`sifr_workspace_design.md`](./sifr_workspace_design.md).

This keeps CLI mode resolution as the boundary that selects the rooted entrypoint shape while preserving one internal build architecture.

driver/package architecture decomposed `sifr_driver` into the following stable internal boundaries:

- `diagnostics.rs`: compile/public result types, panic boundaries, diagnostic serialization, and stderr rendering helpers
- `stdlib/`: embedded stdlib sources, intrinsic mapping, cache lifecycle, and bootstrap compilation
- `frontend/`: single-file parse/lower/type-check entrypoints and metadata extraction
- `project/`: import-closure discovery, reachable module parsing, export collection, and deterministic compile ordering
- `build/`: rooted-entrypoint planning, generated-project materialization, Cargo manifest generation, and generated-artifact cache management for repeated `sifr run` builds
- `test_runner/`: test root discovery, generated test harness assembly, reusable cached Cargo test workspaces, and cargo test execution orchestration

### Generated Artifact Cache Boundary

Generated artifact cache work moved `run`/`test` away from invocation-scoped temp directories as the default cache boundary.

- `sifr build` still materializes into the caller-provided output directory and does not reuse a hidden cache.
- `sifr run` now lowers/codegens on each invocation but materializes the generated Cargo project into a content-addressed cache rooted under the system temp directory. The cache key includes:
  - rooted entrypoint scope
  - generated Cargo manifest and Rust sources
  - cargo/rustc toolchain signature plus relevant build env vars
- cache misses build inside an isolated staging directory and promote atomically into the stable cache path only after `cargo build --release` succeeds
- cache hits execute the previously built binary directly without paying the generated-project rebuild cost again
- `sifr test` uses the same cache discipline for generated test-runner Cargo projects: unchanged input reuses the prior workspace and its `target/` artifacts, while still running `cargo test` on every invocation
- `sifr run` emits human build progress only for cache misses, omits the final
  `Binary:` footer because program output follows, and emits no build progress
  for cache hits or `--quiet`. `sifr test` keeps explicit cache reporting in
  validation logs so reuse and invalidation remain visible there.

---

## Cross-cutting Rules

These are design decisions that span multiple future capabilities. They must be resolved early to prevent those capabilities from diverging and breaking each other.

### 1. Runtime Type Representation

Union types, `Unknown`, and class instances all need a coherent runtime representation in generated Rust code. This rules ensures type_system/classes/protocols/generics produce compatible code.

**Rules:**

- **Primitive unions** (`int | str`): generate Rust `enum` with one variant per member type. The enum name is deterministic from the sorted member types (e.g., `IntOrStr`). Narrowing via `isinstance` generates `match` arms.
- **Optional types** (`T | None`): generate Rust `Option<T>`. Narrowing via `is not None` generates `if let Some(x) = x`.
- **Class unions** (`Circle | Square`, classes/protocols): generate Rust `enum` with one variant per class. Discriminated union narrowing via tag field generates `match` on the tag.
- `**Unknown` type**: generates `Box<dyn std::any::Any>` in Rust. The compiler enforces that every use site is guarded by a narrowing check (`isinstance`, equality, etc.) before any operation. At runtime, `downcast_ref::<T>()` is used after narrowing. This is the only type that requires runtime type information (RTTI).
- `**Any` type**: generates the same `Box<dyn Any>` but the compiler does NOT enforce narrowing. This is the escape hatch.
- **Generics** (generics): monomorphized at compile time (like Rust). No runtime type erasure for generic types. Under the integer-model amendment, `list[int]` generates storage over the canonical `SifrInt` representation, while fixed-width lists use the corresponding Rust primitive storage.
- **Protocol/trait objects** (protocols): when a protocol is used as a type (not just a bound), generate `Box<dyn Trait>` with vtable dispatch. This is the only case of dynamic dispatch besides `Unknown`/`Any`.

**Invariant:** Every supported `Type` variant has one structured `RustType`
representation. `sifr_type_to_rust_type` owns the mapping. Field conversion
uses `sifr_type_to_rust_field_type` when Rust forbids `impl Trait` storage.
Only the Rust IR renderer converts these nodes to Rust source text. The
structured-type guard rejects direct methods and indirect string fallbacks.

### 2. Borrow and Lifetime Strategy

Sifr uses **borrow-by-default** semantics for function parameters. Move-type arguments are immutably borrowed (`&T`) unless the programmer opts in to mutable borrowing (`mut`), ownership transfer (`own`), or owned mutable parameters (`own mut`). Scalar value-semantic primitives (`int`, fixed-width integers, `float`, `bool`) do not expose use-after-move friction at the source level; under the integer-model amendment, `int` is not Rust `Copy`, but codegen owns the borrow/clone/primitive-local optimization needed to preserve scalar source semantics.

**Rules:**

- **Function arguments:** borrow by default (immutable). The compiler models parameter passing along two axes:
  - ownership:
    - borrowed
    - owned
  - mutability:
    - immutable
    - mutable
  Valid surface forms are:
  - `x: list[int]` -> borrowed immutable -> `x: &Vec<SifrInt>` under the integer-model amendment
  - `mut x: list[int]` -> borrowed mutable -> `x: &mut Vec<SifrInt>` under the integer-model amendment
  - `own x: list[int]` -> owned immutable -> `x: Vec<SifrInt>` under the integer-model amendment
  - `own mut x: list[int]` -> owned mutable -> `mut x: Vec<SifrInt>` under the integer-model amendment
  Scalar value-semantic types (`int`, fixed-width integers, `float`, `bool`) remain reusable after calls regardless of annotation; `mut` on those parameters only affects local rebinding/mutation semantics, not observable ownership transfer.
- **Method receivers:** source syntax is authoritative. `self` selects
  `SharedBorrow`, `mut self` selects `MutableBorrow`, `own self` selects
  `Owned`, and `own mut self` selects `OwnedMutable`. Lowering stores that explicit contract in method
  signatures, `HirFunction`, and every resolved method call. Protocol checking,
  codegen, and flow analysis consume the declaration and do not infer or
  reinterpret it from the method body, delegation, fields, generics, protocols,
  or inheritance. The active compatibility-removal issue tracks deletion of the
  current inference implementation before this contract is considered closed.
  Constructor `self` is
  fresh mutable storage even though the Rust constructor is a static `new`
  function. Constructor field values and self-independent statements are
  evaluated in source order. At the first statement that needs `self`, codegen
  materializes a mutable synthetic instance from the already-initialized
  fields, structurally re-roots expression and statement storage names to that
  instance, and emits every remaining statement in source order before
  returning it. A constructor that reaches `self` before all declared fields
  (or inherited storage through `super().__init__`) exist is rejected with
  `SIFR-OWN-0014` during checking rather than leaking partial Rust
  initialization.
- **Mutable receiver and argument places:** lowering proves mutable storage as
  a `BindingId` root plus nominal field projections. Stable owned locals,
  mutable/`own mut` parameters, explicitly mutable `self`, and supported
  non-optional/non-recursive field paths are accepted. Indexes, slices,
  optional/recursive projections, callable fields, loop/comprehension
  elements, and match captures are rejected before codegen. Owned receiver
  temporaries and owned temporaries passed to mutable arguments carry separate
  explicit proofs. Conditional expressions qualify only when both result
  branches are themselves proven owned rvalues; storage-selecting expressions
  such as `a if flag else b` are not treated as temporaries. Slices produce
  fresh owned values and may be mutated as temporaries; walrus expressions are
  binding expressions, not temporary proofs, and are rejected as mutable
  receivers. Module constants are also rejected as mutable roots because
  codegen re-materializes their values on every access.
- **Audited indexed-storage exception:** general mutable indexing remains
  unsupported. The existing membership-guarded/narrowed `dict[K, list[V]]`
  `bucket[key].append(...)` and zero-argument `bucket[key].pop()` lowering,
  plus the typed `defaultdict` list/set aliases' supported in-place bucket
  methods, are compiler-owned exceptions. Unguarded plain-dict indexing
  retains its optional value type and is rejected by ordinary type checking.
  Matching stable expression keys and literal string keys both retain their
  guard fact. Lowering proves the dictionary base place and codegen accepts
  only the dedicated indexed-storage target plus the resolved value type's
  in-place-method set, so these paths cannot become a generic mutable index
  fallback.
- **Same-call exclusivity:** mutable receiver and `mut` argument places use one
  prefix-overlap rule. Equal or ancestor/descendant places conflict with every
  overlapping read, borrow, or move in the same call; sibling fields and
  different binding identities remain disjoint. The only evaluation-order
  exception is typed `defaultdict` list `extend` and set update-family
  lowering: codegen inserts the entry, materializes every argument, and only
  then borrows the destination bucket. Lowering records this exact
  compiler-owned order and otherwise retains conservative overlap rejection.
- **Checked place emission:** mutable receivers, mutable arguments, and
  canonical iterator advancement emit directly from the proven root through
  every field hop. This path never enters ordinary field-value cloning.
  Shared receiver calls use a separate structural borrow path so an operation
  such as `self.items.len()` does not clone the collection; standalone field
  value reads retain ordinary clone semantics. The late Rust mutability
  optimizer runs on production and test-module assembly and preserves roots
  recorded by checked place emission. Its method-name fallback exists only for
  compiler-synthesized IR locals without HIR place provenance.
- **Closure captures (generics):** inferred from usage inside the closure body:
  - Read-only access: capture by `&T`
  - Mutation: capture by `&mut T`
  - Move into closure: capture by value (when the closure outlives the variable's scope, or when explicitly requested with `move` keyword)
- **Temporary lifetimes:** temporaries created in expressions live until the end of the enclosing statement. Method chains like `x.upper().split(",")` work without explicit borrows.
- **Escape analysis:** the compiler tracks whether a reference escapes its scope. If it does, the compiler emits a diagnostic rather than silently cloning. The programmer must choose: clone explicitly, or restructure to avoid the escape.
- **No lifetime annotations in user code:** Sifr does not expose Rust's `'a` lifetime syntax. The compiler infers lifetimes using the rules above. If inference fails, the compiler emits a clear error suggesting `.clone()` or restructuring.
- **Shared mutable state requires explicit opt-in:** the compiler does NOT auto-wrap shared data in `RefCell` or `Mutex`. If multiple variables reference the same mutable data, the programmer must use explicit sharing primitives (deferred to post-protocols). Default behavior is borrow-by-default with explicit `mut`, `own`, and `own mut` parameter rules rather than hidden runtime borrowing. This keeps ownership rules predictable and avoids hidden runtime borrow panics.
- **Return semantics follow ownership, not mutability:** returning a Move-type parameter by value is only valid when the callee owns that parameter (`own` / `own mut`). Borrowed parameters, including `mut` borrows, cannot escape by return or store unless the programmer clones explicitly.

**Implementation responsibilities:**

- classes: preserve explicit method receiver conventions (`&self` / `&mut self` / `self` / `mut self`)
- borrow_default: implement borrow-by-default parameter conventions and codegen
- borrow_hardening: implement exclusivity checking and error diagnostics
- own-mut-parameter-convention: extend parameter conventions so owned mutable parameters are first-class and lower canonically to Rust `mut x: T`
- generics: implement closure capture inference
- task/thread boundary ownership: implement task/thread boundary ownership and Send/Sync capture rules.
- Post-protocols: evaluate explicit shared mutable abstractions (e.g., `Shared[T]` mapping to `Rc<RefCell<T>>`)

### 3. Error Semantics

Sifr replaces Python's exception model with Rust's `Result`/`Option` model (error_handling). **All fallible operations return `Result` or `Option`; the compiler enforces handling via `#[must_use]`.** The only user-facing error handling mechanism is `try`/`except` -- there is no user-facing `?` operator. The compiler uses `?` internally (as an HIR node) when auto-unwrapping `Result` values inside `try` blocks.

**Rules:**

**Error mechanism matrix:**

| Context                          | Error mechanism                   | Handling                                                    | Codegen                                                    |
| -------------------------------- | --------------------------------- | ----------------------------------------------------------- | ---------------------------------------------------------- |
| Sync function                    | `Result[T, E]` return             | `try`/`except` with exhaustiveness checking                 | `Result<T, E>`                                             |
| Async function (async HIR/type substrate/async call semantics) | `Result[T, E]` return             | `try`/`except` works across same-task `.await`; spawned task observation returns `TaskResult[T, E]` | `Result<T, E>` inside the task; task handles observe `TaskResult<T, E>` |
| `try`/`except` block             | Pattern match on `Result`         | `except` arms match error types; compiler checks coverage   | `match result { Ok(v) => ..., Err(e) => match e { ... } }` |
| Indexing                         | `Option[T]` return                | Type narrowing (`if val is not None`)                       | `.get(i).cloned()` / `.chars().nth(i)`                     |
| Division                         | `Result[T, DivisionError]`        | `try`/`except`                                              | Checked division with zero-check                           |
| Exact integer arithmetic (`int`) | Exact value-semantic arithmetic; explosive operations such as exponentiation/large shifts are budgeted and fallible when needed | Use fixed-width integer APIs only when representation matters; no silent wrap/panic in normal `int` arithmetic | `SifrInt` inline-small runtime type; fixed-width types map to Rust primitives |
| Type conversion                  | `Result[T, ParseError]`           | `try`/`except`                                              | `.parse::<T>()`                                            |
| Unused `Result`                  | **Compile-time error**            | Must handle via `try`/`except` or discard with `_ = ...`    | `#[must_use]` attribute on `Result`                        |
| Rust interop           | Rust panics caught at boundary    | `catch_unwind` at generated Rust interop entry points       | Panic -> `Result::Err` conversion                          |
| C FFI (ffi)            | Crashes are non-recoverable       | Safe wrappers validate inputs                               | Process terminates on segfault/abort                       |
| `assert` statement               | Panic (programmer invariant only) | Not catchable                                               | `assert!()` or `panic!()`                                  |
| Main function                    | `Result` printed as exit code     | Non-zero exit on `Err`                                      | `fn main() -> Result<(), Box<dyn Error>>`                  |

**`try`/`except` semantics and auto-unwrap:**

Inside a `try` block, when a `Result`-returning function is called and the result is assigned to a non-`Result` variable, the compiler **auto-unwraps** the `Result`. If the call returns `Err`, execution jumps to the matching `except` arm. No explicit `?` is needed in user code:

```python
class ValidationError(Error):
    message: str

def validate(x: int) -> Result[str, ValidationError]:
    if x > 0:
        return "positive"                          # auto-wrapped in Ok("positive")
    raise ValidationError("must be positive")      # maps to return Err(ValidationError {...})

def main():
    try:
        r: str = validate(5)    # auto-unwrapped: compiler inserts ? in HIR
        print(f"ok: {r}")
    except ValidationError as e:
        print(f"caught: {e}")                      # Display formats e.message
```

**`except` exhaustiveness checking:**

The compiler enforces that all error types from fallible calls inside a `try` block are covered by the `except` arms. The rules are:

1. **`except Error as e`** is the catch-all for ordinary errors. All ordinary error types are classes that extend `Error`. Since `Error` is the root of the ordinary error hierarchy, this covers every possible ordinary error type. The compiler is satisfied -- no further checking needed.
2. **`except SpecificError as e`** triggers exhaustiveness checking. The compiler collects every error type from every `Result`-returning call in the `try` block and verifies that each one is covered by some `except` arm.
3. **Mixing is allowed.** Specific `except` arms are checked first; an `except Error as e` arm at the end covers all remaining uncovered ordinary error types. This mirrors Python's `except` ordering (specific before general).
4. **Each `try` block is checked independently.** Nested or sequential `try` blocks each have their own exhaustiveness scope.
5. **Uncovered error types are a compile error.** If any error type from a fallible call is not covered by an `except` arm, the compiler emits a diagnostic listing the uncovered types and the calls that produce them.

**Error type constraint:** The ordinary `E` in `Result[T, E]` must always be a class that extends `Error`. Primitive types like `str` are not valid error types. This ensures every ordinary error has a structured type that the compiler can track for exhaustiveness checking, and that `except Error as e` is a true catch-all for ordinary errors. Spawned task observation uses `TaskResult[T, E]`; its `Cancelled(Failure[CancellationError])` branch is not an ordinary `Result` error, and `CancellationError` is not an `Error` subclass.

**Example -- catch-all (compiler satisfied):**

```python
class ProcessError(Error):
    message: str

def process(path: str) -> Result[str, ProcessError]:
    try:
        content: str = read_text(path)           # can fail with IOError
        config: dict = parse_toml(content)       # can fail with TOMLDecodeError
        return config["name"]
    except Error as e:
        raise ProcessError(f"pipeline failed: {e}")   # catches everything
```

**Example -- specific handling (compiler enforces exhaustiveness):**

```python
class ProcessError(Error):
    message: str

def process(path: str) -> Result[str, ProcessError]:
    try:
        content: str = read_text(path)           # IOError
        config: dict = parse_toml(content)       # TOMLDecodeError
        return config["name"]
    except IOError as e:
        raise ProcessError(f"read failed: {e}")
    except TOMLDecodeError as e:
        raise ProcessError(f"parse failed: {e}")
```

If the developer omits `except TOMLDecodeError`, the compiler emits:

```
error[S0042]: unhandled error type in try block
  --> process.sifr:4:22
   |
4  |         config: dict = parse_toml(content)
   |                        ^^^^^^^^^^^^^^^^^^^ can fail with TOMLDecodeError
   |
   = help: add `except TOMLDecodeError as e`, or use `except Error as e` to catch all
```

**Example -- mixed (specific + catch-all):**

```python
class PipelineError(Error):
    message: str

def pipeline(path: str) -> Result[str, PipelineError]:
    try:
        content: str = read_text(path)              # IOError
        config: dict = parse_toml(content)          # TOMLDecodeError
        validated: int = validate_range(x, 0, 100)  # ValidationError
        return "done"
    except IOError as e:
        raise PipelineError(f"io: {e}")             # handles IOError specifically
    except Error as e:
        raise PipelineError(f"other: {e}")          # covers TOMLDecodeError, ValidationError
```

**Typed error hierarchies:** All ordinary error types are classes that extend `Error`. The `raise` keyword maps to `Err(ErrorInstance)`. `return value` in a `Result`-returning function auto-wraps in `Ok(value)`. Using a non-`Error` type (e.g., `str`, `int`) as the `E` in `Result[T, E]` is a compile-time error. Task cancellation is handled by the async cancellation rules rather than ordinary catch-all matching.

**Built-in error classes:** Sifr provides a standard set of error classes for common failure modes (e.g., I/O, parsing, validation). These are used by the stdlib and available to user code.

#### Error Hierarchy

All errors have `message: str` populated from Rust's `Display` (for built-ins) or the constructor (for user-defined errors). `print(e)` is the idiomatic way to display error messages.

**Design Principles:**
1. The type tells you the error kind; the message tells you the details
2. All errors have `message: str` — inherited from the base `Error` class
3. `print(e)` is the idiomatic way to display errors (via `Display` which formats `self.message`)
4. Additional structured fields where Rust provides data the developer can't know in advance
5. Subclasses at Sifr level = `kind` field dispatch at Rust level

**Error Type Reference:**

| Sifr type | Parent | Fields | Rust source |
|---|---|---|---|
| `Error` | — | `message: str` | Base class |
| `IOError` | `Error` | `message: str`, `kind: str` | `std::io::Error` |
| `FileNotFoundError` | `IOError` | `message: str` | `io::ErrorKind::NotFound` |
| `PermissionError` | `IOError` | `message: str` | `io::ErrorKind::PermissionDenied` |
| `FileExistsError` | `IOError` | `message: str` | `io::ErrorKind::AlreadyExists` |
| `IsADirectoryError` | `IOError` | `message: str` | `io::ErrorKind::IsADirectory` |
| `NotADirectoryError` | `IOError` | `message: str` | `io::ErrorKind::NotADirectory` |
| `DirectoryNotEmptyError` | `IOError` | `message: str` | `io::ErrorKind::DirectoryNotEmpty` |
| `ParseError` | `Error` | `message: str` | `ParseIntError`, `FromUtf8Error`, etc. |
| `ValueError` | `Error` | `message: str` | Manually constructed |
| `DivisionError` | `Error` | `message: str` | Compiler-generated |
| `KeyError` | `Error` | `message: str` | Compiler-generated |
| `JSONDecodeError` | `Error` | `message: str`, `line: int`, `column: int` | `serde_json::Error` |
| `TOMLDecodeError` | `Error` | `message: str`, `line: int`, `column: int` | `toml::de::Error` |
| `RegexError` | `Error` | `message: str`, `detail: str` | `regex::Error` |
| `OverflowError` | `Error` | `message: str` | Fixed-width narrowing and representation-preserving fixed-width arithmetic overflow |
| `ArithmeticLimitError` | `OverflowError` | `message: str`, `limit: int` | Exact integer operation exceeds configured output budget |
| `FloatOverflowError` | `OverflowError` | `message: str` | Exact integer cannot be represented as finite `float` |
| `FloatPrecisionLossError` | `OverflowError` | `message: str` | Exact integer to `float` conversion would silently lose precision |
| `JsonIntegerRangeError` | `Error` | `message: str`, `path: str`, `profile: str` | JSON integer output violates selected precision/range profile |
| `JsonLimitError` | `Error` | `message: str`, `limit: int` | JSON/text integer token or document exceeds configured decoder limit |
| `CancellationError` | -- | `message: str` | Task-control evidence for a cancelled child task; not an `Error` subclass and never matched by broad `except Error` |
| `TaskCancelled` | `Error` | `message: str` | Ordinary wrapper when user code intentionally converts child cancellation evidence into an error channel |
| `TimeoutError` | `Error` | `message: str`, `duration: float` | `sifr.task.timeout` deadline expiry |
| `ScopeFailure` | `Error` | `primary: ScopeFailureCause`, `secondary: list[SecondaryError]` | Scope-exit evidence for unobserved child task failure/cancellation |
| `GeneratorCloseError` | `Error` | `message: str` | Explicit async generator close failed during cleanup |
| `GeneratorBusyError` | `Error` | `message: str` | Reentrant async generator advancement protocol error |
| `SecondaryError` | -- | `CleanupFailed`, `CleanupTimedOut`, `SiblingFailed`, or `CancellationDuringCleanup` | Closed secondary evidence defined by the async model; never masks the primary result |

**Exhaustiveness with Subclasses:**

```python
# Specific subclass handling
try:
    content: str = read_text("data.txt")
except FileNotFoundError as e:
    print(f"File not found: {e.message}")
except IOError as e:
    print(f"Other I/O error: {e.message}")

# Parent catches all subclasses
try:
    content: str = read_text("data.txt")
except IOError as e:
    print(f"Any I/O error: {e.message}")
    print(f"Error kind: {e.kind}")
```

**Codegen: subclasses = enum variants via kind field:**
- At the Sifr level, `FileNotFoundError` looks like a subclass of `IOError`
- At the Rust level, `IOError` is a struct with a `kind: String` field
- `except FileNotFoundError` generates a guard: `Err(ref e) if e.kind == "FileNotFound"`
- `except IOError` catches all variants (no guard)

**User-defined error classes:** User-defined error classes inherit `message: str` from `Error`. The constructor accepts a message string, and `print(e)` formats it via `Display`. Users can add additional fields as needed.

The inherited message is required constructor storage. An auto-generated
constructor takes `message: str` before additional fields when the class does
not declare its own message. Explicit string declarations retain their field
order, including the five-field PythonError contract. A message field cannot
have an incompatible type or a field default. Custom constructors must
initialize the string (directly, from a matching required parameter, or through
`super().__init__`); construction cannot finish with missing storage. A
custom constructor requires a caller-supplied string parameter; a zero-argument
constructor or a defaulted-only message input cannot satisfy this contract. A child
without new fields or an explicit constructor forwards the parent's constructor
signature and initialization. An inherited message is stored only in its data
parent. Consuming root conversions move that parent or the owned message;
they never derive a message from formatting or substitute missing text.

```python
# Simple user-defined error — inherits message from Error
class AppError(Error):
    pass                                       # only has message: str (inherited)

def connect() -> Result[str, AppError]:
    raise AppError("connection refused")       # message = "connection refused"

try:
    conn: str = connect()
except AppError as e:
    print(e.message)                           # field access: "connection refused"
    print(e)                                   # Display: "connection refused" (same thing)
    print(f"failed: {e}")                      # f-string: "failed: connection refused"
```

```python
# User-defined error with additional fields
class DbError(Error):
    query: str
    code: int

try:
    result: str = execute(query)
except DbError as e:
    print(e.message)                           # inherited from Error
    print(e.query)                             # additional field access
    print(e.code)                              # additional field access
    print(e)                                   # Display: formats e.message
```

**Common error type patterns:**

- **Application code:** define a simple error class per module or feature (e.g., `class AppError(Error)`). The inherited `message` field carries the human-readable text. Use `except Error as e` as a catch-all when fine-grained ordinary error handling is not needed.
- **Library code:** define a domain error class (e.g., `class ConfigError(Error)`) and wrap internal errors at API boundaries. Callers only see the domain error type.
- **Stdlib functions:** return `Result[T, SpecificError]` using the built-in error classes. For example, `read_text(path)` returns `Result[str, IOError]`, `int(s)` returns `Result[int, ParseError]`.

### 4. Package Resolver and Reproducibility (import semantics work/CLI semantics work/package-management work)

These rules have three owners: import semantics work
(multi-file compilation and import semantics), CLI semantics work (structural
workspace-boundary selection), and package-management work (package management
with dependency resolution). Import semantics work maps to import semantics
architecture (Import and Externals Correctness), CLI semantics work maps to CLI
semantics architecture (Project and CLI Semantics Correctness), and
package-management work lands in driver/package architecture (Package
Management).

**Rules (import semantics work -- imports and modules):**

- **Import cycle detection:** the compiler builds a dependency graph of modules during compilation. Cycles are a compile-time error with a clear diagnostic showing the cycle path.
- `**__init__.sifr` semantics:** defines the public API of a package. Symbols not re-exported from `__init__.sifr` are private to the package. No side effects on import (unlike Python's `__init__.py`).
- **Import-form matrix is explicit:** behavior for `from x import ...`, `from .x import ...`, `from ..x import ...`, `from . import ...`, and `import x` is explicitly defined as supported, unsupported, or non-activating with stable diagnostics.
- **Import caching:** each module is compiled exactly once per compilation. The driver maintains a module cache keyed by canonical path.
- **Multi-file diagnostics:** error messages show correct source file and line numbers across module boundaries.

**Rules (CLI semantics work -- workspace boundary):**

- **Structural mode selection:** the nearest valid ancestor `sifr.toml` selects
  project mode; without one, every explicit file uses single-file mode.
  Entrypoint filenames, source imports, and sibling modules are not inspected
  during mode selection.
- **Malformed manifest authority:** a discovered malformed `sifr.toml` is a hard
  diagnostic and never falls back to single-file mode.
- **Command equivalence:** `run`, `build`, `check`, `emit`, and `trace` use the
  same resolver and produce equivalent mode selection and error-class outcomes
  for identical inputs.
- **Rules synchronization:** resolver behavior, regression tests, and CLI semantics documentation must remain aligned.

**Rules (package-management work -- package management, package-management architecture):**

- **Cargo-backed package substrate:** `Cargo.toml` and `Cargo.lock` own external dependency resolution, lockfile behavior, registries, Git/path sources, workspaces, publishing, vendoring, and backend Rust/native dependencies.
- **Sifr compiler metadata:** `sifr.toml` owns the Sifr package name, edition, compiler requirement, one source root, privacy, aliases, and native trust policy. Source `__init__.sifr` files own public exports. The manifest does not own external dependency resolution or registry credentials.
- **Package graph:** `crates/sifr_package` consumes `cargo metadata --format-version 1`, normalizes Cargo packages and resolved dependency edges, and derives `SifrPackageGraph` plus `PackageSourceMap` for the normal frontend/lowering/codegen pipeline. Package source-map construction uses the SourceProvider boundary for source-root traversal and `__init__.sifr` API reads, and preserves otherwise legal ambiguous module candidates for import-site `SIFR-IMPORT-0005` diagnostics instead of failing construction as `SIFR-PACKAGE-*`.
- **Package identity:** package instance identity includes Cargo package id, version, and source identity. Multiple Cargo-selected versions are allowed when each package's direct dependency scope remains unambiguous.
- **Distribution:** a Sifr package is a valid Cargo package carrying `.sifr` source and `[package.metadata.sifr] manifest = "sifr.toml"`. Pure Sifr packages include only the canonical Rust marker target; Rust-backed packages must declare and pass backend trust validation.
- **No Sifr-native lockfile in package-management architecture:** there is no committed `sifr.lock`; reproducibility is derived from `Cargo.toml`, `Cargo.lock`, `sifr.toml`, selected Sifr source, compiler/toolchain inputs, and package feature/selector inputs.
- **Interop package surfaces:** Python interop consumes externally managed `pyproject.toml`, `uv.lock`, and `.venv` metadata without forking package resolution. Rust interop consumes Cargo dependencies and bridge metadata through declaration-level Cargo integration. Both lanes must lower into the same package graph/import semantics instead of creating alternate resolvers.

### 4.1 Deterministic Const Specialization and Structural Metadata

Package-owned static derivation uses the package-neutral compiler contract documented in
[`const_specialization.md`](const_specialization.md). Structural shapes, typed declaration
metadata, bounded pure HIR evaluation, package issues, and integer JSON boundary verification are
frontend authorities shared by CLI, build, tests, and editor analysis. This is compile-time data;
Sifr does not expose runtime reflection or package-name-specific compiler branches.

The frontend retains package-neutral declaration descriptors as typed compile-time values. It
canonicalizes provider and descriptor declarations during lowering, resolves callable identities
by module, owner, symbol, generic arguments, and checked signature, and evaluates descriptor calls
under the const-evaluation budget before class-adapter selection. Descriptor field expressions and
consumed class assignments are therefore metadata inputs, not runtime defaults or storage. The IR
and external-definition boundary carry these records across modules without selecting a package or
adapter implementation.

Class adapters run across an explicit provisional-to-final lowering boundary. The provisional
declaration input flattens inherited fields and methods before local declarations, preserves their
canonical declaring identities, substitutes concrete generic parent arguments, and orders nested
`Annotated` descriptors from inner to outer before right-hand-side descriptors. A validated plan
normalizes every field to required, constant-default, or checked zero-argument factory-default
state; final lowering alone materializes constructor defaults, and factory expressions execute at
each call so mutable values are not shared. Adapter invocation identity hashes the source-location-
free declaration input and semantic provider HIR before evaluation. Provider canonicalization
sorts the const function set, removes lowering-assigned binding and ownership-place IDs, and
normalizes diagnostic ranges while retaining checked interop declarations. Post-adapter identity
separately binds the invocation to the validated output and is an input to static-program identity,
avoiding a cycle with later handler and attached-API outputs.

Adapter inheritance selects parent plans and type-parameter bindings by canonical class identity. A
colliding local bare name cannot redirect an imported parent. Duplicate local aliases prefer a
requested matching key, then use deterministic key order. Declaration reconstruction stops when a
data parent has no canonical class type. Class-base lowering rejects a type alias as data-parent
identity authority.

Generic alias instances specialize their arguments and bodies before structural substitution.
Generic ancestor walks require exact parameter and argument arity. Unspecialized exports pass
their declaration parameters as explicit symbolic arguments instead of leaving bindings absent.
An unresolved handler ancestry never substitutes a partial signature. Structural metadata keeps
an opaque checked-handler contract until complete ancestry becomes available.

Structural shape derivation specializes finalized adapter field-plan types with the concrete
owner arguments before package specialization. Local and imported adapted generic classes use the
same substitution rule, so nested concrete uses do not expose unbound declaration parameters to a
package specializer. Nested nominal declarations rebind their own type parameters before field and
method substitution. An outer parameter with the same name cannot capture the nested parameter.
Local declaration identity and exported parameter metadata are the only binding authorities. An
unresolved nested scope stays symbolic and does not use a consumer-local class with the same name.
The type system owns this substitution visitor. Lowering and structural code generation supply
their declaration resolvers to the same visitor. Substitution also rebuilds unions through the
canonical union constructor. Therefore, structural identity uses normalized concrete types after
generic substitution. Code generation resolves nested scopes from the selected outer declaration's
module. A consumer module cannot redirect an identityless nested field to a same-named class.
Lowering gives user class arguments canonical module identities before substitution. This keeps a
consumer-owned argument distinct from same-named classes in a generic declaration's module.

Rust generic storage keeps the union nesting from its declaration. Therefore, a concrete generic
parent cannot expand one union member into another union or collapse it into a sibling. Lowering
rejects that parent specialization before code generation. Structural support applies the same
rule when a concrete generic class is nested in another record. Lowering selects an imported
parent template by canonical identity. The shared type-system check follows nested generic class
declarations, binds their local parameters, and rejects transitive union-topology changes.
Lowering uses declared class-parameter metadata as the binding authority. The check rejects a
class occurrence when its concrete argument count differs from that declaration.

Checked adapted-handler exports retain the callable target with a signature specialized relative
to the selected owner's type parameters. Generic substitution follows the full local ancestor
chain before export. Imported structural shapes bind those owner parameters to the concrete use
and restore the handler descriptor, order, and declaration metadata. Source origins remain scoped
to the declaration that created them; inherited and imported handler shapes do not expose a
foreign origin to another declaration's package-issue registry.

The same final field state governs structural construction. Structural records match incoming
edges by field name, reject unknown and duplicate names, and fill omitted defaulted fields from
their checked HIR defaults. Factory defaults retain a canonical callable-identity side channel in
HIR because their executable call expression is not a constant literal; bound checking, shape
identity, and code generation all consume that same identity. Required omissions remain typed
structural contract errors rather than generated panics. Construction checks all required fields
before it evaluates any omitted-field default. A later required omission therefore cannot run an
earlier constant or factory default.

Structural support includes a class with one direct data parent when that parent uses the
compiler-generated constructor. The parent cannot have another data parent. The wire record lists
concrete inherited fields before local fields. Rust storage keeps the parent embedded. Structural
construction builds that embedded parent, and projection reads inherited fields through it. A
supported inherited record can also appear inside another structural record. Module exports keep
the same flattened field list, so imported code retains inherited field access.

Attached package APIs are erased compile-time declarations grouped into canonical module-and-set
identities. Adapter output selects exactly one set. Provisional lowering may expose visible
package candidates so class bodies can type-check before adapter execution, while final lowering
uses only the selected set as authority. Imported adapter selections are finalized external facts;
consumer lowering never applies provisional candidates to an imported owner, including when its
final plan selected no set. Attached signatures are added to the adapted
`Type::Class` surface without adding HIR methods, structural members, handler slots, or synthesized
method bodies. Calls lower directly to the declared package function through deterministic hidden
imports. Local and imported declarations use the same public-function filter. Binding keys use the
selected class's canonical identity when its final symbol matches the selected owner. Classes
without a module identity use their local emitted name and do not gain a synthetic module key. The
lowering substitutes the concrete owner and `Self`, infers residual type parameters,
records their concrete arguments in the emitted call identity, and applies the checked package
function defaults to omitted public arguments. Instance bindings remove the hidden owner parameter
before they map default indexes. A generic type alias can forward an attached type call when its
concrete expansion resolves to an adapted class; the class remains the static-program owner, and
the compile-time alias does not become a generated Rust import. Type, shared-borrow,
mutable-borrow, and owned receivers use the normal call and ownership checks; type-directed calls
do not construct or pass a dummy owner value. For a structural Rust bridge, a type receiver's
declared owner is a valid use of its exact `StaticProgram` type parameter. The generated call keeps
that concrete Rust type argument even when no runtime parameter or result contains the owner.
Sifr lowering enforces declared protocol bounds before code generation. Generated Rust generics
therefore emit only the weaker bounds required by generated operations. Rust bounds are not a
second copy of the Sifr protocol contract.

The Native Pydantic-Sifr consumer architecture is documented in
[`native_pydantic_sifr_architecture.md`](native_pydantic_sifr_architecture.md).
The architecture includes the package-neutral static class-adapter contract.
This contract supports typed declaration descriptors, erased marker bases,
checked handler references, and attached package APIs.

The durable native model and adapter design is authoritative here and in
`native_pydantic_sifr_architecture.md`. Delivery records do not define compiler
or package behavior.

### 5. CI Quality Gates

**Rules for every PR:**

- `cargo test` passes (all layers: unit, snapshot, E2E, corpus)
- `cargo clippy -- -D warnings` passes
- No new `unsafe` blocks without explicit justification
- E2E pass tests compile generated Rust and verify runtime stdout
- E2E fail tests verify expected diagnostics

**Feature-specific gates (added as features land):**

- ergonomics+: CPython parity tests -- verify behavioral match with CPython (`/Users/yaseralnajjar/work/sifr/cpython`) for all built-in functions, data structure methods, and stdlib modules, with safe error handling (no panics, `Result`/`Option` where CPython raises exceptions)
- generics+: benchmark suite with regression thresholds (compile time, binary size)
- core_stdlib+: stdlib wrapper tests (each module has integration tests against the underlying Rust crate)
- ecosystem: fuzz testing for parser and type checker (cargo-fuzz or afl)

### 6. Slice and Collection Semantics

Sifr uses Python-like slicing syntax, but must define whether slicing copies or creates a view. This affects performance expectations and ownership behavior.

**Rules:**

- **List slicing copies:** `list[a:b]` produces a new `list` (deep copy of elements). This matches Python semantics and avoids borrow complexity. Codegen: `vec[a..b].to_vec()`.
- **String slicing copies:** `str[a:b]` produces a new `str`. Indices are character positions (not byte offsets). Codegen: `s.chars().skip(a).take(b - a).collect::<String>()`.
- **Dict:** not sliceable. **Tuple:** compile-time slicing supported (ergonomics) -- the compiler can statically verify tuple slice bounds and produce a new tuple type.
- **Views deferred:** an explicit view API (e.g., `list.view(a, b)` mapping to `&[T]`) may be added later for performance-critical paths. Not part of MVP.
- **`for` loop protocol entry:** `for item in collection` lowers through `iter(collection)` first, then iterates the resulting iterator. Collection-backed iterables (list/set/dict/string/range/iterable wrappers) are converted to iterator objects without consuming the original collection. This preserves reusable collection behavior while making the protocol boundary explicit in HIR.
- **For-loop element semantics (borrow_hardening):** Loop elements are independent copies (deep-copy on assignment via `.cloned()`). This matches Python's loop semantics and avoids exposing Rust's borrow/lifetime complexity to Sifr users. The practical consequence: `for x in items: x = transform(x)` does not mutate `items`. Codegen rationale: `.iter().cloned()` copies elements one-at-a-time (like Python), avoids lifetime issues with borrowed elements escaping the loop, and keeps the Sifr ownership model simple for users.
- **Iterator mutation safety in loops (iterator mutation-safety work):** mutating a collection while iterating over it in the same `for` body is rejected at compile time (`cannot mutate '<name>' while iterating over it in a for loop`). No eager fallback or implicit snapshot is inserted.

### 7. String Semantics (UTF-8)

Sifr's `str` maps to Rust `String` (UTF-8). String indexing and length must be defined carefully because UTF-8 is variable-width.

**Rules (safe indexing -- no panics):**

- `**s[i]`:** returns `Option[str]` -- the i-th character (Unicode code point) as a single-character `str`, or `None` if out-of-bounds. Codegen: `s.chars().nth(i).map(|c| c.to_string())`. This is O(n), not O(1).
- `**list[i]`:** returns `Option[T]` -- the i-th element, or `None` if out-of-bounds. Codegen: `vec.get(i).cloned()`. This is O(1).
- `**s.len()`:** returns the number of Unicode code points (not bytes). Codegen: `s.chars().count()`. This is O(n).
- `**s.byte_len()`:** returns the number of bytes (O(1)). Codegen: `s.len()`.
- `**s[a:b]`:** returns characters from position `a` to `b` (exclusive). Codegen: `s.chars().skip(a).take(b - a).collect::<String>()`. Returns empty string if indices are out of range.
- **String literals:** type is `str`, stored as `String` in generated Rust.
- **Complexity documentation:** the compiler should emit a note when string indexing is used in a loop, suggesting `.chars()` iteration instead for performance.
- **Global indexing rules:** all indexable types (`str`, `list`, `dict`) use safe indexing. `x[i]` returns `Option[T]`, never panics. This is enforced uniformly across the language.

### 7.1 Text, Encoding, Unicode, And I18n Substrate

Sifr's production text substrate is owned by `sifr.encoding`, `sifr.io`, `sifr.unicode`, and `sifr.i18n`.
The focused readiness note lives in [text_i18n_architecture.md](./text_i18n_architecture.md).

**Valid text invariant:**

- `str` is always valid Unicode and lowers to Rust `String`/`str`.
- Arbitrary payloads stay in `bytes` until an explicit decode boundary succeeds.
- Invalid Unicode recovery cannot be hidden inside ordinary strings; recovery paths return typed outcomes or typed errors.

**Encoding and text I/O boundaries:**

- `sifr.encoding` uses a static registry for accepted Tier 0/Tier 1 encodings. Runtime registry mutation and dynamic codec lookup are unsupported.
- `str.encode(...)`, `bytes.decode(...)`, `sifr.encoding.encode/decode`, and text-mode file I/O all lower through the same encoding substrate.
- Text file I/O requires an explicit encoding. `open(path, "r")` and other text modes without `encoding=` are rejected; locale-derived default encodings are not part of the language.
- Compiler-recognized `open(...)` mode values must be string literals so lowering can choose binary versus text handle types statically.

**Unicode data and segmentation:**

- Normalization, property lookup, names, numeric values, and case folding use checked-in generated Unicode 17.0.0 table artifacts and regeneration markers.
- Grapheme and word segmentation use the Unicode 17.0.0-aligned `unicode-segmentation` substrate. Public APIs return owned strings and byte-index records; streaming segmentation is deferred.

**Locale and i18n data:**

- Locale identifiers are explicit `LocaleId` values. Object-scoped number, datetime, plural, and collation APIs use ICU4X compiled data behind Sifr-owned wrappers.
- `host_locale()` is read-only and host-limited. It may observe environment locale identifiers, but it cannot mutate process state and cannot make implicit text encodings legal.
- Process-global locale mutation, `gettext.install`, and global `_` binding are unsupported.

**Translation catalogs:**

- `Bundle`, `Message`, and `Translator` are the production translation API.
- `.mo` files are a compatibility backend/import format. Catalog parsing validates byte layout, declared charset, context keys, plural metadata, and malformed paths through typed `CatalogError`.
- `.mo` plural expressions are parsed by a constrained safe parser. Sifr never evaluates catalog metadata through a Python, Sifr, shell, or host expression engine.
- Missing keys fall through explicit fallback chains and finally return the source singular/plural text; corrupt catalogs surface as typed translation errors.

### 8. Concurrency Safety

Sifr must define which types can cross thread/task boundaries. async/runtime architecture planning follows `internal_docs/async_concurrency_model.md`; this section records the high-level rules that implementation requirements must preserve.

**Rules:**

- **Auto-derived Send/Sync:** Sifr types are `Send` and `Sync` when all their fields are `Send` and `Sync` (matches Rust's auto-derivation). The compiler tracks this automatically.
- **Spawn boundaries are checked:** when a value is sent to a spawned task (`scope.spawn`) or thread, the compiler verifies the value satisfies the task/thread boundary rules. If not, it emits a clear error explaining which captured value or field is not sendable/share-safe.
- **No silent upgrades:** the compiler does NOT auto-upgrade `Rc` to `Arc` or `RefCell` to `Mutex`. If a non-sendable type is used across a task boundary, the programmer must fix it explicitly.
- **Shared mutable state across tasks:** requires explicit primitives from the async/concurrency model (`sifr.sync.Lock`, `sifr.sync.RwLock`, or `sifr.sync.Channel`). The compiler rejects sharing mutable references across task boundaries without synchronization.
- **Shared immutable state is deep-safe:** `sifr.sync.Shared[T]` requires `T` to satisfy the async/runtime architecture `ShareSafe` capability (`Send + Sync` and no unsynchronized interior mutability).
- **Lock and channel safety:** `sifr.sync.Lock` is synchronous in v1 and may block an async runtime worker under contention, so it is for short critical sections only. Channels use explicit sender/receiver endpoints; send and receive are async and direct `receive` reports closed-and-drained with `ClosedError`; channel-backed `async for` maps closed-and-drained to `Ok(None)`. Channel endpoint lifetime rules: dropping last sender closes after buffered drain; dropping receiver closes immediately to senders; `close()` on any sender closes entire channel; buffered messages remain receivable; FIFO delivery order.
- **Async callables are distinct:** `AsyncFunction` is not a subtype of sync `Function`/`Callable`; async functions cannot be stored, passed, or invoked through a sync callable path.
- **Coroutine/task/result ladder:** calling an async function returns a linear `Coroutine[T, E]`. Awaiting that coroutine in the same task yields the async function's surface return type. Spawning it with `scope.spawn` consumes the coroutine and returns `Task[T, E]`.
- **Task error types are constrained:** `Task[T, E]` must satisfy `E: Error`, with `Never` accepted for no-error tasks. Awaiting a task from a non-cancelled owner consumes the affine handle and produces `TaskResult[T, E]`; `CancellationError` is a separate `Cancelled(Failure[CancellationError])` branch, not an `Error` subclass, and is not caught by broad `except Error`.
- **Scope failures are typed:** `TaskScope.__aexit__` returns `Result[None, ScopeFailure]`; unobserved child failure or cancellation is type-erased into `ScopeFailure` instead of being silently dropped. `TaskGroup[E]` requires homogeneous child error type `E` in v1 and cancels unfinished siblings on first failure. User-defined async context managers choose their own ordinary `Error` type for their exit error, not necessarily `ScopeFailure`.
- **Timeouts are typed:** `task.timeout(handle, duration)` returns `TaskResult[T, TimeoutResult[E]]`. `async with task.timeout(duration)` is a compiler-recognized same-task cancellation scope using internal delimited cancellation; deadline exits through ordinary `TimeoutError`, not child cancellation evidence.
- **Async generators are async iterables:** `async def` with `yield` returns `AsyncGenerator[T, E]`, not a coroutine. `AsyncGenerator[T, E]` implements `AsyncIterator[T, E]` and `AsyncClosable[GeneratorCloseError]`; `anext()` returns `Result[Option[T], E]`, where `Ok(None)` is normal exhaustion or completed close and `Err(E)` is stream failure. Async generators are not awaitable.
- **Async generator suspension is ownership-checked:** mutable borrows cannot remain live across `yield` or `await` inside an async generator. If an async generator object crosses a spawned-task boundary, all captured values and generated state-machine fields must satisfy the same sendability facts as any other task-boundary value.
- **Async comprehensions are protocol sugar:** list, set, and dict async comprehensions consume `AsyncIterator[T, E]` through `anext()`; they do not create hidden tasks or detached work. Cancellation of a comprehension closes the active `AsyncClosable` iterator it started. Lazy async generator expressions are deferred in v1.
- **`AsyncClosable` is parameterized:** `AsyncClosable[E]` with `aclose() -> Result[None, E]` allows streams, files, sockets, and database cursors to define their own cleanup error type. `AsyncGenerator` implements `AsyncClosable[GeneratorCloseError]`.
- **Cancellation cleanup is bounded:** cancellation defers redelivery during one resource-owned cleanup budget. Expiry drops the cleanup future and invalidates the resource. It retains `SecondaryError.CleanupTimedOut` without replacing the primary outcome.
- **Native async cleanup has one authority:** lowering records the active error type and whether the body can propagate an error. Code generation classifies the concrete body outcome before conversion, uses nested outcome envelopes for LIFO control flow, and runs `__aexit__` or `aclose` under a fresh cancellation carrier. `sifr_runtime::async_cleanup` owns panic-safe future polling and typed cleanup evidence. The active task cancellation carrier owns that evidence until a task observation boundary drains it.
- **Replay safety is compiler-validated:** `@retry_safe` callbacks can call only pure or replay-safe operations and explicit replay capabilities. External replay-safe declarations require certification.
- **Async calls are async-only:** sync code cannot invoke an async function through the async-call path. The compiler rejects async calls from sync functions unless a future explicit runtime bridge is added. This prevents Python-style unawaited coroutine/task leaks.
- **Typed Python async contexts are cancellation-safe:** `@python.context.aenter`/`.aexit` and `cleanup=async_context` run on the generated application's single owned asyncio loop. The compiler preserves original Python exception replay, permits suppression only for originating Python exceptions, masks exit until terminal cleanup, and then resumes timeout/cancellation with cleanup failures retained as secondary evidence.
- **Borrow rules at async boundaries:** immutable borrows may cross `await` only when the borrow remains valid and no conflicting mutation exists; mutable borrows cannot remain live across `await`; v1 spawn boundaries require owned, sendable, static captures; `sync.Shared[T]` is allowed for immutable shared data; unsynchronized mutable state is rejected.
- **Task composition semantics:** `task.timeout` accepts task handles in v1, returns the inner result when the inner task completes before the deadline, timeout expiry cancels and awaits inner cleanup, same-tick completion wins over timeout, and outer cancellation cancels the inner task. `task.gather` is fail-fast with deterministic success ordering; first observed failure cancels unfinished children and cleanup/sibling failures become secondary evidence. `task.select` and `task.race` consume their input handles and cancel losing tasks by default. `BlockingTask[T, E]` is separate from cooperative `Task[T, E]` because blocking cancellation may only abandon the result.
- **Single-threaded by default:** code that does not use `async` or `spawn` has no concurrency overhead. `Rc` and `RefCell` are used internally only when appropriate for single-threaded code.

**Implementation responsibilities:**

- `async architecture import`: copy the complete async/concurrency type, task, cancellation, and runtime rules from `internal_docs/async_concurrency_model.md` into the architecture rules.
- `async HIR/type substrate`: add async HIR/type substrate (`Coroutine`, `Task`, `TaskResult`, `Awaitable`, `AsyncFunction`, `await`, async calls).
- `task/thread boundary ownership`: implement Send/Sync and borrow-boundary checking at spawn boundaries.
- `shared synchronization primitives`: provide `sifr.sync.Shared`, `Lock`, `RwLock`, and `Channel` for explicit cross-task sharing.
- `blocking offload diagnostics` (completed): provide workload annotations and async-context diagnostics, explicit blocking offload through `task.spawn_blocking`, and `BlockingTask[T, E]` result-abandonment cancellation semantics. The later production concurrency/runtime substrate removed public `sifr.concurrent` and `sifr.threading` compatibility surfaces in favor of native `sifr.task`, `sifr.sync`, `sifr.runtime`, and `sifr.parallel` APIs.
- `async context-manager and iterator surface`: implement user-defined async context managers, `AsyncIterator[T, E]`, and `async for` over protocol-conforming streams.
- `async generator surface`: implement `AsyncGenerator[T, E]`, async generator lifecycle/cleanup, and list/set/dict async comprehensions.

**production runtime readiness:** the terminal architecture audit for the production concurrency/runtime substrate lives in [structured_runtime_work_model.md](./structured_runtime_work_model.md#runtime-substrate-audit). It locks the task/process/channel/offload/runtime boundaries, typed IPC policy, blocking/offload policy, sendability/shareability rules, task/request context model, diagnostics/signal global-state policy, and rejected CPython-shaped surface index without reopening the async/runtime architecture async syntax rules.

### 9. Destruction and Cleanup Semantics

Sifr compiles to Rust, which has deterministic destruction (RAII). This rules defines when and how values are cleaned up.

**Rules:**

- **Scope-end destruction:** values are dropped at the end of their enclosing scope, in reverse declaration order. This matches Rust's `Drop` semantics and is deterministic (unlike Python's GC).
- **Move invalidates source:** when a value is moved (assigned to another variable, or passed to a function via `own` parameter), the source is invalidated. Accessing it after move is a compile-time error. Note: default function parameters borrow (`&T`), so passing a value to a function does NOT move it unless the parameter is marked `own`.
- **Partial moves:** when a struct field is moved out, the entire struct becomes partially invalid. The compiler tracks which fields are still valid.
- **User-defined destructors deferred:** Sifr does NOT expose `__del__` or custom destructors in MVP. The compiler auto-generates `Drop` for types that hold resources (file handles, connections) via stdlib wrappers.
- **Explicit cleanup via `with`:** for resource management (files, connections), use `with` blocks that map to Rust's scoped resource patterns. The resource is cleaned up when the `with` block exits. The `with` statement calls `__enter__()` at scope start and `__exit__()` at scope end, with compile-time enforcement of the `ContextManager` protocol.
- **Destructor failure:** auto-generated destructors do not fail. If an underlying Rust `Drop` implementation panics (only possible via FFI-wrapped types), the program aborts. This is a system-level failure, not a Sifr-level concern -- Sifr user code cannot trigger destructor panics.

**Implementation responsibilities:**

- generators: define initial `with` block syntax (scoped block desugaring)
- compiler hardening work (stdlib parity architecture: Stdlib Parity): complete the `with` statement with full `ContextManager` protocol enforcement (`__enter__`/`__exit__` calls, multiple context managers, compile-time protocol checking)
- classes: implement scope-end destruction for class instances
- core_stdlib: implement `with` blocks for file handles and other stdlib resources

### 10. Auto-Derived Traits

Sifr auto-derives common Rust traits when the complete emitted representation
supports them. This is a language rule, not an implementation detail.

**Rules:**

- **Derived when the full representation supports the trait:**
  - `Debug` -- enables debug formatting. Derived when every field and embedded
    parent implements `Debug`; `NonSend` resource hierarchies are conservative
    non-`Debug` unless code generation owns a specific implementation.
  - `Clone` -- enables `.clone()`. Derived when all fields implement `Clone`.
  - `PartialEq` -- enables `==` and `!=`. Derived when all fields implement `PartialEq`.
- **Conditionally derived:**
  - `Eq` -- derived when `PartialEq` is derived AND no fields are `float` (since `f64` is not `Eq` in Rust due to `NaN`).
  - `Hash` -- derived when `Eq` is derived AND all fields implement `Hash`. NOT derived for types containing `float`, `dict`, or other unhashable types.
- **Not auto-derived (require explicit opt-in):**
  - `Ord` / `PartialOrd` -- comparison ordering requires explicit definition via `__lt__`, `__le__`, etc.
  - `Copy` -- only Rust-copy scalar primitives such as fixed-width integers, `float`, and `bool` are `Copy`. Source-level `int` is value-semantic but lowers to `SifrInt` and is not Rust `Copy`.
- **Inheritance:** trait capability includes the entire embedded parent chain,
  including transitive `NonSend` ancestry, not only fields declared by the
  immediate child. Auto-generated child formatting prints the embedded parent
  through that parent's emitted `Display` implementation rather than assuming
  that the child fields alone describe the Rust representation.
- **Generic classes:** generic declarations and constructors do not impose
  unrelated blanket Rust bounds. Conditional derives and generated formatting
  implementations carry only the bounds required by that trait, while ordinary
  methods and operator implementations carry bounds only on the type parameters
  used by their emitted consumers. A clone required for an `A` field therefore
  does not constrain an unrelated `B`. A specialization is rejected at the Sifr
  consumer that needs a bound its concrete type argument cannot satisfy. Import
  aliases retain a collision-safe local declaration identity across the whole
  imported type graph. Annotations, constructors, exported function and
  constant signatures, transitive ancestry, emitted Rust types, and
  specialization metadata therefore continue to refer to the same generic
  class even when factories and aliases appear in separate import statements.
  Class types carry their concrete specialization arguments separately from
  declaration identity, and those arguments are invariant: `Box[int]` is not
  assignable to `Box[str]`. A concrete annotated initializer may bind an
  otherwise-unresolved zero-argument generic return, while optional contextual
  binding matches the non-`None` payload rather than binding a type parameter
  to the complete union. Code generation renders the explicit class arguments
  as the authoritative Rust specialization and preserves the annotated local
  type when Rust needs result-context inference. Every generic class carries a
  compiler-owned non-owning `PhantomData<fn() -> T>` marker, so even a
  fieldless declaration has a valid, inhabited Rust representation without
  inheriting the argument's Rust auto traits. Concrete Clone, Debug, equality,
  and hash capability checks still include every type argument because Rust's
  generated derives impose those bounds. A generic field can therefore receive
  conditional `Eq` and `Hash` derives. Rust applies the required bounds to each
  concrete type argument. A container that is never hashable does not receive
  those derives.
- **Codegen:** the compiler emits only the derives supported by the complete
  generated struct, newtype, or union representation.
- **Enum types (enum type-system work):** enum types unconditionally derive `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`. All enum values are usable as dict keys and set members.
- **Auto-init (auto_init):** when a class has no explicit `__init__`, the compiler auto-generates `__init__`, `__eq__` (if all fields are `PartialEq`), and `__str__` (via `Debug`-style formatting). Explicit definitions always take precedence.
- **Hash-consumer constraint:** set elements and dictionary keys must implement
  `Hash + Eq`. The compiler enforces this for literals, membership, structural
  equality, `hash()`, and dictionary indexing, assignment, augmented assignment,
  deletion, and every non-empty `dict(...)` construction path. Specialized
  generic class instances are rejected at these consumers unless their emitted
  declaration provides the required traits.
- **Clone-consumer constraint:** collection operations that emit Rust cloning
  require recursive `Clone` support before HIR is accepted. This includes list
  copy/extend, dictionary key/value/item projections, dictionary copy/get/
  setdefault, and dictionary source construction; affine resources keep their
  more specific ownership diagnostic.
- **Comparison-consumer constraint:** list count/contains/remove/index require
  recursive `PartialEq`. In-place list sort and `sorted()` require total Rust
  ordering for the element type or callable-key return type used by the emitted
  sort path. Sifr's broader `Comparable`/`PartialOrd` surface does not by itself
  certify `slice::sort`; the dedicated floating-point path uses `total_cmp`,
  while other partial-only orderings are rejected. A `sorted()` key that borrows
  its element receives the comparator's shared reference without cloning. An
  owned key requires a Clone-capable element, mutable-borrow keys are rejected,
  and a preserved source requires Clone-capable elements for result
  materialization; consumed temporaries and iterators retain their elements.
  Conditional iterable sources are materialized branch-by-branch so preserved
  branches remain reusable and consumed branches are tracked exactly. Clone
  admission is therefore required when any reachable branch is preserved, even
  when another branch is a consumed temporary.
- **Generic method specialization constraint:** body-derived Clone, equality,
  partial-order, arithmetic, remainder, division, and negation requirements are
  recorded per method and per declared type parameter. Direct `self` calls close
  those requirements transitively, module exports preserve them, and concrete
  method and operator calls are rejected during lowering when a specialization
  cannot implement the Rust traits that code generation will emit. Generated
  `PartialOrd` implementations execute the declared `__lt__` body and include
  the exact `PartialEq` supertrait requirements of the class representation or
  its custom `__eq__`; generic negation is checked and emitted end-to-end.
- **User-class identity constraint:** nominal identity is the stable declaring
  module plus class, separate from the local spelling used by generated Rust.
  Function and constant signatures, class fields, generic templates, and the
  complete ancestry chain preserve that identity across independent import and
  re-export paths. Two aliases of one declaration therefore remain compatible
  even when the class and its factory travel through different facades, while
  same-named declarations from different modules remain incompatible.
  Imported-parent ancestry follows aliases to this canonical identity, and
  generated child structs implement `Deref`/`DerefMut` to their embedded parent
  so borrowed source-level subclass-to-base compatibility remains executable in
  Rust. Each direct embedding also implements `From<Child> for Parent`.
  Ownership-consuming arguments, local coercions, and returns emit one such
  conversion per ancestor and therefore move, rather than clone or borrow, the
  embedded parent representation across direct, transitive, imported, and
  re-exported upcasts. Structural value coercion recursively maps existing
  union, `Option`, and `Result` representations, and converts a raw payload
  before wrapping it in a target union. Shared-borrow arguments whose structural
  representation changes materialize a Clone-capable value first; mutable
  borrows reject such conversions. Transitive selection prefers the exact
  canonical ancestor identity; a local-name tail is used only when it identifies
  one unambiguous ancestor. Recursive ownership, Clone, equality, Hash, Debug,
  and task-sendability queries key class visits by canonical declaration
  identity plus concrete specialization, so same-basename imports and generic
  instantiations cannot mask one another.
  Generic callable parameters, bounds, and project codegen signatures likewise
  propagate through direct and multi-hop re-export facades.
- **Module return inference:** successful unannotated top-level return types are
  inferred as a mutually visible declaration group before body lowering, making
  forward calls source-order neutral. The prepass reaches a fixed point sized to
  the declaration group, preserves inferred unions, and ignores unreachable
  statement tails. It is diagnostic-neutral; normal reachability-aware body
  lowering remains authoritative for unresolved or dead return expressions.
  Generic calls in the prepass bind type variables from their arguments before
  substituting the return type; unresolved template variables are never
  accepted as a concrete inferred return.
- **Formatting-consumer constraint:** `print`, `str`, f-string interpolation,
  and `repr` validate the exact generated Rust `Display`/`Debug` strategy before
  accepting HIR. `repr` always requires `Debug`; the other surfaces select the
  same Display-versus-Debug path as code generation, including option members,
  recursively derived task/failure/timeout/select runtime wrappers, and the
  compiler-owned `JoinItemId` display implementation. `None` is emitted with
  its Python spelling directly rather than requiring Rust unit `Display`.

### 11. Diagnostic Mapping

Sifr compiles to Rust source code, which is then compiled by `rustc`. This creates a two-stage compilation where errors can originate from either the Sifr compiler or `rustc`. This rules defines how diagnostics are attributed, mapped, and rendered. The corrective semantic diagnostic taxonomy work amends the original diagnostic architecture rules and moves public diagnostic ownership into `crates/sifr_diagnostics`.

**Rules:**

- **Stable Sifr diagnostic codes:** every top-level Sifr compiler diagnostic has a stable family-local code of the form `SIFR-<FAMILY>-dddd`, for example `SIFR-NAME-0001`. Families identify the semantic domain, not merely the compiler stage. Historical `E####`/`W####` and message-embedded pseudo-codes are removed before public stability.
- **Deterministic documentation URL:** every top-level diagnostic exposes `url = "https://docs.sifr.sh/errors/<CODE>"`. This URL is part of the stable rules and must render in `human` and `json` outputs. `compact` intentionally omits URLs unless a future documented verbose compact flag is added.
- **Canonical severity enum:** the shared diagnostic model uses exactly three top-level severities:
  - `Error` -- blocks compilation or the active command
  - `Warning` -- non-blocking but actionable
  - `Note` -- contextual top-level information such as `reveal_type(...)` output and recovery-cap summaries
- **Help and children:** help text is attached through `help` fields or `ChildSeverity::Help`; `Help` is not a top-level diagnostic severity. Diagnostic children are uncoded `Note` or `Help` messages.
- **Canonical diagnostic object:** target migrated parser, lowering, type checking, borrow checking, and codegen paths must emit `SifrDiagnostic` values from `sifr_diagnostics`. Source diagnostics require a `SourceSpan`; internal diagnostics are reserved for compiler failures without source mapping.
- **Canonical suggestion model:** suggestion payloads are structured logical suggestions with one or more text replacement edits plus applicability (`MachineApplicable`, `MaybeIncorrect`, `HasPlaceholders`, or `Unspecified`). Replacement text lives in suggestion edits, not duplicated help children.
- **Span mapping:** semantic diagnostics preserve byte ranges as `SourceSpan` values before rendering. `sifr_source` owns source text, line maps, and UTF-8/UTF-16/UTF-32 position conversion primitives. Renderers derive display paths, byte offsets, 1-based UTF-8 character line/column positions, source snippets, and related spans at the source-map boundary without defining a separate line-map authority. Codegen/rustc diagnostics use `.sifr` source mapping where available; unmapped compiler failures use `SIFR-INTERNAL-*`.
- **Producer/presentation boundary:** producers own canonical diagnostic identity, source spans, related spans, and structured context before a diagnostic reaches output formatting. `sifr_diagnostics` owns source-map rendering and the `human`, `json`, and `compact` presentation once producers have supplied canonical diagnostic data. Workspace and package discovery must attach resolver details as args/children on source-level import diagnostics instead of replacing source problems with workspace-discovery-specific codes.
- **Package diagnostic conversion:** `sifr_driver::diagnostics::render_package_diagnostic` is the shared package-to-rendered conversion path. It preserves `PackageDiagnostic.help` and useful `PackageDiagnosticOrigin` fields as JSON args while leaving diagnostics spanless when no honest source/config byte range is available.
- `**rustc` error translation:** when `rustc` emits an error on generated code, the driver translates it back to `.sifr` coordinates using the span map. If translation fails (e.g., error in compiler-generated boilerplate), the raw `rustc` error is shown with a note: "This error originated in the Rust compilation step."
- **Generation vs rendering separation:** semantic compiler layers construct diagnostics; renderer layers convert them to `human`, `json`, and `compact` presentation formats. Output mode selection must not change diagnostic ownership or semantics.
- **JSON renderer rules:** CLI `json` output preserves the existing `RenderedDiagnostic[]` transport and must preserve the shared diagnostic model fields without human-only lossy reformatting. The checked-in schema is generated from `sifr_diagnostics`.
- **CLI diagnostic-format rules:** the stable renderer flag surface is `--diagnostic-format human|json|compact`. Unknown values fail fast with exit code `2` before semantic compilation work starts.
- **CLI exit-code rules:** compiler commands return exactly:
  - `0` success (including warning-only outcomes)
  - `1` user-facing compile/check/test diagnostics
  - `2` CLI usage/configuration error
  - `3` internal compiler failure after panic/error boundary handling
- **Human renderer rules:** default `human` output is source-aware. It prints severity, code, message, primary file/line/column, source snippets, caret highlights derived from `DiagnosticSpanLine`, related spans, child notes/help, suggestions, and documentation URLs. Spanless internal diagnostics use an explicit no-source fallback.
- **Compact renderer rules:** `compact` is a stable line-oriented summary format for agents, CI summaries, and quick terminal scanning. It must:
  - show one severity-only summary line first
  - render one physical line per retained diagnostic after recovery limiting
  - keep the first four fields stable: severity abbreviation, code, location or `<unknown>`, and message
  - preserve deterministic diagnostic ordering
  - avoid source snippets, default URLs, help counts, and grouped `CompactKey`-style aggregation
- **Suppression policy:** `rustc` warnings on generated code are suppressed by default (generated code includes `#[allow(warnings)]`). Only `rustc` errors are surfaced to the user.
- **Multi-file rendering:** errors that span multiple `.sifr` files show each file's relevant snippet with labeled spans. Uses `miette` or `ariadne` for rich terminal rendering with colors, underlines, and related notes.
- **Diagnostic ownership:** the Sifr compiler should catch as many errors as possible before invoking `rustc`. Over time, the set of errors that reach `rustc` should shrink to near-zero as the type checker and borrow checker mature.
- **No split-brain rule:** `sifr_driver`, future editor integrations, and automation-facing adapters must consume diagnostics through the canonical frontend API. They may render or transport diagnostics differently, but they may not reimplement parse/lower/type-check logic or semantic diagnostic derivation.
- **Canonical frontend API minimum surface:** the shared frontend/query API established in frontend query architecture must expose one canonical project/context handle plus reusable entrypoints for: parse, lower, type-check, collect diagnostics, inspect project/module graph state, and request per-module/per-project analysis results. CLI, editor, and automation adapters may wrap this API, but they must not bypass it for semantic analysis.

**Implementation responsibilities:**

- core_language-type_system: basic span tracking (single-file, Sifr-native errors only)
- import semantics work: multi-file span tracking (import errors reference both files)
- diagnostic architecture diagnostics rules: structured diagnostic schema, stable renderers, and recovery policy
- frontend query architecture shared analysis/query architecture: canonical query/database-backed frontend API consumed by CLI and future tooling
- ffi: FFI-related `rustc` error translation (extern crate mismatches)
- developer-tooling work (developer tooling surface): editor/LSP parity validation and thin tooling adapter boundaries on top of `sifr_frontend`

### 12. Standard Protocol Primitives

Sifr defines a set of built-in protocols (traits) that are used across multiple future capabilities. This section records when each becomes available and what it maps to in Rust.

**Rules:**


| Protocol         | Rust Trait                                      | Available From                                                                      | Purpose                                                       |
| ---------------- | ----------------------------------------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `Comparable`     | `Ord` (+ `PartialOrd`, `Eq`, `PartialEq`)       | Protocol definitions, then generic bounds                       | Ordering for `sort()`, `min()`, `max()`, comparison operators |
| `Addable`        | `Add` (+ `Sum` for `sum()`)                     | Protocol definitions, then generic bounds                       | Arithmetic `+` operator, `sum()` built-in                     |
| `Display`        | `std::fmt::Display`                             | Class auto-derivation for `__str__`, plus explicit protocol impls | String representation via `str()`, f-strings, `print()`       |
| `ContextManager` | Custom trait (`__enter__`/`__exit__` -> `Drop`) | Syntax support plus compiler-enforced protocol rules            | `with` statement resource management                          |
| `Iterable`       | `IntoIterator` / iterable protocol             | codebase issue `first-class-lazy-iterators-and-python-iterable-protocol` (task runtime) | `iter(x)` entry boundary and protocol typing                 |
| `Iterator`       | `Box<dyn Iterator<Item = T>>` runtime surface  | codebase issue `first-class-lazy-iterators-and-python-iterable-protocol` (task runtime, builtin lowering in synchronization primitives) | `next(it)`, single-pass stateful iteration, lazy pipelines  |
| `Reversible`     | `DoubleEndedIterator` capability rule      | codebase issue `canonical-iteration-model-and-lazy-parity-readiness` (runtime architecture lock, task runtime implementation) | capability-gated `reversed(...)` semantics                    |
| `Hashable`       | `Hash` (+ `Eq`)                                 | classes (auto-derived)                                                    | Dict keys, set membership                                     |


**Semantics:**

- **Auto-derived protocols:** `Display`, `Hashable`, `Comparable` are auto-derived for classes where all fields implement the corresponding Rust trait (see rules #10: Auto-Derived Traits). Users can override with explicit `__str__`, `__hash__`, `__lt__` etc.
- **Pre-generics usage:** Before generics, protocols are used for operator overloading and dynamic dispatch (`&dyn Trait`). After generics, they become usable as generic bounds (`T: Comparable`).
- **Primitive types:** `int`, fixed-width integer types, `float`, `str`, and `bool` implement applicable protocols from the start. Under the integer-model amendment, `Addable` must model the operator output type; fixed-width scalar `+` returns exact `int`, so fixed-width types do not satisfy a generic `T + T -> T` rules through ordinary arithmetic. `float` does NOT implement `Comparable` (because `NaN` violates total ordering) -- this is a compile-time error, matching Rust's `f64` not implementing `Ord`.
- **Protocol composition:** a function can require multiple protocols via intersection bounds (generics): `def process[T: Comparable & Display](item: T)`.

**Implementation responsibilities:**

- classes: auto-derive `Display` and `Hashable` for classes with eligible fields
- protocols: define `Comparable`, `Addable`, `Display` as explicit protocols; enable operator overloading via protocol impl
- generics: enable protocols as generic bounds (`T: Comparable`)
- generators: define initial `with` block syntax (scoped block desugaring)
- first-class lazy iterator work: introduces first-class `Iterable[T]` / `Iterator[T]` typing and protocol execution plan (`iter`, `next`, generator rewrite, lazy builtin conversion)
- parity-extension waiver-reduction work: finalizes iterator-returning builtin/stdlib surfaces (`map` parity, approved `itertools` combinators, `re.finditer`, `glob.iglob`, `Path.iterdir/glob/rglob`) and retires broad lazy-waiver claims to narrow residual governance entries
- canonical iteration continuation work: freezes/implements capability-aware canonical iteration semantics across type system, HIR, codegen, generators, builtins, and stdlib adapters
- structured-data/class-surface parity-expansion work: locks bounded rules for `json`, `configparser`, `csv`, `collections`, `argparse`, `uuid`, `datetime`, `textwrap`, and `html` while keeping explicit permanent diffs (`json` dynamic hooks, timezone-db/tzinfo ecosystems, `Counter(**kwargs)`, dynamic csv registry mutation, argparse formatter ecosystems, package-wide html expansion)
- compiler hardening work (stdlib parity architecture: Stdlib Parity): define `ContextManager` protocol; enforce `with` statement compliance with `__enter__`/`__exit__` calls and compile-time protocol checking; fix `Callable`-as-struct-field (`Box<dyn Fn>`)
- generic type-system work (type-system architecture: Type System Completion): complete generic class field/method substitution; protocol bounds on type parameters (`T: Comparable & Display`)
- pattern-matching work (type-system architecture: Type System Completion): `match`/`case` syntax with exhaustiveness checking on union types, literal unions, optional types, class unions, and enum types
- enum type-system work (type-system architecture: Type System Completion): simple enum types with exhaustive pattern matching; enum values implement `Eq`, `Hash`, `Clone`, `Debug`

### Ecosystem Strategy

Sifr's standard library follows a thin wrapper plus declared interop strategy:

- **Thin wrappers (protocols-data_processing):** The stdlib provides Pythonic APIs over best-in-class Rust crates. The sifr compiler generates Cargo dependencies automatically. Users write Python-like code; the generated Rust uses `axum`, `polars`, `sqlx`, `tokio`, etc. directly.
- **Rust interop:** For crates not yet wrapped by stdlib modules, package authors expose Rust-backed Sifr declarations through declaration-level Cargo integration and checked bridge-compatible signatures. This gives Sifr access to the Rust ecosystem without using Rust's private ABI or runtime `dlopen` as the Rust path. <!-- rust-interop-rejected -->
- **Package ecosystem (ecosystem):** A package registry (`sifr.sh`) for sharing and reusing Sifr code, with incremental compilation for fast iteration.
- **No reinventing:** Sifr never reimplements what Rust already has. Every stdlib module wraps a proven Rust crate.

---

## Type System Design

### Core Types (Full)

```rust
enum Type {
    // Exact integer (value-semantic at source, not Rust Copy)
    Int,

    // Fixed-width integer primitives (Copy)
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    ISize,
    USize,

    // Other primitives
    Float,
    Bool,
    Str,
    None,

    // Compound (Move)
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Set(Box<Type>),

    // Literal types -- specific values as types (type_system)
    LiteralInt(SifrIntLiteral),
    LiteralStr(String),
    LiteralBool(bool),

    // Union / Intersection (type_system)
    Union(Vec<Type>),           // int | str -- flattened, deduplicated
    Intersection(Vec<Type>),    // internal only, for narrowing engine

    // Type alias (type_system)
    Alias(String, Box<Type>),   // type HttpMethod = "GET" | "POST"

    // Function
    Function(FunctionType),

    // Async/concurrency model (async/runtime architecture)
    Coroutine(Box<Type>, Box<Type>),  // Coroutine[T, E] -- linear async computation, consumed by await or spawn
    Task(Box<Type>, Box<Type>),       // Task[T, E] -- awaitable task handle, yields TaskResult[T, E]
    TaskResult(Box<Type>, Box<Type>), // TaskResult[T, E] -- Ok(T), Err(Failure[E]), Cancelled(Failure[CancellationError])
    BlockingTask(Box<Type>, Box<Type>), // BlockingTask[T, E] -- blocking offload handle
    Awaitable(Box<Type>),             // structural awaitability protocol
    AsyncFunction(FunctionType),       // async callable, not a subtype of sync Function
    AsyncIterator(Box<Type>, Box<Type>), // AsyncIterator[T, E] -- anext() yields Result[Option[T], E]
    AsyncGenerator(Box<Type>, Box<Type>), // AsyncGenerator[T, E] -- async def with yield

    // Class instance (classes)
    Instance(ClassId),

    // Generics (generics)
    TypeVar(TypeVarId),
    GenericInstance(ClassId, Vec<Type>),

    // Result / Option (error_handling)
    Result(Box<Type>, Box<Type>),

    // Enum (enum type-system work)
    Enum(EnumId),

    // Range (control_flow)
    Range,

    // Protocol iteration model (first-class lazy iterator work)
    Iterable(Box<Type>),
    Iterator(Box<Type>),

    // Safe top type: must be narrowed before use (type_system)
    Unknown,

    // Escape hatch: opts out of type checking
    Any,

    // Bottom
    Never,
}
```

### Literal Type Behavior (TypeScript-inspired)

Literal types represent specific values at the type level. In sifr, values are used directly as types in type position (TypeScript style), avoiding Python's verbose `Literal[...]` wrapper:

```python
type HttpMethod = "GET" | "POST" | "PUT"    # not Literal["GET"] | Literal["POST"] | ...
type StatusCode = 200 | 404 | 500
x: "hello" = "hello"                        # literal type annotation
```

Key behaviors:

- **Fresh literals widen at mutable locations:** `x = 42` infers `x: int` (widened), but `x: 42 = 42` preserves the literal type
- **Literal types are subtypes of their base type:** `42` is assignable to `int`, `"GET"` is assignable to `str`
- **Equality narrows to literals:** `if x == "GET":` narrows `x: str` to `x: "GET"` in the then-branch
- **Union of literals:** `"GET" | "POST"` is a valid type representing exactly two string values

### Union Type Behavior

- **Flattened:** `Union(vec![Union(vec![A, B]), C])` normalizes to `Union(vec![A, B, C])`
- **Deduplicated:** `Union(vec![Int, Int, Str])` normalizes to `Union(vec![Int, Str])`
- **Single-element unions collapse:** `Union(vec![Int])` becomes `Int`
- **Subtyping:** `A` is assignable to `A | B`; `A | B` is assignable to `C` only if both `A` and `C` and `B` and `C` are assignable
- **Codegen:** `int | str` generates a Rust enum with one variant per runtime representation, e.g. `enum IntOrStr { Int(SifrInt), Str(String) }` under the integer-model amendment.

### Type Narrowing (TypeScript-inspired, type_system)

Narrowing refines a variable's type within a control flow branch:

- **Truthiness:** `if x:` removes `None` and falsy types from unions
- **isinstance:** `if isinstance(x, int):` narrows `x: int | str` to `x: int`
- **Equality:** `if x == "GET":` narrows to literal type
- **is None / is not None:** narrows optional types
- **Type predicates:** `def is_str(x: int | str) -> TypeGuard[str]:` enables user-defined narrowing
- **Assertion functions:** `def assert_int(x: int | str) -> AssertType[int]:` narrows after call
- **Exhaustiveness:** after narrowing all variants of a union, the remaining type is `Never` -- compiler error if not exhaustive

### Ownership Model

- All types are **move by default** for assignment (like Rust)
- Fixed-width integer types, `float`, and `bool` are Rust `Copy` values in generated code. Source-level `int` remains scalar and value-semantic, but it lowers to `SifrInt` and is not Rust `Copy`; codegen preserves non-consuming source behavior with borrowing, cloning, or primitive-local optimization.
- Compound types (`str`, `list`, `dict`, classes) **move** on assignment
- Explicit `.clone()` for deep copy
- Function arguments: **borrow by default** (maps to `&T` for Move types)
- Mutable borrow via `mut` keyword on parameters (maps to `&mut T`)
- Ownership transfer via `own` keyword on parameters (maps to `T`)
- Explicit `.clone()` for deep copy when returning or storing borrowed values

### Type Inference Strategy

- **Initializer inference:** `x = 42` infers `x: int` (literal widens to base type)
- **Return type inference:** analyze all return paths
- **Contextual typing (generics):** lambda/callback parameter types inferred from call-site context. E.g., `map_list(numbers, lambda x: x * 2)` infers `x: int` from the `list[int]` argument. Inspired by TypeScript's contextual typing which looks upward in the tree for type annotations.
- **Enforced annotations:** function parameters MUST have types (or be inferable from defaults)
- **Literal preservation:** `x: "GET" = "GET"` preserves the literal type; `x = "GET"` widens to `str`
- **Empty collection inference:** `x = []` and `x = {}` are compile-time errors -- the element type cannot be inferred. Users must annotate: `x: list[int] = []`, `x: dict[str, int] = {}`. This prevents accidental `list[Unknown]` and matches Rust's requirement for explicit types on empty collections.

---

## Test Suite Architecture

This compiler is built entirely by AI agents. The test suite is the rules that ensures correctness across all agents working on different parts of the compiler. It must be:

- **Deterministic:** same input always produces same output
- **Self-documenting:** test files are readable specifications of language behavior
- **Layered:** each compiler stage has its own test layer
- **Easy to extend:** adding a new language feature means adding test files, not modifying test infrastructure
- **Fast to run:** `cargo test` completes in seconds for the full suite

### Testing Strategy Overview

```mermaid
flowchart TD
    subgraph layer1 [Layer 1: Unit Tests]
        LexerUnit["Lexer unit tests\n(token output)"]
        ASTUnit["AST node tests\n(construction, size)"]
        TypeUnit["Type system tests\n(subtyping, inference)"]
    end
    subgraph layer2 [Layer 2: Snapshot Tests]
        ParseSnap["Parser snapshots\n(.sifr -> AST dump)"]
        TypeSnap["Type checker snapshots\n(inline assertions)"]
        CodegenSnap["Codegen snapshots\n(.sifr -> .rs output)"]
    end
    subgraph layer3 [Layer 3: End-to-End Tests]
        E2EPass["Compile + run tests\n(expected stdout)"]
        E2EFail["Compile-fail tests\n(expected errors)"]
        E2EOwnership["Ownership tests\n(move/borrow errors)"]
    end
    subgraph layer4 [Layer 4: CPython Parity Tests]
        CPythonParity["CPython parity tests\n(match behavior, safe errors)"]
        SafetyTests["Safety tests\n(no panics, Result/Option)"]
    end
    subgraph layer5 [Layer 5: Corpus Tests]
        Corpus["Corpus tests\n(no panics on large inputs)"]
    end
    subgraph layer6 [Layer 6: Fuzz + Property Tests - generics plus]
        FuzzParser["Parser fuzz\n(cargo-fuzz)"]
        FuzzChecker["Type checker fuzz\n(random ASTs)"]
        PropTests["Property tests\n(algebraic invariants)"]
    end
    subgraph layer7 [Layer 7: Performance Tests - generics plus]
        CompileBench["Compile-time benchmarks\n(criterion)"]
        BinarySizeBench["Binary-size benchmarks"]
    end
    layer1 --> layer2 --> layer3 --> layer4 --> layer5 --> layer6 --> layer7
```

### Layer 1: Unit Tests (per crate, `#[cfg(test)]`)

Standard Rust unit tests inside each crate. These test individual functions and data structures.

**Where:** `src/*.rs` in each crate, in `#[cfg(test)] mod tests { }` blocks.

**Examples:**

- Lexer: tokenize a string, assert token sequence
- AST: construct nodes, verify `Debug` output, check memory layout sizes
- Type system: `is_subtype(Int, Int) == true`, `is_subtype(Int, Str) == false`
- HIR: name resolution resolves `x` to the correct `DefId`

**Pattern (from ruff_python_ast):**

```rust
#[test]
fn size() {
    assert!(std::mem::size_of::<Stmt>() <= 120);
    assert_eq!(std::mem::size_of::<Expr>(), 64);
}
```

### Layer 2: Snapshot Tests (insta crate)

Snapshot testing using the `insta` crate. The compiler produces output that is compared against stored `.snap` files. When behavior changes intentionally, run `cargo insta accept` to accept new baselines.

**Crate:** `insta` with `glob` feature.

#### 2a. Parser Snapshots

**Inspired by:** ruff_python_parser's fixture-driven snapshot tests.

**Directory structure:**

```
third_party/ruff/crates/ruff_python_parser/
  resources/
    valid/          # .sifr files that must parse successfully
    invalid/        # .sifr files that must produce parse errors
  tests/
    snapshots/      # auto-generated .snap files
    fixtures.rs     # test harness
```

#### 2b. Type Checker Snapshots (Markdown Tests)

**Inspired by:** ty's mdtest framework -- Markdown files with inline assertions.

**Assertion syntax:**

- `# revealed: <type>` -- assert inferred type (like ty)
- `# error: [rule-code] "optional message"` -- assert diagnostic
- `# error: <col> [rule-code]` -- assert diagnostic at specific column

#### 2c. Codegen Snapshots

**Inspired by:** TypeScript's `.js` baseline files. Compile `.sifr` to `.rs` and snapshot the output.

### Layer 3: End-to-End Tests (Compile + Run)

**Inspired by:** Mojo's Lit + FileCheck pattern, adapted for Rust.

These tests compile `.sifr` files to binaries and run them. Runtime validation in pass fixtures is now assertion-first (`assert ...`), using no `# expect-stdout` directives.

**Directory structure:**

```
tests/
  e2e/
    pass/           # must compile and produce expected output
    fail/           # must fail to compile with expected errors
    ownership/      # ownership-specific compile failures
  e2e.rs            # test runner
```

**Test file format (pass tests, assertion-first):**

```python
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    assert factorial(5) == 120
```

**Test file format (fail tests):**

```python
def main():
    x: int = "hello"  # expected to fail at compile-time
```

### Layer 4: CPython Parity and Safety Tests (ergonomics+)

Verify that Sifr's built-in functions, data structure methods, and stdlib modules match CPython's behavior -- but with safe error handling.

**Reference:** `/Users/yaseralnajjar/work/sifr/cpython` -- specifically `Lib/test/test_<module>.py` for expected behavior.

### Layer 5: Corpus Tests (Robustness)

Run the parser and type checker on a large body of Python source code to catch panics, infinite loops, and crashes. These tests don't check correctness -- only that the compiler doesn't blow up.

### Layer 6: Fuzz and Property Tests (generics+)

Discover edge cases and crashes that hand-written tests miss. Use `cargo-fuzz` or `afl` for parser/type checker fuzzing. Property tests verify algebraic invariants (union normalization idempotent, subtyping reflexive/transitive, narrowing preserves subtyping).

### Layer 7: Performance Regression Tests (generics+)

Prevent compile-time and binary-size regressions. Use `criterion` for statistical benchmarking. Regressions beyond threshold block PRs.

### Parser Fixture Migration Plan

The parser snapshot tests currently use `.py` fixtures inherited from ruff. These should be incrementally migrated to `.sifr` fixtures as the language diverges from Python. Start in error_handling when the first non-Python syntax is introduced. Complete by generics.

### Test Infrastructure Crate: `sifr_test_utils`

A shared crate providing test helpers for compile-failure expectations, compilation, execution, and markdown tests. Runtime expectations in pass fixtures use Sifr `assert` statements.

### Test Commands

```bash
cargo test                                    # Run all tests (layers 1-3)
./scripts/run_all_tests.sh --profile create-pr # Fast local-first profile
./scripts/run_all_tests.sh --profile merge   # Authoritative merge gate
./scripts/run_all_tests.sh --profile nightly # Broad nightly validation profile
./scripts/run_all_tests.sh --profile release # Highest-confidence local qualification profile
./scripts/run_all_tests.sh --profile python-interop-live # Explicit opt-in container-runtime Python interop profile
uv run --project verification --locked python -m sifr_verify areas run --area distribution_release --suite full     # Preview installer/artifact/release automation checks
uv run --project verification --locked python -m sifr_verify areas run --area distribution_release --suite incident-governance # Offline rollback/roll-forward generation, retention, and recovery
uv run --project verification --locked python -m sifr_verify areas run --area distribution_release --suite epoch-bootstrap # One-time opaque schema-v2 preview epoch bootstrap contracts
uv run --project verification --locked python -m sifr_verify areas run --area distribution_release --suite protected-drill # Credential-free GA/normal/rollback/first-GA orchestration under local adapters
./verification/runner/e2e/check_report_determinism.sh --profile release # Stable e2e report signature across reruns
uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite cargo-smoke --suite property --suite fuzz-smoke
cargo test --manifest-path third_party/ruff/Cargo.toml -p ruff_python_parser # Parser snapshots
cargo test -p sifr_type_system -- mdtest      # Type checker markdown tests
cargo test -p sifr_codegen                    # Codegen snapshots
cargo test --test e2e                         # End-to-end tests
cargo insta accept                            # Update snapshots after intentional changes
cargo test -- corpus --ignored                # Run corpus tests (slower, layer 4)
cargo fuzz run parser_fuzz -- -max_total_time=300  # Run fuzz tests (layer 5, generics+)
cargo bench                                   # Run benchmarks (layer 6, generics+)
```

Validation profile policy is defined in `verification/profiles/{create-pr,merge,nightly,release,python-interop-live}.json` and executed by `verification/runner/sifr_verify/profile_runner.py` through `uv run --project verification python -m sifr_verify profiles run --profile <profile>`. `scripts/run_all_tests.sh` is only the stable public facade over that runner. Verification areas are owned by schema-version-2 `verification/areas/*/manifest.json` files and executed through `uv run --project verification python -m sifr_verify areas run`. Stable-surface manifests declare owner, resource classes, pinned-corpus policy, skip policy, and baseline metadata policy. Representative `create-pr` and full-corpus `merge` e2e coverage are selected through profile data rather than hard-coded shell assumptions. The `python-interop-live` profile selects only the live Python interop area and explicitly opts into `container-runtime` and live-network policy for testcontainers-backed Python interop evidence. Offline profiles must not select those live suites. Its live examples build native Sifr binaries for Redis, Postgres, Kafka-compatible Redpanda, and LocalStack Pub/Sub-style SNS fanout, SNS-to-SQS delivery, and direct SQS delivery. Testcontainers owns container lifecycle and endpoint discovery only; the compiled binary's hermetic declaration bridge owns every service-client operation, and broker/cloud deliveries cross a foreign-thread typed Sifr callback. Docker absence is recorded as a structured service-execution skip only after every native binary builds. When a mostly offline area has a live subset, the area can keep top-level `network_mode: offline` while the live suite declares suite-level `network_mode: live` and its resource classes. Declarative validation-suite coverage lives in area-owned validation suite manifests under `verification/areas/{core_language,project_workspace}/data/validation_suites/`; profiles select the individual suite names, and the area adapter invokes the Rust-native `tests/validation_suites.rs` harness with that exact suite filter. Fixed bug locks and unresolved crash sentinels live under `verification/areas/regression/`.

Stable incident recovery is a pure extension of the governed release-index
state machine. Canonical request and approved plan digests authorize exactly
one rollback or incident roll-forward generation; all prior releases and
snapshots remain immutable. Stable activation, normal publication, rollback,
and incident roll-forward share one protected publication workflow, one
`stable-release` environment boundary, and one `sifr-release-index` mutation
lease. Read-only prepare and protected revalidation bind the exact evidence,
plans, live index, retained snapshots, candidate artifacts where applicable,
and proposed index before the sole mutable `channels.json` replacement.
Generation reservation, exact resume, site reconciliation, public
install/update/recovery smoke, Marketplace verification, and release/incident
sign-off remain fail-closed. The one-time schema-epoch bootstrap is separately
bound to the exact opaque pre-epoch asset identity and protected approval. The
user-directed single-maintainer exception for that bootstrap and first GA is
itself a canonical, expiring governance artifact. It permits only the named
owner and those three operations, requires a real `stable-release` approval,
is pinned by digest, prefers a distinct approval when one is available, and binds
the selected approval policy plus initiator into retained evidence; normal and
incident operations require distinct approval. The
post-index bootstrap recovery path revalidates the failed mutation and site
attempts, both protected approvals, the already-live generation-1 bytes, and
the reproducible site inputs before retrying only site publication and public
smoke; it cannot create a release, allocate a generation, or replace the
index. The offline fixture core mirrors generation burning, exact resume, site
reconciliation, range eligibility, downgrade consent, and incident sign-off
evidence, as specified in
`internal_docs/stable_incident_response.md`.

The readiness coverage matrix is the executable registry for shipped guarantees, compiler surfaces, owners, profile assignments, and Cargo package/target/feature classification. `coverage_matrix:readiness` is selected by create-pr, merge, nightly, and release. It runs strict mode, rejects temporary statuses such as `expected-missing`, `tests:none`, and `red-blocker`, validates local-first profile policy, checks profile assignments against `profile_assignment_matrix.json`, and runs negative self-tests for the readiness enforcement claims. CI may run broader profiles, but local-vs-CI plan equivalence is checked by comparing emitted profile plans with `sifr_verify profiles compare-plans`.

`profile_runner.py` emits a per-profile runtime report under `target/validation_lane_reports/` (`<profile>.latest.json`, `<profile>.latest.log`, `<profile>.latest.time`). The report summarizes wall/CPU time, e2e compile-build-run timing, cache hits and rebuilt groups, group-skew tail behavior, cache footprints, default worker settings, and advisory resource signals such as swap activity or default-profile RSS regressions.

Blocking representative and full performance measurements use a controlled-host
boundary. Work-controlled measurements use retired instructions and
process-tree RSS from the local macOS host. Latency-controlled measurements use
elapsed time and generic RSS after a quiet-host admission. Non-macOS profiles
retain latency mode because they do not expose the Darwin counter. Both modes reject
competing build work, thermal pressure, and unstable samples. Darwin work mode
reserves 60% of logical CPU capacity at admission. During the attempt, external
CPU pressure is blocking when retired-instruction samples are unstable and is
advisory when those work samples remain stable. This avoids requiring an idle
desktop without weakening the work budget. The accepted report includes an
invocation identity. Producer
failure cannot use stale `*.budget.latest.json` evidence. This design separates
host delay from additional compiler work without changing elapsed-time budgets.

Approved trend-baseline refreshes use the same controlled producer but are a
separate governance path from threshold-budget updates. A refresh is accepted
only for a clean exact commit, the complete benchmark manifest, manifest sample
counts, and the `compiler/performance` approval profile. The checked-in trend
snapshot records environment metadata and an exact reference-capture receipt;
raw host observations stay in content-addressed target evidence. This replaces
expired freshness or metadata deferrals with measured evidence without
weakening the blocking budget baseline.

### Test Placement

Language features have parser fixtures in `resources/valid/` and `resources/invalid/`, type-checker cases in `resources/mdtest/`, code-generation fixtures in `resources/codegen/`, and end-to-end cases in `tests/e2e/`. Snapshot baselines are maintained with `cargo insta accept`.

---

## Design Note: Mojo Comparison

Mojo (`/Users/yaseralnajjar/work/sifr/modular/mojo`) was evaluated as a reference. Key findings:

- **No Rust code to reuse.** Mojo's compiler is proprietary, built on MLIR/LLVM (C++). The open-source repo only contains the stdlib, docs, and design proposals.
- **Ownership model alignment:** Both Mojo and Sifr use **borrow-by-default** for function arguments. Sifr uses `mut` for mutable borrows, `own` for ownership transfer, and `own mut` for owned mutable parameters (Mojo uses `mut`/`owned`). Assignment still moves for heap types (preventing aliasing). This gives Python-like ergonomics with Rust-like safety.
- **Useful design references:** `proposals/value-ownership.md` and `proposals/lifetimes-and-provenance.md` document tradeoffs between move/borrow defaults, ASAP destruction, and lifecycle methods.
- `**def` vs `fn` split:** Mojo uses `def` for dynamic and `fn` for strict. Sifr does not need this split since all code is strictly typed.

## Key Files to Reference During Implementation

### Ruff (parser, AST)

- **Ruff parser:** `third_party/ruff/crates/ruff_python_parser/`
- **Ruff AST:** `third_party/ruff/crates/ruff_python_ast/src/nodes.rs`

### ty (type checker)

- **ty type system:** `/Users/yaseralnajjar/work/sifr/ty/ruff/crates/ty_python_semantic/src/types.rs`

### TypeScript (type system design, narrowing, control flow analysis)

- **Checker architecture:** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker.md`
- **Type narrowing and widening:** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker-widening-narrowing.md`
- **Type relations (subtyping, assignability):** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker-relations.md`
- **Type inference:** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/checker-inference.md`
- **Binder (control flow graph):** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/binder.md`
- **Type definitions:** `/Users/yaseralnajjar/work/sifr/TypeScript-Compiler-Notes/codebase/src/compiler/types.md`
- **TypeScript wiki:** `/Users/yaseralnajjar/work/sifr/TypeScript.wiki/`

### Mojo (ownership model)

- **Mojo ownership design:** `/Users/yaseralnajjar/work/sifr/modular/mojo/proposals/value-ownership.md`
- **Mojo lifetimes design:** `/Users/yaseralnajjar/work/sifr/modular/mojo/proposals/lifetimes-and-provenance.md`
