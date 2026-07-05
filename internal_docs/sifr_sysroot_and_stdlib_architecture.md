# Sifr Sysroot and Stdlib Architecture

Status: target architecture for the Sifr toolchain, stdlib, runtime, and
distribution model.

## Purpose

Sifr is distributed as a versioned toolchain, not as a standalone compiler
binary. The installed toolchain contains the compiler executable, the public
Sifr standard library sources, private standard-library declaration modules,
Rust crates required by generated programs, and vendored third-party Cargo
sources required by Sifr-owned runtime and stdlib crates.

This architecture prevents release binaries from depending on build-machine
paths, gives CLI and LSP the same view of the installed standard library, and
provides a stable substrate for moving compiler-special native stdlib plumbing
onto Rust interop.

## Pre-Migration Baseline

This section records the current implementation boundary before sysroot and
stdlib crate movement starts. It is intentionally descriptive, not a target
state. Later implementation stages update the implementation toward the architecture below.

| Surface | Current owner | Final owner | Migration blocker | Can move before runtime certification? |
| --- | --- | --- | --- | --- |
| Public Sifr stdlib sources | `stdlib/sifr/*.sifr`, validated by `crates/sifr_stdlib_model` and loaded from the resolved source-tree or installed sysroot path | `stdlib/sifr/*.sifr` copied into `<sysroot>/lib/sifr/stdlib/sifr` and loaded through `ResolvedSysroot` | complete for source layout; packaging copy remains later | yes |
| Private stdlib declarations | `stdlib/_sifr/*.sifr` placeholder declaration files exist and are validated against the compiler intrinsic registry; compiler intrinsic metadata remains the current signature owner until Rust interop migration | `stdlib/_sifr/*.sifr` declarations copied into `<sysroot>/lib/sifr/stdlib/_sifr` and loaded through the same sysroot source inventory | concrete Rust interop declarations, synthetic stdlib interop context | mixed |
| Compiler-side stdlib model | `crates/sifr_stdlib_model` owns source inventory, private intrinsic metadata, feature/dependency mapping, and legacy module suggestions | `crates/sifr_stdlib_model` | complete | yes |
| Generated-program stdlib implementation crate | `crates/sifr_stdlib` exists as the generated-program crate foundation with empty defaults, narrow additive leaf features, runtime-backed wrapper APIs for existing runtime primitives, and feature-plan expectations in `sifr_stdlib_model` | `crates/sifr_stdlib`, shipped under `<sysroot>/crates/sifr_stdlib` | full native leaf migration, generated Cargo sysroot dependency emission, and installed sysroot packaging | yes |
| Runtime crate | `crates/sifr_runtime` in the development workspace; generated Cargo finds it through `SIFR_RUNTIME_PATH`, ancestor scanning, or `env!("CARGO_MANIFEST_DIR")` fallback | `<sysroot>/crates/sifr_runtime` path dependency selected by `ResolvedSysroot` | Sysroot resolver, generated dependency plan | yes |
| Generated Cargo planning | `sifr_codegen::generate_project_with_deps_and_crates` asked `sifr_stdlib_model::generated_cargo_dependencies` for dependency strings | `SysrootDependencyPlan` from `sifr_stdlib_model`, consumed by codegen, driver, cache keys, reports, and LSP traces | dependency planner | yes |
| Third-party stdlib/runtime dependencies | Generated projects emit registry dependencies directly, for example `regex`, `serde_json`, `tokio`, `url`, `zip`, and others | Vendored under `<sysroot>/vendor` from the sysroot workspace lockfile for Sifr-owned dependencies | vendor and Cargo config mode matrix | yes |
| Distribution packaging | Preview/self-update artifacts and receipts pair the standalone binary with installer metadata; no sysroot contract is validated | Release archive contains `bin/sifr` plus the complete sysroot tree and replaces them atomically | installer and release artifact update | yes |
| CLI stdlib loading | `sifr_driver::compile_stdlib` resolves `ResolvedSysroot`, validates the public/private stdlib inventory, and compiles physical public stdlib files with source paths | CLI resolves `ResolvedSysroot` and loads physical sysroot stdlib files | complete for public source loading; private declaration lowering remains later | yes |
| LSP/tooling stdlib loading | Analysis hosts call `sifr_driver::stdlib_external_defs()`, so tooling observes the same sysroot-loaded stdlib definitions as CLI | LSP/tooling load source and declaration metadata from the same `ResolvedSysroot` as CLI | deeper stdlib source navigation | yes |

Current public stdlib sources import private `_sifr.*` modules as follows. User
code still cannot import `_sifr.*`; these rows describe only stdlib wrapper
implementation imports under `stdlib/sifr`.

| Private module | Current public wrappers |
| --- | --- |
| `_sifr.bytes` | `bytes.sifr`, `hashlib.sifr` |
| `_sifr.calendar` | `calendar.sifr` |
| `_sifr.collections` | `collections.sifr` |
| `_sifr.compress` | `gzip.sifr`, `zipfile.sifr` |
| `_sifr.crypto` | `base64.sifr`, `hashlib.sifr`, `random.sifr`, `secrets.sifr`, `tempfile.sifr` |
| `_sifr.datetime` | `datetime.sifr` |
| `_sifr.encoding` | `encoding.sifr` |
| `_sifr.fs` | `glob.sifr`, `hashlib.sifr`, `io.sifr`, `json.sifr`, `os.sifr`, `pathlib.sifr`, `shutil.sifr`, `tempfile.sifr` |
| `_sifr.html` | `html.sifr` |
| `_sifr.http` | `http.sifr` |
| `_sifr.i18n` | `i18n.sifr` |
| `_sifr.json` | `json.sifr` |
| `_sifr.logging` | `logging.sifr` |
| `_sifr.math` | `math.sifr` |
| `_sifr.net` | `net.sifr` |
| `_sifr.platform` | `platform.sifr` |
| `_sifr.process` | `process.sifr` |
| `_sifr.python` | `python.sifr`, `python_core.sifr` |
| `_sifr.regex` | `re.sifr` |
| `_sifr.runtime` | `runtime.sifr` |
| `_sifr.signal` | `signal.sifr` |
| `_sifr.sys` | `env.sifr`, `os.sifr`, `sys.sifr` |
| `_sifr.task` | `task.sifr` |
| `_sifr.time` | `datetime.sifr`, `random.sifr`, `time.sifr`, `timeit.sifr` |
| `_sifr.tls` | `tls.sifr` |
| `_sifr.toml` | `tomllib.sifr` |
| `_sifr.unicode` | `unicode.sifr` |
| `_sifr.url` | `url.sifr` |
| `_sifr.uuid` | `uuid.sifr` |

