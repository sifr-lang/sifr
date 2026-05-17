# Phase 37: Package Management

> Status: reviewed planning contract, ready for implementation only after Phase 36 is complete and this phase checklist issue is opened. This phase defines the complete package-management architecture; implementation must not carve out a separate MVP semantics path.

## Objective
Establish Sifr's native, source-first package manager as the canonical workflow for dependency declaration, resolution, lockfiles, source fetching, workspace/monorepo builds, generated Cargo integration, registry publishing, and deterministic validation.

The package manager is a Sifr-owned semantic layer. Users interact with `sifr` commands and `sifr.toml` / `sifr.lock`. Cargo remains the generated Rust/native build backend. uv is not a Sifr package resolver; uv-inspired ergonomics are adopted only where they fit Sifr's source-first model.

## Depends on
- Phase 36

## Design Principles
- Sifr packages are source packages: package archives contain `.sifr` source, `sifr.toml`, docs, license metadata, and optional backend metadata. `sifr add` fetches and locks source metadata; `sifr build` compiles the application plus reachable dependency source through the normal Sifr parser, HIR, type checker, ownership model, and codegen.
- Cargo is the backend build manager: generated Rust crates, native Rust dependencies, `rustc`, target triples, build profiles, and native linking are handled through generated Cargo projects. Cargo never becomes the user-facing Sifr package resolver or import resolver.
- uv is an optional ecosystem bridge only: future `pyproject.toml` / `[tool.sifr]` compatibility, CLI installation, or Python interop must parse into the same internal Sifr manifest model and must not fork resolver behavior.
- Package source roots are package-aware, not flattened into `[source].roots`. A dependency cache path must never become a normal workspace source root because that destroys origin tracking and makes shadowing ambiguous.
- The import resolver must preserve Sifr's existing stdlib protection: embedded `sifr.*` and `_sifr.*` remain separate, protected, and highest priority. User packages cannot override stdlib or intrinsic modules.
- Package boundaries are explicit. A module may import from its own package, the embedded stdlib, and declared direct dependencies. Transitive dependencies are compiled as needed but are not available for direct import unless declared or re-exported through a public API.
- Determinism is mandatory. Every build input that can affect compilation or linking is represented in `sifr.lock`, the package cache, generated Cargo inputs, or the artifact cache key.
- No arbitrary install scripts are allowed for ordinary Sifr packages. Native build behavior is confined to explicit backend Cargo dependencies and gated by trust policy.

## Canonical Roles

### Sifr Package Manager
- Parses and validates `sifr.toml`.
- Resolves Sifr source package versions, features, platform predicates, and workspace member dependencies.
- Fetches registry, Git, URL, and path source packages into a content-addressed cache.
- Writes and validates `sifr.lock`.
- Constructs package-aware module origins for compiler discovery.
- Enforces package boundaries, undeclared dependency errors, namespace conflict diagnostics, and lockfile staleness rules.
- Owns `sifr init`, `sifr add`, `sifr remove`, `sifr update`, `sifr sync`, `sifr fetch`, `sifr tree`, `sifr publish`, and workspace selection behavior.

### Cargo Backend
- Builds generated Rust projects and final native binaries.
- Resolves backend Rust crates required by Sifr stdlib, codegen, FFI, and package-declared backend dependencies.
- Produces generated `Cargo.toml` and `Cargo.lock` material under Sifr-controlled build/cache directories.
- Must be driven from Sifr's resolved graph. If Cargo resolution differs from the backend dependency section recorded in `sifr.lock`, the build fails instead of silently updating native dependencies under `--locked` / `--frozen`.

### uv Compatibility
- uv is not invoked by default for Sifr package resolution, lockfiles, or builds.
- Future `pyproject.toml` support may be accepted only as a compatibility manifest frontend that lowers into the same Sifr manifest structs used by native `sifr.toml`.
- Python package consumption, Python wheels, and virtual-environment behavior are out of scope for Phase 37 unless they are purely CLI-installation documentation and do not affect Sifr packages.

