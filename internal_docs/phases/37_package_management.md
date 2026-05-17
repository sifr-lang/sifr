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

## Reuse Strategy

Package management is not implemented from scratch when mature, well-scoped Rust components already solve a generic subproblem. Sifr must reuse those components at the library boundary where the abstraction matches Sifr's source-first model, and must avoid embedding package managers whose semantics are for another language.

Required direct dependencies:
- `pubgrub = { package = "astral-pubgrub", version = "0.3.3" }` is the dependency solver. Sifr implements its own package ids, version type wrapper, dependency provider, priority policy, and error lowering around PubGrub. The crate is available from crates.io; any future move to a git or vendored source must pin an immutable revision and record the reason in architecture docs.
- `semver = { features = ["serde"] }` parses and orders package versions. Sifr wraps `semver::Version` and `semver::VersionReq` behind Sifr types so unsupported grammar, build metadata policy, pre-release policy, and published-package restrictions are enforced before solver input is built.
- `toml_edit` mutates `sifr.toml` while preserving comments and stable formatting for `sifr add`, `remove`, `update`, `init`, and workspace catalog edits.
- `petgraph` represents workspace graphs, resolved package graphs, dependency/dependent closure filters, cycle diagnostics, and package-boundary validation.
- `globset`, `ignore`, and `walkdir` perform deterministic workspace member expansion, package include/exclude walking, archive input discovery, and changed-file-to-package mapping.
- `gix` is the Git implementation for Git dependencies, locked revisions, registry index mirrors if needed, and changed-package selectors such as `[main...HEAD]`. Sifr must not mix `gix` and `git2` without a specific later design issue.
- `reqwest` or the selected `gix` HTTP transport implements sparse registry HTTP with conditional requests, retries, TLS validation, and credential redaction. The concrete client must be selected before `milestone_37_6` starts.
- `sha2` and `hex` implement package, manifest, source, lockfile, archive, and generated-backend checksums.
- `tar` and `zstd` implement deterministic `tar.zst` source package archives.
- Existing workspace dependencies such as `serde`, `toml`, `url`, `tempfile`, `thiserror`, `anyhow`, and `tokio` remain the base for serialization, URL validation, atomic writes, diagnostics, and async fetch/concurrency.

Reference implementations, not direct semantic dependencies:
- uv's `uv-resolver` is a high-quality reference for PubGrub integration, version maps, lockfile preferences, upgrade sets, yanked-version behavior, dependency-provider priorities, universal-resolution forks, and no-solution reporting. It is not reused directly because it is Python-specific: PEP 440/508, wheels, sdists, extras, groups, markers, interpreter constraints, and Python environment tags do not define Sifr packages.
- uv's `uv-workspace`, `uv-cache`, `uv-client`, and `uv-git` are references for workspace discovery errors, cache layout discipline, HTTP behavior, and Git edge cases. Sifr implements its own workspace/cache/source model because package roots, exports, lockfiles, and compiler integration are Sifr-specific.
- Cargo's internal resolver, registry traits, lockfile behavior, publish/yank rules, and resolver tests are references for semver compatibility, registry/provider boundaries, conflict cases, and native dependency integration. Sifr does not call Cargo's resolver for Sifr source dependencies because Cargo package ids, feature unification, multiple-version behavior, and crate metadata are Rust-specific.
- Cargo's `crates-io` and `cargo-platform` crates are not Sifr package-manager dependencies in Phase 37. Sifr owns its sparse registry client. Target predicates use Sifr-owned syntax and lower to Rust targets during generated Cargo materialization.
- Turborepo's package graph, filters, changed-package selection, graph utilities, boundary concepts, and hashing/cache discipline are monorepo design references. Sifr reuses generic crates such as `petgraph`, `globset`, and `gix`, not Turborepo's JS-package-specific crates.

The implementation must include a short dependency audit before `milestone_37_2`: licenses, maintenance status, public API stability, feature flags, transitive dependency risk, and whether each dependency is used in CLI, compiler, or registry paths.

## Correctness Test Suite Reuse Plan

Sifr must reuse the correctness lessons from Cargo and uv without depending on their unstable or language-specific test crates. Cargo's resolver tests are an oracle and edge-case corpus. uv's integration tests are a behavior matrix for lockfiles, workspaces, caches, registries, Git, auth, and snapshots. Both are ported into Sifr-owned fixtures, helpers, and property tests.

Upstream test material to reuse by porting:
- Cargo `crates/resolver-tests/tests/resolve.rs`: basic dependency resolution, transitive dependencies, same package name from different sources, dev dependencies, many versions, exact versions, maximal/minimal version ordering, incompatible versions, case/name normalization, backtracking, feature backtracking, deep traps, sys/links conflicts, incomplete information, missing packages, cycles, equality constraints, conflict cache regressions, missing features, cyclic error messages, and shortest conflict paths.
- Cargo `crates/resolver-tests/tests/pubgrub.rs`: renamed packages, rename shadowing, pre-release semver, cyclic features, cyclic optional dependencies, package cycles/self-dependencies, build/dev dependencies with the same name, weak dependency-feature references, duplicate sys crates, missing optional dependencies, feature shadowing, repeated dependency feature unification, implicit/default feature edge cases, and dependency feature references.
- Cargo `crates/resolver-tests/tests/validated.rs`: conflict-store regressions, bad-lockfile regressions, registry-with-features, missing feature/dep-feature/weak-feature diagnostics, feature-and-sys conflicts, multiple dependency kinds/targets, optional dependency features, optional dependency rename behavior, and default features across multiple major versions.
- Cargo `crates/resolver-tests/tests/proptests.rs` and `src/sat.rs`: generated-registry validation, SAT-oracle validation, minimal-version existence agreement, dependency-removal monotonicity, irrelevant-version removal, deterministic pretty-printing of minimized failures, and time-bounded property checks.
- uv `crates/uv/tests/it/lock.rs`, `lock_conflict.rs`, and resolver internals such as `preferences.rs`, `yanks.rs`, `upgrade.rs`, `version_map.rs`, `candidate_selector.rs`, and `error.rs`: lockfile snapshots, lock preferences, allowed yanks, upgrade sets, no-solution rendering, source metadata, locked/offline/frozen behavior, conflict declarations, marker/feature deduplication, and exponential lockfile growth guards.
- uv `workspace.rs`, `workspace_list.rs`, and `init.rs`: root and virtual workspaces, from-member discovery, excluded/non-included/hidden/gitignored members, empty and malformed members, workspace path dependencies, workspace lock idempotence from subdirectories, member names shadowing dependencies, path hopping, Git workspaces, complex relative paths, unmanaged members, and deterministic workspace-list output.
- uv `tree.rs`: normal and inverted trees, platform dependencies, repeated dependencies and repeated versions, dev and optional dependencies, cycles without infinite loops, workspace circular dependencies, frozen tree reads, and outdated reporting.
- uv `cache.rs`, `cache_clean.rs`, and `cache_prune.rs`: cache init failures, permission errors, stale directories, stale symlinks, stale revisions, package/index-specific clean behavior, verbatim path handling, and corruption recovery.
- uv `publish.rs` and `auth.rs`: missing/invalid/mixed credentials, token handling, trusted publishing permission failures, redirects, dubious filenames, dry-run reporting all publish errors, keyring/text/native auth behavior, prefix matching, host fallback, malformed helper responses, invalid URIs, and credential redaction.

