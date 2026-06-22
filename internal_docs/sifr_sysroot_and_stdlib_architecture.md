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

## Installed Layout

The canonical standalone installation layout is:

```text
~/.sifr/
  bin/
    sifr
  lib/
    sifr/
      Cargo.toml
      Cargo.lock
      sysroot.toml
      stdlib/
        sifr/
          *.sifr
        _sifr/
          *.sifr
      crates/
        sifr_runtime/
          Cargo.toml
          src/
        sifr_stdlib/
          Cargo.toml
          src/
      vendor/
      .cargo/
        config.toml
```

`~/.sifr/lib/sifr` is the Sifr sysroot. The sysroot is versioned as one unit
with `~/.sifr/bin/sifr`. Release archives contain the executable and the full
sysroot tree. The installer replaces them together.

The sysroot is also a self-contained Cargo workspace. `Cargo.toml` owns
workspace package metadata and dependency versions for shipped sysroot crates.
`Cargo.lock` records the exact third-party dependency graph used to produce
`vendor/`.

`vendor/` contains third-party Cargo sources needed by Sifr-owned sysroot
crates. Generated projects use the sysroot Cargo config so normal stdlib usage
does not require network access after installation. User package dependencies
remain user-owned Cargo inputs and are not part of the Sifr sysroot contract.

## Sysroot Manifest

`sysroot.toml` records the installed toolchain contract:

```toml
schema-version = 1
sifr-version = "0.1.0-beta.N"
toolchain-id = "sifr-0.1.0-beta.N-x86_64-unknown-linux-gnu"
target-triple = "x86_64-unknown-linux-gnu"
built-by-compiler-commit = "<git-sha>"
rust-edition = "2024"
rust-version = "<minimum-supported-rust-version>"
cargo-lock-sha256 = "<sha256>"
stdlib-public-digest = "<sha256-tree>"
stdlib-private-digest = "<sha256-tree>"
runtime-crate-digest = "<sha256-tree>"
stdlib-crate-digest = "<sha256-tree>"
vendor-digest = "<sha256-tree>"

[paths]
workspace-manifest = "Cargo.toml"
workspace-lock = "Cargo.lock"
stdlib = "stdlib"
runtime-crate = "crates/sifr_runtime"
stdlib-crate = "crates/sifr_stdlib"
vendor = "vendor"
cargo-config = ".cargo/config.toml"

[crates]
sifr-runtime = { name = "sifr_runtime", version = "0.1.0-beta.N" }
sifr-stdlib = { name = "sifr_stdlib", version = "0.1.0-beta.N" }
```

The compiler treats a mismatched or incomplete sysroot as an installation
error. It reports a Sifr diagnostic that tells the user to reinstall Sifr or set
`SIFR_SYSROOT`. It must not silently fall back to `env!("CARGO_MANIFEST_DIR")`
in released builds.

The digest fields are identity metadata for build reports, cache keys,
`sifr doctor`, release validation, and bug reports. Normal commands do not
re-hash the entire sysroot or vendor tree on every invocation. They trust the
manifest after the small boundary check described below; deeper digest
verification belongs to release validation and `sifr doctor`.

## Sysroot Resolution

Sysroot resolution is deterministic:

1. An explicit `--sysroot <path>` CLI option, where accepted by developer
   tooling.
2. `SIFR_SYSROOT`.
3. Installed layout relative to the running executable:
   `current_exe()/../lib/sifr`.
4. Source-tree development layout for unreleased local builds.

`SIFR_RUNTIME_PATH` remains a development compatibility override for testing a
runtime crate checkout, but it is not the release mechanism. In release mode,
missing sysroot assets are surfaced as install corruption, not as Cargo path
errors.

Source-tree development mode is gated by a single predicate:

```text
is_source_tree_development_mode()
```

That predicate is true only for local development builds under an explicit
debug/dev build marker or equivalent reviewed build-time configuration. A
released binary must not enter source-tree mode merely because the current
directory, an ancestor, or the executable path contains directories named
`crates/sifr_runtime` or `crates/sifr_stdlib`. Released binaries either resolve
a valid installed sysroot, honor an explicit sysroot override, or emit a sysroot
diagnostic.

## Sysroot Cargo Workspace

The sysroot Cargo workspace exists so shipped sysroot crates are independent of
the source checkout. Sysroot crate manifests may use workspace-inherited
dependency versions, lints, edition, and metadata, but that inheritance must
resolve inside `~/.sifr/lib/sifr/Cargo.toml`, not the development repository.

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