Current generated dependency ownership is mixed. The generated dependency target is for
Sifr-owned dependencies to come from sysroot path crates and sysroot vendor
data, while user/package dependencies remain package-owned Cargo inputs.

| Current generated dependency group | Current owner | Final owner | Migration blocker |
| --- | --- | --- | --- |
| `sifr_runtime` | Sifr-owned path dependency discovered from `SIFR_RUNTIME_PATH`, source-tree ancestors, executable ancestors, or compile-time checkout fallback | Sysroot-owned path dependency under `<sysroot>/crates/sifr_runtime` | Sysroot resolver and `SysrootDependencyPlan` |
| `base64`, `blake2`, `md5`, `sha1`, `sha2`, `regex`, `toml`, `url`, `uuid`, `zip`, `flate2`, `chrono`, `rand`, `rand_distr`, `rayon`, `rust_decimal`, `bigdecimal`, `num-bigint`, `num-traits`, `percent-encoding` | Sifr-owned implementation details emitted directly into generated project manifests | Vendored third-party inputs of sysroot crates; generated manifests should not expose them after migration | generated-program stdlib crate implementation and vendor/dependency planning |
| `serde`, `serde_json`, `postcard`, `bytes`, `tokio`, `tokio-rustls`, `rustls`, `rustls-pemfile`, `rustls-platform-verifier`, `http`, `http-body`, `http-body-util`, `hyper`, `hyper-util`, `h2`, `tower-service`, `metrics`, `tracing`, ICU crates | Sifr-owned runtime/stdlib implementation details emitted directly when selected features require them | Vendored third-party inputs of sysroot crates; direct generated dependencies remain only for retained compiler-language glue with an allowlist | generated-program stdlib crate, dependency-plan, fallible-data, and retained-glue decisions |
| User package dependencies and explicit Rust interop dependencies | Package-owned Cargo graph | Package-owned Cargo graph | None; sysroot vendor policy must not silently replace normal package resolution |

Current generated code and preamble call into `sifr_runtime::*` from these
families. This table groups the call sites by migration concern; the complete
surface-by-surface ownership decision remains the TOML registry.

| Call-site family | Current call sites | Final owner |
| --- | --- | --- |
| Exact integer/runtime bridge | Entry-point imports, module constants, SifrInt render helpers, Rust interop bridge generated types | Retained compiler-language glue backed by `sifr_runtime` |
| JSON exact-int/value helpers | `sifr_stdlib` JSON token adapters reached through `_sifr.json` private Rust interop declarations; shared integer policy primitives remain in `sifr_runtime::json` | `sifr_stdlib` JSON implementation with only shared primitives retained in `sifr_runtime` |
| Encoding, Unicode, and i18n helpers | `_sifr.encoding`, `_sifr.unicode`, and `_sifr.i18n` private Rust interop declarations backed by `sifr_stdlib` | `sifr_stdlib` text/data implementations with shared primitives retained only when justified |
| Network handles | `preamble/net_runtime.rs` | `sifr_stdlib` net resource implementation through certified interop, backed by `sifr_runtime` substrates |
| TLS handles | `preamble/tls_runtime.rs` | `sifr_stdlib` TLS resource implementation through certified interop, backed by `sifr_runtime` substrates |
| HTTP transport | `preamble/url_http_runtime.rs` | `sifr_stdlib` HTTP resource implementation through certified interop, backed by `sifr_runtime` substrates |
| Python objects, buffers, callbacks, and contexts | `registry/python.rs` | `sifr_stdlib` Python interop surface through certified object/resource/callback interop, backed by `sifr_runtime` substrates |

The broad native migration registry used during earlier migration work has been
retired. Final ownership is now determined by location: public APIs live in
`stdlib/sifr`, private native declarations live in `stdlib/_sifr`,
generated-program implementation lives in `crates/sifr_stdlib` and
`crates/sifr_runtime`, and compiler intrinsics are language/runtime semantics
only. Remaining compiler-native stdlib glue is explicitly allowlisted in
`internal_docs/stdlib_retained_compiler_intrinsics.toml`.

Rust interop runtime certification is not fully complete for resource-shaped
surfaces. The active matrix
`verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`
currently has 14 supported rows, 5 bridge-supported rows, 1
unsupported-by-design row, and 11 rows owned by separate certification work. The
separately owned rows are `bridge_type_matrix`, `opaque_resource_matrix`,
`panic_boundary_wrapper_emission`, `async_runtime_reqwest`,
`callbacks_call_scoped`, `callback_subscription_matrix`,
`ecosystem_backend_certification`, `ecosystem_cli_certification`,
`native_build_script`, `proc_macro_trust`, and `cargo_locked_offline`. Resource
migrations must not claim stable support for any row that remains separately
owned by certification work.

