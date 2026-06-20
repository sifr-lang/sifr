# Rust Interop Architecture

Status: design locked for Phase 39 planning.

This document defines Sifr Rust interop as declaration-level Cargo integration. It is a self-contained design for Rust interop and does not depend on the Python interop lane, raw C ABI interop, or earlier interoperability drafts.

## Goals

Rust interop must make Rust implementation code feel like normal Sifr packages while preserving Sifr's core guarantees:

- compiled Sifr source still lowers to generated Rust and builds through Cargo,
- users import normal Sifr APIs, not Rust symbols,
- all foreign effects are explicit in Sifr declarations,
- user-triggerable runtime panics do not cross into Sifr user code,
- blocking, async, CPU-heavy, unsafe, and native-linking behavior is visible to the compiler,
- bridge-compatible Rust signatures are checked before build, not discovered by trial and error,
- there are no backward-compatible fallback paths or compatibility shims for abandoned designs.

The smooth path is:

```text
Sifr user code
  -> normal Sifr import
  -> Sifr declaration annotated with @rust(...)
  -> generated Rust glue
  -> Cargo dependency or package-local bridge
  -> Rust implementation crate
```

Sifr does not use Rust's private ABI, `dlopen` Rust functions, or call arbitrary symbols. Rust interop is source-level integration through Cargo.

## Non-Goals

- No `extern rust` language lane.
- No Deno-style runtime dynamic-library layer for Rust.
- No Rust native ABI dynamic linking.
- No raw pointers, arbitrary Rust lifetimes, trait objects, unconstrained generics, closures, or `unsafe fn` in the public Sifr-facing bridge contract.
- No silent copy fallback for zero-copy declarations.
- No hidden Tokio runtime, hidden `block_on`, or implicit blocking offload.
- No backwards-compatible support for previous draft syntax.
- No fallback behavior when bridge checking fails; the diagnostic must name the rejected contract and the required declaration or bridge fix.

Raw C ABI interop is a separate unsafe lane. Python interop is a separate embedded CPython lane. Rust crates that call C internally remain normal Rust backend crates from Sifr's perspective.

## User Model

Rust-backed APIs are normal Sifr APIs:

```python
from fast_hash import hash_bytes

def main() -> Result[None, HashError]:
    digest = try hash_bytes(b"hello")
    print(digest.hex())
```

The package author declares the Rust implementation boundary:

```python
class HashError(Error):
    message: str

@rust(bridge.blake3.hash_bytes)
def hash_bytes(input: bytes) -> Result[bytes, HashError]:
    pass
```

The Rust bridge lives in package source, not generated output:

`Error` is Sifr's canonical error base type defined by the error-safety contract in [`08_error_safety.md`](../plans/phases/08_error_safety.md); generated bridge error types follow the same `Bridge` naming and module placement rules as records and closed enums.

```rust
// src/bridges/blake3.rs
use crate::__sifr_bridge::fast_hash::HashErrorBridge;

pub fn hash_bytes(input: &[u8]) -> Result<Vec<u8>, HashErrorBridge> {
    Ok(blake3::hash(input).as_bytes().to_vec())
}
```

Consumers do not see the Rust bridge unless they are maintaining the package.

Bridge authors import generated Sifr bridge types from `crate::__sifr_bridge::<sifr_module_path>::<Name>Bridge`. The `Bridge` suffix is reserved for generated interop types; user-authored bridge modules must not define public items with that suffix in the generated bridge namespace.

## Declaration Syntax

Rust interop uses decorators on normal Sifr declarations.

### Direct Function Binding

```python
@rust(crc32fast.hash)
def crc32(data: bytes) -> uint32:
    pass
```

`crc32fast.hash` resolves to a direct Cargo dependency root named `crc32fast` and Rust path `crc32fast::hash`.

Direct binding is allowed only when the target signature is bridge-compatible without adaptation. If the Rust crate exposes a non-compatible API, the package author must write a package-local bridge.

### Package-Local Bridge Binding

