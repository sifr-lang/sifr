# Rust Interop Architecture

Status: design locked for Rust interop implementation planning.

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

```sifr
from fast_hash import hash_bytes

def main() -> Result[None, HashError | RustPanicError]:
    digest = try hash_bytes(b"hello")
    print(digest.hex())
```

The package author declares the Rust implementation boundary:

```sifr
class HashError(Error):
    message: str

@rust(bridge.blake3.hash_bytes)
def hash_bytes(input: bytes) -> Result[bytes, HashError | RustPanicError]: ...
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

```sifr
@rust(crc32fast.hash, panic=trusted_no_panic)
def crc32(data: bytes) -> uint32: ...
```

`crc32fast.hash` resolves to a direct Cargo dependency root named `crc32fast` and Rust path `crc32fast::hash`.

Direct binding is allowed only when the target signature is bridge-compatible without adaptation. If the Rust crate exposes a non-compatible API, the package author must write a package-local bridge. Non-`Result` direct bindings require an explicit panic policy such as `panic=trusted_no_panic`; otherwise they must expose `Result[..., RustPanicError]`.

The body of a Rust interop declaration is an ellipsis-only stub body: exactly
one ellipsis statement. The compiler derives generated behavior from the
validated Rust interop metadata rather than from Sifr statements in the
declaration body. Ellipsis is public Rust interop declaration syntax, not a
general Sifr function body form.

### Package-Local Bridge Binding

```sifr
@rust(bridge.tokenizer.encode)
def encode(text: str) -> Result[Tokenized, TokenizeError | RustPanicError]: ...
```

`bridge.tokenizer.encode` resolves to `crate::bridges::tokenizer::encode` in the generated package crate. Sifr owns projection entries for `src/bridges/mod.rs`, but user-authored bridge files remain user files and must never be overwritten silently.

Package-local bridge bindings are also the adapter boundary for Rust APIs whose
native shape is not already Sifr-compatible. A bridge function may reshape
generated Sifr bridge types into backend Rust types, call one or more Rust
implementation functions, reshape outputs back into generated Sifr bridge
types, and map `Result`-typed Rust errors into the declared Sifr error channel.
The compiler still validates only the bridge-compatible signature named by
`@rust(...)`; it does not interpret per-function input/output converter chains.

Direct bindings are for exact bridge-compatible Rust signatures. Anything that
needs adaptation targets an adapter function owned by the package (`bridge.*`),
a shared bridge crate, or a sysroot crate such as `sifr_stdlib`; sysroot stdlib
adapters are still targeted through direct `@rust(sifr_stdlib.<path>)` bindings.
Decorator-level converter pipelines are intentionally not part of bridge version
1.

If a Rust crate exposes unsupported lifetimes, borrowed returns, trait objects,
generics, closure-valued APIs, or error types whose mapping needs more than
display-text shaping, the package or stdlib author writes a bridge adapter and
targets that adapter.

Bridge code does not install its own panic guard. Panics from the bridge and
from any Rust function it calls are caught at the generated wrapper according
to the declaration's `panic=` policy.

### Shared Bridge Crate Binding

```sifr
@rust(sifr_arrow_bridge.record_batch_from_columns)
def record_batch(columns: dict[str, ArrowArray]) -> Result[ArrowRecordBatch, ArrowError | RustPanicError]: ...
```

Shared bridge crates are ordinary Cargo dependencies that publish stable adapter APIs for common Rust ecosystems.

### Opaque Types

```sifr
@rust.opaque(
    type=bridge.kafka.Consumer,
    send=False,
    sync=False,
    clone=none,
    close=async_close,
    borrow=exclusive,
    thread_affinity=tokio_current_thread,
)
class KafkaConsumer:
    @rust(Self.poll)
    def poll(self) -> Result[Message, KafkaError | RustPanicError]: ...

    @rust(bridge.kafka.consumer_aclose)
    async def aclose(own self) -> Result[None, KafkaError | RustPanicError]: ...
```

Decorator values are symbolic values, not strings. The opaque class declares ownership, close semantics, thread behavior, and borrowing rules before generated glue can expose the handle.

Rust interop decorator values use a deliberately small grammar:

```text
RustInteropDecoratorValue =
    BooleanLiteral
  | IdentifierSymbol
  | IntegerLiteral
  | PolicyCall(IdentifierSymbol, IntegerLiteral | RustTargetPath)
  | RustTargetPath