## Manifest Model

`sifr.toml` remains the canonical manifest. Unknown tables may remain forward-compatible only when they cannot affect package resolution; resolution-related unknown keys must be rejected with package diagnostics once Phase 37 schema validation is active.

```toml
[package]
name = "http"
version = "1.2.3"
edition = "2026"
description = "Typed HTTP client for Sifr"
license = "MIT"
repository = "https://github.com/sifr-lang/http"
readme = "README.md"
publish = ["sifr"] # or false

[workspace]
resolver = "1"
members = ["apps/*", "packages/*"]
exclude = ["packages/legacy-*", "target"]
default-members = ["apps/cli"]

[workspace.package]
edition = "2026"
license = "MIT"

[workspace.dependencies]
json = "1.4"
http = { version = "1.2", default-features = false }

[source]
roots = ["src"]

[exports]
modules = ["http"]

[dependencies]
json = { workspace = true }
tls = { package = "tls", version = "0.8", features = ["rustls"] }
local_utils = { path = "../local_utils" }
git_math = { git = "https://github.com/sifr-lang/math.git", rev = "abc123" }

[dev-dependencies]
test_helpers = { path = "../test_helpers" }

[features]
default = ["tls"]
tls = ["dep:tls", "tls/rustls"]
native-tls = ["dep:tls", "tls/native"]

[target.'cfg(unix)'.dependencies]
unix_paths = "0.2"

[backend.cargo.dependencies]
tokio = { version = "1.52", features = ["macros", "rt", "time"] }

[trust]
native = ["tokio"]
```

Required semantics:
- `[package].name` is the distribution name and must be registry-safe. Import names come from `[exports].modules`, not from arbitrary package names. Export roots must be valid Sifr/Python identifiers or dotted identifier paths.
- Package names support lowercase ASCII identifiers separated by `-`, `_`, or `.`, with optional registry namespace syntax `namespace/name` for published packages. Import/export roots never include `/`, `-`, or registry namespace punctuation.
- `[dependencies]`, `[dev-dependencies]`, and target-specific dependency tables declare Sifr source packages.
- Dependency keys are local dependency aliases. `{ package = "real-name" }` may rename the distribution package while preserving valid local identifiers.
- Workspace member dependencies must be explicit using `{ workspace = true }`, `{ path = "..." }`, or a normal dependency spec. Workspace membership alone does not make a package importable.
- `[workspace.dependencies]` is a central catalog/constraint source, not an implicit dependency. Members opt into catalog entries with `{ workspace = true }`.
- Workspace-level `[target.'cfg(...)'.dependencies]` tables are central catalogs for target-specific constraints; member packages opt in with `{ workspace = true }` just like non-target workspace dependencies.
- `[exports].modules` declares public import roots supplied by the package. A package may export multiple roots only when every root is intentional and non-overlapping.
- `[backend.cargo.dependencies]` is only for backend Rust/native dependencies. It is distinct from Sifr source dependencies.
- Feature activation uses additive union semantics. A feature cannot disable another feature; mutually exclusive backend choices such as `rustls` vs `native-tls` must be declared as conflicts and produce a resolution error if both are selected.
- Target predicates use Sifr-owned target expressions that map cleanly to Rust target triples during backend materialization.

Version requirement grammar:
- Bare versions use Cargo-style caret compatibility: `"1.2.3"` means `>=1.2.3,<2.0.0`; `"0.2.3"` means `>=0.2.3,<0.3.0`; `"0.0.3"` means `>=0.0.3,<0.0.4`.
- Explicit caret requirements use `^1.2.3` with the same compatibility rule as bare versions.
- Tilde requirements use `~1.2.3` for `>=1.2.3,<1.3.0` and `~1.2` for `>=1.2.0,<1.3.0`.
- Comparison requirements support `=`, `!=`, `>`, `>=`, `<`, and `<=`.
- Intersections are comma-separated, for example `">=1.2,<2.0,!=1.4.0"`.
- Wildcard requirements support `1.*` and `1.2.*`; bare `*` is rejected for published packages and allowed only for local path dependencies during development.
- Pre-release versions are ignored unless explicitly requested by a requirement containing the same pre-release base version.
- Build metadata after `+` is ignored for resolution and is rejected in dependency requirements for published packages.
- Lockfiles store the original requirement string and the concrete resolved version.

