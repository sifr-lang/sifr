# Phase 37: Cargo-Backed Sifr Package Coordination

> Status: completed on 2026-05-19. The accepted v3 contract makes Cargo the primary package substrate for Sifr packages and defers uv/Python packaging to a later interop layer. It supersedes the v1 Sifr-native registry/lockfile model and the v2 uv/PyPI-first model.

## Objective

Establish production-grade Sifr package coordination by using Cargo as the canonical resolver, source fetcher, lockfile manager, workspace manager, registry client, publisher, and backend Rust/native dependency manager.

Sifr does not build a parallel package manager in Phase 37. Instead:

- `Cargo.toml` and `Cargo.lock` own package dependency resolution, registry/Git/path sources, source caching, workspaces, publishing, vendoring, offline/locked/frozen behavior, and backend Rust/native dependencies.
- `sifr.toml` is the single source of truth for Sifr compiler semantics: source roots, import/export roots, `__init__.sifr`, package privacy, Sifr edition, compiler compatibility, scoped imports, and native trust policy.
- `crates/sifr_package` derives a compiler-facing Sifr package graph from `cargo metadata` plus Sifr metadata and builds `PackageSourceMap` for the normal frontend/HIR/codegen pipeline.
- `pyproject.toml`, `uv.lock`, and uv are not part of the Phase 37 core. They remain future Python interop and CLI-distribution concerns.

There is no committed `sifr.lock` in v3. `Cargo.lock` is the committed package lockfile.

## Depends On

- Phase 36 developer tooling and workspace analysis surfaces.

## Core Decision

A registry-distributed Sifr package is a valid Cargo package that carries `.sifr` source and Sifr metadata.

Cargo package artifact:

```text
sifr-http/
  Cargo.toml
  sifr.toml
  sifr/
    http/
      __init__.sifr
      client.sifr
      headers.sifr
      _internal.sifr
  src/lib.rs          # minimal Rust target required by Cargo for pure Sifr packages
  README.md
  LICENSE
```

Cargo manifest:

```toml
[package]
name = "sifr-http"
version = "1.2.0"
edition = "2024"
include = ["Cargo.toml", "sifr.toml", "sifr/**/*.sifr", "src/lib.rs", "README.md", "LICENSE"]

[package.metadata.sifr]
manifest = "sifr.toml"

[dependencies]
sifr-json = "1.4"
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
```

Sifr metadata:

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

[features]
default = ["tls"]
tls = { cargo-package = "reqwest", cargo-feature = "rustls-tls" }
json = { cargo-package = "reqwest", cargo-feature = "json" }