## Stdlib Rust Interop Adapter Policy

Native stdlib migration uses the same two-level Rust interop model as packages.
Private `_sifr.*` declarations may bind directly to `sifr_stdlib` functions
only when the Rust signature is already bridge-compatible and exposes the
intended Sifr-shaped error surface. When a stdlib operation needs input
normalization, output conversion, preservation of existing public semantics, or
typed error shaping, the targeted Rust function is a `sifr_stdlib` adapter that
owns those conversions before calling lower-level implementation code.

The compiler should not add per-declaration converter pipelines for the stdlib
rewrite. Generated glue validates the single `@rust(...)` target signature,
records the sysroot interop dependency and trust metadata, and lets
`sifr_stdlib` own implementation adaptation. These migrations are committed to
`bridge-version = 1`; any future callee-injection form requires a new
bridge-versioned design and must not add fallback conversion behavior to existing
sysroot interop declarations.

Private stdlib declarations rely on the compiler-owned sysroot trust policy and
`sifr_stdlib` crate-level no-panic conventions for public APIs whose error
surface does not include `RustPanicError`; user packages do not inherit that
trust.

Resource, async, callback, and runtime-state surfaces stay behind the active
Rust interop runtime certification gate until their lifecycle behavior is
executable evidence, not just adapter code.
`scripts/check_sysroot_stdlib_resource_certification_gate.py` enforces this
boundary during validation by pinning each resource-sensitive stdlib surface to
its required Rust interop compatibility matrix rows. Those rows must remain
`future-owned-by-separate-phase` and owned by the runtime ecosystem
certification issue until that separate work updates the gate deliberately.

Retained compiler-native stdlib glue is guarded separately by
`internal_docs/stdlib_retained_compiler_intrinsics.toml` and
`scripts/check_stdlib_native_intrinsic_allowlist.py`. The guard compares the
active `sifr_codegen` intrinsic dispatcher, registry module files, and preamble
module files against the allowlist so new compiler-native stdlib behavior must
be reviewed as retained language/runtime glue instead of drifting back into
generic stdlib intrinsics.

## Installed Layout

The canonical standalone installation layout is:

```text
~/.sifr/
  bin/
    sifr
  Cargo.toml
  Cargo.lock
  sysroot.toml
  .cargo/
    config.toml
  vendor/
  crates/
    sifr_runtime/
      Cargo.toml
      src/
    sifr_stdlib/
      Cargo.toml
      src/
  lib/
    sifr/
      stdlib/
        sifr/
          *.sifr
        _sifr/
          *.sifr
```

`~/.sifr/` is the Sifr sysroot and toolchain root. The sysroot is versioned as
one unit with `~/.sifr/bin/sifr`. Release archives contain the executable and
the full sysroot tree. The installer replaces them together.

The sysroot is also a self-contained Cargo workspace. `Cargo.toml` owns
workspace package metadata and dependency versions for shipped sysroot crates.
`Cargo.lock` records the exact third-party dependency graph used to produce
`vendor/`.

`vendor/` contains third-party Cargo sources needed by Sifr-owned sysroot
crates. Generated projects use the sysroot Cargo config so normal stdlib usage
does not require network access after installation. User package dependencies
remain user-owned Cargo inputs and are not part of the Sifr sysroot contract.

The canonical repository source roots for standard-library Sifr code are:

```text
stdlib/sifr/*.sifr
stdlib/_sifr/*.sifr
```

Packaging copies those roots into `<sysroot>/lib/sifr/stdlib/**` in the
installed sysroot. The completed architecture has one canonical stdlib source
root in the repository so CLI, LSP, packaging, and tests cannot drift across two
long-lived source trees.

## Sysroot Manifest

`sysroot.toml` records the installed toolchain contract:

```toml
"schema-version" = 1
"sifr-version" = "0.1.0-beta.N"
"target-triple" = "x86_64-unknown-linux-gnu"
"built-by-compiler-commit" = "<git-sha>"
"sysroot-content-sha256" = "<sha256-tree>"
"cargo-lock-sha256" = "<sha256>"
```

The compiler treats a mismatched or incomplete sysroot as an installation
error. It reports a Sifr diagnostic that tells the user to reinstall Sifr or set
`SIFR_SYSROOT`. It must not silently fall back to `env!("CARGO_MANIFEST_DIR")`
in released builds.

The manifest is identity metadata, not a layout map. Paths and crate names are
fixed by the sysroot layout and Cargo manifests. `toolchain-id` is derived as
`{sifr-version}-{target-triple}` where a human-readable identifier is needed.
Rust edition, minimum Rust version, workspace package data, and sysroot crate
versions belong to `Cargo.toml`, not to `sysroot.toml`.

`sysroot-content-sha256` covers `lib/sifr/`, `crates/`, `vendor/`,
`.cargo/config.toml`, `Cargo.toml`, and `Cargo.lock` for release sysroots.
`cargo-lock-sha256` is kept as a direct lockfile identity because Cargo
resolution and bug reports often need it independently. Normal commands do not
re-hash the entire sysroot or vendor tree on every invocation. They trust the
manifest after the small boundary check described below; deeper digest
verification belongs to release validation and `sifr doctor`.

The manifest schema is implementation-owned and documentation-visible. Parser
tests or snapshots must fail when the accepted schema and this documented schema
drift from each other.

Unknown required fields fail manifest parsing. Unknown optional fields may be
ignored only when the active `schema-version` explicitly permits that behavior.
Schema version 1 permits keys prefixed with `optional-`.

Tree digests are canonical. `crates/sifr_sysroot` sorts normalized relative
paths bytewise, uses `/` path separators, includes `.toml`, `.lock`, `.rs`, and
`.sifr` files by default, excludes timestamps/owners/host packaging metadata,
normalizes line endings to LF, skips symlinks unless a caller opts into
following them, and includes executable-bit state where the host reports it.

