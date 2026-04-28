# Ad-hoc Phase: Sifr Workspace Resolution Via `sifr.toml` (2026-04-25)

Status: closed
Owner: ad_hoc_sifr_workspace_sifr_toml_import_resolution
Source issue: `issues/sifr-workspace-sifr-toml-import-resolution-2026-04-25.md`
Execution checklist: `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25-execution.md`
Review status: pass-4 READY (`reviews/sifr-workspace-sifr-toml-import-resolution-2026-04-25-review-pass4.md`); WS6 local gate passed on 2026-04-25.

## Purpose

Add a first-class Sifr workspace concept discovered from native `sifr.toml` so non-`main.sifr` entrypoints and nested fixture/example trees can import shared user modules without promoting those helpers into the embedded `sifr.*` stdlib registry.

This phase closes the import-resolution gap that currently forces LeetCode fixtures and future user libraries into one of three bad choices:

- duplicate helper code in every fixture,
- rename entries to `main.sifr` and keep all helpers as direct siblings,
- or add fixture/user helpers to the production stdlib registry.

The target behavior is intentionally small: the nearest ancestor `sifr.toml` defines the workspace root, and user imports resolve from the entry directory first, then from configured `[source].roots`. Python `pyproject.toml` / `[tool.sifr]` compatibility is deferred; this phase implements the native Sifr manifest directly.

## Problem Statement

Current user-module discovery is entry-parent-only:

- `crates/sifr/src/main.rs::resolve_compilation_mode` enters project mode only for `main.sifr` files with local sibling imports.
- `crates/sifr/src/main.rs::has_local_project_imports` only detects direct sibling modules.
- `crates/sifr_driver/src/project/discovery.rs::module_source_path` maps every dependency to `<entry.parent>/<module>.sifr`.
- `crates/sifr_driver/src/stdlib/registry.rs` remains the only globally available import surface through embedded `sifr.*` and `_sifr.*` modules.

That makes `audits/leetcode/<problem>.sifr` entries single-file by default and blocks shared helpers such as `audits/leetcode/helpers/list_node.sifr`.

## Goals

- Discover a workspace root from the nearest parent `sifr.toml`.
- Parse and validate native `sifr.toml` with optional `[package]` metadata and optional `[source].roots`, defaulting roots to `["."]`.
- Use the clean native `sifr.toml` manifest model directly, borrowing the useful separation of package, workspace, targets, dependencies, and profiles from Rust/Cargo.
- Enter project mode for any entrypoint inside a discovered Sifr workspace, regardless of filename.
- Preserve current behavior when no `sifr.toml` exists.
- Keep stdlib registry resolution highest priority and filesystem-free.
- Resolve user modules in this order:
  1. entry-parent sibling path,
  2. declared workspace source roots in order.
- Detect ambiguity across workspace source roots and emit diagnostics listing all matches.
- Emit unresolved-import diagnostics listing every attempted entry-relative and workspace-relative path.
- Document the resolved design and update architecture/roadmap status.
- Prove the feature with a LeetCode helper pilot after compiler support lands.

## Non-goals

- No package directories, `__init__.sifr`, namespace packages, re-exports, or wildcard imports.
- No dependency manager, package registry, lockfile, external package fetching, or plugin/build hook support.
- No new relative-import semantics; the current `level <= 1` import rule remains.
- No change to embedded `sifr.*` / `_sifr.*` registry contents or precedence.
- No `pyproject.toml` / `[tool.sifr]` compatibility in this implementation slice; if needed later, it must be a separate adapter onto the native manifest model.
- No multi-workspace merge semantics; nearest `sifr.toml` wins and ancestors above it are ignored.
- No CLI workspace override flag in this phase.
- No silent fallback for malformed `sifr.toml`; invalid Sifr workspace config is a hard diagnostic.
- No full package manager implementation in `sifr.toml` during this import-resolution slice; dependency fetching, lockfiles, profiles, and multi-package member expansion are design runway only.

## Target Configuration For This Slice

```toml
[package]
name = "leetcode-fixtures"
version = "0.0.0"
edition = "2026"

[source]
roots = ["audits/leetcode", "."]
```

Validation contract:

- missing `[source]` or `[source].roots` means `roots = ["."]`;
- missing `[package]` is valid and has no semantic effect in this slice;
- `package.name`, when present, must be a string;
- `source.roots`, when present, must be a list of strings;
- unknown top-level tables and unknown nested keys are accepted and ignored in this slice so reserved Cargo-inspired tables remain forward-compatible;
- every source entry must be relative to the workspace root;
- source entries may not escape the workspace root through `..`;
- source entries must resolve to existing directories at discovery time;
- a parseable empty `sifr.toml` is valid and behaves as `roots = ["."]`.

## Native `sifr.toml` Manifest Model

The native Sifr manifest is `sifr.toml`. The compiler should parse it into one internal manifest struct and keep future compatibility adapters, if any, outside resolver/build APIs.

Cargo ideas to reuse:

- make the manifest root explicit and stable;
- separate package metadata from workspace membership;
- keep source/target layout declarative instead of inferred from the current command;
- make resolver-version changes explicit;
- reserve dependencies and profiles without implementing them prematurely;
- prefer deterministic tables over implicit filesystem walks.

Proposed native shape:

```toml
[package]
name = "leetcode-fixtures"
version = "0.0.0"
edition = "2026"

[workspace]
resolver = "1"
members = ["audits/leetcode"]
exclude = ["tmp", "target"]

[source]
roots = ["audits/leetcode", "."]

[[bin]]
name = "merge-two-sorted-lists"
path = "audits/leetcode/0021_merge_two_sorted_lists.sifr"

[dependencies]
# Reserved for future package/dependency work. Empty in this slice.

[profile.dev]
# Reserved for future build/profile work. Empty in this slice.
```

Import-resolution slice decisions:

- `sifr.toml` is the only manifest format implemented in this phase.
- Internal code should use names like `SifrManifest`, `SifrWorkspaceConfig`, and `SourceRoots`, not adapter-specific names.
- `[source].roots` maps directly to the internal source-root list used by `ModuleResolver`.
- `[package].name` maps to display metadata only and has no semantic effect in this slice.
- Future `pyproject.toml` compatibility, if approved, must parse into the same internal manifest model and must not fork resolver behavior.
- Native dependency, lockfile, profile, package-member, and target-table semantics remain reserved and must not affect module resolution until a dedicated package-management phase. Reserved tables and keys are accepted but ignored in this slice.

## Dotted Module Materialization Model

Workspace imports intentionally support dotted module names such as `helpers.list_node`. The current codegen import renderer already treats dots as Rust module path separators, so `from helpers.list_node import ListNode` lowers to a `crate::helpers::list_node::ListNode`-style path. The build pipeline must therefore materialize support modules as a nested Rust module tree rather than as flat filenames.

Chosen implementation model:

- keep the canonical Sifr module ID as the dotted import string, for example `helpers.list_node`;
- keep `ProjectLowering`, compile order, export collection, and codegen keyed by that canonical dotted ID;
- add a shared Rust module layout helper in `crates/sifr_driver/src/project/rust_module_layout.rs` that maps canonical module IDs to Rust module declarations and file paths;
- materialize `helpers.list_node` as `src/helpers/list_node.rs`;
- materialize intermediate namespace files such as `src/helpers/mod.rs` containing `pub mod list_node;`;
- make `src/main.rs` declare only top-level modules, for example `mod helpers;`, not `mod helpers.list_node;`;
- preserve flat modules such as `helper` as `src/helper.rs` with `mod helper;`;
- reject namespace/file collisions such as both `helpers.sifr` and `helpers/list_node.sifr` resolving in the same graph unless a later package-directory design explicitly supports that shape.

Affected files expected in this phase:

- `crates/sifr_driver/src/project/rust_module_layout.rs`: shared helper for canonical module ID to Rust module tree mapping.
- `crates/sifr_driver/src/project/assembly.rs`: emit top-level Rust module declarations from canonical module IDs.
- `crates/sifr_driver/src/build/materialize.rs`: write dotted support modules under nested paths and generate intermediate `mod.rs` files.
- `crates/sifr_driver/src/test_runner/artifacts.rs` and `crates/sifr_driver/src/test_runner/execution.rs`: use the same module layout helper for support modules that contain dotted IDs, while keeping `sifr test` workspace discovery explicitly out of scope.
- `crates/sifr_driver/src/build/project_codegen.rs`: preserve canonical dotted keys in `GeneratedBinaryProject::support_modules` so cache keys remain tied to canonical module identity and content.