[trust]
native = ["reqwest"]
build-scripts = []
proc-macros = []
```

Pure Sifr packages still include a minimal Rust target because Cargo requires at least one target. The generated `src/lib.rs` marker should contain no semantic implementation and must not become a second source of truth for Sifr behavior.

Marker enforcement is part of Sifr package validation:

- `sifr init --lib` for a pure Sifr package writes a canonical marker file:

  ```rust
  // Pure Sifr package marker. Sifr source lives in sifr.toml source roots.
  ```

- `sifr package --dry-run` and package graph validation reject a pure Sifr package whose marker target contains Rust items, macros, module declarations, cfg-driven implementation, includes, generated code hooks, or re-exports.
- A package that intentionally includes Rust implementation must declare itself as Rust-backed Sifr in `sifr.toml` and pass backend trust validation.
- `SIFR-PACKAGE-0501` reports non-trivial Rust marker source in a package declared as pure Sifr.

## Why Cargo Is Primary

Cargo already owns the infrastructure Sifr needs for a compiled Rust-backed language:

- Semver dependency resolution.
- Multiple-version package graphs.
- `Cargo.lock` reproducibility.
- Registry, Git, and path sources.
- Source download and cache management.
- Workspaces and package selection.
- Feature resolution.
- Offline, locked, frozen, fetch, vendor, update, publish, yank, login behavior.
- Native/backend dependency graph.
- Build scripts, proc macros, `links` crates, target triples, and platform-specific dependencies.
- `cargo metadata` JSON exposing selected packages, source ids, manifest paths, custom metadata, dependency edges, features, workspace members, target/build dirs, and package roots.

Sifr is Python-syntax, but operationally it is a compiled language that emits Rust/native binaries. Cargo is the closest existing package manager to Sifr's runtime and backend constraints. uv remains valuable later for Python interop, but it should not be the core package manager for Sifr source packages in Phase 37.

## Non-Goals

- Do not build a Sifr-native registry or Sifr-native lockfile in Phase 37.
- Do not use uv/PyPI as the primary Sifr package substrate in Phase 37.
- Do not link to Cargo internals as stable production APIs unless a later implementation review explicitly approves it.
- Do not make Cargo understand Sifr import/export semantics.
- Do not reject Cargo's multiple-version resolution globally; scoped imports make it useful.
- Do not let pure Sifr packages rely on Rust marker crates for behavior.

## Canonical Files

### `Cargo.toml`

`Cargo.toml` owns:

- Cargo package/distribution name.
- Cargo semver version.
- Cargo dependencies, including Sifr packages and Rust backend crates.
- Cargo features.
- Cargo workspace membership.
- Registry/Git/path source declarations.
- Cargo publish/include/exclude metadata.
- `[package.metadata.sifr]` pointer to the Sifr manifest.

Cargo dependencies are classified after `cargo metadata`:

- A dependency is a Sifr source package if its selected package exposes `packages[].metadata.sifr.manifest` and the referenced `sifr.toml` is valid.
- A dependency is a backend Rust crate if it does not expose Sifr metadata but is reachable from selected Sifr packages or generated Rust.
- A dependency may be both Sifr source package and Rust backend crate, but this must be explicit and trust-checked.

Classification cases:

- Package has valid Sifr metadata and `sifr.toml`: Sifr source package.
- Package has valid Sifr metadata, `sifr.toml`, and non-marker Rust behavior: Rust-backed Sifr package subject to trust policy.
- Package has no Sifr metadata: Rust backend crate only unless the user explicitly selected it as a Sifr package, in which case `SIFR-PACKAGE-0102` is reported.
- Package has Sifr metadata but its manifest is unavailable or invalid: `SIFR-PACKAGE-0002`.

### `Cargo.lock`

`Cargo.lock` is the committed package lockfile.

Sifr uses it through Cargo behavior:

- `sifr build --locked` delegates lock validation to Cargo locked mode.
- `sifr build --offline` delegates source availability to Cargo offline mode.
- `sifr build --frozen` delegates both no-network and no-lock-update behavior to Cargo frozen mode.

Sifr does not mirror Cargo's package graph into another lockfile.

### `sifr.toml`

`sifr.toml` owns compiler semantics:

- Sifr package name.
- Sifr edition.
- Sifr compiler version requirement.
- Source roots.
- Export roots.
- `__init__.sifr` package semantics.
- Private module policy.
- Optional Sifr import aliases.
- Native/backend trust policy.
- Sifr analysis/tooling options.

It does not own external version resolution, source fetching, registry credentials, Cargo dependency features, or Cargo publish metadata.

Phase 37 extends the `sifr.toml` schema described in `internal_docs/sifr_workspace_design.md`. All Phase 37 fields are additive. Unknown top-level tables and unknown nested keys continue to be accepted and ignored according to that design's forward-compatibility rule.

Sifr edition is orthogonal to Cargo edition. `Cargo.toml` `edition` controls Rust crate parsing and Cargo/rustc behavior. `sifr.toml` `[package].edition` controls Sifr language compatibility only. Phase 37 validates that a selected package declares a syntactically supported Sifr edition, but it does not enforce cross-package edition equality. Detailed Sifr edition compatibility, migration, and deprecation policy belongs in a future edition policy document.

### `pyproject.toml` / `uv.lock`

Out of scope for Phase 37 core package management.

Future Phase 43/Python interop may add:

- `pyproject.toml` `[tool.sifr]` pointers.
- uv-managed Python dependencies.
- Python package distribution for Sifr CLI installation.
- mixed Python/Sifr workspace coordination.

That interop layer must lower into the same `SifrPackageGraph` / `PackageSourceMap` model and must not fork import semantics.

## Cargo Integration Strategy

Sifr should prefer Cargo's stable command/API surfaces:

- `cargo metadata --format-version 1` for selected package graph and metadata.
- `cargo fetch` for source availability.
- `cargo update` for updates.
- `cargo add` / `cargo remove` where available and stable enough.
- `cargo build --locked` / `--offline` / `--frozen` for lock/network enforcement.
- `cargo package` / `cargo publish` / `cargo yank` for distribution.
- `cargo vendor` for vendoring.

Production code should avoid direct dependency on Cargo's internal `cargo` crate APIs unless an implementation review records the reason and risk. The `cargo_metadata` crate and Cargo CLI JSON are the preferred integration surface.

Cargo source paths from `cargo metadata` are trusted only as package roots selected by Cargo. Sifr still validates Sifr source roots, archive inclusion, and package metadata before compiling dependency `.sifr` files.

Pin `cargo_metadata` to an exact version through `Cargo.lock`. Do not update it as part of unrelated dependency refreshes. Every upgrade requires a targeted audit entry in `crates/sifr_package/DEPENDENCY_AUDIT.md` recording:

- `cargo_metadata` version;
- Cargo CLI version range and `--format-version` validated against it;
- fields consumed by Sifr;
- known ordering or compatibility risks;
- fallback command behavior if the crate API changes.

`cargo metadata` output must be normalized before graph derivation. Sifr sorts packages by Cargo package id string, dependency edges by `(from, dependency-name, to)`, features by name, workspace members by package id, and all derived Sifr maps/sets by stable owned identifiers. Graph digests are computed from this canonical representation, not raw JSON order.

Cargo metadata source IDs are opaque Cargo identifiers. Sifr may store and compare them for identity, diagnostics, and cache keys, but must not parse their internal string format or infer registry semantics from the prefix beyond Cargo-documented source kind behavior.

Generated Rust namespace rule:

- Package-aware builds materialize generated Rust under Sifr-owned generated output roots.
- Each selected Sifr package instance receives a generated crate/module namespace derived from canonical package identity, for example `sifr_gen_<sanitized-cargo-name>_<stable-package-hash>`.
- The generated module tree mirrors the Sifr module tree under the selected source roots.
- The pure marker `src/lib.rs` in the distributed Cargo package is not the generated implementation crate and is never allowed to re-export generated modules.
- Visibility between generated package namespaces follows Sifr import/export boundaries, not Rust module reachability.

## Sifr Metadata In Cargo

Every Sifr package must expose a Sifr manifest pointer in Cargo metadata. This is only a discovery hook. Sifr compiler metadata lives in `sifr.toml`.

Discovery:

1. Cargo selects packages through normal Cargo resolution.
2. Sifr reads `packages[].metadata.sifr` from `cargo metadata` for each selected Cargo package.
3. If metadata exists, Sifr resolves `manifest` relative to that Cargo package root and loads `sifr.toml`.
4. If the caller expected a selected package to be Sifr-capable and metadata is absent, Sifr reports `SIFR-PACKAGE-0001`.
5. If metadata exists but `manifest` is missing, unreadable, or invalid, Sifr reports `SIFR-PACKAGE-0002`.
6. If unsupported Sifr compiler metadata appears in Cargo metadata instead of `sifr.toml`, Sifr reports `SIFR-PACKAGE-0003`.

Cargo metadata is read from Cargo's own JSON output. Sifr does not re-parse arbitrary `Cargo.toml` files for semantic data. After `cargo metadata` or `cargo fetch` materializes registry/Git/path sources locally, `manifest_path` gives the package root needed to load the pointed `sifr.toml`.

Minimum:

```toml
[package.metadata.sifr]
manifest = "sifr.toml"
```

Rules:

- `manifest` is relative to the Cargo package root.
- `manifest` is the only required key in `[package.metadata.sifr]`.
- Sifr package name, exports, edition, source roots, compiler requirements, features, and trust policy are read from `sifr.toml`.
- every key in `[package.metadata.sifr.aliases]` must point to a selected Cargo dependency name in the declaring package's Cargo manifest;
- alias `import` names are Sifr import roots visible only in the declaring package's direct dependency scope;
- alias metadata lives in `Cargo.toml` because it depends on Cargo dependency names and renamed dependencies; `sifr.toml` may document aliases but does not duplicate the authoritative mapping.
- Missing or mismatched metadata is a Sifr package diagnostic.
- A Cargo package without `[package.metadata.sifr]` is not a Sifr source package.
- Cargo package name and Sifr package name may differ. The Cargo package name is the distribution name; the Sifr package name/import root is compiler-facing.

## Sifr Features And Cargo Features

Sifr feature names are compiler-facing. Cargo feature names are package-manager-facing. Phase 37 connects them explicitly instead of assuming they are the same.

Example:

```toml
# sifr.toml
[features]
default = ["tls"]
tls = { cargo-package = "reqwest", cargo-feature = "rustls-tls" }
json = { cargo-package = "reqwest", cargo-feature = "json" }
```

Rules:

- `sifr build --features tls,json` activates the Sifr features, then maps them to Cargo features before invoking Cargo metadata/build commands.
- `sifr add <pkg> --features f1,f2` always delegates package name and version mutation to Cargo. Feature flags are delegated to Cargo only when the named feature exists in the selected Cargo package feature set. Features without a Cargo equivalent are written to `sifr.toml` `[features]` only.
- Optional Cargo dependencies that affect Sifr imports or generated Rust must be reachable through explicit Sifr feature mappings.
- Backend-only optional Cargo features may remain Cargo-only, but they must be recorded in `FEATURES.md` if they can affect generated Rust or trust behavior.
- If a Sifr feature mapping names a `cargo-package` that does not appear in the resolved Cargo dependency graph, `SIFR-PACKAGE-0303` is reported.
- If a Sifr feature mapping names a `cargo-feature` that does not exist in that package's feature set, `SIFR-PACKAGE-0304` is reported.
- A Sifr feature may be source-only and map to zero Cargo features; source-only features still participate in graph digests and diagnostics.
- Feature resolution is included in graph digests and diagnostics.

## Multiple Versions And Scoped Imports

Cargo may select multiple versions of the same Cargo package or multiple packages exporting the same Sifr import root. Sifr supports this when imports remain unambiguous inside each package's dependency scope.

Core rule:

> Each Sifr package has its own import dependency scope. The same Sifr import root may resolve to different package versions in different scopes. Ambiguity is an error only inside one package's own direct dependency scope.

Definition:

> A direct dependency of package `P` is a selected Cargo package `Q` that appears as one Cargo dependency edge from `P` after Cargo resolution for the active target, dependency kind, and features. Transitive dependencies at distance greater than one are not in `P`'s direct import scope unless `P` declares them directly or imports them through an explicit Sifr re-export.

Example:

```text
app
  -> image-lib
      -> sifr-math 1.4 exports math
  -> physics-lib
      -> sifr-math 2.1 exports math
```

Valid:

```sifr
# inside image-lib
import math  # resolves to sifr-math 1.4

# inside physics-lib
import math  # resolves to sifr-math 2.1
```

Invalid unless `app` directly declares/aliases a math dependency:

```sifr
# inside app
import math
```

If a package needs two versions in the same scope, it must alias them through Cargo dependency rename plus Sifr alias metadata.

```toml
[dependencies]
math_v1 = { package = "sifr-math", version = "1" }
math_v2 = { package = "sifr-math", version = "2" }

[package.metadata.sifr.aliases]
math_v1 = { dependency = "math_v1", import = "math_v1" }
math_v2 = { dependency = "math_v2", import = "math_v2" }
```

Then:

```sifr
import math_v1
import math_v2
```

Type identity is package-instance-specific:

```text
sifr-math@1.4::math.Vector != sifr-math@2.1::math.Vector
```

Cross-scope values keep the package instance that produced their type. Calling code must import or name the same package instance to accept that value.

Example:

```sifr
# app directly aliases two selected versions.
import math_v1
import math_v2