## Sysroot Resolution

Every compiler, CLI, LSP, and build operation starts by resolving a
`ResolvedSysroot`. After that point, installed and development workflows use the
same compiler, stdlib inventory, Cargo planning, and tooling paths.

Sysroot resolution is deterministic:

1. An explicit `--sysroot <path>` CLI option, where accepted by developer
   tooling.
2. `SIFR_SYSROOT`.
3. Installed layout relative to the running executable:
   if the executable is under `<toolchain>/bin/`, the sysroot root is
   `<toolchain>/`.
4. Development sysroot auto-resolution for unreleased local builds when a source
   checkout itself contains the skeleton layout.

Runtime checkout testing uses a source-tree or generated development sysroot
that contains the desired runtime crate. Released tools and normal development
flows do not support a runtime-only override environment variable; tests that
still need old runtime-path behavior during migration should use test-only
helpers rather than a documented environment variable. In release mode, missing
sysroot assets are surfaced as install corruption, not as Cargo path errors.

Development sysroot auto-resolution is gated by a single predicate:

```text
is_source_tree_development_mode()
```

That predicate is true only for local development builds under an explicit
debug/dev build marker or equivalent reviewed build-time configuration. It
answers only whether the tool may auto-resolve or materialize a development
sysroot. It does not allow different stdlib inventory rules, runtime path
rules, embedded stdlib fallback, or repository ancestor scanning after
resolution. A released binary must not auto-resolve a repository sysroot merely
because the current directory, an ancestor, or the executable path contains
directories named `crates/sifr_runtime` or `crates/sifr_stdlib`. Released
binaries either resolve a valid installed sysroot, honor an explicit sysroot
override, or emit a sysroot diagnostic.

End-user `build`, `run`, `check`, and `emit` commands may support `--sysroot`
only as an advanced developer override. Diagnostics and help text should mark
it as such rather than presenting it as a normal package-management mechanism.
LSP may accept a sysroot from settings or environment, but should report
mismatches with the CLI-observed sysroot when both are available.

Command support for sysroot overrides is intentionally narrow:

```text
sifr check/build/run/emit: allowed as hidden or advanced help.
sifr lsp: allowed through settings or environment, not normal CLI UX.
sifr doctor: allowed and visible for install inspection.
sifr self update: ignored or rejected unless multi-sysroot installs are
                  explicitly designed.
```

## Sysroot Cargo Workspace

The sysroot Cargo workspace exists so shipped sysroot crates are independent of
the source checkout. Sysroot crate manifests may use workspace-inherited
dependency versions, lints, edition, and metadata, but that inheritance must
resolve inside `~/.sifr/Cargo.toml`, not the development repository.

The distributed sysroot workspace manifest has this shape:

```toml
[workspace]
members = [
  "crates/sifr_runtime",
  "crates/sifr_stdlib",
]
resolver = "2"

[workspace.package]
version = "0.1.0-beta.N"
edition = "2024"
rust-version = "<minimum-supported-rust-version>"
license = "<license>"

[workspace.dependencies]
bytes = "=..."
encoding_rs = "=..."
rand = "=..."
serde = { version = "=...", features = ["derive"] }
tokio = { version = "=...", features = [...] }
```

Distribution packaging produces sysroot-ready manifests in the fixed layout.
The released result must be self-contained: running `cargo metadata --offline`
from the sysroot workspace succeeds without the source checkout.

The sysroot Cargo config is stored at `.cargo/config.toml` and uses Cargo's
standard vendoring model:

```toml
[source.crates-io]
replace-with = "sifr-vendor"

[source.sifr-vendor]
directory = "vendor"
```

Sifr does not copy sysroot Cargo config into user/package project directories.
It applies sysroot Cargo configuration only through invocation-scoped Cargo
configuration for Sifr-managed builds where this policy permits it:

- Single-file generated builds using only Sifr stdlib/runtime dependencies
  apply sysroot vendor config.
- Generated package builds with no user registry dependencies apply sysroot
  vendor config.
- Package builds with user registry dependencies in default online mode do not
  silently force sysroot-only vendor config onto user dependencies.
- Package builds with explicit `--offline`, `--frozen`, or equivalent locked
  offline mode use a complete combined graph/vendor source or fail with a clear
  Cargo/Sifr diagnostic.
- The initial implementation may choose the clear failure path for
  offline/frozen package builds with user registry dependencies until a
  complete combined vendor graph is designed.
- Explicit Rust interop package dependencies keep the dependency source
  package-owned; Sifr does not hide them behind sysroot vendor policy.

The observable guarantee is that Sifr-owned sysroot dependencies resolve from
the installed sysroot in offline mode without changing ownership of user
dependencies.

Invocation-scoped Cargo config tests prove sysroot vendor config applies to
Sifr-owned stdlib/runtime dependencies in stdlib-only builds and does not
silently replace user registry resolution in online package mode.

The sysroot vendor directory is generated from the sysroot workspace lockfile,
not from a manually curated dependency list. This prevents stdlib features and runtime
features from drifting away from the shipped vendor set.

For stdlib-only generated builds, Cargo resolution must be reproducible from
the sysroot workspace lock/vendor set. If a generated project needs its own
`Cargo.lock`, that lockfile is produced from the resolved sysroot-compatible
graph and validated by offline generated-project fixtures, not only by
`cargo metadata` from the sysroot workspace.

## Crate Boundaries

### `sifr_runtime`

`sifr_runtime` is the low-level generated-program runtime crate. It owns
foundational runtime primitives and shared substrates:

- exact integer support and runtime numeric bridge helpers,
- Rust interop bridge types and opaque-handle helpers,
- Python runtime state and Python object/resource handles,
- network, TLS, HTTP, timeout, and async runtime substrates,
- JSON, encoding, Unicode, and i18n runtime primitives already used by
  generated code,
- other stateful or cross-cutting runtime services that generated language glue
  must call directly.

Generated code may depend on `sifr_runtime` directly when language glue needs
runtime primitives. Public stdlib wrappers should prefer `sifr_stdlib` where a
native operation is part of the standard library surface rather than language
runtime glue.

### `sifr_stdlib`

`sifr_stdlib` is the Rust-native implementation crate for Sifr standard library
operations. It is a generated-program dependency, shipped in the sysroot, and
versioned with the compiler.

The initial implementation crate exists in the development workspace with
`default-features = false`, narrow leaf features, and small wrapper APIs around
runtime primitives that already live in `sifr_runtime`. Later migration stages
move individual native leaves out of compiler/codegen surfaces and into this
crate behind the same feature names.

Current migrated leaves include the stateless `_sifr.platform`, `_sifr.html`,
`_sifr.calendar`, `_sifr.uuid`, `_sifr.math`, `_sifr.regex`, `_sifr.url`,
`_sifr.toml`, `_sifr.datetime`, and `_sifr.compress` private declaration
modules, plus the hash helper subset and full base encoding helper subset of
the shared `_sifr.crypto` private module used by `sifr.hashlib` and
`sifr.base64`, and the `encode_utf8`/`bytes_to_hex` helper subset of
`_sifr.bytes`.
Their public wrappers continue to live in
`stdlib/sifr/platform.sifr`, `stdlib/sifr/html.sifr`,
`stdlib/sifr/calendar.sifr`, `stdlib/sifr/uuid.sifr`,
`stdlib/sifr/math.sifr`, `stdlib/sifr/re.sifr`,
`stdlib/sifr/url.sifr`, `stdlib/sifr/tomllib.sifr`,
`stdlib/sifr/hashlib.sifr`, `stdlib/sifr/base64.sifr`,
`stdlib/sifr/datetime.sifr`, `stdlib/sifr/gzip.sifr`,
`stdlib/sifr/zipfile.sifr`, and `stdlib/sifr/bytes.sifr`, while generated code
emits the private preamble functions and calls `sifr_stdlib::platform::*`,
`sifr_stdlib::html::*`, `sifr_stdlib::calendar::*`,
`sifr_stdlib::uuid::*`, `sifr_stdlib::math::*`, `sifr_stdlib::regex::*`,
`sifr_stdlib::url::*`, `sifr_stdlib::toml::*`, `sifr_stdlib::hash::*`,
`sifr_stdlib::base64::*`, `sifr_stdlib::time::*`,
`sifr_stdlib::gzip::*`, `sifr_stdlib::zipfile::*`, and
`sifr_stdlib::bytes::*` through feature-gated sysroot dependencies. Direct
Rust interop wrappers bridge Sifr `int` values through
`sifr_runtime::interop::SifrIntBridge` at this boundary, including `list[int]`
calendar returns and `int | None` URL ports; `str | None` URL query arguments
clone into `Option<String>` at the sysroot crate boundary. Public math aggregate
helpers such as `dist`, `fsum`, and `sumprod` keep read-only list parameters in
`stdlib/sifr/math.sifr` and copy into private owned-vector Rust interop helpers,
so private bridge ownership does not leak into the public API.
`stdlib/sifr/hashlib.sifr` uses the same boundary pattern for string and bytes
helpers: public `sha*`, `md5`, and `blake2*` functions wrap private underscored
aliases imported from `_sifr.crypto`, keeping borrowed Rust interop parameters
out of public generated call sites. `stdlib/sifr/bytes.sifr` also wraps
`_sifr.bytes` helper declarations so public `encode_utf8` and `bytes_to_hex`
calls with literals and owned bytes do not expose private borrowed Rust
interop signatures. `bytes.from_hex`, `bytes.from_ints`, `bytes(size)`,
`bytes_to_hex_strict`, `str.encode`, and `bytes.decode` remain retained
compiler-owned language glue covered by the retained native-surface allowlist.
`stdlib/sifr/base64.sifr` uses the same
pattern for base64, URL-safe base64, base32, and base32hex encoders, decoders,
and option helpers. `stdlib/sifr/re.sifr` uses the same wrapper pattern for
regex match, find, replace, findall, split, match start/end, and flag variants.
`stdlib/sifr/url.sifr` keeps public `Url` and `UrlQuery` records in Sifr and
reconstructs them from flat private `list[str]` bridge payloads returned by
`sifr_stdlib::url`, because generated tuple and record bridge types are not
sysroot crate API. `stdlib/sifr/tomllib.sifr` keeps public `TomlValue` in Sifr
and reconstructs it from flat private `list[str]` bridge tokens returned by
`sifr_stdlib::toml`, for the same sysroot API boundary reason. `sifr.pathlib`
still retains a direct `regex`
implementation dependency for path-glob lowering until the filesystem/path
surface migrates.
Direct Rust interop maps Rust `Result[..., E: Display]` returns into
message-shaped Sifr error classes such as `ParseError` at the generated wrapper
boundary. It also supports error subclasses whose fields are all strings by
filling `message` and detail fields from the Rust error display text, which
covers `RegexError { message, detail }`. Random helpers remain on intrinsic
fallback until their stateful surface is migrated.

When sysroot Rust interop activates native-link evidence validation in a
generated project that also embeds Python, the selected packaged Python runtime
contributes its own `libpython` link name to the trusted set. That trust is tied
to the interpreter selected by the Python runtime probe and does not make
arbitrary Rust build-script native links trusted.

It owns Rust implementations for stdlib native leaves and resource operations
that are not compiler semantics:

- math, hashing, base encodings, UUID, regex, TOML, HTML, calendar, and other
  stateless native leaves,
- filesystem, environment, platform, path, process, signal, compression, URL,
  HTTP, net, TLS, Python, Unicode, and i18n stdlib-facing functions where the
  surface is exposed through `stdlib/sifr`,
- stdlib-specific error adapters and result types used by private `_sifr`
  declarations.

`sifr_stdlib` may depend on `sifr_runtime` for lower-level primitives. Generated
projects can depend on both crates when necessary, but stdlib wrappers should
route through `sifr_stdlib` where it provides the stable native implementation.

The `sifr_stdlib` crate should avoid re-exporting every runtime primitive. It is
the standard library implementation layer, not a facade over the whole runtime.
Direct `sifr_runtime` dependencies remain appropriate for generated language
glue, Rust interop bridge glue, and shared resource substrates.

Generated dependencies on `sifr_stdlib` use `default-features = false`.
Capability features are narrow, additive leaves. Human-facing umbrella features
may exist for maintenance, but generated Cargo prefers the minimal leaf set.
Representative leaf features include:

```text
json
regex
uuid
hash
base64
toml
url
gzip
zipfile
unicode
i18n
net
tls
http
python
process
fs
signals
runtime-observability
```

Feature tests and generated Cargo snapshots prove that using one stdlib module
does not enable unrelated capability groups.

### `sifr_stdlib_model`

`sifr_stdlib_model` is the compiler-side model of the standard library. It is
linked into the compiler and is not shipped as a generated-program dependency.
It was renamed from the compiler crate previously named `sifr_stdlib`.

It owns:

- stdlib source inventory and module metadata,
- public `sifr.*` import policy and private `_sifr.*` rejection policy,
- legacy CPython-shaped module suggestions,
- private declaration module metadata,
- mapping from stdlib declarations and codegen requirements to generated Cargo
  dependencies and sysroot crate features,
- IPC schema/protocol metadata needed by compiler analysis and verification.

The name `sifr_stdlib` is reserved for the generated-program stdlib crate. The
compiler model uses a distinct name so crate names communicate runtime
responsibility rather than internal implementation history.

`sifr_stdlib_model` speaks in terms of resolved sysroot/module metadata rather
than hard-coded include paths. Released tools must not use embedded fallback
sources for normal stdlib resolution. If the sysroot is missing or invalid,
released tools fail with a sysroot diagnostic.

## Standard Library Source Layers

The public standard library remains Sifr source:

```text
<sysroot>/lib/sifr/stdlib/sifr/*.sifr
```

These modules define the user-facing APIs:

```sifr
from sifr.json import loads
from sifr.io import open
from sifr.math import sqrt
```

They are the stable wrapper layer for CPython-shaped ergonomics, Sifr safety
choices, input validation, type shaping, and compatibility behavior.

Public stdlib sources are user-inspectable. They should favor readable Sifr code
and stable documentation comments over generated declarations. When a public
wrapper exists only to forward to a private native declaration, the wrapper
still owns user-facing names, defaults, error wording, and compatibility
behavior.

Private declaration modules live under:

```text
<sysroot>/lib/sifr/stdlib/_sifr/*.sifr
```

Only sysroot stdlib modules may import `_sifr.*`. User code importing `_sifr.*`
is rejected. Private declaration modules declare native stdlib operations using
Rust interop annotations such as direct functions, opaque handles, async
functions, view/zero-copy contracts, and callback policies.

Private declarations are source files rather than Rust tables so editor tooling
and compiler behavior share the same signature source. They are ordinary Rust
interop declarations with a sysroot trust policy and private source origin, not
a second native ABI. Rust interop owns direct calls, opaque handles, async
calls, callback policy, views, and error conversion.

Stdlib-private rules are limited to:

- declarations live under `stdlib/_sifr/`,
- only sysroot stdlib sources can import `_sifr.*`,
- targets resolve only to canonical crates under the resolved sysroot,
- user packages cannot shadow private declarations or sysroot crate targets,
- diagnostics point to private declarations only in internal/developer
  contexts.

The compiler's semantic core remains compiler-owned. Parsing, HIR, type
checking, ownership, borrow analysis, Result/Option enforcement, safe indexing,
collection lowering, generator lowering, class/enum layout, async entrypoint
wiring, and language task/control-flow glue do not move into stdlib crates.

## Dependency Ownership

Dependency ownership is split deliberately:

- Sifr-owned sysroot crates are path dependencies under the sysroot.
- Sifr-owned third-party dependencies for sysroot crates are vendored under the
  sysroot.
- User package dependencies are resolved by the user's package graph and Cargo
  configuration.
- Explicit Rust interop package dependencies remain package-owned path,
  registry, or Git dependencies.

Sifr must not globally replace the user's Cargo registry for user dependencies
unless the user requested an offline/frozen package operation that already
requires such behavior. The sysroot vendor config exists for generated
std-library support, not as a hidden policy override for a user's dependency
graph.

Generated Cargo must not expose third-party crates that are implementation
details of Sifr-owned stdlib/runtime behavior. It may still emit user/package
dependencies, explicit Rust interop dependencies, and temporary direct
third-party dependencies for unmigrated compiler-special paths when the
corresponding migration stage has a deletion plan and validation coverage.

The broad native stdlib migration registry has been deleted. Current ownership
is checked by location and by targeted guardrails: retained compiler-native
glue is listed in `internal_docs/stdlib_retained_compiler_intrinsics.toml`, and
resource-shaped surfaces stay blocked by
`scripts/check_sysroot_stdlib_resource_certification_gate.py` until their Rust
interop lifecycle evidence lands.