Distribution packaging may either copy checked-in sysroot-ready manifests or
materialize release manifests from the development workspace. The released
result must be self-contained: running `cargo metadata --offline` from the
sysroot workspace succeeds without the source checkout.

The sysroot Cargo config is stored at `.cargo/config.toml` and uses Cargo's
standard vendoring model:

```toml
[source.crates-io]
replace-with = "sifr-vendor"

[source.sifr-vendor]
directory = "vendor"
```

Generated projects can either receive a copied `.cargo/config.toml` or invoke
Cargo with an explicit config path. The implementation must follow this policy:

- Single-file generated builds using only Sifr stdlib/runtime dependencies
  apply sysroot vendor config.
- Generated package builds with no user registry dependencies apply sysroot
  vendor config.
- Package builds with user registry dependencies in default online mode do not
  silently force sysroot-only vendor config onto user dependencies.
- Package builds with explicit `--offline`, `--frozen`, or equivalent locked
  offline mode use a complete combined graph/vendor source or fail with a clear
  Cargo/Sifr diagnostic.
- Explicit Rust interop package dependencies keep the dependency source
  package-owned; Sifr does not hide them behind sysroot vendor policy.

The observable guarantee is that Sifr-owned sysroot dependencies resolve from
the installed sysroot in offline mode without changing ownership of user
dependencies.

The sysroot vendor directory is generated from the sysroot workspace lockfile,
not from an ad hoc dependency list. This prevents stdlib features and runtime
features from drifting away from the shipped vendor set.

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

It owns Rust implementations for stdlib native leaves and resource operations
that are not compiler semantics:

- math, hashing, base encodings, UUID, regex, TOML, HTML, calendar, and other
  stateless native leaves,
- filesystem, environment, platform, path, process, signal, compression, URL,
  HTTP, net, TLS, Python, Unicode, and i18n stdlib-facing functions where the
  surface is exposed through `lib/sifr`,
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
This is the successor of the current compiler crate named `sifr_stdlib`.

It owns:

- stdlib source inventory and module metadata,
- public `sifr.*` import policy and private `_sifr.*` rejection policy,
- legacy CPython-shaped module suggestions,
- private declaration module metadata,
- mapping from stdlib declarations and codegen requirements to generated Cargo
  dependencies and sysroot crate features,
- IPC schema/protocol metadata needed by compiler analysis and verification,
- compatibility shims while old compiler-special intrinsics are removed.

The name `sifr_stdlib` is reserved for the generated-program stdlib crate. The
compiler model uses a distinct name so crate names communicate runtime
responsibility rather than internal implementation history.

`sifr_stdlib_model` may embed fallback stdlib sources for development and
bootstrap, but released tools prefer sysroot files. Its public API should speak
in terms of resolved sysroot/module metadata rather than hard-coded include
paths so CLI, LSP, and tests can exercise installed-layout behavior.

## Standard Library Source Layers

The public standard library remains Sifr source:

```text
stdlib/sifr/*.sifr
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
stdlib/_sifr/*.sifr
```

Only sysroot stdlib modules may import `_sifr.*`. User code importing `_sifr.*`
is rejected. Private declaration modules declare native stdlib operations using
Rust interop annotations such as direct functions, opaque handles, async
functions, view/zero-copy contracts, and callback policies.

Private declarations are source files rather than Rust tables so editor tooling
and compiler behavior share the same signature source. They should be treated
as internal ABI: stable across one compiler/sysroot version, not public API.

Private declaration ABI is specified explicitly before declaration migration.
The ABI spec defines:

- allowed Rust interop annotations in private declarations,
- allowed target crates and module paths,
- canonical path requirements for sysroot crate targets,
- Result and error-class conversion rules,
- opaque handle ownership and close/drop behavior,
- async, cancellation, and timeout semantics,
- zero-copy/view lifetime rules,
- callback thread-safety and reentrancy rules,
- prohibited declaration shapes,
- diagnostic attribution for private declarations without exposing `_sifr` as a
  public API.

Private declarations may target only canonical paths under the resolved sysroot
crates unless a separate reviewed architecture decision adds another trusted
crate boundary. User packages cannot shadow `_sifr` declarations or provide a
same-named crate path that overrides a sysroot target.

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
corresponding milestone has a deletion plan and validation coverage.

## Generated Cargo Projects

`sifr run`, `sifr build`, tests, and package builds materialize generated Cargo
projects. Their manifests are sysroot-driven:

```toml
[dependencies]
sifr_stdlib = { path = "<sysroot>/crates/sifr_stdlib", features = [...] }
sifr_runtime = { path = "<sysroot>/crates/sifr_runtime", features = [...] }
```