v = math_v1.Vector(1, 2)

def accepts_v1(value: math_v1.Vector) -> None:
    pass

def accepts_v2(value: math_v2.Vector) -> None:
    pass

accepts_v1(v)  # valid
accepts_v2(v)  # SIFR-PACKAGE-0204
```

`SIFR-PACKAGE-0204` must include `expected_package_id`, `actual_package_id`, Cargo package ids, Sifr import paths, and the dependency path that introduced each package instance.

Diagnostics must print both Sifr import path and Cargo package identity when version/type identity matters.

## Derived Sifr Package Graph

Sifr derives its graph from Cargo, not from a Sifr resolver.

Inputs:

- root `Cargo.toml`;
- root `Cargo.lock`;
- `cargo metadata` JSON;
- selected Cargo packages;
- selected package `sifr.toml`;
- `[package.metadata.sifr]`;
- active Cargo features/targets;
- active Sifr CLI package selectors/filters;
- Sifr compiler version, target, and profile.

Output:

```rust
struct SifrPackageGraph {
    root: SifrPackageId,
    packages: BTreeMap<SifrPackageId, SifrPackageMetadata>,
    cargo_edges: BTreeMap<SifrPackageId, BTreeSet<SifrPackageId>>,
    direct_dependency_scopes: BTreeMap<SifrPackageId, BTreeMap<ImportRoot, SifrPackageId>>,
    import_aliases: BTreeMap<(SifrPackageId, ImportRoot), SifrPackageId>,
    backend_crates: BTreeMap<SifrPackageId, Vec<CargoPackageId>>,
}