```

`RustTargetPath` is the structured dotted-path form used by `@rust(...)`, `@rust.opaque(type=...)`, and path-valued policy fields such as `panic=map_error(...)`. `PolicyCall` is allowed only where a decorator explicitly permits it; the initial Rust interop contract permits `bounded(N)` for callback backpressure and `custom(path)` for opaque clone policy. Other call-shaped decorator values are rejected with `SIFR-RUST-CONFIG-*` diagnostics instead of being interpreted dynamically.

Opaque decorator keys are fixed:

- `type=` names the Rust type wrapped by the opaque handle.
- `send=` and `sync=` declare whether the generated Sifr handle may cross Sifr task/thread boundaries.
- `clone=` declares generated clone behavior.
- `close=` declares cleanup behavior.
- `borrow=` declares the default method receiver mode: `shared` lowers unannotated `self` on `Self.method` to `&self`, `exclusive` lowers it to `&mut self`, and `owned` lowers it to a consuming `self` call.
- `thread_affinity=` declares runtime or OS-thread affinity that generated glue must preserve.

Method receiver annotations win over the class-level `borrow=` default. `def method(self, ...)` uses the class default, `def method(mut self, ...)` lowers to `&mut self`, and `def method(own self, ...)` consumes the handle and marks it closed or moved. `close=close` requires `def close(own self) -> Result[None, E]`; `close=async_close` requires `async def aclose(own self) -> Result[None, E]`. If the close target is a package-local function rather than `Self.method`, generated glue passes the owned handle value to the bridge function, marks the Sifr handle closed before returning success, and marks it poisoned if the Rust call panics.

Allowed symbolic values are:

- `send=True | False`
- `sync=True | False`
- `clone=none | copy | arc | custom`
- `close=drop | close | async_close | none`
- `borrow=shared | exclusive | owned`
- `thread_affinity=none | tokio_current_thread | current_os_thread`

`clone=arc` creates multiple Sifr handles that share one `HandleState` cell; closing or poisoning one clone closes or poisons all clones. `clone=copy` is allowed only when a generated probe proves `T: Copy`, and each copied handle has independent state because the Rust value is independent. `clone=custom(path)` must name an explicit bridge function that returns a fresh `Handle<T>`; bare `clone=custom` is rejected.

## Path Resolution

`@rust(...)` accepts a dotted path expression with one of these roots:

| Root | Meaning |
| --- | --- |
| Cargo dependency name | Direct binding to an ordinary Cargo dependency. |
| `bridge` | Package-local bridge module under `src/bridges`. |
| Shared bridge crate name | Direct binding to a bridge crate declared as a Cargo dependency. |
| Sysroot-owned crate root | Private stdlib binding to canonical sysroot crates such as `sifr_stdlib` or `sifr_runtime`, resolved only under the compiler-owned synthetic private `_sifr.*` package context. |
| `Self` | Method binding inside a `@rust.opaque` class. |

Resolution is compiler-owned and deterministic:

1. Parse decorator paths as structured AST nodes, not strings.
2. Resolve the root against direct Cargo dependencies, generated bridge modules, or the current opaque type.
3. Validate that the resolved item exists.
4. Validate the Rust signature against the Sifr declaration.
5. Emit generated Rust glue and structured `InteropBuildPlan` metadata.

Same-workspace crates are not special. They must still be declared as ordinary Cargo dependencies with a package, path, git, registry, and feature set.

`bridge` and `Self` are reserved roots inside `@rust` paths. A Cargo dependency named `bridge` or `Self` cannot be targeted directly; package authors must use a Cargo dependency alias such as `bridge_crate = { package = "bridge", version = "..." }`. Reserved-root collisions produce `SIFR-RUST-RESOLVE-*` diagnostics with the dependency and decorator spans.

The same roots resolve Rust type paths for `@rust.opaque(type=...)`. Probe code validates type existence with a generated type assertion before any opaque wrapper is emitted.

### `Self` Resolution

Inside a `@rust.opaque` class, `Self.method` resolves against the Rust type declared by `@rust.opaque(type=...)`.

Resolution order is:

1. inherent methods on the resolved Rust type,
2. diagnostic failure when the target is missing or is not an inherent method.

For the `KafkaConsumer.poll` example above, `Self.poll` lowers to an inherent call on the wrapped `bridge.kafka.Consumer` value:

```rust
crate::bridges::kafka::Consumer::poll(handle.inner_mut()?)
```

Trait methods are not resolved through `Self` because Rust trait lookup depends on imports and can become ambiguous. Package authors expose trait-backed behavior through a package-local bridge shim:

```sifr
@rust(bridge.kafka.consumer_aclose)
async def aclose(own self) -> Result[None, KafkaError | RustPanicError]: ...
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
native-links = []
unsafe-rust-bridges = ["src/bridges/tokenizer.rs"]
build-env = ["LIBTOKENIZER_PATH"]
rust-no-panic = []
rust-panic-abort = []
```

Sifr may generate projection modules and guarded regions, but it must not overwrite user-authored Rust bridge files without an explicit diagnostic and user action.

File ownership is fixed:

- `src/bridges/*.rs` files are user-owned. Sifr never overwrites them.
- `src/bridges/mod.rs` is Sifr-owned for Rust-backed packages. If it is missing or user-authored, Sifr emits a repair/projection diagnostic before build.
- `src/lib.rs` is Sifr-managed for packages using local bridges. Pure packages keep the pure marker target; Rust-backed packages receive managed module declarations for bridges and generated bridge types.
- `crate::__sifr_bridge` is generated and reserved. User code cannot define this namespace.

Rust-backed package archives must include `sifr.toml`, Sifr source, `Cargo.toml`, declared `src/bridges/*.rs` files, Sifr-managed projection files, and the `[rust].bridge-version` value declared in `sifr.toml`. `sifr package` rejects archives whose managed projections, source digests, or bridge-version metadata do not match the interop build plan.

Backend crates are linked statically as ordinary Rust library crates. The supported backend crate type is `lib`; `cdylib`, `dylib`, and runtime-loaded backend crates are rejected for the Rust interop lane because they imply a dynamic ABI boundary.

`bridge-version = 1` is the schema version for generated bridge modules, generated bridge type naming, decorator lowering, and runtime glue contracts. A package compiled by a compiler that does not support the declared bridge version fails during package validation.

Bridge version 1 covers:

- generated bridge type paths under `crate::__sifr_bridge::<module>::<Name>Bridge`,
- closed enum `repr(u32)` discriminant rules,
- `sifr_runtime::interop::{SifrIntBridge, IndexMap, Handle}` helper versions,
- `Handle<T>` closed/poisoned state semantics,
- `bridge` and `Self` namespace reservation,
- managed projection file ownership.

Same-workspace backend crates are ordinary dependencies:

```toml
[dependencies]
tokenizer_backend = { path = "backend" }
```

They are targeted through normal dotted paths:

```sifr
@rust(tokenizer_backend.encode, panic=map_error(bridge.tokenizer.map_panic))
def encode(text: str) -> Result[Tokenized, TokenizeError]: ...
```

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
- RustBridgeProbePlan entries for direct binding checks,
- opaque handle layouts,
- async/blocking classifications,
- zero-copy/view contracts,
- callback contracts,
- panic and trust requirements,
- Cargo feature and package metadata that must enter the build cache key.

The driver materializes generated glue from this plan. It must not scan emitted Rust text to infer dependencies.

## Rust Signature Probing

Sifr does not implement a Rust resolver. Direct binding signatures are validated through a Sifr-generated `RustBridgeProbePlan` that asks Cargo/rustc to type-check item existence, visibility, arity, receiver mode, asyncness, and bridge-compatible types in an isolated probe module before final generated binary build.

Probe code uses generated type assertions rather than semantic Rust item metadata:

```rust
const _: () = {
    fn assert_signature(f: fn(&[u8]) -> u32) {}
    let _ = assert_signature(crc32fast::hash);
};
```

Probe failures map rustc diagnostics back to the original `@rust(...)` span as `SIFR-RUST-RESOLVE-*` or `SIFR-RUST-TYPE-*` diagnostics. The final generated binary is not built until probe diagnostics are clean. This is the precise meaning of "checked before build": incompatibility is detected in a Sifr-controlled probe/check step rather than surfacing as raw application build noise.

Async probes assert the callable and future output shape:

```rust
fn assert_async_signature<F, Fut>(f: F)
where
    F: Fn(&str) -> Fut,
    Fut: std::future::Future<Output = Result<ResponseBridge, HttpErrorBridge>>,
{
}

const _: () = {
    assert_async_signature(http_client::fetch);
};
```

Method probes assert receiver mode explicitly:

```rust
fn assert_method<T>(f: fn(&mut T, &str) -> Result<TokenizedBridge, ErrorBridge>) {}

const _: () = {
    assert_method::<crate::bridges::tokenizers::Tokenizer>(
        crate::bridges::tokenizers::Tokenizer::encode,
    );
};
```

Message-shaped error probes assert the Rust error's display contract when the
Sifr error type is eligible for generated message construction:

```rust
fn assert_result_signature<__SifrBridgeError: std::fmt::Display>(
    f: fn(&str) -> Result<String, __SifrBridgeError>,
) {
}

const _: () = {
    assert_result_signature(sifr_stdlib::regex::re_find);
};
```

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
| `dict[str, T]` | `&sifr_runtime::interop::IndexMap<String, T>` | `sifr_runtime::interop::IndexMap<String, T>` | `sifr_runtime::interop::IndexMap<String, T>` |
| `Option[T]` | `Option<T>` | `Option<T>` | `Option<T>` |
| `Result[T, E]` | not a parameter | not a parameter | `Result<T, E>` |
| closed enum | generated bridge enum | generated bridge enum | generated bridge enum |
| record class | generated bridge struct | generated bridge struct | generated bridge struct |
| opaque class | `&sifr_runtime::interop::Handle<T>` / `&mut sifr_runtime::interop::Handle<T>` | `sifr_runtime::interop::Handle<T>` | `sifr_runtime::interop::Handle<T>` |
| `Callback[[...], R]` | generated call-scoped callback handle | not storable | not a return type |
| `ThreadsafeCallback[[...], R]` | generated thread-safe callback handle | generated thread-safe callback handle | not a return type |

Exact `int` is not a native ABI integer. `SifrIntBridge` lives in `sifr_runtime::interop` and is an owned, immutable, cloneable exact-integer value with `Eq`, `Ord`, `Hash`, `Send`, and `Sync`, no `Copy` implementation, and no `repr(C)` guarantee. Borrowed parameters use `&SifrIntBridge`; owned parameters and returns use `SifrIntBridge`. Bridges that need fixed-width storage or ABI layout must declare fixed-width integer types instead.

`dict[str, T]` preserves Sifr/Python insertion order through `sifr_runtime::interop::IndexMap`, a runtime re-export of the pinned `indexmap::IndexMap` version used by generated bridge glue. Non-`str` dict keys are not bridge-compatible until a later design defines stable hashing/equality and ordering semantics for those key types.

Nested borrowed forms are generated from the outer ownership mode. For example, borrowed `list[str]` is not `&[&str]`; it is a generated list view whose elements are borrowed string views with the same lifetime as the list view. `Option[str]` and `Option[bytes]` use generated optional borrowed views for borrowed parameters and owned `Option<String>` / `Option<Vec<u8>>` for owned parameters and returns.

For `Result[T, E]`, the error position may be either the generated bridge error
type or a Rust error type that implements `std::fmt::Display` when the Sifr
error class can be constructed from strings. A message-shaped Sifr error class
receives `error.to_string()` as `message`; an error subclass whose fields are
all strings receives the same display text for `message` and its extra string
fields. Any richer error mapping requires an explicit adapter function that
returns a bridge-compatible error shape. This is a single generated wrapper
rule, not a per-declaration converter pipeline.

Sifr container types not listed above, including `set[T]`, `tuple[...]`, and arbitrary iterator/generator types, are not bridge-compatible in the initial Rust interop contract and produce `SIFR-RUST-TYPE-*` diagnostics. No implicit conversion is allowed.

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

Generated bridge module paths use the same deterministic Rust module-name mangling as generated Sifr modules. Every Sifr module path maps bijectively to a Rust module path; keyword escapes, invalid-identifier escapes, package aliases, renamed imports, and nested public namespace segments are stable and included in `bridge-version`. Bridge authors may import generated bridge types directly, so changing this mangling is a bridge-versioned compatibility break.

Generated record bridge structs preserve declared Sifr field order and expose generated constructors/accessors. When a bridge declaration needs layout-sensitive Rust access, the generated struct uses an explicit layout contract owned by the bridge schema; otherwise bridge authors must use accessors and must not rely on Rust default layout.

Closed enum bridge types use explicit numeric discriminants assigned by declaration order unless the Sifr enum declares stable discriminant values. The default representation is `repr(u32)`. Values returned as a typed generated enum from Rust do not need runtime discriminant validation because Rust's type system has already constructed a valid enum value. Values returned through integer or wire adapters must validate the discriminant before becoming a Sifr value; invalid discriminants return a `SIFR-RUST-TYPE-*` runtime conversion error.

### Opaque Handle Representation

Opaque handles use `sifr_runtime::interop::Handle<T>`. The runtime type stores:

- the Rust value `T`,
- a closed flag,
- a poisoned flag,
- generated metadata for handle type identity and thread-affinity checks.

The fields are private. Generated glue and bridge authors use accessors:

- `Handle::new(value: T) -> Handle<T>`
- `inner_ref() -> Result<&T, HandleStateError>`
- `inner_mut() -> Result<&mut T, HandleStateError>`
- `into_inner() -> Result<T, HandleStateError>`
- `mark_closed(GeneratedGlueToken)`
- `mark_poisoned(GeneratedGlueToken, RustPanicError)`

Package-local bridge code may call `Handle::new` when returning a newly owned opaque value. State-mutating APIs require a generated private capability token:

```rust
pub struct GeneratedGlueToken {
    _private: (),
}
```

Generated wrappers construct the token through the hidden `sifr_runtime::interop::__generated_glue::token()` namespace. That namespace is generator-owned, not part of the package-author bridge API, so only generated wrappers can call `mark_closed` and `mark_poisoned`. Package-local bridge code may borrow or consume handles through the accessors but must not manipulate state flags directly.

`HandleStateError` lives at `sifr_runtime::interop::HandleStateError` and has two variants: `Closed` and `Poisoned(RustPanicError)`. Generated wrappers convert `Closed` into the stable closed-handle error for the Sifr surface and reuse the stored `RustPanicError` for `Poisoned` before propagating into the declared Sifr error channel.

`Handle<T>` implements `Send` only when `@rust.opaque(send=True)` is declared and the Rust target satisfies the generated `T: Send` probe. `Handle<T>` implements `Sync` only when `@rust.opaque(sync=True)` is declared and the Rust target satisfies the generated `T: Sync` probe. Without those declarations, the generated Sifr handle is task/thread local even if `T` would be `Send` or `Sync` in Rust.

Generated Send/Sync probes use Rust trait assertions:

```rust
const _: fn() where crate::bridges::kafka::Consumer: Send = || {};
const _: fn() where crate::bridges::kafka::Consumer: Sync = || {};
```

State transitions are deterministic:

| Starting state | Event | New state | Later access |
| --- | --- | --- | --- |
| open | successful consuming close/move | closed | returns closed-handle error |
| open | caught panic during any call | poisoned | returns stored panic error |
| open | caught panic during close | poisoned | poisoned wins over closed |
| closed | any method call | closed | returns closed-handle error |
| poisoned | any method call | poisoned | returns stored panic error |

Generated `Self.method` lowering calls `inner_ref`, `inner_mut`, or `into_inner` according to the receiver mapping. Free bridge functions may accept `Handle<T>`, `&Handle<T>`, or `&mut Handle<T>` only when the Sifr receiver/parameter declaration grants the same ownership or borrow capability.

Before moving an owned opaque value into Rust, generated glue creates a `PoisonOnPanic` guard tied to the Sifr handle state. The guard is disarmed only after the Rust call returns successfully or returns an ordinary `Result` error. If unwinding crosses the boundary, the guard marks the original handle state poisoned even when the inner Rust value was moved into the bridge call.

### Shared Bridge Crate Boundaries

Package-local bridges compile inside the generated package crate and may use package-generated `crate::__sifr_bridge::<module>::<Name>Bridge` types.

Shared bridge crates are ordinary Cargo dependencies and cannot import package-specific generated bridge modules. They may expose only stable Rust types, `sifr_runtime::interop` helper types, or their own opaque handle types. Generated glue adapts package-local Sifr records, enums, and errors to the shared bridge crate's public types outside the shared bridge crate.

### Future Callee Injection

Bridge version 1 does not support callee injection. Sifr source must not store,
return, capture, or dynamically dispatch Rust functions; Rust functions are not
Sifr values. Any future callee-injection form requires a new bridge-versioned
design that proves its ownership, lifetime, panic, trust, and cache-key
behavior.

The current stdlib rewrite does not require this extension. For migrated
stdlib leaves, direct binding is used both for exact-shape `sifr_stdlib`
functions and for `sifr_stdlib` adapter functions that own input reshaping,
output reshaping, and typed error mapping. In both cases the private `_sifr.*`
declaration binds via `@rust(sifr_stdlib.<path>)`; there is no `bridge.*`
package-local module for sysroot stdlib.

## Error Semantics

Every fallible Rust call exposed to Sifr returns `Result`. Panics are not user errors.

## Panic Surface Policy

Every package-authored `@rust` declaration declares its panic surface in the
Sifr source.

Result-returning functions convert caught Rust panics to `RustPanicError`, and that error must enter the declared Sifr error channel through one of:

- a declared union/member error type such as `Result[T, E | RustPanicError]`,
- a declared `panic=map_error` adapter that maps `RustPanicError` into the public error type.

`panic=map_error` names a Sifr or bridge adapter resolved through the same dotted-path rules as `@rust` targets:

```sifr
@rust(bridge.tokenizer.encode, panic=map_error(bridge.tokenizer.map_panic))
def encode(text: str) -> Result[Tokenized, TokenizeError]: ...
```

The adapter must be non-async, non-blocking, and shape-checked:

- Rust bridge adapter: `fn map_panic(error: RustPanicErrorBridge) -> EBridge`
- Sifr adapter: `def map_panic(error: RustPanicError) -> E`

If the mapper panics, generated glue ignores the failed mapper and surfaces the original `RustPanicError` through a stable `SIFR-RUST-PANIC-*` path. If the public error channel cannot represent that original panic after mapper failure, the declaration is rejected.

The initial compile-time panic contract validates the public panic surface and `panic=map_error(path)` shape. Full generated wrapper emission, mapper signature validation, and mapper-panic fallback behavior are future-owned by [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../plans/issues/active/rust-interop-runtime-ecosystem-certification.md) through the `panic_boundary_wrapper_emission` compatibility row.

Non-`Result` functions cannot return a recoverable panic without changing their public type. Package-authored declarations are rejected unless they declare `panic=trusted_no_panic` or `panic=abort` and satisfy the corresponding trust policy. `panic=trusted_no_panic` is a package trust assertion, not a compiler proof, and requires `[trust].rust-no-panic`. `panic=abort` opts into process-aborting behavior through `[trust].rust-panic-abort` and does not preserve recoverable interop semantics.

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

Generated wrappers use `AssertUnwindSafe` at the boundary because opaque handles and mutable bridge state are commonly not `UnwindSafe`. The generated wrapper marks the opaque receiver plus any mutable or owned opaque handles passed to the Rust call as poisoned automatically when `catch_unwind` returns `Err`; bridge authors do not implement poisoning manually and must not depend on additional bridge code running after a panic. Re-entering a poisoned handle returns a stable `SIFR-RUST-PANIC-*` error instead of calling Rust again.

The initial panic-boundary contract surface enforces that package-authored Result-returning Rust interop declarations either expose `RustPanicError`, declare `panic=map_error(path)`, or use an explicit trusted/abort panic policy. Package-authored non-`Result` declarations require `panic=trusted_no_panic` or `panic=abort`; `panic=abort` requires both `[trust].rust-panic-abort` evidence and a selected Cargo profile whose panic strategy is `abort`. Runtime bridge helpers redact panic payloads when converting caught panics into `RustPanicErrorBridge`.

Private sysroot stdlib declarations are compiled in a synthetic compiler-owned
package context. A private `_sifr.*` declaration that targets a direct
`sifr_stdlib.*` path uses the compiler-owned sysroot no-panic trust policy as
its effective panic surface. The effective policy is recorded in Rust interop
trust metadata, bridge probes, dependency-plan cache fingerprints, and audit
diagnostics. This sysroot policy is restricted to canonical private stdlib
declarations and does not apply to user packages, package-local bridges,
`Self`, `sifr_runtime`, or arbitrary Cargo dependency roots.

`Drop` panics are backend contract violations. Fallible cleanup must be modeled as explicit `close` or `aclose`, not as hidden destructor failure.

## Opaque Handles

Opaque handles represent Rust-owned state with declared lifetime and thread behavior.

Required declarations:

- ownership model: owned, shared, or exclusive,
- destructor or explicit close contract,
- whether use-after-close is diagnosed at runtime with a stable error,
- `Send` and `Sync` status,
- clone strategy: `none`, `copy`, `arc`, or custom bridge function,
- close strategy: `drop`, `close`, `async_close`, or `none`,
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

```sifr
@rust(http_client.fetch)
async def fetch(url: str) -> Result[Response, HttpError | RustPanicError]: ...
```

Blocking and CPU-heavy APIs must be classified:

```sifr
@blocking_io
@rust(postgres_bridge.query_blocking)
def query(sql: str) -> Result[Rows, DbError | RustPanicError]: ...

@cpu_heavy
@rust(image_bridge.resize)
def resize(input: bytes, width: uint32, height: uint32) -> Result[bytes, ImageError | RustPanicError]: ...
```

Direct calls to classified blocking or CPU-heavy Rust functions from async Sifr code are compile-time errors unless explicitly offloaded through the Sifr task/offload APIs.

`@blocking_io` and `@cpu_heavy` classify synchronous Rust-backed declarations. They are rejected on `async def` Rust interop declarations unless a future design adds an explicit async-blocking adapter surface. The initial Rust interop contract does not generate hidden blocking adapters, hidden `block_on`, or implicit offload for async Rust declarations.

Default async bridge requirements:

- generated futures must be `'static` when spawned,
- `Send` is required for work that leaves the current-thread runtime,
- non-`Send` futures are allowed only when pinned to the current Sifr Tokio runtime through `thread_affinity=tokio_current_thread` on the opaque type or through an explicit function-level `@rust.async(thread_affinity=tokio_current_thread)` declaration,
- non-`Send` futures without current-thread affinity are rejected,
- cancellation is cooperative and must map to stable Sifr cancellation errors,
- runtime shutdown must drain or cancel registered Rust interop tasks deterministically.

The initial compile-time async contract surface enforces that `@rust.async(...)` is valid only on `async def`, async Rust probes require returned futures to be `Send` by default, `thread_affinity=tokio_current_thread` is the only function-level non-`Send` opt-out, and blocking/CPU-heavy classifications are rejected on async Rust declarations with `SIFR-RUST-ASYNC-0001`.

For async Rust targets that borrow converted inputs, generated glue owns the converted values inside the wrapper future. The raw Rust future may borrow those owned wrapper values, but any Sifr-exposed future must satisfy Sifr async lifetime and spawn rules after wrapping. This allows ordinary Rust APIs such as `async fn fetch(url: &str) -> Result<_, _>` without exposing borrowed futures that outlive their generated wrapper state.

Free async functions that require current-thread affinity declare it explicitly:

```sifr
@rust.async(thread_affinity=tokio_current_thread)
@rust(bridge.local_client.fetch)
async def fetch(url: str) -> Result[Response, HttpError | RustPanicError]: ...
```

## Zero-Copy and Views

Zero-copy is first-class and explicit, including bytes, Arrow-style columnar data, tensor buffers, and application-defined views.

```sifr
@rust.zero_copy(owner=input, view=bridge.hash.DigestView)
@rust.view(
    owner=input,
    lifetime=owner,
    mutability=immutable,
    send=False,
    sync=False,
)
@rust(bridge.hash.digest_view)
def digest_view(input: bytes) -> Result[DigestView, HashError | RustPanicError]: ...
```

`@rust.zero_copy(...)` declares that the API cannot copy. `@rust.view(...)` declares the view's lifetime, mutability, and thread behavior. They compose: `@rust.zero_copy` is required whenever copying is prohibited, `@rust.view` is required whenever the return value is a borrowed view, and a borrowed zero-copy API uses both decorators.

```sifr
@rust.view(
    owner=input,
    lifetime=owner,
    mutability=immutable,
    send=False,
    sync=False,
)
@rust(bridge.parser.tokens_view)
def tokens_view(input: bytes) -> Result[TokenView, ParseError | RustPanicError]: ...
```

Allowed `lifetime` values are `call`, `owner`, and `static`. `call` views cannot escape the bridge call and are valid only for callback-local or bridge-internal views that do not become the function return value. Returned views must use `lifetime=owner` or `lifetime=static`. `owner` views are tied to the named owner parameter or opaque handle. `static` requires the Rust target to return owned or globally valid data and is rejected for borrowed returns. `mutability=mutable` requires exclusive Sifr ownership of the owner for the full view lifetime.

Rules:

- no silent copy fallback for `@rust.zero_copy` or `@rust.view`,
- borrowed views cannot outlive their owner,
- mutable views require exclusive Sifr ownership,
- aliasing is checked at the Sifr boundary,
- Send/Sync for views must be declared and validated,
- fallible downgrade to copying must be a different API name and declaration,
- views crossing async suspension points must satisfy the same lifetime and pinning requirements as native Sifr borrows.

The initial compile-time zero-copy contract surface enforces explicit `owner=` and `view=` on `@rust.zero_copy(...)`, explicit `owner=`, `lifetime=`, `mutability=`, `send=`, and `sync=` on `@rust.view(...)`, paired view contracts for zero-copy borrowed returns, returned-view rejection for `lifetime=call`, mutable-view rejection for non-exclusive owners, copy-fallback rejection, async borrowed-view suspension rejection, and view Send/Sync probe metadata. View probes now derive Send/Sync obligations from the explicit `@rust.view(...)` contract rather than implicit ABI flags, so contract authors must declare the thread behavior that generated probes certify. Runtime-observed generated wrapper behavior and crate-backed certification for `bytes`, `memmap2`, `bytemuck`, and `zerocopy` are future-owned by [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../plans/issues/active/rust-interop-runtime-ecosystem-certification.md).

Data-oriented bridges must support explicit contracts for:

- Python/Rust-independent buffers,
- Arrow C Data Interface compatible record batches and arrays,
- tensor buffers with shape, dtype, layout, strides, device, and ownership metadata,
- DLPack-style tensor handoff through a shared `sifr_tensor_bridge` crate,
- dataframe adapters that preserve ownership and schema identity.

The initial advanced data contract surface is contract-only and extends
`@rust.view(...)` with `data=`, `schema=`, `dtype=`, `rank=`, `shape=`,
`layout=`, `strides=`, `device=`, `ownership=`, and `protocol=` metadata. Arrow
and dataframe views require schema identity and the `sifr_arrow_bridge` shared
bridge crate root; tensor and DLPack views require dtype/shape/layout/strides
and CPU-only device metadata plus the `sifr_tensor_bridge` shared bridge crate
root. DLPack handoff must declare `ownership=transfer`, an owned owner
parameter, and `protocol=` explicitly. Runtime-observed crate-backed
certification for `arrow`, `datafusion`, `polars`, `ndarray`, and `candle` is
future-owned by
[`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../plans/issues/active/rust-interop-runtime-ecosystem-certification.md)
until both evidence directions pass.

## Callbacks

Callbacks are supported only with declared lifetime and threading policy.

Call-scoped callback:

```sifr
@rust(bridge.parser.visit)
def visit(text: str, callback: Callback[[Token], Result[None, ParseError]]) -> Result[None, ParseError | RustPanicError]: ...
```

The Rust implementation cannot store a call-scoped callback, call it after the bridge call returns, or call it from an unmanaged thread.

Thread-safe callback registration:

```sifr
@rust.callback(
    backpressure=bounded(1024),
    overflow=error,
    shutdown=drain,
)
@rust(bridge.kafka.on_message)
def on_message(
    consumer: KafkaConsumer,
    callback: ThreadsafeCallback[[Message], Result[None, KafkaError]],
) -> Result[Subscription, KafkaError | RustPanicError]: ...
```

Thread-safe callbacks require:

- captured values to satisfy the same Sifr task-spawn/offload ownership rule used for values that may leave the current task: no borrowed stack-local values, no non-send opaque handles, and no values with current-thread or current-OS-thread affinity,
- a returned cancellation/subscription handle,
- runtime entry guards for non-Sifr threads,
- explicit backpressure policy,
- explicit cancellation and shutdown behavior,
- panic-to-error handling at both callback and Rust boundaries.

Thread-safe callback policy values are:

- `backpressure=direct | bounded(N) | unbounded`
- `overflow=error | drop_oldest | drop_newest`
- `shutdown=drain | cancel | detach_forbidden`

`@rust.callback(...)` is required for `ThreadsafeCallback` parameters. Missing policy is a `SIFR-RUST-CB-*` diagnostic.

The initial callback contract surface is contract-only. It lowers
`@rust.callback(...)` metadata, requires named `backpressure=`, `overflow=`,
and `shutdown=` policy, rejects malformed or duplicate callback contracts with
`SIFR-RUST-CB-0001`, and makes top-level Sifr `Callable[[...], R]` bridge
parameters compatible only when the same declaration also has an explicit
callback contract. A single `@rust.callback(...)` policy applies uniformly to
all top-level callback parameters on that declaration; per-parameter callback
policy requires a later extension. Callable parameters without
`@rust.callback(...)`, nested callback containers, and callback returns remain
rejected as unsupported bridge types. Runtime-observed call-scoped storage
rejection, cross-thread capture enforcement, callback invocation panic mapping,
and `tokio-tungstenite`/`redis`/`notify` ecosystem certification are
future-owned by
[`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../plans/issues/active/rust-interop-runtime-ecosystem-certification.md)
through the `callbacks_call_scoped` and `callback_subscription_ecosystem`
compatibility rows. The supported `callback_subscription_core` row is limited
to the signal-style declaration and policy contract; it does not claim
subscription lifecycle execution.

## Trust Policy

Rust interop extends existing native trust policy with Rust-specific evidence:

```toml
[trust]
rust-build-scripts = ["openssl-sys", "tokenizer_backend"]
rust-proc-macros = ["serde_derive"]
native-links = ["openssl"]
unsafe-rust-bridges = ["src/bridges/tokenizer.rs"]
build-env = ["OPENSSL_DIR"]
rust-no-panic = ["crc32fast.hash", "bridge.hash.fast_hash"]
rust-panic-abort = ["legacy_backend.run"]
```

Trust gates:

- build scripts,
- procedural macros,
- native links reported through Cargo `links` metadata or trusted build-script output,
- unsafe code in first-party bridge files,
- environment variables consumed by build scripts,
- no-panic assertions through `rust-no-panic`,
- process-aborting panic profiles through `rust-panic-abort`,
- optional strict allowlist for Rust crate roots.

Safe Rust crates are not inherently unsafe, but code execution during build is. The trust model must separate build-time execution, native linking, unsafe bridge code, and ordinary safe dependency use.

Trust evidence is split into pre-execution and post-execution checks.

Pre-execution evidence must be rejected before any untrusted build script or proc macro runs:

- Cargo targets with `custom-build`,
- Cargo targets with `proc-macro`,
- manifest `links` keys,
- declared dependency roots and features,
- local bridge files that contain `unsafe` when not listed in `unsafe-rust-bridges`,
- declared build environment variables.

Post-execution evidence is allowed only after the build script or proc macro has already passed pre-execution trust:

- `cargo:rustc-link-lib` output,
- emitted `cfg`, environment, and link-search paths,
- generated native or bindgen artifacts,
- native library names discovered through trusted build output.

Native trust names native link identities such as `openssl` or `rdkafka`, not Rust sys crates. The sys crate that runs build code must be listed in `rust-build-scripts`; the native link it exposes must be listed in `native-links`.

Trust entries that name Rust call targets use canonical Sifr dotted target paths, not lowered Rust `::` paths. For direct bindings this is the target written in `@rust(...)`, such as `crc32fast.hash`; for package-local bridges this is the Sifr decorator target such as `bridge.hash.fast_hash`; for aborting legacy integrations this is the canonical target such as `legacy_backend.run`. Diagnostics point from the trust entry back to the matching decorator span.

Package-local bridges compile with `#![deny(unsafe_code)]` by default. Files listed in `unsafe-rust-bridges` receive a generated module-local allow boundary for that file only. Untrusted `unsafe` in a local bridge file produces `SIFR-RUST-TRUST-*` before final build acceptance; token scanning may provide early diagnostics, but rustc linting is the authoritative backstop.

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
- `SIFR-RUST-TRUST-*`: missing build-script, proc-macro, native-link, unsafe bridge, build-env, no-panic, or panic-abort trust,
- `SIFR-RUST-TYPE-*`: signature mismatch or unsupported bridge type,
- `SIFR-RUST-HANDLE-*`: invalid opaque handle ownership, close, clone, or thread contract,
- `SIFR-RUST-ASYNC-*`: hidden blocking, invalid future, unsupported non-`Send`, cancellation mismatch,
- `SIFR-RUST-ZC-*`: zero-copy/view lifetime, aliasing, mutability, or copy-fallback violation,
- `SIFR-RUST-CB-*`: callback lifetime, thread, backpressure, or shutdown violation,
- `SIFR-RUST-PANIC-*`: panic strategy or panic-boundary issue,
- `SIFR-RUST-CARGO-*`: Cargo metadata, feature, lockfile, or offline/frozen violation.

Every diagnostic must include source span, resolved target when available, required fix, and documentation URL.

The initial verification scaffold reserves the first code in every family:

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

LSP support expands through the same analysis/compiler facts that back `sifr check`:

1. Complete canonical Rust interop decorator dotted paths and policy keys (`@rust`, `@rust.async`, `@rust.opaque`, `@rust.zero_copy`, `@rust.view`, `@rust.callback`, and their policy arguments).
2. Resolve decorator roots from Sifr package metadata and Cargo metadata.
3. Parse package-local bridge module names and exported functions for completions.
4. Integrate rust-analyzer metadata for richer signatures and go-to-definition.

Completion must prefer valid dotted paths. Invalid string-style Rust targets are rejected instead of tolerated.

`sifr check` is the source of truth for Rust-backed packages. Plain `cargo check` on a source package may require prior Sifr projection generation because local bridge files can import generated `crate::__sifr_bridge` modules. Tooling must provide:

- `sifr bridge check` for bridge projection and probe validation through the same package check path,
- `sifr repair --check` for detecting missing or stale managed projection files,
- `sifr repair` for regenerating Sifr-managed projection files without touching user-owned bridge files.

## Verification Area

Rust interop creates a first-class verification area:

```text
verification/areas/rust_interop/
  README.md
  data/
    rust_interop_fixture_matrix.json
    rust_interop_compatibility_matrix.json
    rust_interop_tiers.toml
  fixtures/
    direct_crate_crc32/
    direct_crate_matrix/          # sha2, uuid, regex
    direct_crate_negative_type/
    dotted_path_resolution/
    bridge_type_matrix/           # serde, serde_json, thiserror, bytes, indexmap
    local_bridge_blake3/
    same_workspace_crate/         # cargo-probed workspace dependency behavior
    shared_bridge_crate/          # cargo-probed shared bridge crate boundary
    opaque_handle_tokenizer/
    opaque_resource_core/         # stdlib-owned opaque-resource lifecycle
    opaque_resource_matrix/       # reqwest::Client, rusqlite, tokio-postgres, redis
    close_after_use/
    bridge_version_mismatch/
    panic_boundary/               # contract-only panic-to-error behavior
    panic_boundary_wrapper_emission/
    panic_abort_profile/          # contract-only abort-profile rejection
    async_runtime_core/
    async_runtime_reqwest/
    async_ecosystem_matrix/       # futures, tower, http, http-body
    blocking_diagnostics/
    callbacks_call_scoped/
    callbacks_threadsafe/
    callback_subscription_core/   # signal-style stdlib subscriptions
    callback_subscription_ecosystem/ # tokio-tungstenite, redis pub/sub, notify
    zero_copy_bytes/
    zero_copy_view_matrix/        # memmap2, bytemuck, zerocopy
    arrow_record_batch/
    tensor_dlpack_bridge/         # contract-only DLPack ownership handoff
    advanced_data_matrix/         # datafusion, polars, ndarray, candle
    ecosystem_backend_certification/ # axum, tower-http, sqlx
    ecosystem_cli_certification/  # clap, tracing, tracing-subscriber, anyhow
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

- Tier 0 allows only `compiler-diagnostic`: parser, lowering, metadata, and
  diagnostics without a Cargo build claim.
- Tier 1 allows only `cargo-probe`: generated direct, local, same-workspace,
  and shared-bridge package builds.
- Tier 2 allows `contract-only`, `cargo-probe`, or `runtime-observed` evidence
  for opaque handles, panic boundaries, async/blocking, callbacks, and
  zero-copy; each row names its exact scope.
- Tier 3 allows only `cargo-probe` for build scripts, proc macros, native
  linking, trust, and offline/locked Cargo behavior.
- Tier 4 allows `contract-only`, `cargo-probe`, or `runtime-observed`
  production ecosystem evidence, explicitly scoped by each row.

Tier is breadth and ownership; `execution_kind` is evidence strength.
`contract-only` cannot satisfy a build or runtime claim. Compiler-diagnostic
rows that name crates carry a structured `diagnostic_crate_rationale` with
`linked = false` and `executed = false`; those crate examples supply diagnostic
API shapes only.

The Crate Verification Matrix must cover representative crates, not just synthetic fixtures.

Required core fixtures:

| Capability | Required crates |
| --- | --- |
| Direct compatible functions | `crc32fast`, `blake3`, `sha2`, `uuid`, `regex` |
| Bridge type generation and conversion | `serde`, `serde_json`, `thiserror`, `bytes`, `indexmap` |
| Build and proc-macro trust | `serde_derive`, `prost-build` |
| Native/build links | `cc`, `bindgen`, `cxx`, `zstd` |
| Async/Tokio ecosystem | `tokio`, `futures`, `reqwest`, `tower`, `http`, `http-body` |
| Opaque resources | `reqwest::Client`, `rusqlite`, `tokio-postgres`, `redis` |
| Blocking and CPU-heavy calls | `rusqlite`, `rayon`, `flate2` |
| Zero-copy core views | `bytes`, `memmap2`, `bytemuck`, `zerocopy` |

Required advanced fixtures:

| Capability | Required crates |
| --- | --- |
| Arrow and dataframe exchange | `arrow`, `datafusion`, `polars` |
| Tensor and array exchange | `ndarray`, `candle` |
| Thread-safe callbacks and subscriptions | `tokio-tungstenite`, `redis` pub/sub, `notify` |

Ecosystem certification fixtures:

| Area | Representative crates |
| --- | --- |
| Backend/service certification | `axum`, `tower-http`, `sqlx` |
| CLI/tooling certification | `clap`, `tracing`, `tracing-subscriber`, `anyhow` |

Ecosystem certification for `axum`, `tower-http`, and `sqlx` is limited to canonical-package compilation and probe coverage; product-level web framework workflows remain out of the Rust interop scope.

`tokio` runtime behavior is exercised through `async_runtime_reqwest`, `async_ecosystem_matrix`, `opaque_resource_matrix`, and `callback_subscription_ecosystem`; do not add a redundant standalone Tokio fixture unless a future runtime contract requires it.

Feature-sensitive fixtures must pin Cargo features in `rust_interop_fixture_matrix.json`:

- `reqwest`: `default-features = false`, `features = ["rustls-tls", "json"]`; do not enable `blocking` in async fixtures.
- `tokio-postgres`: `default-features = false`, `features = ["runtime"]`; TLS is not part of the primary opaque-resource fixture.
- `rusqlite`: `features = ["bundled"]`; the unbundled system-sqlite variant is intentionally not certified in the Rust interop scope.
- `redis`: `default-features = false`, `features = ["tokio-comp"]`; pub/sub fixtures use loopback service infrastructure.
- `tokio-tungstenite`: `default-features = false`; add `features = ["rustls-tls-webpki-roots"]` only for explicit network/TLS coverage.
- `sqlx`: `default-features = false`, `features = ["runtime-tokio-rustls", "postgres", "macros"]`; query-macro fixtures must use checked-in `.sqlx/` offline artifacts instead of requiring `DATABASE_URL` during Cargo execution.
- `axum` and `tower-http`: use default feature sets unless a fixture documents a narrower feature requirement.
- `tracing-subscriber`: include `env-filter`.
- `flate2`: `default-features = false`, `features = ["rust_backend"]`.
- `candle`: CPU-only default backend; GPU and accelerator backend features are out of scope for Rust interop.
- `prost-build`: use default features over a checked-in `.proto` input; generated output must be deterministic.

Runtime-service fixtures must declare whether they are compile/probe-only, loopback-service backed, or in-process stub backed. `reqwest`, `tokio-tungstenite`, and `notify` fixtures should prefer loopback or local filesystem inputs. `tokio-postgres` and `redis` fixtures require explicit local service configuration and must be skippable only by fixture-tier policy, not by silently degrading the interop behavior under test.

The matrix proves the Rust interop contract and package model. It does not create first-party Sifr wrappers for every listed crate and it does not move game, GUI, embedded, or product-level web framework work into Rust interop.

The area must record positive and negative fixtures for every declared capability. A feature is not complete until its failure mode is as deliberate as its success path.

### Compatibility Matrix

`verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`
is the machine-readable source for Rust interop support statements. Every fixture
listed in `rust_interop_fixture_matrix.json` must have exactly one compatibility
row. The compatibility validator rejects any `supported`,
`supported-through-bridge`, or `unsupported-by-design` row unless both positive
and negative fixture evidence are `passing`.

Compatibility categories are:

- `supported`: passing positive and negative fixture evidence for the stated
  execution kind.
- `supported-through-bridge`: passing evidence when the package author uses an
  explicit local or shared bridge contract; direct binding is not implied.
- `unsupported-by-design`: passing diagnostics for a rejected surface with no
  fallback path.
- `future-owned-by-separate-phase`: documented separately because at least one
  evidence direction is not passing. Future-owned rows must reference a concrete
  active issue or phase; the current active issue is
  [`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../plans/issues/active/rust-interop-runtime-ecosystem-certification.md).
