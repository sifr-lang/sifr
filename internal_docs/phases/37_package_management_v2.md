# Phase 37: Package Coordination With uv And Cargo

> Status: proposed replacement planning contract for Phase 37. This v2 plan supersedes the native `sifr.lock` / Sifr registry direction if accepted. It intentionally makes uv/PyPI and Cargo first-class package managers, while keeping Sifr compiler semantics Sifr-owned.

## Objective

Establish package coordination for Sifr without building a full independent package manager in Phase 37.

Sifr uses each ecosystem's native package manager for the dependency graph it already owns:

- `sifr.toml` owns Sifr compiler and package semantics.
- `pyproject.toml` plus `uv.lock` own Sifr/Python distribution dependencies, PyPI/private-index resolution, downloads, cache, auth, publishing, and PEP 440 versioning.
- `Cargo.toml` plus `Cargo.lock` own backend Rust/native dependencies used by generated Rust, the Sifr runtime, Rust interop, and package-declared native capabilities.

There is no committed `sifr.lock` in this architecture. Sifr derives its compiler-facing package graph on each package-aware command from `pyproject.toml`, `uv.lock`, `sifr.toml`, dependency Sifr metadata, `Cargo.toml`, and `Cargo.lock`.

## Depends On

- Phase 36 developer tooling and workspace analysis surfaces.

## Core Decision

Sifr packages are distributed as Python distributions when fetched from an external registry. A Sifr package can be published to PyPI or a private Python package index as a wheel/sdist containing `.sifr` source plus Sifr metadata.

This deliberately accepts modern Python packaging as the transport layer:

- PEP 440 versions are accepted for Sifr packages distributed through PyPI.
- PEP 508 requirements are accepted in `pyproject.toml` for dependency resolution.
- uv's resolver, lockfile, downloader, cache, auth, workspaces, and publishing flows are reused as much as possible behind Sifr adapters.
- Python wheels/sdists are valid carriers for Sifr source.
- `uv.lock` is the committed distribution lockfile.

Sifr still owns the compiler semantics that uv and PyPI do not understand:

- Sifr package name and import roots.
- Sifr source roots and `__init__.sifr` package semantics.
- Public exports and private module boundaries.
- Direct-dependency-only import visibility.
- Sifr edition and compiler compatibility.
- Package-aware source mapping.
- Generated Rust preview mapping.
- Backend Cargo dependency validation.
- Native/build-script/proc-macro trust policy.
- Sifr diagnostics and compiler integration.

## Non-Goals

- Do not build a Sifr-native hosted registry in Phase 37.
- Do not build a Sifr-native dependency resolver in Phase 37.
- Do not create a committed `sifr.lock` in Phase 37.
- Do not make Cargo resolve Sifr source packages.
- Do not require pure Sifr packages to have Python runtime code.
- Do not let uv or Python packaging metadata decide Sifr import/export semantics.
- Do not rely on undocumented uv cache paths as compiler API without an adapter contract and fallback.

## Canonical Files

### `sifr.toml`

`sifr.toml` is the compiler and Sifr package semantics manifest.

It owns:

- Sifr package identity.
- Sifr edition.
- Source roots.
- Export roots.
- Public/private package boundaries.
- Sifr compiler version requirements.
- Optional package aliases for Sifr import ergonomics.
- Backend Cargo manifest linkage.
- Native trust policy.
- Tooling preferences that affect Sifr analysis only.

It does not own:

- External package version resolution.
- Downloading.
- Registry credentials.
- PyPI publish metadata.
- Python dependency groups.
- Rust crate dependency solving.

Example:

```toml
[package]
name = "http"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
roots = ["sifr"]

[exports]
modules = ["http"]

[distribution]
manifest = "pyproject.toml"

[backend]
manifest = "Cargo.toml"

[trust]
native = []
build-scripts = []
proc-macros = []
```

Empty trust arrays are intentional safety defaults. Packages with backend Rust/native behavior must explicitly declare which backend crates or capabilities are trusted.

### `pyproject.toml`

`pyproject.toml` owns distribution metadata and uv/PyPI resolution.

It owns:

- PyPI/private-index distribution name.
- PEP 440 version.
- PEP 508 dependencies.
- Dependency groups.
- Optional Python interop dependencies.
- Build backend for producing wheels/sdists.
- Publish metadata.
- uv workspace membership where applicable.

Pure Sifr package example:

```toml
[project]
name = "sifr-http"
version = "1.2.0"
description = "Typed HTTP client for Sifr"
requires-python = ">=3.12"
dependencies = [
  "sifr-json>=1.4,<2",
]

[dependency-groups]
dev = [
  "pytest>=8",
  "ruff>=0.15",
]

[build-system]
requires = ["sifr-build>=0.1"]
build-backend = "sifr_build"

[tool.sifr]
manifest = "sifr.toml"
```

Python interop package example:

```toml
[project]
name = "sifr-numpy-bridge"
version = "0.3.0"
requires-python = ">=3.12"
dependencies = [
  "numpy>=2",
  "sifr-array>=0.4",
]

[tool.sifr]
manifest = "sifr.toml"
```

### `uv.lock`

`uv.lock` is the committed distribution lockfile. Sifr must not duplicate uv's selected distribution graph in a committed Sifr lockfile.

Sifr uses `uv.lock` to discover:

- Selected distributions.
- Selected versions.
- Distribution sources.
- Dependency edges where uv exposes them.
- Workspace/path/Git/index sources.
- Lock freshness and frozen/offline status.