struct SifrPackageMetadata {
    package_id: SifrPackageId,
    cargo_package_id: CargoPackageId,
    cargo_package_name: String,
    cargo_version: String,
    cargo_source: Option<String>,
    package_root: PathBuf,
    sifr_manifest: PathBuf,
    sifr_name: SifrPackageName,
    edition: SifrEdition,
    compiler_requirement: CompilerRequirement,
    source_roots: Vec<PackageSourceRoot>,
    exports: BTreeSet<ImportRoot>,
    aliases: BTreeMap<ImportRoot, CargoDependencyName>,
    trust: TrustPolicy,
}
```

This graph is not committed. It may be emitted for diagnostics/cache transparency under generated output directories:

```text
target/sifr/package-graph.json
target/sifr/package-source-map.json
target/sifr/graph-digest.json
```

Graph digest includes:

- `Cargo.lock` digest;
- relevant `Cargo.toml` digests;
- normalized `cargo metadata` selected package ids, dependency edges, features, source ids, and workspace members;
- Sifr metadata digests;
- `.sifr` source file digests under selected source roots;
- compiler version, target, profile, features, selectors, and environment inputs affecting generated Rust.

The graph digest file under `target/sifr/` is the incremental cache invalidation key for package-aware builds. It is disposable generated state, not a source of truth.

## Package Discovery Flow

For package-aware `sifr check`, `build`, `run`, `test`, `tree`, and editor analysis:

1. Discover nearest Cargo workspace/package root.
2. Run or read `cargo metadata` with the active target/features.
3. Validate Cargo lock/network mode through the requested command mode.
4. Classify selected Cargo packages into Sifr source packages and backend Rust crates.
5. Load each selected Sifr package's `sifr.toml`.
6. Validate `[package.metadata.sifr]` points to loadable `sifr.toml`.
7. Build `SifrPackageGraph`.
8. Build `PackageSourceMap`.
9. Validate scoped imports, exports, aliases, and private modules.
10. Validate backend trust policy for reachable backend crates.
11. Invoke the normal Sifr frontend/HIR/codegen pipeline.
12. Materialize generated Rust and invoke Cargo as needed.

Subdirectory invocation:

- Sifr always discovers the nearest Cargo workspace root for package graph derivation, lock/network mode enforcement, and shared `Cargo.lock` lookup.
- Running `sifr build`, `check`, `run`, or `test` from a member subdirectory without `--workspace`, `-p`, or `--filter` selects only that member when Cargo identifies the current directory as a workspace member.
- Running the same command from a workspace root without explicit selectors follows Cargo's default package selection. For a package root this is the root package; for a virtual workspace this is Cargo `default-members` when configured, otherwise Cargo's default workspace selection.
- Running with `--workspace` selects the workspace member set, then applies Sifr metadata filtering and any Sifr selectors.
- All subdirectory invocations use the shared workspace `Cargo.lock`; `--locked`, `--offline`, and `--frozen` are enforced at the workspace root.
- `sifr fetch` from a member subdirectory invokes Cargo fetch using the shared workspace root/lock, then validates source availability for the selected Sifr package graph. Cargo may materialize more sources than the Sifr selection because fetch is a Cargo manifest/workspace operation.
- `sifr fetch --workspace` validates source availability for all selected Sifr workspace members after Cargo fetch completes.

Fetch lifecycle:

- `sifr fetch` explicitly runs Cargo fetch for the discovered Cargo manifest/workspace and then validates selected Sifr package metadata from available source. It must not attempt to reimplement Cargo's fetch selection rules.
- `sifr check`, `build`, `run`, and `test` may lazily invoke Cargo fetch when a selected source package is unavailable and the mode is not `--offline` or `--frozen`.
- `--offline` fails immediately with `SIFR-PACKAGE-0104` if any selected Sifr source package is unavailable in Cargo's cache.
- `--frozen` never performs network access or manifest/lock mutation.
- Private registry credential acquisition is delegated to Cargo; Sifr wraps credential-related Cargo failures in `SIFR-PACKAGE-0101` with redacted Cargo context.

## Import And Source Semantics

Dependency `.sifr` source is compiled through the same parser, HIR, type checker, ownership model, and codegen as application source.

The module resolver gains package-aware origins:

```rust
enum ModuleOrigin {
    EntryParent,
    WorkspaceSource { source_root: PathBuf },
    PackageSource {
        package_id: SifrPackageId,
        cargo_package_id: CargoPackageId,
        source_root: PathBuf,
        export_root: ImportRoot,
    },
    EmbeddedStdlib,
}
```

Resolution order inside package `P`:

1. Embedded `sifr.*` and `_sifr.*` stdlib/intrinsics.
2. `P`'s own source roots.
3. `P`'s direct Cargo dependency scope, mapped through Sifr exports and aliases.
4. Transitive dependency source only when compiling that transitive package's own modules or validating explicit re-exports.

Rules:

- Cargo source cache paths must not be flattened into `[source].roots`.
- A package may import its own modules, stdlib modules, and direct dependencies in its own Cargo dependency scope.
- Transitive dependencies are compiled when needed but are not directly importable unless declared directly or re-exported.
- Two direct dependencies exporting the same import root in the same scope are an ambiguity diagnostic unless one is aliased.
- `__init__.sifr` defines explicit package/subpackage re-exports.
- Wildcard re-exports are rejected in Phase 37.
- Private modules such as `_internal.sifr` are visible only inside their declaring package unless explicitly exported.
- Phase 37 does not support platform-specific Sifr source roots. Platform-specific behavior should be expressed through Cargo features/target dependencies plus Sifr feature mappings; target-conditional Sifr source roots are a future extension.

## Cargo Workspace And Sifr Workspace Semantics

Cargo workspace membership is the package selection source of truth in Phase 37.

Sifr adds compiler-aware package filtering over the Cargo workspace graph:

- `sifr test --workspace` uses Cargo workspace members that expose Sifr metadata.
- `sifr build --workspace` compiles all Sifr-capable Cargo workspace members. Rust-only packages are built only when reachable as backend dependencies of those Sifr packages.
- `sifr build -p app` selects the Cargo package `app` and derives the Sifr package graph reachable from it.
- `sifr tree --sifr-only` displays only selected Sifr source packages.
- `sifr tree --all` may include backend Rust crates from Cargo metadata.

Virtual workspaces:

- A Cargo workspace root with `[workspace]` and no `[package]` is a virtual workspace. It has no Sifr package identity of its own.
- Sifr must not require `[package.metadata.sifr]` or `sifr.toml` at a virtual workspace root.
- `--workspace` from a virtual workspace selects every workspace member that exposes `[package.metadata.sifr]`, after Cargo applies workspace membership and `exclude` rules.
- A workspace member without `[package.metadata.sifr]` is classified as Rust-only unless the user explicitly selects it as a Sifr package, in which case `SIFR-PACKAGE-0102` is reported.

Cargo workspace selection:

- Sifr consumes the flattened workspace member list from `cargo metadata`; it does not expand Cargo `members` globs itself.
- Cargo `exclude` rules are honored because excluded packages do not appear as selected workspace members in Cargo metadata.
- Commands run from a workspace root without `--workspace`, `-p`, or `--filter` follow Cargo's default package selection, including `default-members`.
- Commands with `--workspace` select all Cargo workspace members, then filter to Sifr-capable members for build/check/test commands. `sifr tree --all` is a display-only mode that includes backend Rust crates from Cargo metadata.
- There is no separate Sifr exclude mechanism in Phase 37. Users use Cargo workspace membership plus Sifr `--filter` selectors.

`[workspace.dependencies]`:

- Cargo `[workspace.dependencies]` provides shared dependency declarations for members that opt into them through Cargo's `workspace = true` dependency syntax.
- Workspace dependencies cannot be declared as `optional`, and features declared in `[workspace.dependencies]` are additive with member-level dependency features.
- Sifr does not treat workspace dependencies as globally importable by every member. A workspace dependency becomes part of package `P`'s Sifr import scope only when Cargo metadata reports it as a resolved dependency edge from `P`.
- A member's explicit dependency declaration and Cargo's feature unification rules remain Cargo-owned. Sifr consumes the resolved dependency edge and package id from Cargo metadata.
- Workspace-inherited Sifr packages participate in scoped imports exactly like normal direct dependencies once Cargo reports them as direct dependencies of the member.

Path dependencies between workspace members:

- When package `A` has a path dependency on package `B` and both expose Sifr metadata, `B` is a direct Sifr dependency in `A`'s package scope.
- Path dependency workspace members use the same package identity, aliasing, type identity, and generated namespace rules as registry and Git packages.
- `sifr build --workspace` validates and schedules selected Sifr workspace members in Cargo topological order. Independent packages may be checked in parallel after graph validation.
- Path dependency cycles reported by Cargo are surfaced as Cargo-backed package diagnostics. If Sifr detects a cycle in the Sifr package graph before Cargo reports it, `SIFR-PACKAGE-0205` reports the full dependency path.
- A path dependency on a workspace member without Sifr metadata is a Rust backend dependency. It is an error only if Sifr source attempts to import it as a Sifr package or the user explicitly selected it as Sifr-capable.

Mixed Sifr/Rust workspaces:

- A Cargo workspace may contain Sifr packages, Rust-only packages, and Rust-backed Sifr packages.
- `sifr build --workspace` and `sifr test --workspace` select Sifr-capable members. Rust-only members are built by Cargo only when reachable as backend dependencies of selected Sifr packages or generated Rust.
- Rust-only members not reachable from any selected Sifr package are ignored by Sifr in Phase 37.
- A Rust-only workspace member depending on a Sifr package is not a supported Phase 37 integration pattern. `SIFR-PACKAGE-0106` reports this and suggests converting the Rust member into a Rust-backed Sifr package or making the dependency an implementation detail behind Cargo features.
- Trust validation only considers backend crates reachable from selected Sifr packages. Workspace Rust-only members outside that reachability closure do not affect Sifr trust diagnostics.

Sifr workspace manifest vs Cargo workspace:

- Phase 37 delegates package workspace membership to Cargo.
- Per-package `sifr.toml` remains the only Sifr compiler metadata file for package management.
- A root `sifr.toml` has package-management meaning only when the Cargo root is also a Cargo package and `[package.metadata.sifr].manifest` points to it.
- A virtual Cargo workspace root does not need and must not imply a root Sifr package manifest.
- The `[workspace]` table described in `internal_docs/sifr_workspace_design.md` is a source-resolution/workspace-configuration concept from earlier phases, not a package dependency resolver. Phase 37 keeps that compatibility behavior but does not use it to select packages or resolve external dependencies.
- Workspace-wide Sifr policies such as shared trust, edition migration, or monorepo diagnostic policy are deferred to a future workspace-policy phase.

Selectors adapted from Turborepo:

- `pkg` selects one Sifr/Cargo package by name or configured alias.
- `{path/glob}` selects packages by root path.
- `pkg...` selects `pkg` plus dependency closure.
- `...pkg` selects `pkg` plus dependent closure.
- `...^pkg` selects dependents only.
- `[base...head]` selects packages owning changed files using Git ranges.

Additional selector flags:

- `--no-default-features` deactivates default Sifr and Cargo feature activation for selected packages.
- `--all-features` activates all Sifr and Cargo features for selected packages.
- Repeated `--filter` flags are ORed.
- Comma-separated selectors inside one `--filter` are ANDed.
- `--filter '!pkg'` removes `pkg` from the current selection set.

Changed-file detection may use `git` CLI or a Git library behind an adapter. Sifr owns the mapping from changed files to Sifr package roots. Files outside selected package roots are ignored unless the changed path is a workspace manifest, lockfile, package manifest, or shared config file that can affect the whole selected graph; unmappable changed files that were explicitly selected produce `SIFR-PACKAGE-0603`.

LSP and editor integration:

- The Sifr language server uses the same workspace discovery as the CLI: nearest Cargo workspace root plus `cargo metadata`.
- Multi-root editor sessions treat each Cargo workspace root as an independent Sifr workspace.
- LSP package graph discovery is read-only and uses frozen-equivalent behavior: no lock mutation, no network access, no manifest writes.
- If the workspace lockfile or source cache is stale, the LSP reports package diagnostics and asks the user to run the relevant CLI command rather than mutating state.
- The LSP uses the same `PackageSourceMap` as CLI builds for hover, completion, goto-definition, rename, and diagnostics.
- Package graph recomputation is incremental by package root. Changes to workspace manifests, `Cargo.lock`, or shared config invalidate the whole selected graph.

## Package Name Resolution And Discovery

Cargo owns registry resolution after a Cargo package name is known. Sifr owns the user-facing mapping from Sifr import/package names to Cargo distribution names.

Initial lookup order for `sifr add <name>`:

1. If `<name>` is already an exact Cargo package name, try it directly.
2. If `<name>` has no `sifr-` prefix, try the configured Sifr naming convention `sifr-<name>`.
3. If the workspace or user config defines an explicit alias, use that mapping.
4. If a Cargo alternate registry is configured for Sifr packages, search that registry according to Cargo's registry configuration.
5. If multiple candidates remain, fail with a diagnostic and require an explicit Cargo package name or alias.

Cargo registry indexes are not required to understand Sifr exports or compiler metadata. Discovery is intentionally coarse: naming convention and registry selection find candidate Cargo packages, then Cargo resolves and fetches them, and Sifr validates `sifr.toml` from the selected package root. A future Sifr registry index may improve search quality, but it must remain an index over Cargo packages rather than a separate resolver.

Compiler version compatibility is validated after Cargo resolution. Every selected Sifr package's `sifr-version` requirement must match the active Sifr compiler before source compilation. Cargo may select a package version that is semver-valid but compiler-incompatible; Sifr rejects that graph with a package diagnostic rather than attempting fallback resolution.

## Organization Demo Repositories

Phase 37 should include concrete demos backed by Git repositories in the `sifr-lang` GitHub organization. The current public org repositories are the compiler/tooling repos (`sifr`, `sifr-website`, `leetcode`, `ruff`, `sifr-vscode`, and `editor-integrations`), so the package demos below require creating small package repositories under the same org rather than pretending those existing tooling repos are Sifr packages.

Closeout note: these repositories now exist under `sifr-lang/` and are checked
out through git submodules under
`verification/package_management/demo_repositories/`. The local guardrail
validates the submodule declarations, required files, trust declarations, Git tag
references, alias coverage, lockfile shape, and workspace shape.

Required demo package repos:

- `https://github.com/sifr-lang/sifr-demo-json`: pure Sifr source package exporting `demo_json`.
- `https://github.com/sifr-lang/sifr-demo-http`: Rust-backed Sifr package exporting `demo_http`, depending on `sifr-demo-json` and a trusted Rust HTTP backend.
- `https://github.com/sifr-lang/sifr-demo-test-support`: dev-only Sifr package exporting `demo_test_support`.
- `https://github.com/sifr-lang/sifr-demo-app`: consumer app/workspace that exercises Git dependencies, aliases, lock modes, and workspace filters.