Generated projects also receive Cargo configuration that points Cargo at the
sysroot vendor directory for Sifr-owned third-party dependencies. The generated
project is still a normal Cargo project: user package dependencies, explicit
Rust interop package dependencies, and user-selected registries remain
controlled by Cargo and package metadata.

The generated dependency planner is owned by `sifr_stdlib_model`. It maps
stdlib modules, private declarations, language runtime requirements, and Rust
interop bridge requirements to sysroot crate dependencies and feature sets.

Generated project cache keys include:

- compiler version,
- sysroot manifest digest,
- sysroot crate manifest digests,
- relevant sysroot crate source digests,
- selected sysroot crate features,
- generated Rust source digests,
- package dependency graph digests when package mode is active,
- Rust interop probe/bridge contract digests.

This prevents cached binaries from surviving sysroot updates or stdlib feature
changes.

Generated build reports include the resolved sysroot path, toolchain id, and
sysroot manifest digest in human/trace/debug surfaces. This makes CI failures
and bug reports attributable to a concrete stdlib/runtime bundle.

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

Compiler-special codegen intrinsics shrink to language semantics and temporary
compatibility shims. The old handwritten intrinsic registry is removed once all
supported private declarations have Rust interop equivalents or explicit
unsupported-by-design decisions.

Runtime/resource/callback surfaces follow the Rust interop certification gate.
Surfaces still marked future-owned or uncertified in the compatibility matrix
must not be claimed as stable stdlib interop support.

Private stdlib Rust interop uses a compiler-owned trust policy. That trust does
not extend to arbitrary user crates. Sysroot declarations may target only
version-matched sysroot crates unless an explicit architecture decision adds a
new trusted crate boundary.

## LSP and Tooling

CLI, LSP, formatter, linter, analysis, and documentation tooling all resolve
the same sysroot. LSP uses physical sysroot source files for:

- hover over stdlib functions and types,
- completion from public `sifr.*` modules,
- go-to-definition into installed stdlib sources,
- private `_sifr.*` declaration awareness for stdlib implementation files,
- version-correct diagnostics.

Embedded stdlib sources may remain available to source-tree development builds,
tests, and bootstrap tooling, but released tools use the installed sysroot as
the authoritative stdlib source.

Tooling must surface stdlib locations as sysroot file paths in installed mode.
For source-tree development mode, it may surface repository paths. Mixed modes
where LSP reads embedded sources while CLI reads sysroot sources are considered
configuration bugs.

Source maps distinguish at least these origins:

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
sifr
lib/sifr/Cargo.toml
lib/sifr/Cargo.lock
lib/sifr/sysroot.toml
lib/sifr/stdlib/sifr/*.sifr
lib/sifr/stdlib/_sifr/*.sifr
lib/sifr/crates/sifr_runtime/**
lib/sifr/crates/sifr_stdlib/**
lib/sifr/vendor/**
lib/sifr/.cargo/config.toml
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

## Diagnostics and Doctor

Normal commands perform a small sysroot boundary check before they need sysroot
assets. The check verifies:

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
- no `SIFR_RUNTIME_PATH`,
- no `SIFR_SYSROOT` unless testing override behavior,
- generated project outside the source checkout,
- network disabled for stdlib-only offline cases,
- failure on any generated source checkout path.

## Source-Tree Development Mode

Developer builds can use the repository as a sysroot. In that mode:

- `lib/sifr` or the canonical sysroot source tree supplies stdlib sources,
- workspace crates under `crates/` supply `sifr_runtime` and `sifr_stdlib`,
- normal workspace dependency metadata is allowed,
- local tests may use embedded source fallbacks to validate bootstrap behavior.

Development mode must be explicit in code paths. Released binaries should not
discover arbitrary parent directories and treat them as a sysroot unless the
resolved layout contains a valid `sysroot.toml` or is the known source-tree
layout compiled for development.

## Versioning and Compatibility

The compiler, sysroot manifest, `sifr_runtime`, `sifr_stdlib`, private
declarations, and public stdlib sources are versioned as a single toolchain.
Patch-level sysroot-only updates are not a supported standalone operation until
the toolchain defines a compatibility policy for them.

Generated artifacts should record the sysroot version in build reports and
trace/debug output so users and CI logs can identify which stdlib/runtime
bundle was used.

Release certification includes feature-tree inspection. Representative programs
for `sifr.re`, `sifr.json`, `sifr.http`, `sifr.python`, and pure Sifr stdlib
modules record `cargo tree -e features` snapshots so feature creep is visible
in review.

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