## Lockfile Model

`sifr.lock` is committed to version control and is the authoritative reproducibility artifact for Sifr package builds.

```toml
version = 1
resolver = "1"
workspace-root = "."
manifest-digest = "sha256:..."

[[package]]
id = "registry+sifr/http@1.2.3"
name = "http"
version = "1.2.3"
source = "registry+sifr"
checksum = "sha256:..."
manifest-checksum = "sha256:..."
source-roots = ["src"]
exports = ["http"]
features = ["default", "tls"]
dependencies = [
  { name = "json", requirement = "1.4", resolved = "registry+sifr/json@1.4.2" },
]

[[package]]
id = "path+../local_utils"
name = "local_utils"
version = "0.1.0"
source = "path+../local_utils"
checksum = "sha256:..."
source-roots = ["src"]
exports = ["local_utils"]

[[backend.cargo-package]]
name = "tokio"
version = "1.52.3"
source = "registry+crates-io"
checksum = "sha256:..."
features = ["macros", "rt", "time"]

[metadata]
generated-by = "sifr"
generated-at = "2026-05-17T00:00:00Z"
```

Required semantics:
- The lockfile records resolved Sifr package ids, checksums, source roots, exports, features, transitive dependencies, and target predicates.
- The lockfile records reachable backend Cargo packages or an equivalent canonical backend lock section sufficient to verify generated `Cargo.lock` deterministically.
- Registry and Git dependencies are locked by immutable version/checksum or commit id. Branch names may be accepted in manifests for development, but the lockfile must pin the resolved commit.
- Path dependencies record source checksums for cache invalidation but remain editable during local development.
- `--locked` fails if manifests, selected features, target predicates, package source metadata, or backend dependencies would change the lockfile.
- `--offline` forbids network access and fails on cache misses.
- `--frozen` is `--locked --offline` and is the recommended CI mode.
- The lockfile must never store credentials.

## Source Package And Import Semantics

Package source layout:

```text
http/
  sifr.toml
  src/
    http/
      __init__.sifr
      client.sifr
      headers.sifr
      _internal.sifr
  tests/
  README.md
  LICENSE
```

Required semantics:
- `__init__.sifr` defines package or subpackage public re-exports. It has no import-time side effects.
- Direct imports use Python-compatible syntax such as `import http`, `from http.client import get`, and `from http import Client`.
- Public modules and public symbols are controlled by `[exports]` and `__init__.sifr` re-exports. Private modules such as `_internal` are accessible only inside the declaring package unless explicitly exported.
- Package directories are first-class. The Phase 36-era namespace-file collision rejection is replaced by deterministic package-directory semantics.
- Re-exports must be explicit and type-checked through the normal frontend. Wildcard re-exports are rejected in Phase 37. Public package APIs use explicit statements in `__init__.sifr`, for example `from client import Client as Client` and `from headers import HeaderMap as HeaderMap`.
- Multiple packages exporting the same import root in one dependency scope are rejected unless the consuming manifest explicitly aliases one package to a different valid import root. Aliasing must preserve type identity in diagnostics and generated Rust module names.
- Sifr rejects unresolved, ambiguous, undeclared, private, cyclic, and shadowed imports with package diagnostics before HIR lowering emits downstream type noise.

The `ModuleResolver` must grow package-aware origins:

```rust
enum ModuleOrigin {
    EntryParent,
    WorkspaceSource { source_root: PathBuf },
    PackageSource {
        package_id: PackageId,
        source_root: PathBuf,
        checksum: String,
        export_root: String,
    },
    EmbeddedStdlib,
}
```