Do not reuse directly:
- Cargo `resolver-tests` APIs, helper DSL, Cargo `Dependency` / `Summary` / `PackageId` types, or Cargo's internal resolver.
- uv's `uv_snapshot!` macro, Python interpreter/venv fixtures, wheel/sdist metadata fixtures, PyPI-specific snapshots, or uv resolver crates as implementation dependencies.
- uv resolver library implementation files such as `preferences.rs`, `version_map.rs`, `candidate_selector.rs`, and `error.rs` are reference patterns, not test ports. Port behavior from uv's `tests/it/` integration tests and encode Sifr-specific resolver checks in Sifr-owned tests.
- Cargo's multiple-version behavior where it conflicts with Sifr import-root semantics.
- Python-specific extras/groups/markers except as patterns that lower into Sifr features, target predicates, and workspace catalogs.

Sifr-owned correctness harnesses:
- Add a package-manager model test harness under the Phase 37 implementation crate, with Sifr-owned helpers such as `pkg`, `dep`, `feature`, `target_dep`, `workspace_member`, `registry_source`, `git_source`, `path_source`, and expected resolved graph snapshots.
- Add a fake sparse registry server that can serve versions, yanks, auth failures, malformed records, checksum mismatches, conditional request responses, redirects, and offline/cache-only metadata.
- Add temporary Git repository helpers using the selected `gix` stack for tags, branches, locked revisions, branch movement, subdirectories, workspaces inside Git dependencies, and changed-package selectors.
- Use `insta` snapshots with deterministic filters for temp paths, registry URLs, commit ids, checksums, elapsed time, and target triples.

Sifr SAT/metamorphic oracle:
- Implement a Sifr-specific SAT oracle inspired by Cargo's `SatResolver`. It validates the existence or non-existence of a solution and validates that a PubGrub result satisfies Sifr package semantics.
- The oracle must encode these constraints explicitly:
  1. At most one selected version per Sifr import root in one dependency scope unless the consuming manifest aliases a package to a distinct import root.
  2. At most one selected package per identical source identity: source kind, registry name, package name, and path/Git/URL identity.
  3. Workspace members are fixed local candidates at their manifest version and source identity.
  4. If a dependency edge is selected, exactly one package candidate satisfying its source, target, feature, and version requirement is selected.
  5. Non-optional dependencies are always selected when their owning package is selected and their target predicate is active.
  6. Optional dependencies are selected only through explicit feature activation semantics described below.
  7. If a package feature is active, its owning package is selected.
  8. If a dependency feature reference is active, the dependency package is selected and that dependency feature is active unless the reference is weak.
  9. Feature conflicts are pre-solver constraints: two conflicting feature atoms cannot both be active in the same solve.
  10. Target predicates are hard pre-filters for Phase 37: false-target dependencies and candidates never enter the active solver input.
  11. Yanked versions are excluded unless the version is already locked with matching checksum or explicitly pinned by an allowed future policy.
  12. Native-capable backend dependencies, build scripts, `links` crates, and proc macros require explicit root trust entries; every selected native-capable package without trust fails the oracle.
  13. Path, Git, URL, registry, and workspace sources remain distinct solver identities and preserve their source constraints in conflict paths.
  14. Structural workspace package cycles and non-dev package cycles are rejected before PubGrub and are not treated as valid SAT solutions.
  15. Direct-dependency import boundaries are validated over the resolved graph: transitive dependencies are compiled but not directly importable from a consuming package unless explicitly declared or re-exported.

Optional dependency activation semantics:
- Optional dependencies are declared with `optional = true` or by being attached only to a feature.
- `dep:name` activates optional dependency `name`.
- `name/feature` activates feature `feature` on dependency `name` and requires `name` to be non-optional or activated by `dep:name` in the same feature fixed point; otherwise Sifr emits `SIFR-PACKAGE-0103`.
- `name?/feature` is a weak dependency-feature reference. It activates `feature` only if dependency `name` is already active through another edge; otherwise it is a no-op.
- Feature expansion runs to a deterministic fixed point before PubGrub input is built. Cyclic feature aliases are rejected; cyclic optional dependency graphs are rejected unless expansion reaches a stable finite fixed point without introducing a package cycle.

Sifr versus Cargo multiple-version policy:
- Same source identity and semver-incompatible versions of the same package are a hard error.
- Different registries with the same package name and export root are a hard error unless the consuming manifest aliases one dependency to a distinct valid import root.
- Path versus registry, workspace versus registry, and Git versus registry instances of the same package/export root are hard errors unless explicitly aliased.
- Aliasing preserves package identity in diagnostics and generated Rust names but gives the consumer a distinct import root.
- Every Cargo resolver case that Cargo accepts through multiple semver-incompatible versions must have a Sifr negative test showing the intentional rejection, plus a positive alias test where Sifr intentionally accepts the graph.