Sifr does not mutate `uv.lock` directly. It asks uv APIs or uv command functionality to update it.

Sifr packages are detected from the selected distribution set, not from `uv.lock` alone. A selected distribution is a Sifr package when one of these is true:

- a workspace, path, editable, or Git distribution exposes `[tool.sifr]` in its `pyproject.toml`;
- a distribution archive contains `pyproject.toml` with `[tool.sifr].manifest`;
- a distribution archive contains `sifr.toml` at the linked metadata location;
- a configured Sifr package index/name map marks the distribution as Sifr-capable, after which Sifr must still verify package metadata in the artifact.

Distributions without Sifr metadata remain ordinary Python dependencies and do not enter the Sifr import graph.

### `Cargo.toml`

`Cargo.toml` owns backend Rust/native dependencies for packages that need Rust crates during generated Rust compilation.

Example:

```toml
[package]
name = "sifr-http-backend"
version = "1.2.0"
edition = "2024"
publish = false

[dependencies]
tokio = { version = "1", features = ["rt", "time"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
```

Pure Sifr packages do not need a `Cargo.toml`.

### `Cargo.lock`

`Cargo.lock` is the committed backend Rust/native lockfile when a package or workspace declares backend Cargo dependencies. Sifr validates backend drift through Cargo rather than mirroring Cargo data into `sifr.lock`.

## Single Source Of Truth Rules

Fields must not have two active owners.

Distribution-owned fields:

- `project.name`
- `project.version`
- `project.dependencies`
- dependency groups
- publish metadata
- index/auth/publish configuration

Sifr-owned fields:

- `[package].name`
- `[package].edition`
- `[source].roots`
- `[exports].modules`
- import privacy
- compiler compatibility
- backend manifest linkage
- native trust

The distribution name is derived from linked `pyproject.toml` and must not be duplicated in `sifr.toml`.

Cargo-owned fields:

- Rust crate package identity.
- Rust dependency requirements.
- Rust features.
- Cargo target/build/profile behavior.
- Cargo lockfile content.

If both `pyproject.toml` and `sifr.toml` exist, one must explicitly reference the other:

- `pyproject.toml` uses `[tool.sifr].manifest = "sifr.toml"`.
- `sifr.toml` uses `[distribution].manifest = "pyproject.toml"`.

Missing reciprocal references are warnings in the discovery phase and become deterministic package diagnostics once Phase 37 package mode is active.

## Package Layout

Wheel/sdist packages carrying Sifr source must include source in a deterministic layout.

Recommended layout:

```text
sifr-http/
  pyproject.toml
  sifr.toml
  sifr/
    http/
      __init__.sifr
      client.sifr
      headers.sifr
      _internal.sifr
  README.md
  LICENSE
```

Installed distribution layout is implementation-defined by Python tooling, so Sifr must locate source through package metadata and uv/Python distribution APIs rather than hard-coding virtualenv internals.

Required packaging checks:

- `.sifr` files under `[source].roots` are included in wheel and sdist.
- `sifr.toml` or `[tool.sifr]` metadata is included.
- Source roots do not escape the distribution root.
- Archive paths reject traversal, absolute paths, device files, and unsupported symlink patterns.
- `__init__.sifr` files are present where package directories require them.
- Private modules are not exported unless explicitly listed.

## uv Integration Strategy

Sifr uses uv deeply, but behind Sifr-owned adapters.

The rule is direct reuse first:

> Prefer direct uv crate/API reuse behind a Sifr adapter whenever the uv API can be driven by real Python distribution data and can return enough information for Sifr without fabricating fake Python metadata.

Fallback to Sifr-native code only when:

- uv APIs are not exposed as stable-enough library boundaries.
- the uv type forces Sifr to pretend a Sifr concept is a wheel/sdist when it is not.
- the module's behavior is inseparable from virtualenv installation rather than distribution resolution.
- direct reuse would leak uv/Python types into compiler-facing Sifr APIs.

Sifr-owned public APIs must not expose uv types.

Allowed public Sifr types:

```rust
SifrDistributionName
SifrPackageName
ImportRoot
PackageId
ResolvedDistribution
ResolvedDistributionGraph
SifrPackageMetadata
SifrPackageGraph
PackageSourceMap
OperationPlan
```

Disallowed public Sifr types:

```rust
uv_pep440::Version
uv_pep508::Requirement<_>
uv_distribution_types::*
uv_pypi_types::*
uv_cache::Cache
uv_client::RegistryClient
```

Those may appear inside adapter modules only.

Adapter targets:

| Sifr adapter | uv reuse target | Responsibility |
| --- | --- | --- |
| `uv_project` | `uv-workspace`, `uv-configuration`, `uv-settings` where practical | discover project/workspace and dependency groups |
| `uv_lock` | uv lock/project APIs where available | read selected distributions and freshness state |
| `uv_sync` | uv resolver/sync command path or library path | update `uv.lock`, download distributions, respect frozen/offline |
| `uv_cache` | `uv-cache` primitives where generic | locate/stage/validate cached distributions without leaking cache internals |
| `uv_client` | `uv-client` base client, retry, cache policy | HTTP/index behavior, redirects, conditional requests, retries |
| `uv_auth` | `uv-auth`, `uv-netrc`, `uv-keyring`, `uv-redacted` | credentials and redaction |
| `uv_git` | `uv-git` / `gix` where practical | Git/path dependency handling and changed-file support |
| `uv_publish` | uv publish/auth flows where practical | packaging upload and dry-run validation |