Resolution order:
1. Embedded `sifr.*` / `_sifr.*` stdlib and intrinsic modules.
2. The current package's own source roots.
3. Declared direct workspace-member dependencies.
4. Declared direct path, Git, URL, and registry dependencies from `sifr.lock`.
5. Transitive dependency source only when compiling that dependency's own modules or validating an explicit re-export.

Conflicts between local modules and declared dependency export roots are hard errors in package mode. Single-file mode may preserve entry-parent behavior when no manifest package graph is active.

## Workspace And Monorepo Model

Phase 37 completes workspace semantics rather than treating package management as only single-package dependency fetching.

Required workspace behavior:
- Root package workspaces and virtual workspaces are both supported.
- `members`, `exclude`, and `default-members` use deterministic glob expansion sorted lexicographically.
- The workspace root owns the single `sifr.lock`.
- Commands accept `-p/--package`, `--workspace`, `--exclude`, and `--filter` selectors.
- `--filter` supports package names, directory globs, dependency closure (`pkg...`), dependent closure (`...pkg`), dependents-only (`...^pkg`), and changed-package selectors such as `[main...HEAD]`.
- `--dry-run` and `--dry-run=json` expose selected packages, dependency edges, lockfile changes, cache hits/misses, and planned command execution without mutating files.
- Workspace member graph cycles are hard errors.
- Packages may not import files outside their package root except through declared dependencies and public exports.
- Dependencies belong where used. Root manifests may define shared constraints/catalogs and repo-level tools, but application/library dependencies belong in the member that imports them.
- Workspace package boundaries are validated in `sifr check`, `sifr build`, `sifr test`, and editor analysis.

Package-aware tooling must update Phase 36 analysis surfaces: workspace diagnostics, completion, auto-import, rename, references, document/workspace symbols, generated Rust preview, and test discovery all need package graph awareness without a separate resolver.

## CLI Contract

Package commands:

```bash
sifr init [--lib|--bin] [name]
sifr add <spec> [--dev] [--optional <feature>] [--features f1,f2] [--package member]
sifr add <name> --path <path>
sifr add <name> --git <url> [--rev <sha>|--tag <tag>|--branch <branch>]
sifr remove <name> [--dev] [--package member]
sifr update [name] [--recursive] [--dry-run]
sifr sync [--locked|--frozen|--offline]
sifr fetch [--locked|--offline]
sifr tree [--workspace|-p package] [--duplicates] [--features]
sifr outdated [--workspace|-p package]
sifr vendor <dir>
sifr package [--dry-run]
sifr publish [--dry-run] [--registry name]
sifr yank <name>@<version> [--registry name]
sifr login [--registry name]
sifr owner <name> --add <user>
```

Build/test commands become package-aware:

```bash
sifr check [file.sifr] [--locked|--frozen|--offline]
sifr build [file.sifr] [--locked|--frozen|--offline]
sifr run [file.sifr] [--locked|--frozen|--offline]
sifr test [--workspace|-p package|--filter selector] [--locked|--frozen|--offline]
```

Default ergonomics:
- `sifr build`, `sifr run`, `sifr check`, and `sifr test` automatically validate manifest freshness, update `sifr.lock` when allowed, fetch missing packages, and then compile.
- `sifr sync` explicitly resolves and fetches dependencies without compiling.
- `sifr fetch` downloads locked packages without changing resolution.
- `sifr vendor <dir>` writes a deterministic vendor tree containing every non-path source package in `sifr.lock`, emits a registry replacement config, and leaves `sifr.lock` package ids/checksums unchanged.
- `sifr outdated` compares locked package versions against the newest registry versions allowed by the manifest requirement and separately reports newer incompatible major versions.
- Mutating commands preserve TOML formatting as much as practical and produce deterministic table ordering.
- All commands preserve the existing exit-code contract: success `0`, compilation/semantic/package diagnostics `1`, CLI usage/config errors `2`, and internal/toolchain failures `3`.

## Registry, Publishing, And Trust

