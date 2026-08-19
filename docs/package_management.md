# Package Management

Cargo is the package substrate for Sifr packages. A distributed Sifr library is a Cargo package that carries Sifr metadata and `.sifr` source files, while Sifr owns the user-facing source layout, package API, and diagnostics.

## Package Shape

Minimum production library layout:

```text
sifr-http/
  Cargo.toml
  Cargo.lock
  sifr.toml
  src/
    __init__.sifr
    client.sifr
    lib.rs
```

`Cargo.toml` owns distribution metadata, dependency resolution, lockfiles, registries, Git/path sources, workspaces, publishing, vendoring, and backend Rust dependencies.

`sifr.toml` owns compiler semantics: Sifr package name, edition, compiler requirement, source root, dependencies, scripts, privacy, and native trust policy.

`[source].root` defaults to `src`. New production packages do not use `[exports].modules` or Sifr manifest `[[bin]]` target tables. Public package APIs are declared in source with `__init__.sifr`.

Minimum app layout:

```text
sifr-app/
  Cargo.toml
  Cargo.lock
  sifr.toml
  src/
    main.sifr
    bin/
      admin.sifr
    lib.rs
```

`src/main.sifr` is the default app target. Additional app targets are discovered from `src/bin/*.sifr`.

## Public APIs

A package root is public when `src/__init__.sifr` exists. Public names are the top-level classes, functions, and type aliases defined there, plus explicit relative re-exports:

```sifr
from .client import HttpClient
from .errors import HttpError as Error
```

A public child namespace is a directory with its own `__init__.sifr`. Implementation files such as `src/client.sifr` are private across package boundaries unless their names are re-exported through an accessible namespace.

Local package code may import its own implementation files directly. Cross-package imports must go through the dependency's public namespace API.

## Manifest-Less Files

Explicit file commands remain first-class for learning and scripting:

```bash
sifr run path/to/file.sifr
sifr check path/to/file.sifr
sifr emit path/to/file.sifr
```

When no `sifr.toml` exists in the file's directory or an ancestor directory,
these commands run in manifest-less single-file mode with no package dependency
graph. The resolver does not inspect the filename, source imports, or sibling
modules. A local import therefore requires an explicit `sifr.toml` workspace.
Inside a package, an explicit `.sifr` file must live under the selected package
source root. A discovered malformed manifest is a hard diagnostic rather than
a reason to fall back to manifest-less mode.

## Package Sessions And Commands

Package-aware commands first create a `PackageSession`. The session records the workspace root, selected `sifr.toml`, source root, lock mode, manifest-less state, selected runnable target, and any script expansion before a Cargo-backed command is invoked.

`sifr fetch --locked` and `sifr tree --locked` delegate to Cargo through Sifr command plans and preserve Cargo lock/network flags. `sifr check` without an explicit `.sifr` file delegates to the package check plan; `sifr check path/to/file.sifr` stays in explicit-file mode and is rejected inside a package when the file is outside the source root.

`sifr run` resolves package targets in this order: explicit `.sifr` files, `--bin <name>`, `--script <name>`, positional app target or script name, `[package].default-run`, then `src/main.sifr` or a single discovered `src/bin/*.sifr` target. If an app target and script share a name, Sifr reports [`SIFR-PACKAGE-0605`](/errors/SIFR-PACKAGE-0605) and requires `--bin` or `--script`.

`[scripts]` entries are structured Sifr command plans:

```toml
[scripts]
dev = { command = "run", args = [] }
check-all = { command = "check", args = ["--workspace"] }
```

Scripts do not execute shell strings or external programs. A script may not call another script; nested script expansion reports [`SIFR-PACKAGE-0714`](/errors/SIFR-PACKAGE-0714).

## Cargo Metadata Hook

Every Sifr package exposes a manifest pointer through Cargo metadata:

```toml
[package.metadata.sifr]
manifest = "sifr.toml"
```

Sifr reads this from `cargo metadata --format-version 1`, resolves `sifr.toml` relative to the selected Cargo package root, and derives a Sifr package graph from Cargo's resolved package graph. Cargo source ids are opaque; Sifr does not parse Cargo registry or Git cache internals.

## Pure Sifr Packages

Pure Sifr packages include a minimal Rust library target because Cargo requires a target:

```rust
// Pure Sifr package marker. Sifr source lives in the sifr.toml source root.
```

Sifr rejects non-trivial Rust marker contents for packages that do not declare Rust-backed behavior.

## Initialization And Repair

`sifr init --lib demo_json --name demo_json` creates a canonical library package:

```text
demo_json/
  Cargo.toml
  sifr.toml
  src/
    __init__.sifr
    lib.rs
```

`sifr init --bin demo_app --name demo_app` creates `src/main.sifr` instead of `src/__init__.sifr`. Generated Cargo package names use `sifr-<kebab-case-name>` and include the discovery pointer:

```toml
[package.metadata.sifr]
manifest = "sifr.toml"
```