Regression requirements:

- A project fixture with `from helpers.list_node import ListNode` must `check`, `emit`, `build`, and `run`.
- Generated Rust for that project must contain `mod helpers;`, `src/helpers/mod.rs`, and `src/helpers/list_node.rs`.
- The cache key must change when `helpers/list_node.sifr` content changes.
- A graph containing both `helpers` and `helpers.list_node` must fail with a deterministic diagnostic until package directories are intentionally designed.

## Workstreams

### WS0: Workspace Discovery And Config Validation

Status: completed

Scope:

- Add a new driver module for user workspace discovery. Use a name that does not collide semantically with `crates/sifr_driver/src/build/workspace.rs`, for example `crates/sifr_driver/src/workspace/`.
- Add a TOML parsing dependency through workspace dependencies and the `sifr_driver` crate.
- Define an internal manifest/config model populated from native `sifr.toml`.
- Implement:
  - `find_workspace_root(entry: &Path) -> Result<Option<WorkspaceRoot>, Vec<CompileError>>`
  - `WorkspaceRoot { dir: PathBuf, config: SifrWorkspaceConfig }`
  - `SifrWorkspaceConfig { source_roots: Vec<PathBuf>, package_name: Option<String> }`
- Walk from the entry file's parent upward. Return the first parent containing a parseable `sifr.toml`.

Implementation notes:

- Keep diagnostics in `CompilePhase::Build`.
- Do not use ad hoc string parsing for TOML.
- Avoid adapter-specific type names; `SifrManifest` and `SifrWorkspaceConfig` are acceptable because `sifr.toml` is now the native source of truth.
- Normalize validation around path components rather than string prefix checks, so `..` escape attempts cannot pass through spelling tricks.
- Keep this module independent of module-graph parsing so WS0 can land as a low-risk PR.
- Use `toml = "0.8"` unless implementation discovers a concrete compatibility problem that is documented in the PR.

Acceptance criteria:

- Unit tests cover nearest-workspace discovery, missing `sifr.toml`, malformed TOML, wrong `package.name` type, wrong `source.roots` type, omitted roots, missing `[package]`, empty manifest, ignored unknown tables/keys, source escape, absolute source paths, empty source strings, leading `./` normalization, source missing, source not-a-directory, path-separator robustness, and nearest-wins behavior.
- Malformed `sifr.toml` files are hard errors when encountered before a closer valid workspace is found. Once a closer valid Sifr workspace is found, ancestors above it are ignored.
- Unit tests must include a closer valid `sifr.toml` with a farther malformed ancestor `sifr.toml` and prove the farther ancestor is ignored.
- `cargo test -p sifr_driver workspace -- --nocapture` or an equivalent targeted test selector passes.
- `cargo fmt --check` passes.

### WS1: Workspace-Aware Compilation Mode

Status: completed

Scope:

- Update `crates/sifr/src/main.rs::resolve_compilation_mode`.
- If `find_workspace_root(entry)` returns a workspace, enter `CompilationMode::Project` for that entry.
- Otherwise preserve the existing `main.sifr` plus sibling-import heuristic exactly.
- Add CLI-facing tests for:
  - non-`main.sifr` entry inside a Sifr workspace enters project mode,
  - non-`main.sifr` entry outside a workspace remains single-file,
  - no `sifr.toml` keeps the entry in the legacy mode path,
  - malformed `sifr.toml` reports a diagnostic rather than silently falling back.

Implementation requirements:

- `resolve_compilation_mode` must return `Result<CompilationMode, Vec<CompileError>>` or an equivalent error-carrying type so workspace parse/config failures reach `build`, `run`, `check`, and `emit`.
- Keep diagnostic formatting centralized with existing CLI error rendering.
- The old `has_local_project_imports` behavior remains for non-workspace `main.sifr` projects.

Acceptance criteria:

- Existing single-file and `main.sifr` project tests remain green.
- Workspace activation does not depend on the entry filename.
- Invalid Sifr workspace config cannot be hidden by single-file fallback.

### WS2: Module Resolver Refactor With No Behavior Change

Status: completed

Scope:

- Refactor `crates/sifr_driver/src/project/discovery.rs` from a `project_dir`-only resolver to an explicit resolver object before adding workspace sources.
- Introduce:
  - `ModuleResolver`
  - `ResolvedModule { module_name: String, path: PathBuf, origin: ModuleOrigin }`
  - `ResolutionError { module_name: String, tried_paths: Vec<PathBuf>, matches: Vec<PathBuf> }`
- Keep the initial resolver configured with only the entry parent directory.
- Update `parse_import_closure_modules` and test-runner call sites to consume `ModuleResolver`.
- Update `crates/sifr_driver/src/test_runner/orchestrator.rs` to pass an entry-parent-only `ModuleResolver` and preserve its current discovery scope.

Implementation notes:

- This PR must be a semantic no-op. It should not start resolving from workspace roots yet.
- Preserve deterministic pending-module traversal with `BTreeSet`.
- Preserve current stdlib/import exclusions for `typing`, `enum`, `sifr.*`, and `_sifr.*`.
- Keep manifest naming neutral through resolver types: use `ModuleResolver` plus a manifest-source/config enum if needed, not parallel `SifrTomlResolver` or `WorkspaceModuleResolver` types.

Acceptance criteria:

- Existing project discovery tests pass unchanged or with only mechanical fixture updates.
- Negative read/parse diagnostics remain stable where no workspace is configured.
- `cargo test -p sifr_driver discovery -- --nocapture` or an equivalent targeted selector passes.

### WS3: Workspace Source Resolution And Diagnostics

Status: completed

Scope:

- Extend `ModuleResolver` to search:
  1. entry parent directory,
  2. each `<workspace_root>/<source>` in declaration order.
- Convert dotted import names to nested paths: `helpers.list_node` resolves to `helpers/list_node.sifr`.
- Keep entry-parent resolution as an unconditional winner over workspace matches.
- Detect ambiguity across workspace source roots only.
- Emit stable diagnostics for unresolved and ambiguous modules.
- Add `crates/sifr_driver/src/project/rust_module_layout.rs` as the shared Rust module layout helper for canonical dotted module IDs and use it in project assembly tests, without yet changing all build materialization call sites.

Diagnostic contract:

- Parse/config:
  - `SIFR-WORKSPACE-0001`: `could not parse sifr.toml at '<path>': <reason>` with URL `https://sifr.sh/docs/errors/SIFR-WORKSPACE-0001`
  - `SIFR-WORKSPACE-0002`: `[source].roots entry '<value>' escapes the workspace root via '..'` with URL `https://sifr.sh/docs/errors/SIFR-WORKSPACE-0002`
  - `SIFR-WORKSPACE-0003`: `[source].roots entry '<value>' is not a directory under the workspace root` with URL `https://sifr.sh/docs/errors/SIFR-WORKSPACE-0003`
  - `SIFR-WORKSPACE-0004`: `[source].roots entry '<value>' must be a relative non-empty path under the workspace root` with URL `https://sifr.sh/docs/errors/SIFR-WORKSPACE-0004`
- Resolution:
  - `SIFR-WORKSPACE-0101`: `could not resolve import '<module>'; tried entry-relative '<path>' and workspace-relative '<path1>', '<path2>'` with URL `https://sifr.sh/docs/errors/SIFR-WORKSPACE-0101`
  - `SIFR-WORKSPACE-0102`: `module '<module>' is ambiguous in workspace '<root>': matches '<path1>' and '<path2>'; reorder [source].roots or rename one module to disambiguate` with URL `https://sifr.sh/docs/errors/SIFR-WORKSPACE-0102`
  - `SIFR-WORKSPACE-0103`: `module '<module>' resolves to file '<path>' but parent name '<parent>' is also a module file '<parent_path>'; package directories are not supported in this phase` with URL `https://sifr.sh/docs/errors/SIFR-WORKSPACE-0103`

Implementation notes:

- Do not include entry-parent matches in ambiguity detection.
- Include every tried path in unresolved diagnostics, even when no source directories are configured beyond the default.
- Avoid diagnostic wording that depends on map iteration or filesystem directory iteration order.
- Keep stdlib registry resolution untouched and before filesystem lookup.
- Do not add package-directory behavior to resolve namespace/file collisions; reject it until a dedicated package-directory design exists.

Acceptance criteria:

- Unit tests prove sibling-wins behavior, workspace source ordering, dotted path resolution, ambiguity diagnostics, unresolved diagnostics, and stdlib-name exclusion.
- Verification-suite diagnostic snapshots cover every new user-facing diagnostic in `human`, `json`, and `compact` formats, including diagnostic code and URL.
- Rust module layout unit tests prove `helpers.list_node` emits `mod helpers;` at the crate root and `pub mod list_node;` in `helpers/mod.rs`.
- `cargo test -p sifr_driver` passes.

### WS4: Build/Run/Check/Emit Wiring And Cache Correctness

Status: completed

Scope:

- Extend `crates/sifr_driver/src/build/entrypoint.rs::RootedEntrypoint` and `RootedEntrypointPlan` with optional workspace context.
- Update `crates/sifr_driver/src/build/api.rs` so project entrypoints discover and pass workspace context internally.
- Ensure `build`, `run`, `check`, and `emit` use the same resolver contract.
- Update cached builds so workspace-resolved module paths and contents participate in the cache key.
- Finish dotted module materialization by writing nested Rust files and intermediate `mod.rs` files through `crates/sifr_driver/src/project/rust_module_layout.rs`.

Implementation notes:

- The existing generated artifact cache already hashes generated project content. Add an explicit dotted-import regression ensuring a workspace helper content change invalidates the cache for `run`/cached builds.
- Keep check/codegen behavior aligned; do not reimplement discovery in CLI code.
- Test-runner workspace discovery remains out of scope, but test-runner Rust materialization must use the same dotted module layout helper so future workspace-aware test support does not fork module-tree behavior.

Acceptance criteria:

- `build`, `run`, `check`, and `emit` all resolve the same workspace helper graph.
- Cache tests prove dotted helper content changes change the cache key and do not produce stale binaries.
- Cache tests prove an unrelated source-root order change that does not alter resolved module content does not invalidate the cache.
- Cache tests prove a source-root order change that changes which duplicate helper resolves does change the cache key.
- Existing artifact cache hit behavior remains for unchanged workspace graphs.

### WS5: Verification-Suite Fixtures, Design Note, And LeetCode Pilot

Status: completed

Scope:

- Add verification-suite coverage for workspace-rooted user imports under `crates/sifr/tests/verification/project/<case_id>/` and register each case in `verification/suites/manifest.json`:
  - pass case: non-`main.sifr` entry imports `helpers.list_node` from a configured workspace source;
  - fail case: ambiguous module under two workspace source roots;
  - fail case: malformed Sifr `sifr.toml`;
  - fail case: unresolved workspace import lists all tried paths.
- Add `internal_docs/sifr_workspace_design.md`.
- Include the native `sifr.toml` manifest model in the design note, with `pyproject.toml` / `[tool.sifr]` compatibility explicitly deferred.
- Update `internal_docs/architecture.md` with workspace discovery in the frontend/build pipeline.
- Pilot one LeetCode fixture using a shared helper:
  - add `audits/leetcode/helpers/list_node.sifr`;
  - add or update repo-root `sifr.toml` with `[package]` and `[source]`;
  - migrate `audits/leetcode/0021_merge_two_sorted_lists.sifr` to import the helper;
  - regenerate `verification/leetcode/leetcode_pair_diff_scan_<YYYYMMDD>.json`;
  - regenerate `verification/leetcode/full_corpus_current_results_<YYYYMMDD>_workspace_pilot.json`.

Implementation notes:

- The compiler support PRs should land before the LeetCode pilot PR.
- The pilot PR must not move broad fixture-helper cleanup into the compiler PRs.
- The pilot source-root list is fixed as `roots = ["audits/leetcode", "."]` unless implementation finds a concrete conflict and updates this phase with the reason before changing it.
- Do not implement reserved dependency/profile/package-member semantics through the pilot.
- Do not place workspace tests under the flat `crates/sifr/tests/e2e/pass` or `fail` directories; that harness discovers only direct `.sifr` files and does not model sifr.toml-rooted fixture trees.
- Document that `sifr test` remains workspace-unaware in this slice and is deferred to a later frontend-mode-parity follow-up.
- Note that `audits/leetcode/helpers/` may already exist locally; the pilot PR owns populating it with `list_node.sifr`, not broad cleanup.

Acceptance criteria:

- The pilot fixture compiles and runs through the normal CLI command.
- Pair scan shows the expected helper-boilerplate reduction for the migrated fixture.
- Full LeetCode corpus has no new `CHECK_ERROR`, `RUN_ERROR`, or `TIMEOUT`.
- Docs link back to the source issue and this phase plan.
- `scripts/run_verification_suites.py --suite project` or the repo-equivalent project verification command passes with the new workspace cases.

### WS6: Final Gate, Review, And Closure

Status: completed

Scope:

- Run full local validation.
- Produce or update closure artifacts:
  - execution checklist with PR links,
  - validation evidence,
  - review notes,
  - final status in this phase file.
- Update `internal_docs/roadmap.md` with the completed ad-hoc phase status.

Required validation:

- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`
- Full LeetCode corpus rerun with no new `CHECK_ERROR`, `RUN_ERROR`, or `TIMEOUT`

Acceptance criteria:

- All workstream PRs are merged.
- Required docs and artifacts are updated.
- External review has no blocking findings.
- Phase status is changed from `ready_to_implement` to `closed`.

## PR Sequencing

1. WS0 discovery/config parser and tests.
2. WS2 resolver refactor as a no-op.
3. WS3 workspace source resolution, diagnostics, and dotted module layout helper.
4. WS1 compilation-mode activation, after workspace source resolution exists so users do not hit an intermediate project-mode-without-source-roots state.
5. WS4 build/check/run/emit wiring plus dotted materialization and cache regression.
6. WS5 verification-suite fixtures, docs, and LeetCode pilot.
7. WS6 final validation/review/closure.

Between WS1 and WS4, workspace activation may produce clean unresolved-import diagnostics for workspace-source imports that are not wired through build APIs yet; this is acceptable only as an intermediate PR state and must not survive WS4.

## Quality Contract

Entry criteria:

- Source issue is present and reviewed for problem scope.
- Existing import graph behavior from Phases 17, 18, 19, 22, and 23 is understood and preserved unless explicitly superseded here.

Milestone quality checks:

- No fallback path may hide malformed `sifr.toml` config.
- No fixture/user helper may be added to `crates/sifr_driver/src/stdlib/registry.rs` for this problem.
- Config code must keep native `sifr.toml` as the source of truth; future compatibility adapters must not hard-code a one-off schema into resolver APIs.
- Every new diagnostic must be stable, deterministic, and tested.
- Every new top-level user-facing diagnostic must include a stable code and `https://sifr.sh/docs/errors/<CODE>` URL.
- No user-triggerable compiler panic may be introduced.
- No data-dependent `.unwrap()` / `.expect()` may be added to generated user runtime paths.
- Every compiler PR must include targeted positive and negative tests.
- Validation evidence must be recorded in the execution checklist before merge.

Exit gate:

- Non-`main.sifr` entries inside a Sifr workspace can import shared user modules from configured workspace sources.
- Existing no-workspace single-file and `main.sifr` project behavior is unchanged.
- Stdlib import resolution is unchanged.
- Diagnostics explain malformed workspace config, ambiguous imports, and unresolved imports with concrete paths.
- Cache invalidation accounts for workspace helper changes.
- LeetCode pilot proves shared helper import without stdlib pollution.
- Required local validation passes.

## Implementation Checklist

- [x] Add workspace config parser/discovery module.
- [x] Add workspace discovery unit tests.
- [x] Refactor module discovery to `ModuleResolver` with no behavior change.
- [x] Update `test_runner/orchestrator.rs` to use entry-parent-only `ModuleResolver` with no scope change.
- [x] Activate workspace-driven project mode in the CLI.
- [x] Make compilation-mode resolution error-carrying so malformed workspace config cannot silently fall back.
- [x] Add workspace source roots to the resolver.
- [x] Add ambiguity and unresolved-import diagnostics.
- [x] Add dotted module Rust layout helper and namespace-conflict diagnostic.
- [x] Wire workspace context through build/check/run/emit.
- [x] Materialize dotted modules as nested Rust module trees.
- [x] Add cache invalidation regression for workspace helper changes.
- [x] Add verification-suite pass/fail workspace fixtures.
- [x] Add `internal_docs/sifr_workspace_design.md`.
- [x] Update `internal_docs/architecture.md`.
- [x] Add LeetCode helper pilot and regenerate corpus artifacts.
- [x] Run required local validation.
- [x] Complete external review and close this phase.