```python
@rust(bridge.tokenizer.encode)
def encode(text: str) -> Result[Tokenized, TokenizeError]:
    pass
```

`bridge.tokenizer.encode` resolves to `crate::bridges::tokenizer::encode` in the generated package crate. Sifr owns projection entries for `src/bridges/mod.rs`, but user-authored bridge files remain user files and must never be overwritten silently.

### Shared Bridge Crate Binding

```python
@rust(sifr_arrow_bridge.record_batch_from_columns)
def record_batch(columns: dict[str, ArrowArray]) -> Result[ArrowRecordBatch, ArrowError]:
    pass
```

Shared bridge crates are ordinary Cargo dependencies that publish stable adapter APIs for common Rust ecosystems.

### Opaque Types

```python
@rust.opaque(
    type=bridge.kafka.Consumer,
    send=false,
    sync=false,
    clone=none,
    close=async_close,
    borrow=exclusive,
    thread_affinity=tokio_current_thread,
)
class KafkaConsumer:
    @rust(Self.poll)
    def poll(self) -> Result[Message, KafkaError]:
        pass

    @rust(bridge.kafka.consumer_aclose)
    async def aclose(self) -> Result[None, KafkaError]:
        pass
```

Decorator values are symbolic values, not strings. The opaque class declares ownership, close semantics, thread behavior, and borrowing rules before generated glue can expose the handle.

Opaque decorator keys are fixed:

- `type=` names the Rust type wrapped by the opaque handle.
- `send=` and `sync=` declare whether the generated Sifr handle may cross Sifr task/thread boundaries.
- `clone=` declares generated clone behavior.
- `close=` declares cleanup behavior.
- `borrow=` declares the default method receiver mode: `shared` lowers `Self.method` to `&self`, `exclusive` lowers to `&mut self`, and `owned` lowers to a consuming `self` call.
- `thread_affinity=` declares runtime or OS-thread affinity that generated glue must preserve.

## Path Resolution

`@rust(...)` accepts a dotted path expression with one of these roots:

| Root | Meaning |
| --- | --- |
| Cargo dependency name | Direct binding to an ordinary Cargo dependency. |
| `bridge` | Package-local bridge module under `src/bridges`. |
| Shared bridge crate name | Direct binding to a bridge crate declared as a Cargo dependency. |
| `Self` | Method binding inside a `@rust.opaque` class. |

Resolution is compiler-owned and deterministic:

1. Parse decorator paths as structured AST nodes, not strings.
2. Resolve the root against direct Cargo dependencies, generated bridge modules, or the current opaque type.
3. Validate that the resolved item exists.
4. Validate the Rust signature against the Sifr declaration.
5. Emit generated Rust glue and structured `InteropBuildPlan` metadata.

Same-workspace crates are not special. They must still be declared as ordinary Cargo dependencies with a package, path, git, registry, and feature set.

### `Self` Resolution

Inside a `@rust.opaque` class, `Self.method` resolves against the Rust type declared by `@rust.opaque(type=...)`.

Resolution order is:

1. inherent methods on the resolved Rust type,
2. diagnostic failure when the target is missing or is not an inherent method.

For the `KafkaConsumer.poll` example above, `Self.poll` lowers to an inherent call on the wrapped `bridge.kafka.Consumer` value:

```rust
crate::bridges::kafka::Consumer::poll(&mut handle.inner)
```

Trait methods are not resolved through `Self` because Rust trait lookup depends on imports and can become ambiguous. Package authors expose trait-backed behavior through a package-local bridge shim:

```python
@rust(bridge.kafka.consumer_aclose)
async def aclose(self) -> Result[None, KafkaError]:
    pass
```

Missing, trait-only, ambiguous, or non-method `Self` targets produce `SIFR-RUST-RESOLVE-*` diagnostics with the opaque class span and target decorator span.

## Package Layout

A Rust-backed package may use this shape:

```text
fast-tokenizer/
  Cargo.toml
  sifr.toml
  src/
    __init__.sifr
    bridges/
      mod.rs              # Sifr-managed projection region
      tokenizer.rs        # user-authored Rust bridge
  backend/
    Cargo.toml
    src/lib.rs
```

`Cargo.toml` owns Rust dependency resolution. `sifr.toml` owns Sifr compiler semantics and trust policy.

```toml
[rust]
bridge-version = 1
bridges = ["src/bridges"]
direct-crate-bindings = true

[trust]
rust-build-scripts = ["tokenizer_backend"]
rust-proc-macros = []
native = ["tokenizer_backend"]
unsafe-rust-bridges = ["src/bridges/tokenizer.rs"]
build-env = ["LIBTOKENIZER_PATH"]
```

Sifr may generate projection modules and guarded regions, but it must not overwrite user-authored Rust bridge files without an explicit diagnostic and user action.

Backend crates are linked statically as ordinary Rust library crates. The supported backend crate type is `lib`; `cdylib`, `dylib`, and runtime-loaded backend crates are rejected for the Rust interop lane because they imply a dynamic ABI boundary.

`bridge-version = 1` is the schema version for generated bridge modules, generated bridge type naming, decorator lowering, and runtime glue contracts. A package compiled by a compiler that does not support the declared bridge version fails during package validation.

## Compiler Model

Codegen returns interop metadata beside generated Rust:

```rust
pub struct GeneratedBinaryProject {
    pub main_rs: String,
    pub support_modules: BTreeMap<String, String>,
    pub used_stdlib_modules: HashSet<StdlibModule>,
    pub required_features: HashSet<StdlibFeature>,
    pub interop: InteropBuildPlan,
}

pub struct InteropBuildPlan {
    pub rust: RustInteropPlan,
}
```

The Rust plan records:

- decorator source spans,
- resolved Rust target paths,
- direct Cargo dependency roots,
- local bridge module paths and source digests,
- bridge signature requirements,
- opaque handle layouts,
- async/blocking classifications,
- zero-copy/view contracts,
- callback contracts,
- panic and trust requirements,
- Cargo feature and package metadata that must enter the build cache key.

The driver materializes generated glue from this plan. It must not scan emitted Rust text to infer dependencies.

## Bridge Type Contract

Sifr-facing Rust bridge signatures are intentionally small and explicit.

| Sifr type | Rust parameter | Rust owned parameter | Rust return |
| --- | --- | --- | --- |
| `bool` | `bool` | `bool` | `bool` |
| `int8`/`int16`/`int32`/`int64` | matching signed integer | matching signed integer | matching signed integer |
| `uint8`/`uint16`/`uint32`/`uint64` | matching unsigned integer | matching unsigned integer | matching unsigned integer |
| `float32` | `f32` | `f32` | `f32` |
| `float64` / `float` | `f64` | `f64` | `f64` |
| `int` | `&SifrIntBridge` | `SifrIntBridge` | `SifrIntBridge` |
| `str` | `&str` | `String` | `String` |
| `bytes` | `&[u8]` | `Vec<u8>` | `Vec<u8>` |
| `list[T]` | `&[T]` | `Vec<T>` | `Vec<T>` |
| `dict[str, T]` | `&interop::IndexMap<String, T>` | `interop::IndexMap<String, T>` | `interop::IndexMap<String, T>` |
| `Option[T]` | `Option<T>` | `Option<T>` | `Option<T>` |
| `Result[T, E]` | not a parameter | not a parameter | `Result<T, E>` |
| closed enum | generated bridge enum | generated bridge enum | generated bridge enum |
| record class | generated bridge struct | generated bridge struct | generated bridge struct |
| opaque class | `&Handle<T>` / `&mut Handle<T>` | `Handle<T>` | `Handle<T>` |
| `Callback[[...], R]` | generated call-scoped callback handle | not storable | not a return type |
| `ThreadsafeCallback[[...], R]` | generated thread-safe callback handle | generated thread-safe callback handle | not a return type |