If a uv crate cannot be used directly without Python-shaped leakage, the Sifr adapter must still port the relevant uv behavior into Sifr-owned tests.

### uv API Consumption Strategy

Phase 37 defaults to a CLI-first uv adapter.

The reason is maintenance, not capability: uv's internal crates are strong implementation material, but they are not a stable public downstream API contract for compilers. The CLI behavior is the stable product surface.

Requirements:

- Sifr may invoke the `uv` binary only behind `crates/sifr_package::uv::*`.
- Structured output is preferred. When uv lacks structured output for a required operation, the adapter defines the smallest stable parsing surface and covers it with snapshots.
- raw uv stderr/stdout is redacted and lowered into `SIFR-PACKAGE-*` diagnostics.
- subprocesses are cancellable.
- timeouts are explicit per operation class.
- `DEPENDENCY_AUDIT.md` records every place where Sifr shells out versus directly links uv crates.

Direct uv crate reuse remains preferred per adapter when a crate exposes a sufficiently narrow and stable API. Each direct-use decision must be documented in `DEPENDENCY_AUDIT.md` with pinned uv version, public API risk, Python-semantic leakage risk, and fallback plan to the CLI adapter.

### uv Crate Pinning And Optional Reference Checkout

Cargo is the canonical pinning mechanism for any uv crate used by Sifr production code.

If an adapter directly links a uv crate, that dependency must be declared in the relevant Sifr `Cargo.toml` with either:

```toml
uv-cache = { git = "https://github.com/astral-sh/uv.git", rev = "d19f1cd498202e04da70224573bbd5b79b94a726" }
```

or a reviewed crates.io version if the crate is published and usable as a downstream dependency.

The repository `Cargo.lock` is then the authoritative record of the exact uv crate revisions used by production code. `DEPENDENCY_AUDIT.md` must record the uv crate, chosen source, pinned revision/version, enabled features, direct-use justification, public API risk, Python-semantic leakage risk, and fallback plan to the CLI adapter.

An optional uv source checkout may exist only for audits, adapter planning, and review by agents. The checkout is not vendored and uv source is not committed to the Sifr repository.

Initial pinned reference:

```text
path: third_party/uv
remote: https://github.com/astral-sh/uv.git
revision: d19f1cd498202e04da70224573bbd5b79b94a726
describe: 0.11.14-27-gd19f1cd49
```

The checkout is materialized with:

```bash
scripts/prepare_uv_reference.sh --status
```

Rules:

- `third_party/uv/` is ignored by git.
- `scripts/prepare_uv_reference.sh` prepares only the optional local reference tree.
- Any uv revision used by production code must be pinned through `Cargo.toml` and `Cargo.lock`; the optional reference checkout should match that revision when audits need source inspection.
- Any uv revision change must update the relevant `Cargo.toml`, `Cargo.lock`, this section if the reference revision changes, `scripts/prepare_uv_reference.sh` if its default changes, and `crates/sifr_package/DEPENDENCY_AUDIT.md`.
- uv source may be read for implementation planning and adapter audits, but production code must depend on uv through the documented adapter strategy rather than by importing from `third_party/uv` paths.
- CI and agents may run the script when they need to inspect uv internals; normal Sifr builds must not require the checkout unless a Phase 37 validation lane explicitly opts into uv-reference audits.

## Cargo Integration Strategy

Sifr does not wrap Cargo dependency resolution with a Sifr resolver.

Cargo owns:

- backend Rust dependency resolution;
- backend features;
- `links` crates;
- build scripts;
- proc macros;
- target-specific Rust dependencies;
- `Cargo.lock`;
- final native build invocation.

Sifr owns:

- deciding which Sifr packages are active;
- deciding which backend Cargo manifests are relevant;
- checking native trust policy before invoking Cargo;
- checking `cargo metadata` and `cargo build --locked` results;
- mapping backend failures to Sifr diagnostics when they originate from Sifr package metadata.

Backend validation:

- If a selected Sifr package declares `[backend].manifest`, that `Cargo.toml` participates in backend validation.
- `sifr build --locked` invokes Cargo in locked mode for relevant backend manifests.
- `sifr build --frozen` requires both uv frozen/offline success and Cargo locked/offline success where backend manifests exist.
- Sifr must not edit `Cargo.toml` or `Cargo.lock` except through explicit user commands that are documented as Cargo-coordinating commands.
- `sifr build --frozen` with backend manifests requires a `Cargo.lock` for each relevant backend Cargo workspace or manifest root. If it is absent, Sifr fails with `SIFR-PACKAGE-0301`.
- `sifr build --offline` with backend manifests invokes `cargo build --offline` when `Cargo.lock` exists. If `Cargo.lock` is absent, Sifr fails unless the package explicitly opts into documented `backend.offline-lock-optional = true`; that opt-in is intended only for local development and is rejected for publish validation.
- `sifr build --locked` with backend manifests invokes `cargo build --locked` and fails if Cargo would update the lockfile.

## Derived Sifr Package Graph

Sifr derives the compiler-facing package graph on demand.

Inputs:

1. Root `sifr.toml`.
2. Root `pyproject.toml`.
3. `uv.lock`.
4. uv workspace/project metadata.
5. selected installed or cached distributions.
6. dependency package `sifr.toml` or `[tool.sifr]` metadata.
7. relevant `Cargo.toml` and `Cargo.lock` files.
8. active CLI selectors and target.