Each package repo must use the same Phase 37 file model:

```text
sifr-demo-json/
  Cargo.toml
  Cargo.lock
  sifr.toml
  sifr/demo_json/__init__.sifr
  sifr/demo_json/parse.sifr
  src/lib.rs
```

Pure package manifest:

```toml
# https://github.com/sifr-lang/sifr-demo-json/blob/main/Cargo.toml
[package]
name = "sifr-demo-json"
version = "0.1.0"
edition = "2024"
include = ["Cargo.toml", "sifr.toml", "sifr/**/*.sifr", "src/lib.rs", "README.md", "LICENSE"]

[package.metadata.sifr]
manifest = "sifr.toml"
```

```toml
# https://github.com/sifr-lang/sifr-demo-json/blob/main/sifr.toml
[package]
name = "demo_json"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
roots = ["sifr"]

[exports]
modules = ["demo_json"]
```

Rust-backed package manifest:

```toml
# https://github.com/sifr-lang/sifr-demo-http/blob/main/Cargo.toml
[package]
name = "sifr-demo-http"
version = "0.1.0"
edition = "2024"
include = ["Cargo.toml", "sifr.toml", "sifr/**/*.sifr", "src/**/*.rs", "README.md", "LICENSE"]

[package.metadata.sifr]
manifest = "sifr.toml"

[dependencies]
sifr-demo-json = { git = "https://github.com/sifr-lang/sifr-demo-json", tag = "v0.1.0" }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
```

```toml
# https://github.com/sifr-lang/sifr-demo-http/blob/main/sifr.toml
[package]
name = "demo_http"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
roots = ["sifr"]

[exports]
modules = ["demo_http"]

[trust]
native = ["reqwest"]
build-scripts = []
proc-macros = []
```

Consumer demo:

```toml
# https://github.com/sifr-lang/sifr-demo-app/blob/main/Cargo.toml
[package]
name = "sifr-demo-app"
version = "0.1.0"
edition = "2024"

[package.metadata.sifr]
manifest = "sifr.toml"

[dependencies]
sifr-demo-json = { git = "https://github.com/sifr-lang/sifr-demo-json", tag = "v0.1.0" }
sifr-demo-http = { git = "https://github.com/sifr-lang/sifr-demo-http", tag = "v0.1.0" }

[dev-dependencies]
sifr-demo-test-support = { git = "https://github.com/sifr-lang/sifr-demo-test-support", tag = "v0.1.0" }
```

```toml
# https://github.com/sifr-lang/sifr-demo-app/blob/main/sifr.toml
[package]
name = "demo_app"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
roots = ["sifr"]

[exports]
modules = ["app"]
```

```sifr
# https://github.com/sifr-lang/sifr-demo-app/blob/main/sifr/app/__init__.sifr
from app.main import main
```

```sifr
# https://github.com/sifr-lang/sifr-demo-app/blob/main/sifr/app/main.sifr
from demo_json.parse import parse_json
from demo_http.client import get

def main() -> Result[int, Error]:
    response = get("https://example.com/config.json")?
    config = parse_json(response.body)?
    return Ok(config["status_code"].as_int()?)
```

The package repos should include enough source to make these imports self-verifying. At minimum:

```sifr
# https://github.com/sifr-lang/sifr-demo-json/blob/main/sifr/demo_json/parse.sifr
class DemoJsonError(Error):
    message: str

class DemoJsonValue:
    def as_int(self) -> Result[int, DemoJsonError]:
        ...

def parse_json(text: str) -> Result[dict[str, DemoJsonValue], DemoJsonError]:
    ...
```

```sifr
# https://github.com/sifr-lang/sifr-demo-http/blob/main/sifr/demo_http/client.sifr
class DemoHttpError(Error):
    message: str

class DemoHttpResponse:
    body: str

def get(url: str) -> Result[DemoHttpResponse, DemoHttpError]:
    ...
```

`sifr-demo-http` must also include a minimal Rust shim under `src/lib.rs` or another included Rust source file that uses `reqwest` so the trust policy is exercised by the demo rather than merely declared.

Multiple-version alias demo:

```toml
# Additions to https://github.com/sifr-lang/sifr-demo-app/blob/main/Cargo.toml
[dependencies]
demo_json_v1 = { package = "sifr-demo-json", git = "https://github.com/sifr-lang/sifr-demo-json", tag = "v0.1.0" }
demo_json_v2 = { package = "sifr-demo-json", git = "https://github.com/sifr-lang/sifr-demo-json", tag = "v0.2.0" }

[package.metadata.sifr.aliases]
demo_json_v1 = { dependency = "demo_json_v1", import = "demo_json_v1" }
demo_json_v2 = { dependency = "demo_json_v2", import = "demo_json_v2" }
```

```sifr
import demo_json_v1
import demo_json_v2

def migrate(raw: str) -> Result[demo_json_v2.Value, Error]:
    old = demo_json_v1.parse(raw)?
    return demo_json_v2.from_legacy(old)?
```

The demo suite must validate:

- Git dependency fetch through Cargo.
- `Cargo.lock` pins Git revisions/tags.
- `sifr build --locked` succeeds after lock creation.
- `sifr build --offline` fails with `SIFR-PACKAGE-0104` before fetch and succeeds after `sifr fetch`.
- package archive validation catches a branch that omits `sifr/**/*.sifr` from Cargo packaging.
- pure marker validation rejects a branch where `sifr-demo-json/src/lib.rs` contains Rust implementation.
- Rust-backed trust validation accepts `sifr-demo-http` only with explicit `reqwest` trust.
- aliasing two Git-tagged versions of `sifr-demo-json` produces distinct package-instance type identities.
- `sifr test --workspace --filter ...sifr-demo-http` selects the demo package plus its dependency closure.

Additional monorepo demo repo required:

- `https://github.com/sifr-lang/sifr-demo-workspace`: Cargo workspace demonstrating real monorepo behavior.

Required shape:

```text
sifr-demo-workspace/
  Cargo.toml                  # virtual workspace root
  Cargo.lock                  # shared workspace lock
  packages/
    core/
      Cargo.toml
      sifr.toml
      sifr/demo_core/__init__.sifr
    utils/
      Cargo.toml
      sifr.toml
      sifr/demo_utils/__init__.sifr
    app/
      Cargo.toml
      sifr.toml
      sifr/app/__init__.sifr
      sifr/app/main.sifr
    backend-utils/            # Rust-only member
      Cargo.toml
      src/lib.rs
```

Workspace root:

```toml
[workspace]
members = ["packages/*"]
default-members = ["packages/app", "packages/core"]
exclude = ["packages/experimental-*"]
resolver = "3"

[workspace.dependencies]
sifr-demo-core = { path = "packages/core" }
sifr-demo-utils = { path = "packages/utils" }
serde = "1"
```

The demo must validate:

- virtual workspace root has no Sifr package identity;
- `sifr build` from `packages/app` builds only `app` using the root `Cargo.lock`;
- `sifr build --workspace` from `packages/app` builds all selected Sifr workspace members;
- default package selection from the workspace root follows Cargo `default-members`;
- `exclude`d members are not selected;
- `[workspace.dependencies]` inherited through `workspace = true` appears as a direct scoped dependency in Cargo metadata;
- path dependencies compile in topological order;
- path dependency cycles produce `SIFR-PACKAGE-0205`;
- Rust-only `backend-utils` is ignored unless reachable as a backend dependency of a selected Sifr package;
- changed-file filters map files under each member to the correct package and treat root `Cargo.toml` / `Cargo.lock` as whole-graph invalidations;
- multi-root editor sessions can open this workspace beside another Cargo workspace without graph leakage.