Exact `int` is not a native ABI integer. `SifrIntBridge` lives in `sifr_runtime::interop` and is an owned, immutable, cloneable exact-integer value with `Eq`, `Ord`, `Hash`, `Send`, and `Sync`, no `Copy` implementation, and no `repr(C)` guarantee. Borrowed parameters use `&SifrIntBridge`; owned parameters and returns use `SifrIntBridge`. Bridges that need fixed-width storage or ABI layout must declare fixed-width integer types instead.

`dict[str, T]` preserves Sifr/Python insertion order through `sifr_runtime::interop::IndexMap`, a runtime re-export of the pinned `indexmap::IndexMap` version used by generated bridge glue. Non-`str` dict keys are not bridge-compatible until a later design defines stable hashing/equality and ordering semantics for those key types.

Nested borrowed forms are generated from the outer ownership mode. For example, borrowed `list[str]` is not `&[&str]`; it is a generated list view whose elements are borrowed string views with the same lifetime as the list view. `Option[str]` and `Option[bytes]` use generated optional borrowed views for borrowed parameters and owned `Option<String>` / `Option<Vec<u8>>` for owned parameters and returns.

Sifr container types not listed above, including `set[T]`, `tuple[...]`, and arbitrary iterator/generator types, are not bridge-compatible in Phase 39 and produce `SIFR-RUST-TYPE-*` diagnostics. No implicit conversion is allowed.

Rejected public bridge surfaces include:

- raw pointers,
- borrowed return values,
- arbitrary lifetimes,
- higher-ranked trait bounds,
- trait objects,
- unconstrained generics,
- `unsafe fn`,
- closures as plain values,
- `repr(Rust)` layout assumptions for records crossing the boundary,
- self-referential structs,
- arbitrary `Pin` projections.

Bridge-compatible records and enums are generated with explicit representation rules. Layout-sensitive interop must use generated bridge structs or declared wire layouts, not Rust's default layout.

### Generated Bridge Types

Every Sifr record, closed enum, and error type reachable across an `@rust` boundary materializes under:

```rust
crate::__sifr_bridge::<sifr_module_path>::<Name>Bridge
```

Generated record bridge structs preserve declared Sifr field order and expose generated constructors/accessors. When a bridge declaration needs layout-sensitive Rust access, the generated struct uses an explicit layout contract owned by the bridge schema; otherwise bridge authors must use accessors and must not rely on Rust default layout.

Closed enum bridge types use explicit numeric discriminants assigned by declaration order unless the Sifr enum declares stable discriminant values. The default representation is `repr(u32)`. Values returned as a typed generated enum from Rust do not need runtime discriminant validation because Rust's type system has already constructed a valid enum value. Values returned through integer or wire adapters must validate the discriminant before becoming a Sifr value; invalid discriminants return a `SIFR-RUST-TYPE-*` runtime conversion error.

## Error Semantics

Every fallible Rust call exposed to Sifr returns `Result`. Panics are not user errors.

Generated wrappers catch panics at the boundary:

```rust
pub fn call_encode(text: &str) -> Result<Tokenized, NativeErrorOr<TokenizeError>> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokenizer_backend::encode(text)
    })) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(NativeErrorOr::User(error)),
        Err(payload) => Err(NativeErrorOr::RustPanic(RustPanicError::from_payload(payload))),
    }
}
```

`panic = "abort"` profiles are rejected for recoverable bridge builds unless the package explicitly opts into process-aborting behavior through `[trust].rust-panic-abort` and the Sifr API documents that it cannot preserve the no-panic guarantee for that backend. Aborts, segmentation faults, and process kills are outside recoverability.

Generated wrappers use `AssertUnwindSafe` at the boundary because opaque handles and mutable bridge state are commonly not `UnwindSafe`. The generated wrapper marks an opaque handle as poisoned automatically when `catch_unwind` returns `Err`; bridge authors do not implement poisoning manually and must not depend on additional bridge code running after a panic. Re-entering a poisoned handle returns a stable `SIFR-RUST-PANIC-*` error instead of calling Rust again.