Some surfaces are intentionally split while migration is underway. For example,
`_sifr.collections` set helpers and legacy JSON-string defaultdict helper
leaves now use private `@rust(sifr_stdlib.collections.*)` declarations backed
by `crates/sifr_stdlib`, while Counter behavior, defaultdict language glue, and
core collection layout remain compiler-owned language semantics.

## Generated Cargo Projects

`sifr run`, `sifr build`, tests, and package builds materialize generated Cargo
projects. Their manifests are sysroot-driven:

```toml
[dependencies]
sifr_stdlib = { path = "<sysroot>/crates/sifr_stdlib", default-features = false, features = [...] }
sifr_runtime = { path = "<sysroot>/crates/sifr_runtime", default-features = false, features = [...] }
```

Sifr-managed Cargo invocations apply sysroot vendor configuration for
Sifr-owned third-party dependencies when permitted by the Cargo ownership mode.
The generated project is still a normal Cargo project: user package
dependencies, explicit Rust interop package dependencies, and user-selected
registries remain controlled by Cargo and package metadata.

The generated dependency planner is owned by `sifr_stdlib_model`. It maps
stdlib modules, private declarations, language runtime requirements, and Rust
interop bridge requirements to one compiler-facing artifact:

```rust
SysrootDependencyPlan {
    crates: Vec<SysrootCrateDependency>,
    features: BTreeMap<SysrootCrate, BTreeSet<Feature>>,
    cargo_vendor_mode: CargoVendorMode,
    cache_fingerprint: String,
}
```

Generated Cargo, build reports, cache keys, LSP traces, feature expectations,
and tests consume this plan instead of recomputing sysroot crate dependencies,
features, vendor mode, or cache fragments independently.

Generated project cache keys include:

- compiler version,
- sysroot content digest,
- selected sysroot crate features,
- generated Rust source digests,
- package dependency graph digests when package mode is active,
- Rust interop probe/bridge contract digests.

This prevents cached binaries from surviving sysroot updates or stdlib feature
changes.

Generated build reports include the resolved sysroot path, derived toolchain id,
and sysroot content digest in human/trace/debug surfaces. This makes CI
failures and bug reports attributable to a concrete stdlib/runtime bundle.

## Rust Interop Stdlib Backend

The final native stdlib backend uses Rust interop as the default mechanism for
stdlib native leaves and resources.

The implementation stack is:

```text
stdlib/sifr/*.sifr public wrappers
  -> stdlib/_sifr/*.sifr private declarations
  -> @rust(...) / @rust.opaque(...) / @rust.async(...) declarations
  -> generated Rust interop glue
  -> sifr_stdlib and sifr_runtime sysroot crates
```

Compiler-special codegen intrinsics are restricted to language semantics. The
old handwritten intrinsic registry is removed or reduced to a retained
compiler-language-glue allowlist once supported native stdlib surfaces route
through private declarations and Rust interop.

Runtime/resource/callback surfaces follow the Rust interop certification gate.
Surfaces still marked future-owned or uncertified in the compatibility matrix
must not be claimed as stable stdlib interop support.
The sysroot stdlib resource certification gate is part of core validation so
resource migration cannot advance a resource-shaped surface ahead of the
runtime evidence recorded by the Rust interop matrix.
The stdlib native intrinsic allowlist guard is also part of core validation:
it freezes every retained compiler intrinsic name, prefix dispatcher, registry
file, and preamble file until that entry is migrated, deleted, or explicitly
kept as compiler-language glue.

Private stdlib Rust interop uses the normal Rust interop contract plus a
compiler-owned sysroot trust policy. That trust does not extend to arbitrary
user crates. Sysroot declarations may target only version-matched sysroot
crates unless an explicit architecture decision adds a new trusted crate
boundary.

Private `_sifr` declaration interop uses a synthetic compiler-owned package
context rather than a user package. The context maps private declaration
modules to the resolved sysroot, exposes only canonical `sifr_stdlib` and
`sifr_runtime` backend roots, records sysroot trust requirements as satisfied by
the compiler-owned sysroot policy, and keeps generated Cargo planning in
sysroot-only vendor mode for those roots. Rust bridge probes for private
declarations use the resolved sysroot runtime crate and invocation-scoped
sysroot vendor configuration.

When stdlib private declarations are merged with user package declarations, the
trust boundary is still package-keyed: private `_sifr` declarations must resolve
to the synthetic sysroot package id, and normal user declarations continue to
resolve against the user package. Sysroot interop crates injected after stdlib
feature planning are recorded in both generated Cargo dependencies and the
dependency-plan cache fingerprint.

## LSP and Tooling

CLI, LSP, formatter, linter, analysis, and documentation tooling all consume
the same `ResolvedSysroot`. LSP uses physical sysroot source files for:

- hover over stdlib functions and types,
- completion from public `sifr.*` modules,
- go-to-definition into installed stdlib sources,
- private `_sifr.*` declaration awareness for stdlib implementation files,
- version-correct diagnostics.

Tooling surfaces stdlib locations from the resolved sysroot. Installed sysroots
produce installed paths; development sysroots may produce repository or
`target/sifr-dev-sysroot` paths. Mixed modes where LSP and CLI resolve different
sysroots are configuration bugs.

The editor symbol index has a distinct stdlib bucket populated from public
`sifr.*` sysroot sources. Private `_sifr.*` declaration files are still present
in source maps for internal analysis, but they are not public completion
candidates for user code.

Source maps distinguish at least these origins. The editor source map includes
user, public stdlib, and private declaration sources. Generated Rust preview
metadata emits production source-map entries for generated support and compiler
synthetic contexts from the actual Rust source produced by the compiler.

- `UserSource`
- `SysrootPublicStdlib`
- `SysrootPrivateDeclaration`
- `GeneratedSupport`
- `CompilerSynthetic`