Output:

```rust
struct SifrPackageGraph {
    root: PackageId,
    packages: BTreeMap<PackageId, SifrPackageMetadata>,
    distribution_edges: BTreeMap<PackageId, BTreeSet<PackageId>>,
    direct_dependency_scopes: BTreeMap<PackageId, BTreeSet<PackageId>>,
    import_roots: BTreeMap<ImportRoot, PackageId>,
    backend_manifests: BTreeMap<PackageId, CargoManifestRef>,
}

struct SifrPackageMetadata {
    package_id: PackageId,
    distribution: ResolvedDistribution,
    sifr_name: SifrPackageName,
    import_alias: Option<SifrPackageName>,
    edition: SifrEdition,
    compiler_requirement: CompilerRequirement,
    source_roots: Vec<PackageSourceRoot>,
    exports: BTreeSet<ImportRoot>,
    python_interop: bool,
    backend: Option<CargoManifestRef>,
    trust: TrustPolicy,
}
```

This graph is not committed. It may be emitted for debugging and caching under generated output directories such as:

```text
target/sifr/package-graph.json
target/sifr/package-source-map.json
target/sifr/graph-digest.json
```

Those files are disposable derived artifacts.

The derived graph must still be reproducible without a committed `sifr.lock`. Sifr computes a graph digest from:

- SHA-256 of `uv.lock`;
- SHA-256 of root and member `pyproject.toml` files that participate in the selected graph;
- SHA-256 of root and dependency `sifr.toml` files or `[tool.sifr]` metadata content;
- SHA-256 of all `.sifr` source files under selected package `[source].roots`, computed from wheel/sdist artifact contents after archive validation for immutable distributions and from the current source tree for path, editable, Git, and workspace distributions;
- SHA-256 of each relevant `Cargo.toml` and `Cargo.lock`;
- active Sifr compiler version, target, profile, package selector, and relevant environment inputs already used by generated artifact caches.

The digest is emitted to `target/sifr/graph-digest.json` for diagnostics and cache transparency. In locked and frozen modes, Sifr recomputes the digest and fails if previously materialized generated artifacts were produced from a different digest and regeneration is disallowed by the command mode.

## Package Discovery Flow

For package-aware `sifr check`, `build`, `run`, `test`, `tree`, and editor analysis:

1. Discover nearest `sifr.toml`.
2. Discover linked `pyproject.toml`.
3. Ask uv adapter to validate or update `uv.lock` according to command mode.
4. Ask uv adapter for the selected distribution graph.
5. Filter selected distributions to those with Sifr metadata.
6. Load each dependency's `sifr.toml` or `[tool.sifr]`.
7. Validate Sifr metadata.
8. Build `SifrPackageGraph`.
9. Build `PackageSourceMap`.
10. Validate import/export boundaries.
11. Validate backend Cargo trust and locked state.
12. Invoke the normal frontend/HIR/codegen pipeline.
13. Invoke Cargo for generated Rust/native build when needed.

### Source Discovery

The uv adapter must expose a Sifr-owned artifact interface:

```rust
enum DistributionArtifact {
    Wheel { distribution: ResolvedDistribution, archive: ArtifactReader },
    Sdist { distribution: ResolvedDistribution, archive: ArtifactReader },
    Path { distribution: ResolvedDistribution, root: PathBuf },
    Git { distribution: ResolvedDistribution, checkout: PathBuf, resolved_commit: String },
    WorkspaceMember { distribution: ResolvedDistribution, root: PathBuf },
    Installed { distribution: ResolvedDistribution, root: PathBuf },
}
```

`ArtifactReader` is a Sifr-owned abstraction. It may be backed by uv cache APIs, a wheel/sdist file, an installed distribution, or a temporary extraction directory, but callers outside the uv adapter must not rely on uv cache paths.

Metadata lookup:

- Wheel: inspect archive contents for `pyproject.toml` with `[tool.sifr]` and/or the linked `sifr.toml`; extraction is into a staged temporary directory only after archive path validation.
- Sdist: inspect archive contents the same way; build backend output is not trusted until Sifr packaging validation confirms `.sifr` files are included.
- Path/editable: read directly from the resolved project root.
- Git: read from the uv-resolved checkout or adapter checkout at the exact locked commit.
- Workspace member: read directly from the member root.
- Installed: use distribution metadata/location APIs when available; otherwise fail closed instead of guessing virtualenv internals.

If Sifr metadata cannot be found for a distribution expected to provide a Sifr package, emit `SIFR-PACKAGE-0104`. If a configured Sifr package index or name map marked the distribution as Sifr-capable but artifact verification finds no Sifr metadata, emit `SIFR-PACKAGE-0105`.

Path, editable, Git, and workspace dependencies are mutable local inputs. At graph derivation time Sifr computes a content hash of each selected dependency's Sifr metadata plus all `.sifr` files under `[source].roots`; this hash participates in the graph digest.

## Import And Source Semantics

Dependency `.sifr` source is compiled through the same parser, HIR, type checker, ownership model, and codegen as application source.

The `ModuleResolver` gains package-aware origins:

```rust
enum ModuleOrigin {
    EntryParent,
    WorkspaceSource { source_root: PathBuf },
    PackageSource {
        package_id: PackageId,
        distribution: SifrDistributionName,
        source_root: PathBuf,
        export_root: ImportRoot,
    },
    EmbeddedStdlib,
}
```