Registry protocol:
- Uses a sparse HTTP index by default so dependency solving fetches only relevant metadata.
- Index metadata includes package name, version, yanked status, checksum, dependencies, features, source roots, exports, compiler version range, target predicates, license, and required backend native trust metadata.
- Package tarballs are immutable once published. Deletion is not allowed; yanking hides versions from new resolution while preserving existing lockfiles.
- Alternate registries are named in Sifr config and referenced from dependency specs. Registry URLs must be HTTPS unless an explicit local development override is used.

Sparse index contract:
- `GET /index/config.json` returns registry protocol version, download URL template, publish API URL, and registry public metadata.
- `GET /index/{prefix}/{name}.json` returns newline-delimited JSON records, one immutable record per published version. The prefix is deterministic from the normalized package name.
- Each version record contains `name`, `version`, `checksum`, `yanked`, `dependencies`, `features`, `source_roots`, `exports`, `sifr_version`, `target_predicates`, `backend_cargo_dependencies`, and `native_trust_required`.
- `GET /api/v1/packages/{name}/{version}/download` returns the package archive whose content hash must match the index checksum.
- `PUT /api/v1/packages/new` publishes one deterministic archive plus metadata using a bearer token from `sifr login`.
- `PATCH /api/v1/packages/{name}/{version}/yank` marks a version as yanked without changing the archive or checksum.
- Existing lockfiles may continue to use yanked versions if the checksum matches; new resolution ignores yanked versions unless the user explicitly requests a yanked version and accepts the diagnostic prompt through a future governance-approved flag.

Package archive structure:
- Archives are deterministic `tar.zst` files with sorted entries, normalized permissions, normalized timestamps, and no absolute paths.
- Required entries are `sifr.toml`, all included `.sifr` source files, and declared `README` / license files.
- Archive entries may not contain `..`, absolute paths, unsupported symlinks, generated build outputs, credentials, or files outside the package root.

Publishing workflow:
1. Validate manifest schema, package name, version, exports, include/exclude rules, license/readme metadata, and compiler compatibility.
2. Reject package roots that contain path traversal, generated build artifacts, credentials, unsupported symlinks, or files outside the package root.
3. Run `sifr check`, package-boundary validation, lockfile validation, and package tests unless `--no-verify` is explicitly approved in a later governance phase.
4. Build a deterministic source archive.
5. Compute checksums.
6. Upload metadata and archive.
7. Verify index round-trip and print the immutable package id.

Trust model:
- Ordinary Sifr packages cannot execute install scripts.
- Backend native behavior is explicit through `[backend.cargo.dependencies]` and `[trust]`.
- Native trust is default-deny for build scripts, `links` crates, proc-macro backend crates, and any package metadata marked `native_trust_required = true`.
- Trust does not propagate by package author declaration. The consuming workspace root must trust every native-capable backend crate or Sifr package id that can execute build-time native code.
- A trusted direct dependency does not automatically trust an untrusted transitive native dependency; the diagnostic must report the full dependency path and the exact trust entry required.
- Cargo build scripts and `links` crates require explicit trust and appear in lockfile metadata.
- Credential storage must use platform credential stores or Sifr config with redaction; credentials never enter manifests, lockfiles, diagnostics, logs, or generated Cargo files.
- Package names, export roots, registry names, URLs, Git refs, archive entries, and cache paths are validated before filesystem writes.

## Package Diagnostics

Phase 37 reserves the `SIFR-PACKAGE-*` namespace. Implementations may add more codes, but the following codes and meanings are mandatory and stable:

| Code | Meaning |
| --- | --- |
| `SIFR-PACKAGE-0001` | malformed package manifest or resolution-affecting unknown key |
| `SIFR-PACKAGE-0002` | invalid dependency requirement or unsupported source spec |
| `SIFR-PACKAGE-0003` | invalid package name, dependency alias, export root, feature name, or registry name |
| `SIFR-PACKAGE-0101` | package/version could not be found in the selected source |
| `SIFR-PACKAGE-0102` | dependency version conflict with a reported conflict path |
| `SIFR-PACKAGE-0103` | feature conflict, unknown feature, or cyclic feature dependency |
| `SIFR-PACKAGE-0104` | yanked package selected during new resolution |
| `SIFR-PACKAGE-0201` | lockfile missing, stale, malformed, or incompatible with the active manifest graph |
| `SIFR-PACKAGE-0202` | lockfile checksum, manifest checksum, source checksum, or backend Cargo lock mismatch |
| `SIFR-PACKAGE-0203` | offline/frozen cache miss |
| `SIFR-PACKAGE-0301` | undeclared dependency import |
| `SIFR-PACKAGE-0302` | ambiguous package export root or module provider |
| `SIFR-PACKAGE-0303` | local/dependency namespace shadowing in package mode |
| `SIFR-PACKAGE-0304` | private package module or symbol imported outside its declaring package |
| `SIFR-PACKAGE-0305` | package directory, `__init__.sifr`, or explicit re-export contract violation |
| `SIFR-PACKAGE-0401` | untrusted native/build-script/proc-macro/backend dependency |
| `SIFR-PACKAGE-0402` | unsupported target predicate or native target selection |
| `SIFR-PACKAGE-0501` | registry protocol, authentication, or authorization failure |
| `SIFR-PACKAGE-0502` | package archive path traversal, forbidden file, checksum, or deterministic archive violation |
| `SIFR-PACKAGE-0503` | credential leak attempt or redaction failure |
| `SIFR-PACKAGE-0601` | workspace member selection, filter, boundary, or package graph cycle error |

Every package diagnostic must include structured origin data where applicable: manifest path and key, package id, source kind, dependency path, import site, selected target, registry name, and remediation suggestion.

## Artifact Cache And Generated Cargo Integration

The artifact cache key must include:
- Sifr compiler version and artifact cache schema version.
- Root manifest and all member manifests affecting the selected graph.
- `sifr.lock` contents or selected lockfile package subset digest.
- Selected package ids, source checksums, features, target predicates, and trust metadata.
- Embedded stdlib module digests.
- Generated Rust source and generated Cargo manifest content.
- Backend Cargo lock section or generated `Cargo.lock` digest.
- Target triple, profile, relevant environment variables, and explicit cache inputs.

Generated Cargo projects:
- Must be deterministic for the same selected package graph.
- Must preserve source-to-generated mapping for diagnostics and generated Rust preview.
- Must not expose package cache paths as normal user source roots.
- Must fail closed when generated Cargo resolution changes under `--locked` or `--frozen`.

Cache invalidation and recovery:
- Corrupt package-cache entries are deleted only after checksum verification fails and the diagnostic is emitted; `sifr sync` may refetch them when network access is allowed.
- Artifact-cache entries are disposable and may be evicted by size, age, or schema version without affecting reproducibility.
- Package-cache entries are content-addressed and are not evicted automatically while referenced by any lockfile known to the workspace unless a future cache-prune command explicitly does so.
- Cache summaries in `--dry-run=json` must expose cache hit/miss/corruption reasons without leaking credentials or absolute registry tokens.

## Milestones

### milestone_37_1: Manifest, Workspace, And Dependency Schema
- Scope:
  - Extend manifest parsing with package metadata, dependency specs, source types, features, target dependencies, workspace inheritance, workspace members/default-members, exports, backend Cargo dependencies, and trust metadata.
  - Add deterministic schema diagnostics for invalid tables/keys/values.
  - Add workspace member discovery and package selection planning.
- Definition of done:
  - Manifest and workspace graphs parse deterministically for root, virtual, and member workspaces.
  - Invalid dependency specs, invalid export roots, invalid workspace globs, package cycles, and invalid backend/native trust config produce stable package diagnostics.
  - Existing non-package workspace behavior remains compatible where manifests do not use Phase 37 package tables.

### milestone_37_2: Resolver, Lockfile, And Source Cache
- Scope:
  - Implement Sifr source dependency solving, feature resolution, target predicate selection, `sifr.lock`, staleness detection, content checksums, and a content-addressed package cache.
  - Support registry, Git, URL, and path source records at the model level.
  - Implement `sifr sync`, `sifr fetch`, `--locked`, `--offline`, and `--frozen`.