LSP go-to-definition prefers public wrapper locations for user code. Private
declaration locations are shown only in internal or developer contexts where the
request is operating on sysroot implementation files.

## Release and Install Contract

Release archives contain:

```text
bin/sifr
Cargo.toml
Cargo.lock
sysroot.toml
lib/sifr/stdlib/sifr/*.sifr
lib/sifr/stdlib/_sifr/*.sifr
crates/sifr_runtime/**
crates/sifr_stdlib/**
vendor/**
.cargo/config.toml
```

The installer verifies the archive contains the executable and required sysroot
roots before replacing the installation. Binary and sysroot replacement is
treated as one toolchain update. A self-update receipt records both the binary
path and sysroot path.

Installation replacement uses a staging directory:

1. Extract archive to a temporary directory.
2. Validate binary and sysroot shape.
3. Acquire the install lock.
4. Move the existing binary/sysroot aside or replace through temporary names.
5. Move the new binary and sysroot into place.
6. Write the install receipt atomically.
7. Remove old staged data only after success.

The implementation does not need a complex rollback system, but it must avoid
committing a new binary with an old sysroot or a new sysroot with an old binary.

Installer implementation must handle platform-specific failure modes:

- Windows executable replacement behavior,
- cross-device rename fallback,
- executable permission preservation,
- symlink rejection or canonicalization for archive entries and install paths,
- partial extraction cleanup,
- receipt atomic write,
- old sysroot cleanup only after successful replacement.

Release validation includes a mechanical no-path-leakage guardrail. The scanner
checks generated `Cargo.toml`, `Cargo.lock`, generated Rust sources, build
reports, LSP trace snapshots, installed sysroot manifests, release archives,
self-update receipts, and binary strings where feasible. It rejects build or CI
paths such as `/home/runner/`, `/workspace/`, checkout-root
`crates/sifr_runtime`, checkout-root `crates/sifr_stdlib`, and other
`CARGO_MANIFEST_DIR`-derived source paths in release artifacts.

Generated Cargo builds must never contain build-machine absolute paths in
release artifacts. Build-time paths are acceptable only in source-tree
development builds and tests where the source checkout is the intended sysroot.

## Diagnostics, Doctor, and Release Validation

Sysroot validation is a three-level ladder:

```text
Boundary check:
  cheap, always run before sysroot use

Doctor:
  expensive, local install health

Release certification:
  exhaustive, artifact integrity and no path leakage
```

Normal commands perform the boundary check before they need sysroot assets. The
check verifies:

- `sysroot.toml` exists and parses,
- `Cargo.toml` and `Cargo.lock` exist for the sysroot workspace,
- the sysroot version matches the compiler version,
- public stdlib source root exists,
- private declaration root exists,
- `crates/sifr_runtime/Cargo.toml` exists,
- `crates/sifr_stdlib/Cargo.toml` exists,
- `.cargo/config.toml` and `vendor/` exist for bundled dependency mode.

`sifr doctor` provides fuller human-readable diagnostics for installation
health, Cargo/rustc availability, sysroot paths, vendor configuration, and
self-update receipt consistency. Command execution diagnostics remain concise
and actionable.

The boundary check is intentionally small. It catches broken installs before
Cargo emits confusing path errors. It does not re-hash every vendored crate or
re-run every stdlib probe on every command. Deeper checks belong to
`sifr doctor` and release validation.

Hermetic installed-layout fixtures use an isolated environment:

- run outside the repository,
- empty `HOME` except the test install directory,
- isolated Cargo home,
- no runtime-only override environment variable,
- no `SIFR_SYSROOT` unless testing override behavior,
- generated project outside the source checkout,
- network disabled for stdlib-only offline cases,
- failure on any generated source checkout path.

## Development Sysroots

Developer builds use a sysroot-shaped tree. That tree may be materialized under
`target/sifr-dev-sysroot/` or resolved from a repository layout that already has
the installed shape:

```text
target/sifr-dev-sysroot/
  Cargo.toml
  Cargo.lock
  sysroot.toml
  lib/sifr/
  crates/
  vendor/ or dev Cargo config
```

Development only changes how a `ResolvedSysroot` is found or materialized. It
does not change compiler semantics, stdlib source inventory, runtime dependency
rules, LSP behavior, Cargo dependency ownership, or generated Cargo ownership.
Released binaries should not discover arbitrary parent directories and treat
them as a sysroot unless the resolved layout contains a valid `sysroot.toml`.

## Versioning and Compatibility

The compiler, sysroot manifest, `sifr_runtime`, `sifr_stdlib`, private
declarations, and public stdlib sources are versioned as a single toolchain.
Sysroot-only patching is not a supported standalone operation.

Generated artifacts should record the sysroot version in build reports and
trace/debug output so users and CI logs can identify which stdlib/runtime
bundle was used.

Release certification includes feature-tree inspection. Representative programs
for `sifr.re`, `sifr.json`, `sifr.http`, `sifr.python`, and pure Sifr stdlib
modules derive expected feature sets from `SysrootDependencyPlan`; `cargo tree
-e features` snapshots are evidence that Cargo resolution matches the planner.
The planner-owned expectation is the source of truth for minimal features.

## Compatibility Guarantees

- Public `sifr.*` APIs remain stable unless changed through normal stdlib API
  review.
- User imports of `_sifr.*` remain rejected.
- Generated runtime code must not contain data-dependent `unwrap()` or
  `expect()` in user-triggerable paths.
- Sysroot crate versions match the compiler version.
- CLI and LSP observe the same stdlib source and private declaration set.
- Standard-library native implementation moves through Rust interop without
  weakening Sifr's static safety and error-handling guarantees.