Workspace catalog and selection tests:
- A workspace catalog entry with no consumers is allowed and does not enter the solve.
- A member using `{ workspace = true }` inherits version, default-features, features, target predicates, and source constraints from the catalog.
- A member that references a catalog entry without `{ workspace = true }` does not inherit it.
- A member may not silently override a catalog entry through `{ workspace = true }`; overrides require an explicit non-workspace dependency spec and must produce deterministic diagnostics when they conflict with other selected members.
- Workspace-level target dependency catalogs are tested with active and inactive targets.
- Selection tests cover `--workspace`, default members, `-p`, `--exclude`, `--filter pkg`, `{path/glob}`, `pkg...`, `...pkg`, `...^pkg`, `[base...head]`, empty selections, and global file changes that affect all members.

Resolution mode and command matrix:

| Command | Online | Offline | Locked | Frozen |
| --- | --- | --- | --- | --- |
| `sifr sync` | resolve, fetch, and write lock when needed | solve from cached metadata and fail on cache miss | validate lock and fail on required changes | locked plus offline cache enforcement |
| `sifr fetch` | download locked or newly resolved sources as allowed | download nothing and fail on missing cache entries | fetch only packages already present in the lock | locked plus offline cache enforcement |
| `sifr check` | resolve/fetch as needed, then type-check | solve from cache and fail on missing metadata/source | validate lock, then type-check | locked plus offline cache enforcement |
| `sifr build` | resolve/fetch as needed, then build | solve from cache and fail on missing metadata/source | validate lock and backend Cargo lock, then build | locked plus offline cache enforcement |
| `sifr run` | same as build, then execute | same as offline build, then execute | same as locked build, then execute | same as frozen build, then execute |
| `sifr test` | resolve selected workspace/package graph, then test | solve from cache and fail on missing metadata/source | validate lock for selected graph, then test | locked plus offline cache enforcement |
| `sifr tree` / `outdated` | read or refresh metadata according to command flags | use cached metadata only | read lock without mutation | read lock without mutation and no network |
| `sifr publish --dry-run` | validate package, archive, lock, and registry metadata without upload | fail if registry metadata is required and absent | validate lock before packaging | locked plus offline cache enforcement |

Mandatory property tests:
- Repeated resolution of the same input produces byte-identical `sifr.lock`, resolved graph JSON, and diagnostics.
- Solver results satisfy all active direct and transitive requirements.
- PubGrub success implies SAT oracle success; SAT oracle unsat implies PubGrub unsat for the supported Sifr model subset.
- Locked versions are preserved unless manifest, feature, target, source metadata, or update policy requires movement.
- `--locked` and `--frozen` never write manifests, lockfiles, caches, or generated Cargo state.
- Offline mode performs no network calls and fails on missing metadata or source archive cache entries.
- Removing inactive package versions does not make a previously satisfiable graph unsatisfiable.
- Removing dependency edges cannot make a satisfiable graph unsatisfiable.
- Feature activation is order-independent and idempotent.
- Optional dependency cycle expansion terminates in a deterministic bounded number of iterations derived from the feature/dependency graph's strongly connected components, or emits a stable package diagnostic before PubGrub runs.
- Target-false dependencies never appear in solver input or lockfile dependency edges for the active-target solve.
- Yanked versions are selected only through an existing valid lockfile or an explicitly pinned future policy.
- Diagnostic conflict paths are deterministic and shortest according to package-edge count, then lexical tie-breakers.

Mandatory integration categories:
- Lockfile round-trip, schema version rejection, manifest digest/staleness, checksum mismatch, exact requirement-string preservation, and stable ordering.
- Registry behavior: sparse-index metadata, yanks, malformed records, auth failures, alternate registries, registry priority, redirects, HTTPS enforcement, and credential redaction.
- Git behavior: locked commit remains stable after branch movement, branch/tag specs lock to a commit, short revs normalize, subdirectories resolve, Git workspaces resolve, and changed-package selectors use Git ranges deterministically.
- Package graph cycles: direct package cycles, self-dependencies, optional dependency cycles, workspace member cycles, dev-dependency cycles, and cycle diagnostics.
- Source/package import integration: dependency source compiles through frontend/HIR/codegen, direct-dependency-only imports, private module rejection, explicit re-export validation, export-root ambiguity, aliasing, and PackageSourceMap stability.
- Generated Cargo backend integration: backend dependency collection, native trust gates, `links` conflicts, generated `Cargo.lock` verification, backend lock drift under `--locked`, and cache invalidation.
- CLI mutation: `sifr add/remove/update` TOML preservation, atomic write failure recovery, dry-run manifest/lock diffs, workspace package targeting, and idempotent repeated commands.
- Cache and archive: cache init failure, corruption recovery, stale staged directory cleanup, deterministic `tar.zst` archive entries, path traversal rejection, symlink policy, and cache prune behavior once prune exists.
- Publish/auth: missing metadata, duplicate immutable version, unauthorized namespace, invalid token, mixed credentials, no credentials, dry-run reporting all errors, suspicious filenames/paths, namespace ownership, token storage, token redaction, and helper/auth malformed responses.
- Tree/outdated: normal and inverted tree, repeated dependency display, cycles without infinite loops, dev/optional/target dependency views, workspace selection, frozen mode, and outdated compatible/incompatible reporting.

Traceability and upstream update process:
- Maintain a Phase 37 traceability matrix mapping each borrowed Cargo/uv category to Sifr test files, milestone, diagnostic code, expected behavior, and intentional divergence.
- When upgrading `astral-pubgrub`, `semver`, `gix`, registry/HTTP dependencies, archive/checksum dependencies, or solver-adjacent code, the implementation must:
  1. Run the full Sifr SAT oracle suite.
  2. Run property tests with recorded seeds for any failure.
  3. Review upstream Cargo resolver-tests for new semver, feature, backtracking, lock, registry, or conflict cases and port relevant categories.
  4. Review upstream uv lock/workspace/tree/cache/auth/publish tests for new behavior categories and port relevant categories.
  5. Update the traceability matrix with new test files or explicit non-port decisions.
  6. Run the repository validation gate before merging.

## Maintainability Architecture

Phase 37 introduces one package-manager crate first: `crates/sifr_package`. This keeps implementation simple while giving the package manager a hard ownership boundary separate from the CLI, driver, frontend, HIR, and generated Cargo materialization code. The crate must be decomposed into focused modules with public facade types so it can later split into multiple crates only if compile time, ownership, or API pressure justifies it.

