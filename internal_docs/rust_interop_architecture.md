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

```rust
// src/bridges/blake3.rs
pub fn hash_bytes(input: &[u8]) -> Result<Vec<u8>, HashErrorBridge> {
    Ok(blake3::hash(input).as_bytes().to_vec())
}
```

Consumers do not see the Rust bridge unless they are maintaining the package.

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

    @rust(Self.aclose)
    def aclose(self) -> Result[None, KafkaError]:
        pass
```

Decorator values are symbolic values, not strings. The opaque class declares ownership, close semantics, thread behavior, and borrowing rules before generated glue can expose the handle.

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
| `str` | `&str` | `String` | `String` |
| `bytes` | `&[u8]` | `Vec<u8>` | `Vec<u8>` |
| `list[T]` | `&[T]` | `Vec<T>` | `Vec<T>` |
| `dict[str, T]` | `&HashMap<String, T>` | `HashMap<String, T>` | `HashMap<String, T>` |
| `Option[T]` | `Option<T>` | `Option<T>` | `Option<T>` |
| `Result[T, E]` | not a parameter | not a parameter | `Result<T, E>` |
| closed enum | generated bridge enum | generated bridge enum | generated bridge enum |
| record class | generated bridge struct | generated bridge struct | generated bridge struct |
| opaque class | `&Handle<T>` / `&mut Handle<T>` | `Handle<T>` | `Handle<T>` |

Exact `int` is not a native ABI integer. A bridge may accept exact integers only through Sifr's exact integer bridge representation. Fixed-width integer declarations are required for representation-sensitive APIs.

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

## Error Semantics

Every fallible Rust call exposed to Sifr returns `Result`. Panics are not user errors.

Generated wrappers catch unwind-safe panics at the boundary:

```rust
pub fn call_encode(text: &str) -> Result<Tokenized, NativeErrorOr<TokenizeError>> {
    match std::panic::catch_unwind(|| tokenizer_backend::encode(text)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(NativeErrorOr::User(error)),
        Err(payload) => Err(NativeErrorOr::RustPanic(RustPanicError::from_payload(payload))),
    }
}
```

`panic = "abort"` profiles are rejected for recoverable bridge builds unless the package explicitly opts into process-aborting behavior and the Sifr API documents that it cannot preserve the no-panic guarantee for that backend. Aborts, segmentation faults, and process kills are outside recoverability.

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
- non-`Send` futures are rejected until Sifr has an explicit local-task surface for them,
- cancellation is cooperative and must map to stable Sifr cancellation errors,
- runtime shutdown must drain or cancel registered Rust interop tasks deterministically.

## Zero-Copy and Views

Zero-copy is first-class and explicit, including bytes, Arrow-style columnar data, tensor buffers, and application-defined views.

```python
@rust.zero_copy(owner=bytes, view=bridge.hash.BytesView)
@rust(bridge.hash.digest_view)
def digest_view(input: bytes) -> Result[DigestView, HashError]:
    pass
```

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
- DLPack-style tensor handoff through shared bridge crates,
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
```

Trust gates:

- build scripts,
- procedural macros,
- native library linking,
- unsafe code in first-party bridge files,
- environment variables consumed by build scripts,
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
- `@rust` target paths,
- `@rust.opaque`, callback, and zero-copy declarations,
- trust policy,
- build-script/proc-macro/native evidence,
- `rustc` and Cargo versions,
- selected Sifr runtime metadata,
- declared build environment variables and their values when policy allows them.

Any change to bridge declarations, local bridge code, Cargo lock state, selected features, target triple, or trust policy invalidates the relevant interop build plan.

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