`Drop` panics are backend contract violations. Fallible cleanup must be modeled as explicit `close` or `aclose`, not as hidden destructor failure.

## Opaque Handles

Opaque handles represent Rust-owned state with declared lifetime and thread behavior.

Required declarations:

- ownership model: owned, borrowed, shared, or exclusive,
- destructor or explicit close contract,
- whether use-after-close is diagnosed at runtime with a stable error,
- `Send` and `Sync` status,
- clone strategy: `none`, `copy`, `arc`, or custom bridge function,
- close strategy: `drop`, `close`, `async_close`, or prohibited,
- thread affinity: none, current Sifr Tokio runtime, current OS thread, or custom guard.

The compiler rejects owning opaque handles that have neither safe `Drop` cleanup nor explicit close semantics. Handles that require explicit close must produce diagnostics for missing close in paths where Sifr's ownership analysis can prove the leak.

## Async and Tokio Runtime

Sifr uses Tokio as its async runtime. Current generated async entrypoints use the current-thread flavor, and runtime features are intentionally feature-minimal. Rust interop must work with that model:

- no hidden Tokio runtime per interop call,
- no `block_on` inside generated glue,
- no assumption that `rt-multi-thread` is enabled,
- no implicit offload for blocking or CPU-heavy Rust code,
- futures returned to Sifr must be compatible with Sifr's runtime and lifetime requirements.

Async Rust APIs are declared explicitly:

```python
@rust(http_client.fetch)
async def fetch(url: str) -> Result[Response, HttpError]:
    pass
```

Blocking and CPU-heavy APIs must be classified:

```python
@blocking_io
@rust(postgres_bridge.query_blocking)
def query(sql: str) -> Result[Rows, DbError]:
    pass

@cpu_heavy
@rust(image_bridge.resize)
def resize(input: bytes, width: uint32, height: uint32) -> Result[bytes, ImageError]:
    pass
```

Direct calls to classified blocking or CPU-heavy Rust functions from async Sifr code are compile-time errors unless explicitly offloaded through the Sifr task/offload APIs.

Default async bridge requirements:

- generated futures must be `'static` when spawned,
- `Send` is required for work that leaves the current-thread runtime,
- non-`Send` futures are allowed only when pinned to the current Sifr Tokio runtime through `thread_affinity=tokio_current_thread` on the opaque type or through an explicit function-level `@rust.async(thread_affinity=tokio_current_thread)` declaration,
- non-`Send` futures without current-thread affinity are rejected,
- cancellation is cooperative and must map to stable Sifr cancellation errors,
- runtime shutdown must drain or cancel registered Rust interop tasks deterministically.

Free async functions that require current-thread affinity declare it explicitly:

```python
@rust.async(thread_affinity=tokio_current_thread)
@rust(bridge.local_client.fetch)
async def fetch(url: str) -> Result[Response, HttpError]:
    pass
```

## Zero-Copy and Views

Zero-copy is first-class and explicit, including bytes, Arrow-style columnar data, tensor buffers, and application-defined views.

```python
@rust.zero_copy(owner=bytes, view=bridge.hash.BytesView)
@rust(bridge.hash.digest_view)
def digest_view(input: bytes) -> Result[DigestView, HashError]:
    pass
```

`@rust.zero_copy(...)` declares that the API cannot copy. `@rust.view(...)` declares the view's lifetime, mutability, and thread behavior. They compose: `@rust.zero_copy` is required whenever copying is prohibited, `@rust.view` is required whenever the return value is a borrowed view, and a borrowed zero-copy API uses both decorators.

```python
@rust.view(
    owner=input,
    lifetime=call,
    mutability=immutable,
    send=false,
    sync=false,
)
@rust(bridge.parser.tokens_view)
def tokens_view(input: bytes) -> Result[TokenView, ParseError]:
    pass
```