## CLI Contract

Sifr remains the user-facing compiler UX. Package-management commands are Cargo-coordinating commands, not independent resolver commands.

```bash
sifr init [--lib|--bin] [name]
sifr add <package> [--dev] [--features f1,f2] [--package member]
sifr remove <package> [--dev] [--package member]
sifr update [package]
sifr fetch [--locked|--offline]
sifr vendor <dir>
sifr tree [--workspace|-p package] [--sifr-only|--all] [--depth N]
sifr package [--dry-run]
sifr publish [--dry-run]
sifr check [file.sifr] [--locked|--frozen|--offline] [--features f1,f2|--all-features|--no-default-features]
sifr build [file.sifr] [--locked|--frozen|--offline] [--features f1,f2|--all-features|--no-default-features]
sifr run [file.sifr] [--locked|--frozen|--offline] [--features f1,f2|--all-features|--no-default-features]
sifr test [--workspace|-p package|--filter selector] [--locked|--frozen|--offline] [--features f1,f2|--all-features|--no-default-features]
```

Behavior:

- `sifr add demo_http` maps Sifr package name `demo_http` to Cargo package `sifr-demo-http` through configured naming policy or registry search, then delegates package name/version mutation to Cargo.
- `sifr add sifr-demo-http` may be accepted directly as a Cargo package name.
- `sifr add <package> --features f1,f2` validates requested features against the selected Cargo package feature set before mutating manifests. Cargo-backed features are passed to `cargo add`; source-only Sifr features are written only to `sifr.toml` `[features]`.
- `sifr update` delegates package updates to Cargo and follows Cargo's selected-package/update semantics; Sifr does not invent a separate recursive update mode.
- `sifr fetch` delegates source fetching to Cargo.
- `sifr vendor` delegates vendoring to Cargo and validates Sifr metadata after vendoring.
- `sifr tree` displays the package dependency tree. `--sifr-only` shows only Sifr source packages. `--all` includes backend Rust crates from Cargo metadata. `--workspace` shows the Cargo workspace graph filtered by Sifr metadata. `--depth N` limits display depth. Cycles are marked instead of recursing indefinitely.
- `sifr package --dry-run` runs Cargo package validation plus Sifr package validation.
- `sifr publish` runs Cargo dry-run packaging, Sifr package validation, then delegates upload to Cargo after validation succeeds.
- Cargo authentication prompts, token lookup, alternate registries, and registry config are Cargo-owned. Sifr only redacts and classifies errors.

Mode semantics:

| Sifr mode | Cargo behavior | Sifr behavior |
| --- | --- | --- |
| default | normal Cargo behavior | derive graph and validate metadata |
| `--locked` | Cargo locked mode | no lock mutation; derived graph only |
| `--offline` | Cargo offline mode | fail if selected Sifr source unavailable |
| `--frozen` | Cargo frozen mode | no network, no lock mutation, no non-disposable generated state |

`--frozen` implies `--locked --offline`. Disposable generated Rust artifacts may be created only under generated output roots such as `target/` or `.sifr-gen/`; frozen mode must not write user manifests, lockfiles, registry caches, package source files, or user source tree files.

## Publishing

Publishing is Cargo-compatible by default.

`sifr package --dry-run` must validate before `cargo publish`:

- `Cargo.toml` is valid.
- `Cargo.lock` policy is satisfied when required.
- `[package.metadata.sifr]` exists and points to the validated `sifr.toml`.
- `.sifr` source files are included in Cargo package archive.
- `sifr.toml` is included in Cargo package archive.
- source roots and exports are valid.
- imports parse/type-check according to command mode.
- backend Rust/native trust policy is explicit.
- package archive rejects traversal and unsupported file types through Cargo packaging plus Sifr checks.
- credentials are never printed.

Archive inclusion policy:

- Sifr does not require authors to write explicit Cargo `include` patterns.
- Sifr computes required files from `sifr.toml` source roots, exports, package manifests, and marker/backend Rust classification.
- `sifr package --dry-run` invokes Cargo package dry-run/list behavior where available, inspects the exact package archive file set Cargo would publish, and verifies every required `.sifr` source file and `sifr.toml` entry is present.
- If the Cargo CLI surface cannot provide a reliable file list, Sifr reads the generated `.crate` archive directly during dry-run validation.
- If Cargo `include`/`exclude` rules omit required Sifr files, Sifr reports `SIFR-PACKAGE-0403`.

Publish flow:

1. Run Cargo package dry-run without mutating the lockfile.
2. Run Sifr package validation against the exact archive file set Cargo would publish.
3. Run Sifr check/type validation for exported modules under the requested lock/network mode.
4. Invoke `cargo publish`.
5. If final publish fails because of credentials, network, yanked/conflicting versions, or registry-side policy, report `SIFR-PACKAGE-0402` with the redacted Cargo cause. The failed publish does not create Sifr-owned persistent state.

Cargo's package verification compiles Rust targets. Sifr package verification additionally compiles/checks Sifr source packages.

Yanking, owner management, alternate registries, and credentials follow Cargo registry behavior in Phase 37.

## Rust Backend And Trust

Because Cargo is primary, backend Rust crates and Sifr package crates may live in one Cargo dependency graph.

Sifr must classify backend Rust crates and gate native behavior:

- build scripts;
- proc macros;
- `links` crates;
- native library linking;
- generated Rust dependency requirements;
- package-declared Rust interop.

Trust policy lives in `sifr.toml`. Empty trust arrays mean no native capability is trusted by default. Packages with native/backend behavior must explicitly declare trust.

Trust is validated only against direct Cargo dependencies declared in the package's `Cargo.toml`. Transitive trust is not inherited; each Sifr package declares trust for its own direct backend dependencies. If a package declares trust for a crate that is not a direct dependency, `SIFR-PACKAGE-0305` is reported. If a reachable backend crate has native behavior but is not declared in `[trust]`, `SIFR-PACKAGE-0301` applies.

Sifr diagnostics should map backend trust failures to the Sifr package that introduced the backend dependency where Cargo metadata can determine that path.

## Diagnostics

Sifr diagnostics own user-facing reporting even when Cargo provides the underlying failure.

Diagnostic examples:

| Code | Meaning |
| --- | --- |
| `SIFR-PACKAGE-0001` | missing or invalid `[package.metadata.sifr]` |
| `SIFR-PACKAGE-0002` | missing or invalid `sifr.toml` |
| `SIFR-PACKAGE-0003` | unsupported or misplaced Sifr compiler metadata in Cargo metadata |
| `SIFR-PACKAGE-0101` | Cargo metadata/lock/source command failed |
| `SIFR-PACKAGE-0102` | selected Cargo package expected to be Sifr-capable but has no Sifr metadata |
| `SIFR-PACKAGE-0103` | Cargo metadata parsing or normalization error |
| `SIFR-PACKAGE-0104` | package source unavailable in offline/frozen mode |
| `SIFR-PACKAGE-0105` | retired; Cargo credential failures use `SIFR-PACKAGE-0101` |
| `SIFR-PACKAGE-0106` | Rust-only workspace member depends on a Sifr package in an unsupported Phase 37 pattern |
| `SIFR-PACKAGE-0201` | ambiguous import root in one package dependency scope |
| `SIFR-PACKAGE-0202` | undeclared direct dependency import |
| `SIFR-PACKAGE-0203` | private package module access |
| `SIFR-PACKAGE-0204` | type identity mismatch between two package instances |
| `SIFR-PACKAGE-0205` | circular path dependency between workspace Sifr packages |
| `SIFR-PACKAGE-0301` | backend native trust violation |
| `SIFR-PACKAGE-0303` | Sifr feature mapping references unavailable Cargo package |
| `SIFR-PACKAGE-0304` | Sifr feature mapping references unavailable Cargo feature |
| `SIFR-PACKAGE-0305` | trust policy references a non-direct Cargo dependency |
| `SIFR-PACKAGE-0401` | Cargo package archive missing required Sifr source or metadata |
| `SIFR-PACKAGE-0402` | publish validation failed |
| `SIFR-PACKAGE-0403` | Cargo include/exclude rules omit required Sifr source |
| `SIFR-PACKAGE-0501` | pure Sifr Rust marker contains implementation |
| `SIFR-PACKAGE-0601` | package selector matches multiple packages without disambiguation |
| `SIFR-PACKAGE-0602` | duplicate Sifr package import root across workspace members sharing one dependency scope |
| `SIFR-PACKAGE-0603` | changed-file mapping failed for an explicitly selected path |
| `SIFR-PACKAGE-0604` | outdated query unsupported for package source |