Core maintainability rules:
- `crates/sifr` remains a thin CLI parser/renderer layer. It does not resolve packages directly, mutate TOML directly, fetch registries, call Git, or inspect package cache internals.
- `sifr_driver` consumes package operation plans and generated Cargo backend plans. It does not know registry HTTP, Git, sparse-index, or archive extraction details.
- `sifr_frontend` and `sifr_hir` consume immutable `PackageSourceMap` / package-origin data for import closure, diagnostics, and analysis. They do not depend on registry, cache, Git, TOML mutation, or lockfile writer modules.
- The package manager exposes Sifr-owned domain types: `SifrVersion`, `SifrVersionReq`, `PackageName`, `FeatureName`, `Checksum`, `PackageId`, `PackageIdentity`, `PackageSource`, `DependencySpec`, `ManifestGraph`, `FeaturePlan`, `SolverInput`, `ResolvedPackageGraph`, `SifrLock`, `NativeCapability`, `PackageSourceMap`, `CargoBackendPlan`, `CargoLockDiff`, and `OperationPlan`.
- External package-manager/library types must not cross the public boundary of `sifr_package`. `Path`, `Url`, strings, byte buffers, and Sifr-owned newtypes are acceptable boundary types; `semver::Version`, PubGrub state/types, `gix` repository types, `toml_edit` AST nodes, `petgraph` graph types, `reqwest` responses, and archive implementation types are not.

Anti-corruption adapter modules:
- `version::semver_adapter` is the only module that imports `semver`.
- `solver::pubgrub_adapter` is the only module that imports `pubgrub`.
- `manifest::edit` is the only module that imports `toml_edit`.
- `workspace::graph` / `graph::petgraph_adapter` are the only modules that import `petgraph`.
- `sources::git::gix_adapter` and `workspace::changed` are the only modules that import `gix`.
- `sources::registry::http_adapter` is the only module that imports the selected HTTP client.
- `archive::tar_zstd_adapter` is the only module that imports `tar` and `zstd`.
- `checksum::sha2_adapter` is the only module that imports `sha2` and checksum formatting helpers.

Pure core versus IO:
- Pure deterministic core includes manifest model validation, version requirements, feature fixed-point expansion, solver input construction, lockfile staleness checks, workspace selection, conflict path lowering, `PackageSourceMap` construction, trust validation, and `CargoBackendPlan` construction.
- IO adapters include registry HTTP, Git fetch, source cache writes, archive extraction/creation, manifest mutation writes, lockfile writes, credential storage, and generated Cargo materialization.
- The pure core must be testable without network, filesystem mutation, Git repositories, registry servers, or generated Cargo directories.

Module map:

```text
crates/sifr_package/src/
  lib.rs
  manifest/{model,parse,validate,edit}.rs
  workspace/{discover,catalog,select,filters,changed,graph}.rs
  version/{model,req,semver_adapter}.rs
  features/{model,expand,conflicts}.rs
  solver/{input,provider,pubgrub_adapter,preferences,conflict}.rs
  lockfile/{model,read,write,stale}.rs
  sources/{model,registry,git,path,url,cache}.rs
  archive/{pack,unpack,verify,tar_zstd_adapter}.rs
  checksum/{model,sha2_adapter}.rs
  trust/{model,validate}.rs
  imports/{source_map,boundaries}.rs
  backend/{cargo_plan,cargo_lock_verify,trust_bridge}.rs
  ops/{plan,mutate,resolve,read,publish}.rs
  diag/{origins,redaction}.rs
  test_support/   # cfg(test) helpers only
  test_assets/    # fixture packages, sparse registry fixtures, Git fixture templates
```

`diag` constructs `sifr_diagnostics::SifrDiagnostic` values and uses the existing diagnostic renderers. `SIFR-PACKAGE-*` codes are added to `sifr_diagnostics::codes` during `milestone_37_1`; `sifr_package` must not create a parallel diagnostic registry, renderer, JSON schema, or redaction pipeline.

`ops` is grouped by mutation class rather than by every command name:
- `ops::mutate` plans manifest-changing commands such as `add`, `remove`, and dependency-spec `update`.
- `ops::resolve` plans `sync`, `fetch`, lock validation, cache fill, and source availability work.
- `ops::read` plans `tree`, `outdated`, dry-run summaries, and graph-only queries.
- `ops::publish` plans package/archive/registry/auth operations.
- `ops::plan` defines shared plan types and execution boundaries.

Operation planning:

```rust
struct OperationPlan {
    selected_packages: Vec<PackageId>,
    manifest_edits: Vec<ManifestEdit>,
    lockfile_writes: Vec<LockfileWrite>,
    fetch_actions: Vec<FetchAction>,
    cache_actions: Vec<CacheAction>,
    generated_cargo_changes: Vec<CargoEdit>,
    diagnostics: Vec<SifrDiagnostic>,
}

enum ManifestEdit {
    AddDependency { manifest_path: PathBuf, alias: DependencyAlias, spec: DependencySpec },
    RemoveDependency { manifest_path: PathBuf, alias: DependencyAlias },
    UpdateDependency { manifest_path: PathBuf, alias: DependencyAlias, spec: DependencySpec },
}

enum CacheAction {
    Verify { package_id: PackageId, expected_checksum: Checksum },
    Extract { package_id: PackageId, archive: PathBuf, destination: PathBuf },
    Skip { package_id: PackageId, reason: CacheSkipReason },
}
```

Every mutating command must produce an `OperationPlan` before it writes manifests, lockfiles, package caches, credentials, archives, or generated Cargo directories. `--dry-run` and `--dry-run=json` render the same plan without executing it. Editor/LSP paths may request read-only plans for diagnostics and graph previews, but they must not execute mutating plan actions.

Integration with compiler and driver:
- `PackageSourceMap` is produced by `imports::source_map` from `ResolvedPackageGraph` and passed into package-aware project assembly before import-closure discovery. It augments existing workspace source-root resolution; it does not replace embedded stdlib precedence or local package source roots.
- `sifr_frontend` receives package origins as part of its project query input so editor diagnostics, completion, rename, references, and generated Rust preview use the same package graph as CLI builds.
- `sifr_hir` receives package-origin metadata only for diagnostics, privacy/import-boundary checks, and source mapping. It does not fetch packages or read manifests.
- `CargoBackendPlan` is produced by `backend::cargo_plan`, consumed by `sifr_driver` generated Cargo materialization, and verified after Cargo resolution.