Allowed `lifetime` values are `call`, `owner`, and `static`. `call` views cannot escape the bridge call. `owner` views are tied to the named owner parameter or opaque handle. `static` requires the Rust target to return owned or globally valid data and is rejected for borrowed returns. `mutability=mutable` requires exclusive Sifr ownership of the owner for the full view lifetime.

Rules:

- no silent copy fallback for `@rust.zero_copy` or `@rust.view`,
- borrowed views cannot outlive their owner,
- mutable views require exclusive Sifr ownership,
- aliasing is checked at the Sifr boundary,
- Send/Sync for views must be declared and validated,
- fallible downgrade to copying must be a different API name and declaration,
- views crossing async suspension points must satisfy the same lifetime and pinning requirements as native Sifr borrows.

Data-oriented bridges must support explicit contracts for:

- Python/Rust-independent buffers,
- Arrow C Data Interface compatible record batches and arrays,
- tensor buffers with shape, dtype, layout, strides, device, and ownership metadata,
- DLPack-style tensor handoff through a shared `sifr_tensor_bridge` crate,
- dataframe adapters that preserve ownership and schema identity.

## Callbacks

Callbacks are supported only with declared lifetime and threading policy.

Call-scoped callback:

```python
@rust(bridge.parser.visit)
def visit(text: str, callback: Callback[[Token], Result[None, ParseError]]) -> Result[None, ParseError]:
    pass
```

The Rust implementation cannot store a call-scoped callback, call it after the bridge call returns, or call it from an unmanaged thread.

Thread-safe callback registration:

```python
@rust(bridge.kafka.on_message)
def on_message(
    consumer: KafkaConsumer,
    callback: ThreadsafeCallback[[Message], Result[None, KafkaError]],
) -> Result[Subscription, KafkaError]:
    pass
```

Thread-safe callbacks require:

- captured values to satisfy Sifr's `Send + 'static` equivalent,
- a returned cancellation/subscription handle,
- runtime entry guards for non-Sifr threads,
- explicit backpressure policy,
- explicit cancellation and shutdown behavior,
- panic-to-error handling at both callback and Rust boundaries.

## Trust Policy

Rust interop extends existing native trust policy with Rust-specific evidence:

```toml
[trust]
rust-build-scripts = ["openssl-sys", "tokenizer_backend"]
rust-proc-macros = ["serde_derive"]
native = ["openssl-sys"]
unsafe-rust-bridges = ["src/bridges/tokenizer.rs"]
build-env = ["OPENSSL_DIR"]
rust-panic-abort = []
```

Trust gates:

- build scripts,
- procedural macros,
- native library linking,
- unsafe code in first-party bridge files,
- environment variables consumed by build scripts,
- process-aborting panic profiles through `rust-panic-abort`,
- optional strict allowlist for Rust crate roots.

Safe Rust crates are not inherently unsafe, but code execution during build is. The trust model must separate build-time execution, native linking, unsafe bridge code, and ordinary safe dependency use.

## Cargo and Build Cache

Cargo remains the source of truth for Rust dependency resolution. Sifr must preserve Cargo flags such as `--locked`, `--offline`, and `--frozen`.

The interop build cache key includes:

- Sifr source digests,
- generated Rust digests,
- local bridge source digests,
- `Cargo.lock`,
- resolved Cargo package IDs, source IDs, features, and target triples,
- selected Cargo profile name and profile settings that affect code generation or panic behavior,
- resolved panic strategy, `lto`, `codegen-units`, `incremental`, target features, and rustc target-spec hash,
- `@rust` target paths,
- `@rust.opaque`, callback, and zero-copy declarations,
- trust policy,
- build-script/proc-macro/native evidence,
- `rustc` and Cargo versions,
- selected Sifr runtime metadata and bridge-version schema,
- declared build environment variables and their values when policy allows them.

Any change to bridge declarations, local bridge code, Cargo lock state, selected features, target triple, profile, panic strategy, bridge version, or trust policy invalidates the relevant interop build plan.

## Diagnostics

Rust interop diagnostics use stable diagnostic families:

- `SIFR-RUST-CONFIG-*`: malformed decorators, manifest conflict, missing bridge directory,
- `SIFR-RUST-RESOLVE-*`: unresolved dependency root, module, item, or `Self` target,
- `SIFR-RUST-TRUST-*`: missing build-script, proc-macro, native, unsafe, or build-env trust,
- `SIFR-RUST-TYPE-*`: signature mismatch or unsupported bridge type,
- `SIFR-RUST-HANDLE-*`: invalid opaque handle ownership, close, clone, or thread contract,
- `SIFR-RUST-ASYNC-*`: hidden blocking, invalid future, unsupported non-`Send`, cancellation mismatch,
- `SIFR-RUST-ZC-*`: zero-copy/view lifetime, aliasing, mutability, or copy-fallback violation,
- `SIFR-RUST-CB-*`: callback lifetime, thread, backpressure, or shutdown violation,
- `SIFR-RUST-PANIC-*`: panic strategy or panic-boundary issue,
- `SIFR-RUST-CARGO-*`: Cargo metadata, feature, lockfile, or offline/frozen violation.

Every diagnostic must include source span, resolved target when available, required fix, and documentation URL.

Milestone 39.0 reserves the first code in every family:

| Code | Reserved meaning |
| --- | --- |
| `SIFR-RUST-CONFIG-0001` | malformed Rust interop decorator |
| `SIFR-RUST-RESOLVE-0001` | unresolved Rust target path |
| `SIFR-RUST-TRUST-0001` | missing Rust interop trust declaration |
| `SIFR-RUST-TYPE-0001` | unsupported Rust bridge type |
| `SIFR-RUST-HANDLE-0001` | invalid opaque handle contract |
| `SIFR-RUST-ASYNC-0001` | invalid Rust async/thread-affinity contract |
| `SIFR-RUST-ZC-0001` | invalid zero-copy/view lifetime contract |
| `SIFR-RUST-CB-0001` | invalid callback lifetime or threading contract |
| `SIFR-RUST-PANIC-0001` | incompatible Rust panic strategy or poisoned handle |
| `SIFR-RUST-CARGO-0001` | Cargo metadata or lock/profile mismatch |

## Tooling

LSP support is staged:

1. Resolve decorator roots from Sifr package metadata and Cargo metadata.
2. Parse package-local bridge module names and exported functions for completions.
3. Integrate rust-analyzer metadata for richer signatures and go-to-definition.

Completion must prefer valid dotted paths. Invalid string-style Rust targets are rejected instead of tolerated.

## Verification Area

Phase 39 creates a first-class verification area:

```text
verification/areas/rust_interop/
  README.md
  data/
    rust_interop_fixture_matrix.json
    rust_interop_tiers.toml
  fixtures/
    direct_crate_crc32/
    direct_crate_negative_type/
    dotted_path_resolution/
    local_bridge_blake3/
    same_workspace_crate/
    shared_bridge_crate/
    opaque_handle_tokenizer/
    close_after_use/
    panic_boundary/
    panic_abort_profile/
    async_runtime_reqwest/
    blocking_diagnostics/
    callbacks_call_scoped/
    callbacks_threadsafe/
    zero_copy_bytes/
    arrow_record_batch/
    tensor_dlpack_bridge/
    native_build_script/
    proc_macro_trust/
    cargo_locked_offline/
  runner/
    cargo_probe.py
    bridge_check.py
    trust_check.py
    native_probe.py
    report.py
```

Verification tiers:

- Tier 0: parser, lowering, metadata, and diagnostics without Cargo build.
- Tier 1: local bridge and direct crate build fixtures.
- Tier 2: opaque handles, panic boundary, async/blocking, callbacks, and zero-copy.
- Tier 3: package ecosystem fixtures for build scripts, proc macros, native linking, and offline/locked Cargo behavior.
- Tier 4: production examples and compatibility matrix for selected Rust ecosystems.

The area must record positive and negative fixtures for every declared capability. A feature is not complete until its failure mode is as deliberate as its success path.