- Definition of done:
  - Builds are reproducible from `sifr.toml`, `sifr.lock`, and the package cache.
  - Lockfile drift, checksum mismatch, missing cache entries, yanked packages, unsupported protocols, and version conflicts fail with deterministic diagnostics.
  - Lockfile output is stable across repeated runs.

### milestone_37_3: Package-Aware Import Resolution
- Scope:
  - Replace deferred package-directory behavior with `__init__.sifr`, explicit re-exports, public/private package APIs, and package-aware `ModuleOrigin`.
  - Enforce direct-dependency imports and transitive dependency boundaries.
  - Integrate package source modules into import-closure parsing and project lowering.
- Definition of done:
  - User and dependency `.sifr` sources compile through the same frontend/HIR/codegen path.
  - Package roots are not flattened into `[source].roots`.
  - Ambiguous exports, undeclared imports, private module access, local/dependency shadowing, and package directory errors fail before downstream type noise.

### milestone_37_4: Cargo Backend And Native Dependency Bridge
- Scope:
  - Materialize generated Cargo projects from the selected Sifr package graph.
  - Resolve and lock reachable backend Cargo dependencies from stdlib/codegen/package metadata.
  - Gate native/build-script behavior through trust metadata.
  - Extend artifact cache keys and generated Rust preview metadata.
- Definition of done:
  - Generated Cargo manifests and locks are deterministic and verified against `sifr.lock`.
  - Native dependency changes invalidate caches predictably.
  - Untrusted native/build-script dependencies fail closed with actionable diagnostics.

### milestone_37_5: Workspace CLI, Filters, And Tooling Integration
- Scope:
  - Implement `sifr init`, `add`, `remove`, `update`, `tree`, `outdated`, `vendor`, package-aware `check/build/run/test`, workspace selectors, filters, dry-run/json summaries, and TOML mutation.
  - Update Phase 36 analysis surfaces to use the package graph.
- Definition of done:
  - Monorepo workflows support root/virtual workspaces, member package selection, dependency/dependent filtering, changed-package selection, and package boundary diagnostics.
  - Editor/tooling queries use the same package graph as CLI builds.
  - CLI mutation commands are deterministic and respect `--package`.

### milestone_37_6: Registry, Publish, Yank, And Credential Flows
- Scope:
  - Implement sparse registry protocol, registry config, login, package archive creation, publish dry-run, publish, yank, owner management, alternate registries, and metadata verification.
  - Add package archive security validation and credential redaction.
- Definition of done:
  - Published packages are immutable, checksum-addressed, and reproducible from source archives.
  - Yanked packages remain usable from existing lockfiles but are excluded from new resolution.
  - Credentials never appear in manifests, lockfiles, diagnostics, generated Cargo files, or logs.

### milestone_37_7: Validation, Documentation, And Ecosystem Gate
- Scope:
  - Add positive and negative E2E fixtures, package registry test infrastructure, workspace fixtures, cache tests, CLI docs, public package docs, architecture updates, and phase execution checklist evidence.
  - Add maintainability guardrails for package manager module decomposition.
- Definition of done:
  - `scripts/run_all_tests.sh --profile quick` and full local validation pass.
  - Package workflows are stable enough for broader ecosystem usage.
  - Validation evidence is recorded in the phase execution checklist issue with fixture names, commands, expected diagnostics, and merged PR links.

## Quality Contract
- Entry criteria: Phase 36 is completed and tooling contracts are stable.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Sifr package management workflows are stable enough for broader ecosystem usage across single-package projects, workspaces, source packages, generated Cargo/native dependencies, lockfiles, and registry publishing.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler/package-manager code: strict typing, deterministic behavior, explicit invariants, stable diagnostics, explicit trust boundaries, and unforgiving correctness standards.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
  - Package manager code must be decomposed into focused modules; no monolithic resolver, registry, CLI, or lockfile file is accepted.