Generated Cargo projection files include Sifr-owned marker comments and deterministic include metadata for `sifr.toml`, `src/**/*.sifr`, and `src/lib.rs`. `sifr repair --check` reports projection drift without writing. `sifr repair` may regenerate Sifr-owned Cargo projection metadata and the pure marker when it is missing.

## Publishing And Vendoring

`sifr package` validates Sifr metadata and package archive contents before delegating to Cargo archive assembly. `sifr publish --dry-run` runs the same Sifr preflight before delegating to Cargo's publish dry-run path. Validation rejects missing `sifr.toml`, missing `.sifr` source files, Cargo include/exclude omissions, archive traversal paths, invalid exports, and backend trust violations.

Publishing and vendoring reuse Cargo-compatible behavior. Credentials remain Cargo-owned and must never appear in Sifr diagnostics, generated files, package metadata, or logs.

## Demo Workflow

The checked-in `verification/areas/package_management/corpora/demo_repositories/sifr-demo-*` subrepositories use the production `src/` layout and the same commands expected for public demos.

First clone and fetch:

```bash
scripts/clone_subrepos.sh
cd verification/areas/package_management/corpora/demo_repositories/sifr-demo-app
sifr fetch --locked
```

Run the default app, an explicit file, and a structured script:

```bash
sifr run --locked
sifr run src/main.sifr --locked
sifr run --script dev --locked
```

Offline and workspace checks:

```bash
sifr check --locked --offline
cd ../sifr-demo-workspace
sifr check --workspace --locked
sifr check --workspace --exclude sifr-demo-app --locked
sifr run -p sifr-demo-app --locked
sifr run -p sifr-demo-app --bin status --locked
sifr run -p sifr-demo-app --script status-smoke --locked
```

Development-only dependencies stay in `[dev-dependencies]`; normal `sifr run`, `sifr check`, `sifr package`, and `sifr publish` only expose runtime `[dependencies]` as import roots.
Use Cargo-compatible tree arguments when you want to inspect them explicitly:

```bash
sifr tree --locked --edges dev
```

Release dry-run and vendoring:

```bash
cd ../sifr-demo-json
sifr package --allow-dirty
sifr publish --dry-run --allow-dirty --no-verify
cd ../sifr-demo-app
sifr vendor vendor --versioned-dirs
```

## Lock Modes

Sifr delegates lock and network semantics to Cargo:

- `--locked` asks Cargo to reject lockfile updates.
- `--offline` asks Cargo to avoid network access and requires selected package sources to be locally available.
- `--frozen` combines locked and offline semantics.

`sifr check`, `sifr build`, and `sifr run` carry the selected mode through
Rust interop signature probes and the final generated Cargo build. Before a
constrained generated build, Sifr derives a generated lockfile from the
authoritative package/sysroot locks, validates every registry package identity
and checksum against those locks or trusted vendored sources, and caches that
prepared lock by manifest and authority digest. The actual Cargo probe/build
still runs with the requested flag; internal probes additionally use frozen
strength so a constrained cache entry can never depend on a lockfile write.
Sifr verifies that the prepared generated lock remains byte-identical.
Those three commands reject lock-constrained manifestless operation because
there is no package `Cargo.lock` that can serve as the resolution authority.

Package-owned builds and their probes preserve the package's Cargo source
selection; they do not replace crates.io with the Sifr sysroot vendor.
Sysroot-only builds continue to use the sysroot vendor. In either mode,
`--offline` and `--frozen` require every selected source to be present in the
applicable Cargo cache or vendor tree.

Normal and lock-constrained probe results use separate cache identities. All
constrained probe modes may share an entry because each internal probe executes
with frozen strength; final build behavior still preserves the requested
`--locked`, `--offline`, or `--frozen` mode. Missing locks, stale selections,
source/checksum drift, feature drift, or unavailable offline sources produce
`SIFR-RUST-CARGO-0001` for Rust interop rather than falling back to normal
resolution.

## Cargo Failure Diagnostics

Sifr wraps Cargo process failures in [`SIFR-PACKAGE-0101`](/errors/SIFR-PACKAGE-0101). The wrapper carries the Cargo action, working directory, redacted argument vector, lock/network mode, exit status when available, and a bounded redacted Cargo output excerpt. Credential-looking Cargo failures, including private registry and Git credential failures, use the same wrapper rather than a separate Sifr credential taxonomy.

[`SIFR-PACKAGE-0105`](/errors/diagnostic-codes) is retired. `sifr --explain SIFR-PACKAGE-0105` points users to [`SIFR-PACKAGE-0101`](/errors/SIFR-PACKAGE-0101), and `sifr --explain SIFR-PACKAGE-0101` prints diagnostic help without performing package operations.

## Python Interop

`pyproject.toml`, `uv.lock`, and uv/Python package coordination are future interop work. They are not part of the core package-management rules and must lower into the same Cargo-backed `SifrPackageGraph` and `PackageSourceMap` model when implemented.