Generated Cargo verification:
- `backend::cargo_lock_verify` produces a structured `CargoLockDiff`.
- Critical diffs fail under package mode: package name, version, source, checksum, activated features, `links`, build script/proc-macro/native capability, or dependency edge changes.
- Non-critical diffs such as ordering-only changes are normalized before comparison and must not produce false failures.
- Any diff rendered to the user must map back to the Sifr package or backend dependency that introduced it.

```rust
struct CargoLockDiff {
    critical: Vec<CriticalDiff>,
    normalized: Vec<NormalizedDiff>,
}

enum CriticalDiff {
    PackageNameMismatch { expected: PackageName, actual: PackageName },
    VersionMismatch { expected: SifrVersion, actual: SifrVersion },
    SourceMismatch { expected: PackageSource, actual: PackageSource },
    ChecksumMismatch { expected: Checksum, actual: Checksum },
    FeaturesMismatch {
        package: PackageId,
        expected: BTreeSet<FeatureName>,
        actual: BTreeSet<FeatureName>,
    },
    LinksMismatch { expected: Option<String>, actual: Option<String> },
    BuildScriptMismatch { package: PackageId, expected: bool, actual: bool },
    ProcMacroMismatch { package: PackageId, expected: bool, actual: bool },
    NativeCapabilityMismatch {
        package: PackageId,
        expected: NativeCapability,
        actual: NativeCapability,
    },
    DependencyEdgeAdded { from: PackageId, to: PackageId },
    DependencyEdgeRemoved { from: PackageId, to: PackageId },
}

enum NormalizedDiff {
    OrderingOnly { reason: &'static str },
    TimestampOnly,
    OptionalMetadataOnly { field: &'static str },
}
```

`trust::validate` owns Sifr package trust policy evaluation. `backend::trust_bridge` is only the adapter that translates reachable backend Cargo requirements into trust validation inputs and maps trust diagnostics back to the Sifr package or stdlib/codegen component that introduced the backend dependency.

Feature expansion termination:
- Feature expansion has an explicit bound derived from package count, feature count, and dependency-feature references in the selected graph.
- If fixed-point expansion does not stabilize within that bound, Sifr emits `SIFR-PACKAGE-0103` before PubGrub runs.
- The bound and the feature graph strongly connected components are included in debug/test traces for minimization.

Dependency audit and adapter contracts:
- `crates/sifr_package/DEPENDENCY_AUDIT.md` records each external crate, pinned version/source, owning adapter module, enabled features, public API stability, license, maintenance risk, adapter contract tests, and intentional behavior differences from upstream.
- `crates/sifr_package/TRACEABILITY.md` maps Cargo/uv correctness categories to Sifr test files, milestones, diagnostics, and non-port decisions.
- `crates/sifr_package/FEATURES.md` maps Sifr package-manager feature flags to external crate feature flags and explains why each is enabled.
- If `Cargo.lock` changes for a `sifr_package` dependency, local validation must fail until `DEPENDENCY_AUDIT.md` is updated or the change is explicitly marked unrelated.
- Before `milestone_37_5`, the implementation must open or complete a small design issue for `gix` runtime integration: blocking versus async calls, cancellation behavior, credential redaction, and Git range handling for changed-package filters.

`TRACEABILITY.md` uses a stable table schema: borrowed category, upstream source/test reference, Sifr test file, milestone, expected diagnostic code, expected behavior, intentional divergence, and non-port decision. Empty divergence/non-port fields mean the upstream behavior is expected to be preserved in Sifr's model.

Maintainability guardrails:
- Add `scripts/check_package_manager_guardrails.py` and run it from local validation once `crates/sifr_package` exists.
- The guardrail enforces file-size limits for package-manager modules, with no package-manager source file over 600 lines and stricter limits for adapter modules.
- The guardrail enforces dependency boundaries: each external crate appears only in its approved adapter module subtree.
- The guardrail rejects public `pub use`, public fields, or public function signatures that expose external crate types from `sifr_package`.
- The guardrail verifies mutating CLI command paths call into `OperationPlan` planning before execution.
- The guardrail verifies `TRACEABILITY.md`, `DEPENDENCY_AUDIT.md`, and `FEATURES.md` exist and are updated when package-manager dependency, adapter, or correctness-fixture files change.
- Code review is supplementary; automated guardrails are the source of truth for module-boundary enforcement.

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
- Target predicates use a Sifr-owned parser for a Cargo-compatible `cfg(...)` subset: `unix`, `windows`, `target_os`, `target_arch`, `target_env`, `target_vendor`, `target_family`, `target_pointer_width`, `all(...)`, `any(...)`, and `not(...)`. Unsupported keys or malformed expressions are package diagnostics. The parsed predicate lowers to Rust target triples during backend materialization, but Sifr does not depend on Cargo's internal platform parser in Phase 37.

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

## Resolver Architecture

Version resolution is an explicit part of the package manager model. Sifr does not hand this problem to Cargo or uv; it reuses PubGrub directly with Sifr-owned package metadata.

Resolution pipeline:

```text
ManifestGraph
  -> FeaturePlan
  -> SolverInput
  -> PubGrubDependencyProvider
  -> ResolvedPackageGraph
  -> LockPlan
  -> SourceCachePlan
  -> PackageSourceMap
  -> ModuleResolver
  -> CargoBackendPlan
```

Core types:

```rust
struct SolverInput {
    roots: Vec<RootRequirement>,
    workspace_members: BTreeMap<PackageName, WorkspaceMember>,
    registries: BTreeMap<RegistryName, RegistryConfig>,
    feature_plan: FeaturePlan,
    target: TargetSelection,
    preferences: ResolutionPreferences,
    mode: ResolutionMode,
}

struct FeaturePlan {
    package_features: BTreeMap<PackageName, BTreeSet<FeatureName>>,
    conflicts: Vec<FeatureConflict>,
}

enum ResolutionMode {
    Online,
    Offline { metadata_cache_only: bool },
    Locked { lockfile: SifrLock },
    Frozen { lockfile: SifrLock },
}

struct ResolutionPreferences {
    locked_versions: BTreeMap<PackageIdentity, Version>,
    upgrade: UpgradePolicy,
}

enum UpgradePolicy {
    PreserveLocked,
    UpdateDirect(BTreeSet<DependencyAlias>),
    UpdatePackageAndAffected { package: DependencyAlias, recursive: bool },
    UpdateWorkspace,
}

struct ResolvedPackageGraph {
    packages: BTreeMap<PackageId, ResolvedPackage>,
    dependency_edges: Vec<ResolvedDependencyEdge>,
    activated_features: BTreeMap<PackageId, BTreeSet<FeatureName>>,
    target: TargetSelection,
    backend_cargo: CargoBackendPlan,
}
```