## Validation Planning Goals
- `milestone_37_1` (Manifest, Workspace, And Dependency Schema):
  - Positive: parse root package, virtual workspace, member workspace, dependency catalog inheritance, path/Git/registry specs, export roots, and backend Cargo dependency specs.
  - Negative: malformed dependency syntax, invalid export root, unknown resolution-affecting key, workspace member cycle, invalid glob, invalid target predicate, invalid trust entry.
- `milestone_37_2` (Resolver, Lockfile, And Source Cache):
  - Positive: resolve registry/path/Git graph, write stable `sifr.lock`, fetch into cache, rebuild without lockfile churn, build offline after fetch.
  - Negative: stale lock under `--locked`, cache miss under `--offline`, checksum mismatch, yanked version on new resolution, unsupported protocol, version conflict with clear conflict path.
- `milestone_37_3` (Package-Aware Import Resolution):
  - Positive: import package modules, import re-exported symbols from `__init__.sifr`, compile transitive dependency source, support workspace member dependency import.
  - Negative: undeclared dependency import, ambiguous export root, local/dependency shadowing, private module access, missing `__init__.sifr`, package directory/file collision.
- `milestone_37_4` (Cargo Backend And Native Dependency Bridge):
  - Positive: generated Cargo manifest includes reachable backend crates from stdlib and package metadata, generated Cargo lock verifies against `sifr.lock`, native cache invalidates on backend feature change.
  - Negative: untrusted build-script dependency, backend lock drift under `--locked`, unsupported native target, conflicting backend feature requirements.
- `milestone_37_5` (Workspace CLI, Filters, And Tooling Integration):
  - Positive: `sifr add/remove/update` mutate selected member manifests, `--workspace` builds all members, `-p` builds one member, filters select dependencies/dependents/changed packages, dry-run/json reports planned changes.
  - Negative: unknown package selector, filter selecting no packages, import outside package root, dependency used but not declared, tool/editor query using stale package graph.
- `milestone_37_6` (Registry, Publish, Yank, And Credential Flows):
  - Positive: package dry-run archive is deterministic, publish uploads immutable package, yank hides package from new resolution, alternate registry resolves, credential redaction is verified.
  - Negative: publish missing metadata, duplicate immutable version, unauthorized namespace, insecure registry URL, archive path traversal, credential leak attempt, yanked package selected without existing lockfile.
- `milestone_37_7` (Validation, Documentation, And Ecosystem Gate):
  - Positive: docs cover app/library/workspace/package-author flows; package fixtures cover single package, monorepo, registry, Git, path, features, native deps, publish, and cache workflows.
  - Negative: maintainability guardrail rejects monolithic package manager files; docs/fixture drift checks catch undocumented CLI flags or diagnostics.
- Exit-gate evidence explicitly demonstrates: Package management workflows are stable enough for broader ecosystem usage with deterministic source package resolution, lockfiles, package-aware imports, generated Cargo/native integration, monorepo support, registry publishing, and Phase 27 non-regression guarantees.

## Exit Gate
- `sifr.toml` package tables and `sifr.lock` are the single documented package-management contract.
- `sifr add/remove/update/sync/fetch/tree/outdated/package/publish/yank/login/owner` and package-aware `check/build/run/test` behavior are documented, implemented, tested, and deterministic.
- Source packages from path, Git, URL, registry, and workspace members compile through the same Sifr frontend/HIR/codegen pipeline as application source.
- Package directories, `__init__.sifr`, re-exports, public/private package APIs, direct-dependency import enforcement, and package-aware diagnostics are implemented.
- Workspace/monorepo builds support members, excludes, default members, package selection, filters, dry-run/json summaries, and boundary diagnostics.
- Generated Cargo/native dependencies are locked, verified, trust-gated, and included in artifact cache keys.
- Registry publish/yank/owner/login flows are production-grade enough for controlled ecosystem usage.
- Architecture, roadmap, CLI docs, and phase checklist issues are updated with final status and merged PR links.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