Resolution order:

1. Embedded `sifr.*` and `_sifr.*` stdlib/intrinsics.
2. Current package source roots.
3. Declared direct workspace member dependencies from the uv/Sifr graph.
4. Declared direct distribution/path/Git dependencies from the uv/Sifr graph.
5. Transitive dependency source only when compiling that dependency's own modules or validating explicit re-exports.

Rules:

- Dependency package cache or installation paths must not become normal `[source].roots`.
- Transitive dependencies are compiled when needed but are not directly importable by a consumer unless declared directly or re-exported.
- Multiple selected distributions exporting the same Sifr import root are hard errors unless the consuming package declares an explicit Sifr alias.
- `__init__.sifr` defines explicit package and subpackage re-exports.
- Wildcard re-exports are rejected in Phase 37.
- Private modules such as `_internal.sifr` are visible only inside their declaring package unless explicitly exported.

## Manifest Models

### Pure Sifr Package

```toml
# pyproject.toml
[project]
name = "sifr-json"
version = "1.4.2"
dependencies = []

[build-system]
requires = ["sifr-build>=0.1"]
build-backend = "sifr_build"

[tool.sifr]
manifest = "sifr.toml"
```

```toml
# sifr.toml
[package]
name = "json"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
roots = ["sifr"]

[exports]
modules = ["json"]
```

### Sifr Package With Rust Backend

```toml
# pyproject.toml
[project]
name = "sifr-http"
version = "1.2.0"
dependencies = [
  "sifr-json>=1.4,<2",
]

[tool.sifr]
manifest = "sifr.toml"
```

```toml
# sifr.toml
[package]
name = "http"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
roots = ["sifr"]

[exports]
modules = ["http"]

[backend]
manifest = "Cargo.toml"

[trust]
native = ["http"]
```

```toml
# Cargo.toml
[package]
name = "sifr-http-backend"
version = "1.2.0"
edition = "2024"
publish = false

[dependencies]
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
```

### Python Interop Package

```toml
# pyproject.toml
[project]
name = "sifr-numpy"
version = "0.2.0"
requires-python = ">=3.12"
dependencies = [
  "numpy>=2",
]

[tool.sifr]
manifest = "sifr.toml"
```

```toml
# sifr.toml
[package]
name = "numpy"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
roots = ["sifr"]

[exports]
modules = ["numpy"]

[python]
interop = true
```

## CLI Contract

Sifr remains the user-facing compiler UX. Package-management commands are coordination wrappers over uv and Cargo, not independent resolvers.

Commands:

```bash
sifr init [--lib|--bin] [name]
sifr add <sifr-package-or-distribution> [--dev] [--group name] [--package member]
sifr remove <sifr-package-or-distribution> [--group name] [--package member]
sifr sync [--locked|--frozen|--offline]
sifr fetch [--locked|--offline]
sifr tree [--workspace|-p package] [--sifr-only|--all]
sifr outdated [--workspace|-p package]
sifr package [--dry-run]
sifr publish [--dry-run]
sifr check [file.sifr] [--locked|--frozen|--offline]
sifr build [file.sifr] [--locked|--frozen|--offline]
sifr run [file.sifr] [--locked|--frozen|--offline]
sifr test [--workspace|-p package|--filter selector] [--locked|--frozen|--offline]
```

Behavior:

- `sifr add http` maps a Sifr package name to a distribution name by package index metadata or configured naming policy, then uses uv to mutate `pyproject.toml` and `uv.lock`.
- `sifr add sifr-http` may be accepted directly as a distribution name.
- `sifr sync` delegates distribution sync/fetch/lock behavior to uv, then validates Sifr metadata.
- `sifr build` validates uv state, derives Sifr package graph, validates Cargo state, then compiles.
- `sifr tree --sifr-only` shows the Sifr import/package graph.
- `sifr tree --all` may include full uv distribution graph and backend Cargo graph where available.
- `sifr package --dry-run` validates Python distribution metadata, Sifr package metadata, included source files, and backend trust without uploading.
- `sifr publish` delegates upload to uv/PyPI-compatible publishing after Sifr validation succeeds.

`sifr` may call uv through library APIs where available. Shelling out to the `uv` binary is allowed only behind the uv adapter and must preserve structured error mapping, deterministic output capture, and cancellation behavior.

## Command Modes

| Sifr command mode | uv behavior | Cargo behavior | Sifr behavior |
| --- | --- | --- | --- |
| default | may update `uv.lock` when command semantics allow | may update `Cargo.lock` only for explicit backend commands | derive graph and validate metadata |
| `--locked` | uv lock must be current; no uv lock mutation | Cargo locked mode | no manifest mutation; derived graph only |
| `--offline` | uv offline/cache-only behavior | Cargo offline where configured | fail on missing Sifr metadata/source |
| `--frozen` | uv frozen/offline behavior | Cargo locked/offline behavior | no writes to manifests, locks, package cache, or non-disposable generated state |

`sifr build --frozen` succeeds only if:

1. `uv.lock` is present and satisfies `pyproject.toml`.
2. uv can provide selected distributions without network or lock mutation.
3. all selected Sifr package metadata validates.
4. all `.sifr` source files are present.
5. `Cargo.lock` is present and valid for selected backend manifests.
6. generated Rust/Cargo artifacts may be created only under disposable generated-output roots such as `target/`, `.sifr-gen/`, or another path explicitly configured as generated output and ignored by source control. Frozen mode must not write manifests, lockfiles, uv caches, Cargo registries, package source caches, installed distributions, or any user source tree file.