PubGrub provider requirements:
- Package identities include source kind, registry name, package name, and workspace/path/git/url identity where applicable. Two packages with the same display name but different source identities are distinct solver packages until export-root validation rejects conflicting public imports.
- Available versions come from workspace members, path manifests, locked Git/URL metadata, cached sparse-index records, or online sparse-index records depending on `ResolutionMode`.
- Candidate ordering is deterministic: prefer locked versions that still satisfy all constraints unless the package is in the upgrade set; otherwise prefer the newest non-yanked compatible version; ties break by package id string and source priority.
- Locked versions are preferences, not extra manifest constraints. If a locked version no longer satisfies the active manifest graph, normal resolution selects a compatible replacement unless `--locked` or `--frozen` is active.
- Yanked versions are excluded from new resolution. Existing lockfiles may continue using yanked versions only when the lockfile checksum and package source checksum still match. If a requirement can be satisfied only by yanked versions during new resolution, resolution fails with `SIFR-PACKAGE-0104` and reports the closest non-yanked incompatible versions if known.
- No-solution errors must be lowered into a structured conflict path before rendering.

Feature resolution:
- Sifr uses additive Cargo-like feature union.
- Phase 37 uses pre-expanded feature edges, not PubGrub virtual feature packages. The manifest graph is first expanded into a `FeaturePlan`; unknown features, cyclic feature aliases, and declared feature conflicts fail before PubGrub runs.
- Feature-expanded optional dependencies become ordinary solver requirements with origin metadata pointing back to the feature that activated them.
- Mutually exclusive features are checked as a Sifr diagnostic, not by relying on PubGrub to discover the conflict indirectly.

Target-specific resolution:
- Normal Phase 37 commands solve for one active target. The active target is the explicit CLI target when present, otherwise the configured default target, otherwise the host target.
- Target predicates are evaluated before solver input is built. Dependencies whose target predicate is false for the active target are not part of the solve.
- The lockfile records the target selection and any target predicates that affected the graph.
- Universal multi-target locks are not a separate semantic path. If added later, they must run the same solver once per declared target environment and merge compatible results using uv-style forked-resolution ideas without changing package import semantics.

Workspace member resolution:
- Workspace members are source packages with path identities and fixed local manifests.
- A member participates in the solve only when selected as a root or when another selected package declares it through `{ workspace = true }`, `{ path = "..." }`, or an equivalent explicit dependency.
- Workspace catalogs contribute requirements and defaults only when a member opts into them. They are never implicit imports and never implicit package roots.
- Workspace member cycles are rejected before PubGrub when they are structural package cycles, and solver conflict paths report workspace edges using member manifest paths.

Resolution modes:
- Online mode may refresh registry metadata, resolve missing versions, fetch source archives, and update `sifr.lock`.
- Offline mode may solve using cached index metadata and cached Git/URL/path metadata only. If required metadata or source archives are missing, it fails with `SIFR-PACKAGE-0203`.
- Locked mode does not perform a fresh solve that changes the graph. It validates that the existing `sifr.lock` exactly satisfies the active manifest graph, selected features, target predicates, backend Cargo requirements, and package checksums. Any required change fails with `SIFR-PACKAGE-0201` or `SIFR-PACKAGE-0202`.
- Frozen mode is locked mode plus offline cache enforcement.

Upgrade policy:
- `sifr update` updates all direct dependencies in the selected package or workspace to the newest compatible non-yanked versions, then resolves affected transitive dependencies as needed.
- `sifr update name` updates the named direct dependency and transitive dependencies whose locked versions are no longer preferred or compatible after that change.
- `sifr update name --recursive` additionally allows packages depending on `name` to move when needed to maintain a coherent graph.
- `sifr update --workspace` applies the same policy across selected workspace members while respecting workspace catalogs.
- `sifr add`, `remove`, feature changes, target changes, and workspace catalog changes create a new `FeaturePlan` and `SolverInput`; lockfile preferences still minimize unrelated churn.

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

[[package]]
id = "git+https://github.com/sifr-lang/math.git?rev=abc123#math@0.4.0"
name = "math"
version = "0.4.0"
source = "git+https://github.com/sifr-lang/math.git"
rev = "abc123"
checksum = "sha256:..."
source-roots = ["src"]
exports = ["math"]

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

The resolved graph must also produce the compiler-facing package source map:

```rust
struct PackageSourceMap {
    providers: BTreeMap<ImportRoot, PackageSourceProvider>,
    package_roots: BTreeMap<PackageId, Vec<PackageSourceRoot>>,
    dependency_scopes: BTreeMap<PackageId, BTreeSet<PackageId>>,
}

struct PackageSourceProvider {
    package_id: PackageId,
    origin: ModuleOrigin,
    public_exports: BTreeSet<ImportRoot>,
}
```

`providers` is used for fast import-root lookup and ambiguity diagnostics. `dependency_scopes` records which direct dependencies are importable from each package; transitive dependencies remain available only while compiling their own modules or validating explicit re-exports.

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

Selector semantics are adapted from Turborepo but implemented over Sifr's package graph:
- `pkg` selects one package by name or alias.
- `{path/glob}` selects packages whose root directories match the glob.
- `pkg...` selects `pkg` plus dependency closure.
- `...pkg` selects `pkg` plus dependent closure.
- `...^pkg` selects dependents of `pkg` but not `pkg`.
- `[base...head]` asks `gix` for changed files and maps them to owning package roots. Global files such as the root lockfile, root manifest catalogs, compiler config, registry config, or trust config select every affected package, not an arbitrary single owner.
- Include filters are unioned, exclude filters subtract, and an empty result is a package diagnostic rather than a silent no-op unless a future command explicitly documents empty selection as success.

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

Manifest mutation:
- `toml_edit` is required for mutating `sifr.toml`; commands preserve comments and user grouping where possible while normalizing only the tables they edit.
- Writes are atomic: write to a same-directory temporary file, fsync where supported, then rename. A failed write must leave the original manifest intact.
- `sifr add/remove/update --dry-run` must compute and report the TOML edit plan without touching the manifest or lockfile.