`SIFR-PACKAGE-0302` and `SIFR-PACKAGE-0306` through `SIFR-PACKAGE-0309` are reserved for future backend trust and feature diagnostics.

Every diagnostic must include structured origin data where applicable:

- `Cargo.toml` path and key.
- `sifr.toml` path and key.
- Cargo package id.
- Sifr package id.
- import root.
- source root.
- dependency path.
- Cargo command/action.
- remediation suggestion.

Cargo stderr/stdout must not be surfaced raw in compiler diagnostics except as redacted, structured cause text.

## Maintainability Architecture

Phase 37 introduces one coordination crate first: `crates/sifr_package`.

Module map:

```text
crates/sifr_package/src/
  lib.rs
  cargo/{metadata,commands,lock_modes,package,trust,errors}.rs
  manifest/{sifr,metadata,validate}.rs
  graph/{derive,scopes,workspace,filters,changed,digest}.rs
  imports/{source_map,boundaries,reexports,aliases}.rs
  source/{layout,include,discover}.rs
  ops/{plan,mutate,resolve,read,publish}.rs
  diag/{codes,lower_cargo,origins,redaction}.rs
  test_support/
  test_assets/
```

`ops::plan::OperationPlan` is the gate for every mutating CLI command. It records the requested operation, selected package/workspace scope, lock/network mode, manifest mutations, Cargo commands to invoke, and validated package graph digest. It prevents mutation under `--frozen`, refuses lockfile or manifest writes under `--locked` where Cargo would reject them, and gives diagnostics a single place to explain what would change before `sifr add`, `remove`, `update`, `package`, or `publish` proceeds.

`graph::workspace` owns Cargo workspace interpretation: virtual workspace detection, member/default/exclude selection, subdirectory-to-member mapping, workspace dependency edge normalization, path dependency ordering, and changed-file-to-package mapping. `graph::derive` consumes that normalized workspace view and builds the package graph; it must not re-interpret Cargo workspace manifests directly.

Boundary rules:

- `crates/sifr` CLI parses command flags and renders output; it does not parse Cargo metadata ad hoc or walk Cargo caches directly.
- `sifr_driver` consumes `PackageSourceMap` and package graph validation results; it does not call Cargo directly except through approved driver build paths.
- `sifr_frontend` and `sifr_hir` consume immutable package-origin data only.
- `crates/sifr_package::cargo::*` is the only subtree that shells out to Cargo or imports Cargo metadata crates.
- no Cargo metadata crate type crosses the public `sifr_package` facade.
- diagnostics are constructed as `sifr_diagnostics::SifrDiagnostic`; no parallel diagnostic renderer exists.

`crates/sifr_package/DEPENDENCY_AUDIT.md` records each Cargo integration crate or command surface, why it is used, stability risk, and fallback plan.

`crates/sifr_package/TRACEABILITY.md` maps Cargo behavior categories to Sifr tests, milestones, diagnostics, and intentional divergences.

`crates/sifr_package/FEATURES.md` records Cargo/Sifr feature interactions and which Cargo features matter for Sifr imports.

## Guardrails

Add `scripts/check_package_manager_guardrails.py` when `crates/sifr_package` lands.

It must enforce:

- package-manager source files stay below documented line limits;
- Cargo command/metadata integration is isolated under `crates/sifr_package/src/cargo`;
- public `sifr_package` APIs expose only Sifr-owned types;
- mutating CLI commands route through `OperationPlan`;
- no production path imports Cargo internals directly without audit entry;
- `DEPENDENCY_AUDIT.md`, `TRACEABILITY.md`, and `FEATURES.md` exist once the first Cargo-backed adapter lands;
- package metadata tests exist for every supported package layout;
- no code path walks Cargo registry/source cache internals outside approved Cargo metadata/source discovery adapters.
- pure Sifr marker validation exists before pure Sifr packages can be packaged or published;
- graph derivation tests normalize shuffled Cargo metadata records before digesting or comparing graph output.

## Correctness And Test Reuse

Reuse Cargo tests and behavior more directly than v1/v2.

Cargo categories to port/adapt:

- resolver tests: semver, yanks, features, renamed dependencies, multiple versions, source identity, Git/path/registry, cycles, conflict paths;
- lockfile tests: locked/offline/frozen, stale locks, bad locks, reproducibility;
- metadata tests: package metadata, renamed deps, workspace members, target filtering, feature activation;
- package/publish tests: include/exclude, missing files, dirty trees, verification, alternate registries, credentials, yanks;
- vendor/fetch tests: source availability, offline cache, registry replacement;
- workspace tests: default members, virtual workspaces, package selection;
- build script/proc macro/links tests for trust-gating behavior.

`TRACEABILITY.md` must classify every reused Cargo behavior category as one of:

- `ported`: Cargo test logic reimplemented against Sifr public behavior;
- `adapted`: Cargo test shape reused with Sifr-specific assertions;
- `skipped`: not applicable to the Sifr model, with reason;
- `deferred`: intentionally moved to a later phase, with owner.

Sifr-specific tests:

- Cargo package carrying pure `.sifr` source with marker `src/lib.rs`.
- pure `.sifr` package with non-trivial marker Rust source rejected.
- Cargo package carrying both `.sifr` source and Rust backend deps.
- missing `[package.metadata.sifr]`.
- mismatched Cargo metadata and `sifr.toml`.
- missing `.sifr` files from package archive.
- Cargo `include`/`exclude` omitting required `.sifr` files rejected.
- multiple versions of same Sifr import root in different scopes accepted.
- same import root twice in one scope rejected unless aliased.
- aliasing two versions in one scope accepted.
- type identity mismatch diagnostic across package versions.
- cross-scope type passing succeeds only when expected/actual package instance ids match.
- direct dependency import boundary rejection.
- transitive dependency source compilation.
- offline package source unavailable diagnostic.
- private registry credential error redaction/classification.
- `__init__.sifr` re-export validation.
- private module access rejection.
- backend trust failures.
- `sifr build --frozen` with valid `Cargo.lock`.
- `sifr build --locked` with stale `Cargo.lock`.
- virtual Cargo workspace with no root Sifr package.
- workspace root default selection honors Cargo `default-members`.
- `--workspace` selection honors Cargo workspace `exclude`.
- workspace member imports a Sifr dependency inherited through `[workspace.dependencies]` and `workspace = true`.
- workspace member redeclares a workspace dependency with a different semver requirement and Cargo's resolved member edge wins.
- workspace member depends on another Sifr member through a path dependency.
- path dependency cycle between Sifr members reports `SIFR-PACKAGE-0205`.
- mixed Sifr/Rust workspace ignores Rust-only members outside the selected Sifr reachability closure.
- Rust-only workspace member depending on a Sifr package reports `SIFR-PACKAGE-0106`.
- subdirectory invocation uses the shared workspace `Cargo.lock`.
- changed-file selector maps member files, root manifests, lockfiles, and unknown selected paths deterministically.
- LSP monorepo discovery uses frozen-equivalent graph derivation and does not mutate lockfiles or manifests.

Property tests:

- Derived `SifrPackageGraph` is deterministic for same `cargo metadata`, lockfile, manifests, target, features, and selectors.
- Reordering Cargo metadata packages, dependencies, features, and workspace members does not change derived import graph or graph digest.
- Adding a non-Sifr Rust dependency does not change Sifr import graph unless it is a trusted backend dependency.
- Removing a direct Sifr dependency makes its import root unavailable in that scope.
- Transitive dependency imports are rejected unless declared directly or re-exported.
- Same import root in different dependency scopes can resolve to different package ids.
- Source-map paths are stable after temp/cache path normalization.
- Diagnostics are stable across repeated runs.