Mode combinations:

- `--frozen` implies `--locked --offline`.
- Passing `--frozen --locked` or `--frozen --offline` is accepted and normalized to frozen mode.
- Passing `--locked --offline` without `--frozen` is accepted and behaves like frozen for uv/Cargo network and lock mutation, while still allowing disposable generated artifacts.

## Operation Planning

All mutating Sifr package coordination commands must produce an operation plan before writes.

```rust
struct OperationPlan {
    selected_packages: Vec<PackageId>,
    pyproject_edits: Vec<PyProjectEdit>,
    uv_actions: Vec<UvAction>,
    cargo_actions: Vec<CargoAction>,
    sifr_metadata_checks: Vec<SifrMetadataCheck>,
    package_graph_changes: Vec<PackageGraphChange>,
    diagnostics: Vec<SifrDiagnostic>,
}
```

Examples:

```rust
enum UvAction {
    AddDistribution { name: SifrDistributionName, requirement: String, group: Option<String> },
    RemoveDistribution { name: SifrDistributionName, group: Option<String> },
    Sync { locked: bool, frozen: bool, offline: bool },
    Publish { dry_run: bool },
}

enum CargoAction {
    CheckLocked { manifest: PathBuf },
    Metadata { manifest: PathBuf },
    BuildGenerated { manifest: PathBuf, locked: bool },
}
```

`--dry-run` and `--dry-run=json` render the same plan without applying it. The plan must include uv and Cargo actions even when the actual execution is delegated.

## Workspaces And Monorepos

Sifr supports workspaces by coordinating uv workspaces, Sifr package roots, and Cargo workspaces where present.

Rules:

- uv workspace membership can define distribution packages.
- Sifr package membership is discovered from workspace members with `sifr.toml` or `[tool.sifr]`.
- Cargo workspace membership is used only for backend manifests.
- One repository may contain all three workspace concepts, but Sifr derives one compiler-facing graph.
- Workspace root commands may select packages by Sifr package name, distribution name, or path.
- Empty selections are deterministic package diagnostics unless a command explicitly documents no-op success.

Selectors adapted from Turborepo:

- `pkg` selects one package by Sifr package name or configured alias.
- `{path/glob}` selects packages by root path.
- `pkg...` selects `pkg` plus dependency closure.
- `...pkg` selects `pkg` plus dependent closure.
- `...^pkg` selects dependents only.
- `[base...head]` selects packages owning changed files using Git ranges.

Changed-package selection may reuse uv/git, `gix`, or an equivalent Git implementation behind the adapter. Falling back to the `git` CLI is allowed when `gix` cannot provide the needed behavior, but the fallback must be isolated behind the same adapter and covered by deterministic tests. Sifr owns the mapping from changed files to Sifr package roots.

## Publishing

Publishing is PyPI-compatible by default.

`sifr package --dry-run` must validate before `uv publish` or equivalent upload:

- `pyproject.toml` package metadata is valid.
- distribution name and version are accepted by Python packaging.
- `sifr.toml` or `[tool.sifr]` exists.
- `.sifr` source files are included in wheel/sdist.
- source roots and exports are valid.
- package imports compile or at least parse/type-check according to command mode.
- backend Cargo metadata is valid.
- native trust policy is explicit.
- archive contents reject traversal and unsupported file types.
- credentials are never printed.

`sifr publish` delegates upload/auth to uv where possible after Sifr validation passes.

Yanking and owner management follow PyPI/private-index capabilities. Sifr does not define custom registry owner semantics in Phase 37.

## Diagnostics

Sifr diagnostics own user-facing error reporting even when uv or Cargo provides the underlying failure.

Diagnostic examples:

| Code | Meaning |
| --- | --- |
| `SIFR-PACKAGE-0001` | missing or inconsistent `sifr.toml` / `[tool.sifr]` linkage |
| `SIFR-PACKAGE-0002` | invalid Sifr package metadata |
| `SIFR-PACKAGE-0003` | invalid source root or export root |
| `SIFR-PACKAGE-0101` | uv project or lockfile is stale |
| `SIFR-PACKAGE-0102` | uv resolution/sync failed |
| `SIFR-PACKAGE-0103` | selected distribution lacks required Sifr metadata |
| `SIFR-PACKAGE-0104` | cannot locate Sifr metadata inside selected distribution artifact |
| `SIFR-PACKAGE-0105` | Sifr package index marks distribution as Sifr-capable but artifact verification found no Sifr metadata |
| `SIFR-PACKAGE-0201` | duplicate or ambiguous Sifr import root |
| `SIFR-PACKAGE-0202` | undeclared direct dependency import |
| `SIFR-PACKAGE-0203` | private package module access |
| `SIFR-PACKAGE-0301` | backend Cargo manifest missing or stale |
| `SIFR-PACKAGE-0302` | backend native trust violation |
| `SIFR-PACKAGE-0401` | package archive missing required `.sifr` source |
| `SIFR-PACKAGE-0402` | publish validation failed |

Every diagnostic must include structured origin data where applicable:

- `sifr.toml` path and key.
- `pyproject.toml` path and key.
- distribution name.
- Sifr package name.
- import root.
- source root.
- uv action.
- Cargo manifest path.
- dependency path.
- remediation suggestion.