Dry-run JSON:
- `--dry-run=json` emits stable JSON containing selected packages, selected targets, manifest edits, lockfile edits, package graph edges, activated features, package cache actions, registry/Git fetch actions, generated Cargo changes, diagnostics, and command execution plan.
- Package diagnostics in dry-run JSON use the same canonical diagnostic schema as compiler diagnostics, including `SIFR-PACKAGE-*` code, severity, origin, conflict path, and remediation.
- JSON fields are additive only after Phase 37; removing or renaming fields requires a later schema-version bump.

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

Namespace and ownership:
- Registry namespaces such as `namespace/name` are owned registry resources.
- Publishing under a namespace requires authenticated ownership or delegated package publish rights from the registry.
- Namespace creation, transfer, and owner management must be explicit registry API operations; package publish must not implicitly grant namespace ownership.
- Duplicate immutable versions, unauthorized namespaces, and ambiguous owner state fail before archive upload.

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
- `sifr login` stores registry tokens outside manifests and lockfiles. Registry requests attach bearer tokens only for endpoints that require authentication; token expiry or revocation fails with a registry diagnostic that redacts the token and registry secret material.
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

Version conflict diagnostics must include a structured conflict path:

```rust
struct ConflictPath {
    root: PackageId,
    steps: Vec<ConflictStep>,
    unsatisfied: UnsatisfiedRequirement,
}

struct ConflictStep {
    package: PackageId,
    required_by: Option<PackageId>,
    requirement: String,
    source: DependencySource,
    feature: Option<FeatureName>,
    target: Option<TargetSelection>,
}
```

Human, compact, and JSON renderers all consume this same structure. Human output may summarize the path, but JSON output must preserve the full path.

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
- Are regenerated from `CargoBackendPlan`, not handwritten or mutated in place.
- Record stdlib/codegen backend dependencies in the backend lock section because compiler/runtime changes can alter native dependency resolution.
- Verify generated `Cargo.lock` by comparing package names, versions, sources, checksums, and activated features against the backend section of `sifr.lock`; a digest-only mismatch is not enough for user diagnostics.
- Live under the generated artifact cache root, separate from the package source cache. The existing content-addressed system-temp cache discipline for `sifr run` / `sifr test` remains valid unless a later architecture update moves generated artifacts into a configured Sifr cache directory.

`CargoBackendPlan` is derived from the resolved package graph by collecting backend Cargo requirements from embedded stdlib/codegen metadata, selected Sifr packages, and active package features. It is sorted deterministically, trust-checked, written into `sifr.lock`, materialized as generated `Cargo.toml`, and then verified against generated `Cargo.lock`.

Cache invalidation and recovery:
- Corrupt package-cache entries are deleted only after checksum verification fails and the diagnostic is emitted; `sifr sync` may refetch them when network access is allowed.
- Artifact-cache entries are disposable and may be evicted by size, age, or schema version without affecting reproducibility.
- Package-cache entries are content-addressed and are not evicted automatically while referenced by any lockfile known to the workspace unless a future cache-prune command explicitly does so.
- Cache summaries in `--dry-run=json` must expose cache hit/miss/corruption reasons without leaking credentials or absolute registry tokens.

Concurrency:
- Manifest and lockfile mutations take an exclusive workspace lock file under the workspace root before reading and writing package state.
- Package cache writes are staged in unique temporary directories and made visible only by atomic rename after checksum verification.
- Concurrent readers may use existing verified cache entries, but they must not observe partially extracted archives, partially written lockfiles, or generated Cargo directories in progress.
- IDE/editor package graph refreshes share the same read-only resolver/cache APIs and must back off when a mutating CLI command owns the workspace lock.

## Milestones

### milestone_37_1: Manifest, Workspace, And Dependency Schema
- Scope:
  - Add `crates/sifr_package` with the documented maintainability module map, public facade types, adapter boundaries, and no external crate types in public APIs.
  - Extend manifest parsing with package metadata, dependency specs, source types, features, target dependencies, workspace inheritance, workspace members/default-members, exports, backend Cargo dependencies, and trust metadata.
  - Add deterministic schema diagnostics for invalid tables/keys/values.
  - Add workspace member discovery and package selection planning.
  - Add the Phase 37 dependency audit for `pubgrub`, `semver`, `toml_edit`, `petgraph`, `globset`, `ignore`, `gix`, HTTP, checksum, archive, and async dependencies.
  - Add `crates/sifr_package/FEATURES.md` with initial entries mapping each enabled external crate feature flag to its corresponding Sifr package-manager capability.
  - Add `SIFR-PACKAGE-*` diagnostic code definitions to `sifr_diagnostics`.
- Definition of done:
  - Manifest and workspace graphs parse deterministically for root, virtual, and member workspaces.
  - Invalid dependency specs, invalid export roots, invalid workspace globs, package cycles, and invalid backend/native trust config produce stable package diagnostics.
  - Existing non-package workspace behavior remains compatible where manifests do not use Phase 37 package tables.
  - Dependency audit records license, maintenance status, public API stability, feature flags, transitive dependency risk, runtime path exposure, selected version, and fallback decision if a dependency fails the audit.
  - `crates/sifr_package/DEPENDENCY_AUDIT.md`, `TRACEABILITY.md`, and `FEATURES.md` exist with initial entries for implemented modules.

### milestone_37_2: Resolver, Lockfile, And Source Cache
- Scope:
  - Implement Sifr source dependency solving through `astral-pubgrub`, including `SolverInput`, `FeaturePlan`, `ResolutionMode`, `ResolutionPreferences`, `ResolvedPackageGraph`, deterministic candidate ordering, lockfile preferences, and structured conflict paths.
  - Implement pre-expanded feature resolution, target predicate selection, `sifr.lock`, staleness detection, content checksums, and a content-addressed package cache.
  - Implement the Sifr SAT/metamorphic oracle and port the Cargo resolver matrix categories that apply to Sifr's source package model.
  - Implement package-manager adapter contract tests for `semver`, PubGrub, checksum, lockfile IO, and source-cache boundaries.
  - Support registry, Git, URL, and path source records at the model level.
  - Implement `sifr sync`, `sifr fetch`, `--locked`, `--offline`, and `--frozen`.