## Milestones

### milestone_37_1: Cargo Metadata And Sifr Manifest Linking

Scope:

- Add `crates/sifr_package` with facade types and Cargo adapter boundaries.
- Parse and validate `sifr.toml`.
- Parse and validate `[package.metadata.sifr]` from Cargo metadata.
- Derive selected Cargo package graph from normalized `cargo metadata`.
- Classify Sifr packages vs backend Rust crates.
- Validate pure Sifr marker targets.
- Add package diagnostics.

Definition of done:

- Cargo packages with Sifr metadata validate.
- pure Sifr packages with marker Rust target validate.
- pure Sifr packages with non-trivial marker Rust source fail with `SIFR-PACKAGE-0501`.
- invalid metadata, missing manifests, mismatched names/exports, invalid source roots, and compiler-version mismatches produce stable diagnostics.
- shuffled Cargo metadata input produces the same graph and digest.

### milestone_37_2: Package Graph, Scoped Imports, And Multiple Versions

Scope:

- Build `SifrPackageGraph`.
- Implement per-package direct dependency scopes.
- Support Cargo multiple-version graphs with scoped import resolution.
- Implement import aliases for multiple versions in one scope.
- Emit type identity diagnostics across package instances.
- Define cross-scope type passing through package instance identity.

Definition of done:

- same import root can resolve to different versions in different package scopes.
- ambiguity in one scope is rejected unless aliases are configured.
- type identity includes Cargo package id/version/source in diagnostics.
- values produced by one package instance cannot satisfy types from another package instance without an explicit conversion API.

### milestone_37_3: Package-Aware Source Compilation

Scope:

- Build `PackageSourceMap`.
- Integrate package origins with frontend project assembly.
- Add package directory and `__init__.sifr` re-export semantics.
- Enforce direct-dependency import boundaries.
- Compile dependency `.sifr` source through normal frontend/HIR/codegen.

Definition of done:

- dependency source compiles through the normal compiler pipeline.
- duplicate export roots in scope, undeclared imports, private modules, and missing source roots fail before downstream type noise.
- editor analysis uses the same package source map as CLI builds.

### milestone_37_4: Cargo Commands, Lock Modes, And Backend Trust

Scope:

- Implement Cargo command adapter for metadata, fetch, locked/offline/frozen validation, package, publish, vendor, add/remove/update where supported.
- Implement lazy fetch lifecycle and offline/frozen source availability diagnostics.
- Validate backend native trust policy.
- Extend generated artifact cache keys with Cargo lock digest, Sifr metadata/source digests, compiler version, target, profile, features, and selectors.
- Map Cargo failures to Sifr diagnostics.

Definition of done:

- `sifr build --locked`, `--offline`, and `--frozen` honor Cargo semantics.
- unavailable package sources in offline/frozen mode produce `SIFR-PACKAGE-0104`.
- missing private registry credentials produce redacted `SIFR-PACKAGE-0101`.
- backend trust failures are deterministic.
- generated Rust builds are reproducible from Cargo/Sifr inputs.

### milestone_37_5: Workspaces, Filters, And Tooling

Scope:

- Coordinate Cargo workspaces and Sifr package scopes.
- Implement package selectors and Turborepo-style filters.
- Implement changed-package selection.
- Update Phase 36 analysis surfaces for package graph awareness.
- Implement `sifr tree`, `outdated`, workspace `check/build/test`.
- Define `sifr outdated` as a read-only Cargo-coordinated query: report selected Sifr source packages whose locked Cargo package version is older than the newest compatible version available from the same Cargo source. Do not mutate `Cargo.lock`. For registry dependencies, use Cargo registry index metadata. For Git dependencies, report the locked tag/branch/revision and whether the remote ref advanced when that can be checked without violating lock/network mode. For path dependencies, report the local version as pinned. For alternate or private sources without usable index metadata, report `SIFR-PACKAGE-0604` as an explicit unknown.
- Implement virtual workspace, `default-members`, `exclude`, `[workspace.dependencies]`, path dependency, mixed Sifr/Rust workspace, subdirectory invocation, and LSP monorepo semantics from this phase.

Definition of done:

- `sifr build --workspace` from a workspace subdirectory builds the selected workspace using the shared root `Cargo.lock`; `sifr build` from a subdirectory builds only that member using the shared root `Cargo.lock`.
- `--workspace` selection honors Cargo workspace membership and `exclude`; root default selection honors Cargo `default-members`.
- virtual workspaces require no root Sifr package and select Sifr-capable members correctly.
- path dependency workspace members are compiled in Cargo topological order and cycles are diagnosed.
- `[workspace.dependencies]` inherited through Cargo metadata are importable by the inheriting member.
- mixed Sifr/Rust workspace members are handled correctly; Rust-only members outside the Sifr reachability closure are not compiled by Sifr.
- filters select package, dependency closure, dependent closure, negation, feature mode, and changed packages deterministically.
- tooling queries use the same derived graph as CLI builds.
- `sifr outdated` reports current locked version, newest compatible version, source, and unknown status without changing manifests or lockfiles.
- LSP workspace discovery handles member subdirectories and multi-root editor sessions without network or lockfile mutation.

### milestone_37_6: Packaging, Publishing, And Vendoring

Scope:

- Implement `sifr package --dry-run`.
- Validate Cargo package archive contents for Sifr packages.
- Integrate with Cargo publish/yank/vendor behavior.
- Add package archive security checks and credential redaction.

Definition of done:

- dry-run catches missing Sifr files, bad metadata, invalid exports, archive traversal, and backend trust issues.
- dry-run catches Cargo include/exclude rules that omit required Sifr files.
- publish delegates to Cargo-compatible upload after Sifr validation.
- credentials never appear in diagnostics, logs, generated files, or package metadata.

### milestone_37_7: Validation, Docs, And Guardrails

Scope:

- Add end-to-end fixtures for pure Sifr Cargo packages, Rust-backed Sifr packages, workspaces, path dependencies, Git dependencies, registry dependencies, multiple-version graphs, aliases, and publishing.
- Add `scripts/check_package_manager_guardrails.py`.
- Complete `DEPENDENCY_AUDIT.md`, `TRACEABILITY.md`, and `FEATURES.md`.
- Update public docs and internal architecture docs.
- Document uv/Python interop as future work rather than Phase 37 core.

Definition of done:

- `scripts/run_all_tests.sh --profile create-pr` and full local validation pass.
- Cargo behavior categories are mapped to Sifr tests or explicit non-port decisions.
- guardrails pass locally and are part of the authoritative validation gate.

## Quality Contract

- Phase 27 diagnostic stability and no-user-triggerable-panic invariants still apply.
- No Sifr package command may panic on malformed Cargo metadata, malformed Sifr manifests, missing files, invalid package archives, stale locks, or bad Cargo manifests.
- No command may silently fall back from Cargo locked/frozen/offline behavior to best-effort behavior.
- Package graph derivation must be deterministic.
- Derived graph artifacts are disposable and must never be treated as committed source of truth.
- Cargo failures must be redacted and mapped into stable Sifr diagnostics.
- Implementation must prefer root-cause fixes over compatibility fallbacks.

## Exit Gate

Phase 37 is complete when:

- Sifr packages can be distributed as Cargo packages containing `.sifr` source and Sifr metadata.
- `sifr build --frozen` is reproducible from `Cargo.toml`, `Cargo.lock`, `sifr.toml`, selected `.sifr` source, and compiler/toolchain inputs.
- pure Sifr packages, Rust-backed Sifr packages, multiple-version package graphs, aliases, and workspaces compile through one package-aware compiler path.
- import/export/package boundary diagnostics are stable and actionable.
- Cargo-backed resolution/download/cache/auth/publish/vendor behavior is reused rather than rebuilt.
- no committed `sifr.lock` is required for the Phase 37 model.
- uv/Python interop remains possible later without changing Sifr import semantics.