uv and Cargo stderr/stdout are not surfaced raw in compiler diagnostics except as redacted, structured cause text.

## Maintainability Architecture

Phase 37 introduces one coordination crate first: `crates/sifr_package`.

The crate owns Sifr package graph derivation, manifest validation, uv/Cargo adapter boundaries, operation planning, and compiler-facing source maps. It does not implement a full package resolver.

Module map:

```text
crates/sifr_package/src/
  lib.rs
  manifest/{sifr,pyproject_link,validate}.rs
  distribution/{model,metadata,graph}.rs
  uv/{project,lock,sync,cache,client,auth,git,publish,errors}.rs
  cargo/{manifest,metadata,locked,trust}.rs
  graph/{derive,imports,workspace,filters,changed}.rs
  source/{layout,include,archive,discover}.rs
  imports/{source_map,boundaries,reexports}.rs
  ops/{plan,mutate,resolve,read,publish}.rs
  diag/{codes,lower_uv,lower_cargo,origins,redaction}.rs
  test_support/
  test_assets/
```

Boundary rules:

- `crates/sifr` CLI parses command flags and renders output; it does not inspect uv internals, mutate TOML directly, or walk installed distribution caches.
- `sifr_driver` consumes `PackageSourceMap` and backend Cargo validation results; it does not call uv directly.
- `sifr_frontend` and `sifr_hir` consume immutable package-origin data only.
- `crates/sifr_package::uv::*` is the only subtree that imports uv crates.
- `crates/sifr_package::cargo::*` is the only subtree that invokes Cargo-specific package metadata APIs.
- no uv type crosses the public `sifr_package` facade.
- no Cargo metadata type crosses the public `sifr_package` facade.
- diagnostics are constructed as `sifr_diagnostics::SifrDiagnostic`; no parallel diagnostic renderer exists.

Adapter audit:

`crates/sifr_package/DEPENDENCY_AUDIT.md` must record every uv crate or generic crate reused directly, the reason for direct reuse, the public API stability risk, whether Python concepts cross the adapter, and the fallback plan if uv internals change.

`crates/sifr_package/TRACEABILITY.md` maps uv/Cargo/Turborepo behavior categories to Sifr tests, milestones, diagnostics, and intentional divergences.

`crates/sifr_package/FEATURES.md` records enabled feature flags for uv crates, Cargo-related crates, and generic dependencies.

## Guardrails

Add `scripts/check_package_manager_guardrails.py` when `crates/sifr_package` lands.

It must enforce:

- package-manager source files stay below documented line limits;
- uv crates are imported only under `crates/sifr_package/src/uv`;
- Cargo metadata crates and Cargo invocation helpers are imported only under `crates/sifr_package/src/cargo`;
- public `sifr_package` APIs expose only Sifr-owned types;
- mutating CLI commands route through `OperationPlan`;
- `DEPENDENCY_AUDIT.md`, `TRACEABILITY.md`, and `FEATURES.md` exist once the first uv-backed adapter lands;
- package metadata tests exist for every supported package layout;
- no code path reads uv cache internals outside the uv adapter.

## Correctness And Test Reuse

Reuse uv tests more directly than the v1 plan.

Port or adapt categories from:

- uv lock tests: frozen, locked, stale, conflict, upgrade, yanks, source metadata.
- uv workspace tests: root discovery, virtual projects, members, excluded paths, malformed workspaces, subdirectory invocation, path dependencies.
- uv cache tests: stale entries, permissions, clean/prune behavior, corrupted entries, concurrent writes.
- uv auth/publish tests: token handling, keyring behavior, redirects, missing credentials, malformed helpers, redaction.
- uv tree/outdated tests: repeated dependencies, cycles, frozen reads, dev/group views.
- uv project/dependency group tests: group selection, optional groups, marker behavior.
- Cargo locked/offline/build metadata tests where relevant for backend manifests.
- Turborepo filter behavior for package/dependency/dependent/changed selectors.

Sifr-specific tests:

- Sifr metadata discovery from wheel, sdist, editable/path dependency, Git dependency, and workspace member.
- missing `sifr.toml` or `[tool.sifr]`.
- invalid export roots.
- duplicate import roots from two uv-selected distributions.
- direct-dependency import boundary rejection.
- transitive dependency source compilation.
- `__init__.sifr` re-export validation.
- private module access rejection.
- backend Cargo trust failures.
- `sifr build --frozen` with valid `uv.lock` and `Cargo.lock`.
- `sifr build --frozen` with stale uv lock.
- `sifr build --locked` with stale Cargo lock.
- publish dry-run catches missing `.sifr` files in wheel/sdist.

Property tests:

- Derived `SifrPackageGraph` is deterministic for the same `uv.lock`, manifests, target, and selector.
- Reordering uv lock records does not change derived import graph.
- Adding a non-Sifr Python-only dependency does not change the Sifr import graph.
- Removing a Sifr direct dependency makes its import root unavailable.
- Transitive dependency imports are rejected unless declared directly or re-exported.
- Source-map paths are stable after temp path normalization.
- Diagnostics are stable across repeated runs.

## Milestones

### milestone_37_1: Manifest Linking And Sifr Metadata

Scope:

- Add `crates/sifr_package` with facade types and adapter boundaries.
- Parse and validate `sifr.toml` compiler/package semantics.
- Parse `[tool.sifr]` linkage from `pyproject.toml`.
- Enforce single-source-of-truth rules.
- Add Sifr package metadata diagnostics.
- Add initial dependency audit for uv/Cargo adapter crates.