- Definition of done:
  - Builds are reproducible from `sifr.toml`, `sifr.lock`, and the package cache.
  - Lockfile drift, checksum mismatch, missing cache entries, yanked packages, unsupported protocols, and version conflicts fail with deterministic diagnostics.
  - Lockfile output is stable across repeated runs.

### milestone_37_3: Package-Aware Import Resolution
- Scope:
  - Replace deferred package-directory behavior with `__init__.sifr`, explicit re-exports, public/private package APIs, and package-aware `ModuleOrigin`.
  - Build and consume `PackageSourceMap` from the resolved package graph.
  - Integrate `PackageSourceMap` with `sifr_frontend` project query input and `sifr_driver` package-aware project assembly.
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
  - Implement `CargoLockDiff` with critical versus normalized non-critical difference categories.
  - Gate native/build-script behavior through trust metadata.
  - Extend artifact cache keys and generated Rust preview metadata.
- Definition of done:
  - Generated Cargo manifests and locks are deterministic and verified against `sifr.lock`.
  - Native dependency changes invalidate caches predictably.
  - Untrusted native/build-script dependencies fail closed with actionable diagnostics.

### milestone_37_5: Workspace CLI, Filters, And Tooling Integration
- Scope:
  - Implement `sifr init`, `add`, `remove`, `update`, `tree`, `outdated`, `vendor`, package-aware `check/build/run/test`, workspace selectors, filters, dry-run/json summaries, and TOML mutation.
  - Route mutating commands through `OperationPlan` planning before execution.
  - Implement Turborepo-inspired package filters over Sifr's `petgraph` package graph and `gix` changed-file detection.
  - Complete the `gix` runtime integration design issue for blocking/async behavior, cancellation, credential redaction, and Git range handling.
  - Port uv workspace, tree, lock-mode, and CLI snapshot behavior categories into Sifr-owned integration tests.
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
  - Add `scripts/check_package_manager_guardrails.py` for package manager module decomposition, adapter boundaries, public API leakage, operation planning, and traceability/audit files.
  - Complete the Cargo/uv correctness traceability matrix and document every intentional divergence from Cargo or uv behavior.
  - Complete `crates/sifr_package/FEATURES.md` with the full feature flag map, rationale for each enabled flag, and documentation of disabled flags that were considered.
- Definition of done:
  - `scripts/run_all_tests.sh --profile quick` and full local validation pass.
  - Package workflows are stable enough for broader ecosystem usage.
  - Validation evidence is recorded in the phase execution checklist issue with fixture names, commands, expected diagnostics, and merged PR links.
  - Package manager maintainability guardrails pass locally and are part of the authoritative validation gate.

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
  - Maintainability: `sifr_package` public APIs expose only Sifr-owned types; adapter-only external crate usage is enforced for implemented modules.
- `milestone_37_2` (Resolver, Lockfile, And Source Cache):
  - Positive: resolve registry/path/Git graph, write stable `sifr.lock`, fetch into cache, rebuild without lockfile churn, build offline after fetch.
  - Negative: stale lock under `--locked`, cache miss under `--offline`, checksum mismatch, yanked version on new resolution, unsupported protocol, version conflict with clear conflict path.
  - Oracle: port Cargo resolver matrix cases for semver/pre-release/backtracking/features/cycles/source identity, add Sifr multiple-version divergence tests, and pass Sifr SAT/metamorphic property checks.
  - Maintainability: adapter contract tests cover semver/PubGrub/checksum/lockfile/source-cache wrappers and fail if external types leak.
- `milestone_37_3` (Package-Aware Import Resolution):
  - Positive: import package modules, import re-exported symbols from `__init__.sifr`, compile transitive dependency source, support workspace member dependency import.
  - Negative: undeclared dependency import, ambiguous export root, local/dependency shadowing, private module access, missing `__init__.sifr`, package directory/file collision.
  - Maintainability: frontend/HIR package integration consumes immutable package-origin inputs only and does not import registry/cache/Git/lockfile writer modules.
- `milestone_37_4` (Cargo Backend And Native Dependency Bridge):
  - Positive: generated Cargo manifest includes reachable backend crates from stdlib and package metadata, generated Cargo lock verifies against `sifr.lock`, native cache invalidates on backend feature change.
  - Negative: untrusted build-script dependency, backend lock drift under `--locked`, unsupported native target, conflicting backend feature requirements.
  - Maintainability: `CargoLockDiff` distinguishes critical semantic drift from normalized ordering-only changes.
- `milestone_37_5` (Workspace CLI, Filters, And Tooling Integration):
  - Positive: `sifr add/remove/update` mutate selected member manifests, `--workspace` builds all members, `-p` builds one member, filters select dependencies/dependents/changed packages, dry-run/json reports planned changes.
  - Negative: unknown package selector, filter selecting no packages, import outside package root, dependency used but not declared, tool/editor query using stale package graph.
  - Upstream reuse: port uv workspace/list/tree/init lock-idempotence categories into Sifr snapshot tests with Sifr package manifests and source packages.
  - Maintainability: mutating CLI paths produce and snapshot `OperationPlan` before execution; dry-run and execution use the same plan.
- `milestone_37_6` (Registry, Publish, Yank, And Credential Flows):
  - Positive: package dry-run archive is deterministic, publish uploads immutable package, yank hides package from new resolution, alternate registry resolves, credential redaction is verified.
  - Negative: publish missing metadata, duplicate immutable version, unauthorized namespace, insecure registry URL, archive path traversal, credential leak attempt, yanked package selected without existing lockfile.
- `milestone_37_7` (Validation, Documentation, And Ecosystem Gate):
  - Positive: docs cover app/library/workspace/package-author flows; package fixtures cover single package, monorepo, registry, Git, path, features, native deps, publish, and cache workflows.
  - Negative: maintainability guardrail rejects monolithic package manager files; docs/fixture drift checks catch undocumented CLI flags or diagnostics.
  - Traceability: Cargo resolver-tests and uv integration-test categories are mapped to Sifr tests, intentional divergences, or explicit non-port decisions.
  - Maintainability: package manager guardrail rejects oversized files, adapter boundary violations, external public API leaks, missing audit/traceability files, and mutation paths that bypass `OperationPlan`.
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