Definition of done:

- root packages with `sifr.toml` plus `pyproject.toml` validate.
- pure Sifr package metadata validates without Python runtime code.
- invalid source roots, exports, package names, linkage, and compiler requirements produce stable diagnostics.

### milestone_37_2: uv Project And Lock Integration

Scope:

- Implement uv-backed adapter for project/workspace discovery.
- Implement uv-backed lock reading or command integration.
- Derive selected distribution graph from `uv.lock`.
- Map selected distributions to Sifr package metadata.
- Implement `sifr sync`, `sifr add`, and `sifr remove` as uv-coordinating commands.

Definition of done:

- `sifr add` updates `pyproject.toml` and `uv.lock` through uv behavior.
- `sifr sync --frozen` respects uv frozen/offline semantics.
- stale, missing, or incompatible `uv.lock` produces Sifr diagnostics.
- selected Python-only dependencies do not become Sifr packages.

### milestone_37_3: Package-Aware Import Resolution

Scope:

- Build `SifrPackageGraph`.
- Build `PackageSourceMap`.
- Integrate package origins with frontend project assembly.
- Add `__init__.sifr` package directory and explicit re-export semantics.
- Enforce direct dependency import boundaries.

Definition of done:

- dependency `.sifr` source compiles through the normal compiler pipeline.
- duplicate export roots, undeclared imports, private modules, and missing source roots fail before downstream type noise.
- editor analysis uses the same package source map as CLI builds.

### milestone_37_4: Cargo Backend Coordination

Scope:

- Link selected Sifr packages to backend Cargo manifests.
- Validate backend trust policy.
- Invoke Cargo metadata/build in locked modes as appropriate.
- Map backend failures into Sifr diagnostics.
- Extend generated artifact cache keys with uv lock digest, Sifr metadata digests, Cargo lock digest, compiler version, target, and profile.

Definition of done:

- pure Sifr packages build without Cargo backend manifests.
- Rust-backed packages validate `Cargo.lock`.
- native/build-script/proc-macro trust failures are deterministic.
- generated Rust builds are reproducible under locked/frozen modes.

### milestone_37_5: Workspaces, Filters, And Tooling

Scope:

- Coordinate uv workspaces, Sifr packages, and Cargo workspaces.
- Implement package selectors and Turborepo-style filters.
- Implement changed-package selection through uv/git or `gix` adapter.
- Update Phase 36 analysis surfaces for package graph awareness.
- Implement `sifr tree`, `outdated`, workspace `check/build/test`.

Definition of done:

- root and virtual uv/Sifr workspaces work from subdirectories.
- filters select package, dependency closure, dependent closure, and changed packages deterministically.
- tooling queries use the same derived graph as CLI builds.

### milestone_37_6: Packaging And Publishing

Scope:

- Implement `sifr package --dry-run`.
- Validate wheel/sdist contents for Sifr packages.
- Integrate with uv publish/auth where practical.
- Add package archive security checks and credential redaction.
- Support pure Sifr and Python interop package layouts.

Definition of done:

- dry-run catches missing Sifr files, bad metadata, invalid exports, archive traversal, and backend trust issues.
- publish delegates to uv-compatible upload after Sifr validation.
- credentials never appear in diagnostics, logs, generated files, or package metadata.

### milestone_37_7: Validation, Docs, And Guardrails

Scope:

- Add end-to-end package fixtures for pure Sifr, Rust-backed Sifr, Python interop, workspace, path, Git, and PyPI/private-index flows.
- Add `scripts/check_package_manager_guardrails.py`.
- Complete `DEPENDENCY_AUDIT.md`, `TRACEABILITY.md`, and `FEATURES.md`.
- Update public docs and internal architecture docs.

Definition of done:

- `scripts/run_all_tests.sh --profile quick` and full local validation pass.
- uv/Cargo/Turborepo behavior categories are mapped to Sifr tests or explicit non-port decisions.
- guardrails pass locally and are part of the authoritative validation gate.

## Quality Contract

- Phase 27 diagnostic stability and no-user-triggerable-panic invariants still apply.
- No Sifr package command may panic on malformed manifests, malformed uv locks, malformed package metadata, missing files, invalid archives, or bad Cargo manifests.
- No command may silently fall back from uv/Cargo locked/frozen behavior to best-effort behavior.
- Package graph derivation must be deterministic.
- Derived graph artifacts are disposable and must never be treated as committed source of truth.
- uv/Cargo failures must be redacted and mapped into stable Sifr diagnostics.
- Implementation must prefer root-cause fixes over compatibility fallbacks.

## Exit Gate

Phase 37 is complete when:

- Sifr packages can be distributed through PyPI/private Python indexes as wheels/sdists containing `.sifr` source and Sifr metadata.
- `sifr build --frozen` is reproducible from `sifr.toml`, `pyproject.toml`, `uv.lock`, `Cargo.toml`, and `Cargo.lock`.
- pure Sifr packages, Rust-backed Sifr packages, Python interop packages, and workspaces compile through one package-aware compiler path.
- import/export/package boundary diagnostics are stable and actionable.
- uv-backed resolution/download/cache/auth/publish behavior is reused behind adapters where viable.
- Cargo backend behavior is delegated to Cargo and verified through locked modes.
- no committed `sifr.lock` is required for the Phase 37 model.
